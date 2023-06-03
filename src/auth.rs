use async_recursion::async_recursion;
use axum::{
  http::{Request, StatusCode},
  response::Response,
  middleware::Next, 
  extract::State,
};
use hyper::{Body, Uri};
use chrono::Utc;
use std::sync::{Arc, Mutex};
use serde_json::Value;
use crate::Args;
use serde_json::json;
use jsonwebtoken::jwk::AlgorithmParameters;
use jsonwebtoken::{decode, decode_header, jwk, DecodingKey, Validation};
use std::collections::HashMap;

use crate::error::Error as RestError;
use crate::https::HttpsClient;
use crate::scopes::AuthorizeScope;

type MyResult<T> = std::result::Result<T, RestError>;

#[derive(Clone)]
pub struct AuthJwks {
    replicaset: Option<String>,
    noauth: bool,
    keys: JwksKeys
}

#[derive(Clone)]
pub struct JwksKeys {
    uri: Option<String>,
    audience: Option<String>,
    jwks: Arc<Mutex<Value>>,
    last_read: Arc<Mutex<i64>>,
    client: HttpsClient,
}

impl AuthJwks {
    pub fn new(args: Args, set: Option<String>) -> MyResult<Self> {
        let jwks_keys = JwksKeys::new(args.clone())?;
        
        Ok(AuthJwks { noauth: args.noauth, keys: jwks_keys, replicaset: set })
    }

    #[allow(dead_code)]
    pub async fn keys(&mut self) -> Result<jwk::JwkSet, RestError> {
        self.keys.keys().await
    }

    pub async fn scopes(&mut self, token: &str) -> Result<AuthorizeScope, RestError> {
        let (subject, scopes) = self.keys.scopes(token).await?;
        AuthorizeScope::new(self.replicaset.clone(), scopes, subject)
    }
}

impl JwksKeys {
    pub fn new(args: Args) -> MyResult<Self> {
        Ok(JwksKeys { 
            uri: args.jwks,
            audience: args.audience,
            jwks: Arc::new(Mutex::new(json!(null))),
            last_read: Arc::new(Mutex::new(0i64)),
            client: HttpsClient::default()
        })
    }

    #[async_recursion]
    pub async fn keys(&self) -> Result<jwk::JwkSet, RestError> {

        let jwks = self.jwks.lock().unwrap().clone();
        match jwks {
            Value::Null => {
                log::debug!("Getting keys");
                self.get_keys().await?;
                self.keys().await
            }
            _ => {
                self.renew().await;
                log::debug!("Returning known keys");
                let j: jwk::JwkSet = serde_json::from_value(jwks)?;
                log::debug!("keys: {:?}", j);
                Ok(j)
            }
        }
    }

    pub async fn get_keys(&self) -> Result<(), RestError> {
        let uri = Uri::try_from(self.uri.clone().unwrap().to_string())?;

        log::debug!("jwks uri: {}", uri);

        let req = Request::builder()
            .method("GET")
            .uri(uri)
            .body(Body::empty())
            .expect("request builder");

        let response = self.client.request(req).await?;

        let body = match response.status().as_u16() {
            200 => {
                let contents = hyper::body::to_bytes(response.into_body()).await?;
                let string: Value = serde_json::from_slice(&contents)?;
                string
            }
            _ => {
                log::debug!(
                    "Got bad status code getting config: {}",
                    response.status().as_u16()
                );
                return Err(RestError::BadStatusCode);
            }
        };

        // Save jwks
        let mut jwks = self.jwks.lock().unwrap();
        *jwks = body;

        // Set last_read field
        let now = Utc::now().timestamp();
        let mut last_read = self.last_read.lock().unwrap();
        *last_read = now;

        Ok(())
    }

    pub async fn renew(&self) {
        let last_read = self.last_read.lock().expect("Error getting last_read");
        let diff = Utc::now().timestamp() - *last_read;
        if diff >= 360 {
            log::debug!("jwks has expired, kicking off job to get keys");
            metrics::increment_counter!("proxima_jwks_renew_attempts_total");
            drop(last_read);

            // Kick off background thread to update config
            let me = self.clone();
            tokio::spawn(async move {
                log::debug!("Kicking off background thread to renew jwks");
                if let Err(e) = me.get_keys().await {
                    log::error!("Error gettings updated jwks: {}", e);
                    metrics::increment_counter!("proxima_jwks_renew_failures_total");
                }
            });
        } else {
            log::debug!("\"jwks has not expired, current age is {} seconds\"", diff);
        }
    }

    pub async fn scopes(&self, header: &str) -> Result<(String, Vec<String>), RestError> {
        self.renew().await;

        let token: Vec<&str> = header.split(' ').collect();

        let jwks = self.keys().await?;
        let header = decode_header(token[1])?;
        let kid = match header.kid {
            Some(k) => k,
            None => {
                log::trace!("\"Token doesn't have a `kid` header field\"");
                return Err(RestError::JwtDecode);
            }
        };

        if let Some(j) = jwks.find(&kid) {
            match j.algorithm {
                AlgorithmParameters::RSA(ref rsa) => {
                    let decoding_key = match DecodingKey::from_rsa_components(&rsa.n, &rsa.e) {
                        Ok(k) => k,
                        Err(e) => {
                            log::trace!("\"Error decoding key: {}\"", e);
                            return Err(RestError::JwtDecode);
                        }
                    };
                    let algo = j.common.algorithm.expect("missing algorithm");
                    let mut validation = Validation::new(algo);

                    validation.validate_exp = true;
                    validation.validate_nbf = true;
                    validation.set_audience(&[&self.audience.clone().unwrap()]);

                    log::debug!("Attempting to decode token");
                    let decoded_token = match decode::<HashMap<String, serde_json::Value>>(
                        token[1],
                        &decoding_key,
                        &validation,
                    ) {
                        Ok(e) => Ok(e),
                        Err(e) => {
                            log::debug!("Unable to decode token: {}", e);
                            Err(e)
                        }
                    }?;
                    log::trace!("decoded token: {:?}", decoded_token);

                    let sub = match decoded_token.claims.get("sub") {
                        Some(s) => s.as_str().unwrap_or("none").to_string(),
                        None => "none".to_owned()
                    };

                    let scp = match decoded_token.claims.get("scp") {
                        Some(scopes) => {
                            let vec_values =
                                scopes.as_array().expect("Unable to convert to array");
                            let vec_string = vec_values
                                .iter()
                                .map(|s| s.to_string().replace('"', ""))
                                .collect();
                            vec_string
                        }
                        None => Vec::new(),
                    };
                    log::debug!("token scopes: {:?}", scp);
                    Ok((sub,scp))
                }
                _ => Err(RestError::JwtDecode),
            }
        } else {
            log::warn!("\"No matching JWK found for the given kid\"");
            Err(RestError::JwtDecode)
        }
    }

}

pub async fn auth<B>(State(mut state): State<AuthJwks>, mut req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    if state.noauth {
        req.extensions_mut().insert(AuthorizeScope::default());
        return Ok(next.run(req).await)
    }

    let auth_header = req.headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let scopes = match state.scopes(auth_header).await {
        Ok(i) => i,
        Err(e) => {
            log::debug!("Got error getting token: {}", e);
            return Err(StatusCode::UNAUTHORIZED)
        }
    };

    req.extensions_mut().insert(scopes);
    Ok(next.run(req).await)
}


