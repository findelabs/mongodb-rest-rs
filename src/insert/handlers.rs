use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use bson::Bson;
use serde_json::{json, Value};

use crate::error::Error as RestError;
use crate::insert::structs::{CustomInsertManyOptions, CustomInsertOneOptions};
use crate::State;

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
