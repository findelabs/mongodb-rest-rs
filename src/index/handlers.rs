use axum::{
    body::{Bytes, StreamBody},
    extract::{Path, Query},
    Extension, Json,
};
use futures::Stream;
use serde_json::{json, Value};

use crate::error::Error as RestError;
use crate::index::structs::Index;
use crate::queries::{QueriesDelete, QueriesFormat};
use crate::State;

pub async fn index_delete(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesDelete>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"index_create\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.index_delete(&db, &coll, &queries).await?
    )))
}

pub async fn index_create(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<Index>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"index_create\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.index_create(&db, &coll, payload).await?
    )))
}

pub async fn indexes(
    Extension(state): Extension<State>,
    queries: Query<QueriesFormat>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"coll_indexes\", \"method\":\"get\"}}");
    Ok(Json(json!(
        state.db.coll_indexes(&db, &coll, &queries).await?
    )))
}

pub async fn index_stats(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    log::info!("{{\"fn\": \"coll_indexes\", \"method\":\"get\"}}");
    state.db.coll_index_stats(&db, &coll).await
}
