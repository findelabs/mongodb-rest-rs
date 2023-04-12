use axum::{
    extract::{OriginalUri, Path, Query},
    body::Bytes,
    body::StreamBody,
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use core::time::Duration;
use futures::Stream;
use mongodb::options::{CollationCaseFirst, CollationStrength, CollationAlternate, CollationMaxVariable, Collation, TextIndexVersion, AggregateOptions, FindOptions, FindOneOptions, ChangeStreamOptions};
//use mongodb::change_stream::event::ChangeStreamEvent;
use bson::Document;
use clap::{crate_description, crate_name, crate_version};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Value;

use crate::error::Error as RestError;
use crate::State;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Formats {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "ejson")]
    Ejson
}

impl Default for Formats {
    fn default() -> Self { Formats::Json }
}

impl Default for QueriesFormat {
    fn default() -> Self { QueriesFormat {format: Some(Formats::default())  }}
}

#[derive(Clone, Deserialize)]
pub struct QueriesFormat {
    pub format: Option<Formats>
}

#[derive(Deserialize)]
pub struct QueriesDelete {
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct FindOne {
    pub filter: Document,
    pub options: Option<FindOneOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Find {
    pub filter: Document,
    pub options: Option<FindOptions>,
    pub explain: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Watch {
    pub pipeline: Vec<Document>,
    pub options: Option<ChangeStreamOptions>
}

#[derive(Deserialize, Debug, Clone)]
pub struct Aggregate {
    pub pipeline: Vec<Document>,
    pub options: Option<AggregateOptions>,
    pub explain: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Index {
    pub keys: Document,
    pub options: Option<IndexCreateOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IndexCreateOptions {
    pub unique: Option<bool>,
    pub name: Option<String>,
    pub partial_filter_expression: Option<Document>,
    pub sparse: Option<bool>,
    pub expire_after: Option<Duration>,
    pub hidden: Option<bool>,
    pub collation: Option<Collation>,
    pub weights: Option<Document>,
    pub default_language: Option<String>,
    pub language_override: Option<String>,
    pub text_index_version: Option<TextIndexVersion>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IndexCollation{
    pub locale: Option<String>,
    pub case_level: Option<bool>,
    pub case_first: Option<CollationCaseFirst>,
    pub strength: Option<CollationStrength>,
    pub numeric_ordering: Option<bool>,
    pub alternate: Option<CollationAlternate>,
    pub max_variable: Option<CollationMaxVariable>,
    pub backwards: Option<bool>
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

pub async fn aggregate(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Aggregate>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"find_one\", \"method\":\"post\"}}");
    Ok(Json(json!(state.db.aggregate(&db, &coll, payload, &queries).await?)))
}

pub async fn index_delete(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesDelete>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"index_create\", \"method\":\"post\"}}");
    Ok(Json(json!(state.db.index_delete(&db, &coll, &queries).await?)))
}

pub async fn index_create(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<Index>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"index_create\", \"method\":\"post\"}}");
    Ok(Json(json!(state.db.index_create(&db, &coll, payload).await?)))
}

pub async fn find(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Find>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"find_one\", \"method\":\"post\"}}");
    Ok(Json(json!(state.db.find(&db, &coll, payload, &queries).await?)))
}

pub async fn find_one(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<FindOne>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"find_one\", \"method\":\"post\"}}");
    Ok(Json(json!(state.db.find_one(&db, &coll, payload, &queries).await?)))
}

pub async fn rs_status(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_status\", \"method\":\"get\"}}");
    Ok(Json(state.db.rs_status().await?))
}

pub async fn rs_log(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_log\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.rs_log().await?)))
}

pub async fn rs_operations(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_operations\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.rs_operations().await?)))
}

pub async fn rs_stats(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_stats\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.rs_stats().await?)))
}

pub async fn rs_top(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_top\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.rs_top().await?)))
}

pub async fn rs_conn(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_conn\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.rs_conn().await?)))
}

pub async fn rs_pool(Extension(state): Extension<State>) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"rs_pool\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.rs_pool().await?)))
}

pub async fn db_stats(
    Extension(state): Extension<State>,
    Path(db): Path<String>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"db_stats\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.db_stats(&db).await?)))
}

pub async fn coll_stats(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"coll_stats\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.coll_stats(&db, &coll).await?)))
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

pub async fn coll_indexes(
    Extension(state): Extension<State>,
    queries: Query<QueriesFormat>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"coll_indexes\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.coll_indexes(&db, &coll, &queries).await?)))
}

pub async fn coll_index_stats(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"coll_indexes\", \"method\":\"get\"}}");
    Ok(Json(json!(state.db.coll_index_stats(&db, &coll).await?)))
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

pub async fn echo(Json(payload): Json<Value>) -> Json<Value> {
    log::info!("{{\"fn\": \"echo\", \"method\":\"post\"}}");
    Json(payload)
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
