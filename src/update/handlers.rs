use axum::{
    extract::{Path},
    Extension,
    Json,
};
use serde_json::{json, Value};

use crate::error::Error as RestError;
use crate::update::structs::{Update};
use crate::State;

pub async fn update_one(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<Update>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"update_one\", \"method\":\"post\"}}");
    Ok(Json(json!(state.db.update_one(&db, &coll, payload).await?)))
}

pub async fn update_many(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<Update>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"update_many\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.update_many(&db, &coll, payload).await?
    )))
}
