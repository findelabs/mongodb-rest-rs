use axum::{
    body::Bytes,
    body::StreamBody,
    extract::{OriginalUri, Path, Query},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use bson::to_document;
use bson::to_bson;
use bson::Document;
use bson::{doc};
use clap::{crate_description, crate_name, crate_version};
use futures::Stream;
use mongodb::options::{
    AggregateOptions, 
};
use serde::{Deserialize};
use serde_json::json;
use serde_json::Value;
//use axum_macros::debug_handler;

use crate::error::Error as RestError;
use crate::queries::{ExplainFormat, QueriesFormat};
use crate::State;

#[derive(Deserialize, Debug, Clone)]
pub struct Aggregate {
    pub pipeline: Vec<Document>,
    pub options: Option<AggregateOptions>,
}

pub async fn aggregate(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Aggregate>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    log::info!("{{\"fn\": \"aggregate\", \"method\":\"post\"}}");
    state.db.aggregate(&db, &coll, payload, queries).await
}

pub async fn aggregate_explain(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<ExplainFormat>,
    Json(payload): Json<Aggregate>,
) -> Result<Json<Value>, RestError> {
    use crate::db::AggregateRaw;
    use crate::db::Explain;

    log::info!("{{\"fn\": \"aggregate_explain\", \"method\":\"post\"}}");

    let aggregate_raw = AggregateRaw {
        aggregate: coll.to_string(),
        pipeline: payload.pipeline,
        cursor: doc! {},
    };

   let payload = Explain {
        explain: to_document(&aggregate_raw)?,
        verbosity: queries
            .verbosity
            .as_ref()
            .unwrap_or(&"allPlansExecution".to_string())
            .clone(),
        comment: "mongodb-rest-rs explain".to_string(),
    };

    Ok(Json(json!(
        state.db.run_command(&db, payload, false).await?
    )))
}

pub async fn rs_status(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_status\", \"method\":\"get\"}}");

    let payload = doc! { "replSetGetStatus": 1};

    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_log(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_log\", \"method\":\"get\"}}");
    let payload = doc! { "getLog": "global"};

    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_operations(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
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

pub async fn rs_stats(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_stats\", \"method\":\"get\"}}");
    let payload  = doc! { "serverStatus": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_top(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_top\", \"method\":\"get\"}}");
    let payload = doc! { "top": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_conn(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_conn\", \"method\":\"get\"}}");
    let payload  = doc! { "connectionStatus": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_pool(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_pool\", \"method\":\"get\"}}");
    let payload  = doc! { "connPoolStats": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn db_stats(
    Extension(state): Extension<State>,
    Path(db): Path<String>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"db_stats\", \"method\":\"get\"}}");
    let payload = doc! { "dbStats": 1};
    Ok(Json(json!(
        state.db.run_command(&db, payload, false).await?
    )))
}

pub async fn coll_stats(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"coll_stats\", \"method\":\"get\"}}");
    let payload = doc! { "collStats": coll};
    Ok(Json(json!(
        state.db.run_command(&db, payload, false).await?
    )))
}

pub async fn databases(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"databases\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.databases().await?)))
}

pub async fn db_colls(
    Extension(state): Extension<State>,
    Path(db): Path<String>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"db_colls\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.collections(&db).await?)))
}

pub async fn coll_count(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"coll_count\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.coll_count(&db, &coll).await?)))
}

pub async fn health() -> Json<Value> {
    log::info!("{{\"fn\": \"health\", \"method\":\"get\"}}");
    Json(json!({ "msg": "Healthy"}))
}

pub async fn root() -> Json<Value> {
    log::info!("{{\"fn\": \"root\", \"method\":\"get\"}}");
    Json(
        json!({ "version": crate_version!(), "name": crate_name!(), "description": crate_description!()}),
    )
}

pub async fn help() -> Json<Value> {
    log::info!("{{\"fn\": \"help\", \"method\":\"get\"}}");
    let payload = json!({"paths": {
            "/health": "Get the health of the api",
            "/config": "Get config of api",
            "/reload": "Reload the api's config",
            "/echo": "Echo back json payload (debugging)",
            "/help": "Show this help message",
            "/:endpoint": "Show config for specific endpoint",
            "/:endpoint/*path": "Pass through any request to specified endpoint"
        }
    });
    Json(payload)
}

pub async fn handler_404(OriginalUri(original_uri): OriginalUri) -> impl IntoResponse {
    let parts = original_uri.into_parts();
    let path_and_query = parts.path_and_query.expect("Missing post path and query");
    log::info!(
        "{{\"fn\": \"handler_404\", \"method\":\"get\", \"path\":\"{}\"}}",
        path_and_query
    );
    (
        StatusCode::NOT_FOUND,
        "{\"error_code\": 404, \"message\": \"HTTP 404 Not Found\"}",
    )
}
