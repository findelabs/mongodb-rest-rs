use axum::{extract::Path, Extension, Json};
use serde_json::{json, Value};

use crate::delete::structs::DeleteOne;
use crate::error::Error as RestError;
use crate::State;

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
