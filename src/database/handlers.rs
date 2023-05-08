use axum::{extract::Path, Extension, Json};
use bson::{doc, to_bson, to_document};
use serde_json::{json, Value};

use crate::error::Error as RestError;
use crate::State;

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
    let payload = doc! { "serverStatus": 1};
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
    let payload = doc! { "connectionStatus": 1};
    Ok(Json(json!(
        state.db.run_command(&"admin", payload, false).await?
    )))
}

pub async fn rs_pool(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_pool\", \"method\":\"get\"}}");
    let payload = doc! { "connPoolStats": 1};
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
