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
use bson::{doc, Bson};
use clap::{crate_description, crate_name, crate_version};
use core::time::Duration;
use futures::Stream;
use mongodb::options::{
    Acknowledgment, AggregateOptions, ChangeStreamOptions,
    DeleteOptions, DistinctOptions,
    InsertManyOptions, InsertOneOptions, 
    UpdateModifications, UpdateOptions, WriteConcern,
};
use serde::{Deserialize};
use serde_json::json;
use serde_json::Value;
//use axum_macros::debug_handler;

use crate::error::Error as RestError;
use crate::queries::{ExplainFormat, QueriesFormat};
use crate::State;

#[derive(Deserialize, Debug, Clone)]
pub struct DeleteOne {
    pub filter: Document,
    pub options: Option<DeleteOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct UpdateOne {
    pub filter: Document,
    pub update: UpdateModifications,
    pub options: Option<UpdateOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Distinct {
    pub field_name: String,
    pub filter: Option<Document>,
    pub options: Option<DistinctOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Watch {
    pub pipeline: Vec<Document>,
    pub options: Option<ChangeStreamOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Aggregate {
    pub pipeline: Vec<Document>,
    pub options: Option<AggregateOptions>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomInsertManyOptions {
    pub bypass_document_validation: Option<bool>,
    pub ordered: Option<bool>,
    pub w: Option<Acknowledgment>,
    pub n: Option<u32>,
    pub w_timeout: Option<Duration>,
    pub journal: Option<bool>,
    pub comment: Option<Bson>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomInsertOneOptions {
    pub bypass_document_validation: Option<bool>,
    pub w: Option<Acknowledgment>,
    pub n: Option<u32>,
    pub w_timeout: Option<Duration>,
    pub journal: Option<bool>,
    pub comment: Option<Bson>,
}

impl From<CustomInsertManyOptions> for InsertManyOptions {
    fn from(item: CustomInsertManyOptions) -> Self {
        let w_concern = if let Some(w) = item.w {
            Some(w.into())
        } else if let Some(n) = item.n {
            Some(n.into())
        } else {
            None
        };

        let write_concern = WriteConcern::builder()
            .w(w_concern)
            .w_timeout(item.w_timeout)
            .journal(item.journal)
            .build();

        InsertManyOptions::builder()
            .bypass_document_validation(item.bypass_document_validation)
            .ordered(item.ordered)
            .write_concern(write_concern)
            .comment(item.comment)
            .build()
    }
}

impl From<CustomInsertOneOptions> for InsertOneOptions {
    fn from(item: CustomInsertOneOptions) -> Self {
        let w_concern = if let Some(w) = item.w {
            Some(w.into())
        } else if let Some(n) = item.n {
            Some(n.into())
        } else {
            None
        };

        let write_concern = WriteConcern::builder()
            .w(w_concern)
            .w_timeout(item.w_timeout)
            .journal(item.journal)
            .build();

        InsertOneOptions::builder()
            .bypass_document_validation(item.bypass_document_validation)
            .write_concern(write_concern)
            .comment(item.comment)
            .build()
    }
}

pub async fn watch(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Watch>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    log::info!("{{\"fn\": \"watch\", \"method\":\"post\"}}");
    state.db.watch(&db, &coll, payload, queries).await
}

pub async fn watch_latest(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    log::info!("{{\"fn\": \"watch\", \"method\":\"get\"}}");
    let payload = Watch {
        pipeline: vec![doc! {"$match":{}}],
        options: None,
    };
    state.db.watch(&db, &coll, payload, queries).await
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

pub async fn delete_many(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<DeleteOne>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"delete_many\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.delete_many(&db, &coll, payload).await?
    )))
}

pub async fn delete_one(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<DeleteOne>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"delete_one\", \"method\":\"post\"}}");
    Ok(Json(json!(state.db.delete_one(&db, &coll, payload).await?)))
}

pub async fn update_one(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<UpdateOne>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"update_one\", \"method\":\"post\"}}");
    Ok(Json(json!(state.db.update_one(&db, &coll, payload).await?)))
}

pub async fn update_many(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<UpdateOne>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"update_many\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.update_many(&db, &coll, payload).await?
    )))
}

pub async fn distinct(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Distinct>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"distinct\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.distinct(&db, &coll, payload, &queries).await?
    )))
}

pub async fn insert_many(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<CustomInsertManyOptions>,
    Json(body): Json<Vec<Bson>>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"insert\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.insert_many(&db, &coll, body, queries).await?
    )))
}

pub async fn insert_one(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<CustomInsertOneOptions>,
    Json(body): Json<Bson>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"insert_one\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.insert_one(&db, &coll, body, queries).await?
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
