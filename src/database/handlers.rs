use axum::{extract::Path, Extension, Json};
use bson::{doc, to_bson, to_document};
use serde_json::{json, Value};
use opentelemetry::{Key, global};
use opentelemetry::trace::{Span, Tracer};

use crate::error::Error as RestError;
use crate::scopes::AuthorizeScope;
use crate::State;

pub async fn rs_status(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    let tracer = global::tracer("GET /rs/status");

    // Validate that the client has access
    let mut span = tracer.start("scopes.monitor");
    scopes.monitor(&"admin")?;
    span.end();

    log::info!("{{\"fn\": \"rs_status\", \"method\":\"get\"}}");

    let payload = doc! { "replSetGetStatus": 1};

    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_log(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.monitor(&"admin")?;

    log::info!("{{\"fn\": \"rs_log\", \"method\":\"get\"}}");
    let payload = doc! { "getLog": "global"};

    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_operations(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.monitor(&"admin")?;

    log::info!("{{\"fn\": \"rs_operations\", \"method\":\"get\"}}");
    let payload = doc! { "currentOp": 1};
    let response = state.db.run_command(&"admin", payload, false).await?;

    log::debug!("Successfully got inprog");
    let results = response
        .get("inprog")
        .expect("Missing inprog field")
        .as_array()
        .expect("Failed to get log field")
        .clone();

    let output = results
        .iter()
        .map(|x| {
            let mut doc = to_document(x).expect("Malformed operation doc");
            doc.remove("$clusterTime");
            doc.remove("operationTime");
            let bson = to_bson(&x)
                .expect("Malformed bson operation doc")
                .into_relaxed_extjson();
            bson
        })
        .collect();
    Ok(Json(output))
}

pub async fn rs_stats(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.monitor(&"admin")?;

    log::info!("{{\"fn\": \"rs_stats\", \"method\":\"get\"}}");
    let payload = doc! { "serverStatus": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_top(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.monitor(&"admin")?;

    log::info!("{{\"fn\": \"rs_top\", \"method\":\"get\"}}");
    let payload = doc! { "top": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_conn(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.monitor(&"admin")?;

    log::info!("{{\"fn\": \"rs_conn\", \"method\":\"get\"}}");
    let payload = doc! { "connectionStatus": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_pool(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.monitor(&"admin")?;

    log::info!("{{\"fn\": \"rs_pool\", \"method\":\"get\"}}");
    let payload = doc! { "connPoolStats": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn db_stats(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path(db): Path<String>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.monitor(&db)?;

    log::info!("{{\"fn\": \"db_stats\", \"method\":\"get\"}}");
    let payload = doc! { "dbStats": 1};
    Ok(Json(json!(
        state.db.run_command(&db, payload, false).await?
    )))
}

pub async fn coll_stats(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.monitor(&db)?;

    log::info!("{{\"fn\": \"coll_stats\", \"method\":\"get\"}}");
    let payload = doc! { "collStats": coll};
    Ok(Json(json!(
        state.db.run_command(&db, payload, false).await?
    )))
}

pub async fn databases(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    // If we got this far, the user is at least authenticated to the cluster
    log::info!("{{\"fn\": \"databases\", \"method\":\"get\"}}");

    // This needs to check for admin monitor, and if that fails, return the db's the client has access to
    if scopes.read(&"admin").is_ok() {
        Ok(Json(json!(state.db.databases().await?)))
    } else {
        Ok(Json(json!(scopes.authorized_dbs())))
    }
}

pub async fn db_colls(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path(db): Path<String>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.read(&db)?;

    log::info!("{{\"fn\": \"db_colls\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.collections(&db).await?)))
}

pub async fn coll_count(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access
    scopes.read(&db)?;

    log::info!("{{\"fn\": \"coll_count\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.coll_count(&db, &coll).await?)))
}

pub async fn token_roles(
    Extension(scopes): Extension<AuthorizeScope>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"roles\", \"method\":\"get\"}}");
    Ok(Json(json!(scopes.roles())))
}
