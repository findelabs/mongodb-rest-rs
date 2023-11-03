use axum::{extract::Path, Extension, Json};
use serde_json::{json, Value};

use crate::delete::structs::DeleteOne;
use crate::error::Error as RestError;
use crate::scopes::AuthorizeScope;
use crate::State;

pub async fn delete_many(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<DeleteOne>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.write(&db)?;

    log::info!("{{\"fn\": \"delete_many\", \"db\":\"{}\", \"coll\":\"{}\"}}", &db, &coll);
    Ok(Json(json!(
        state.db.delete_many(&db, &coll, payload).await?
    )))
}

pub async fn delete_one(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<DeleteOne>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.write(&db)?;

    log::info!("{{\"fn\": \"delete_one\", \"db\":\"{}\", \"coll\":\"{}\"}}", &db, &coll);
    Ok(Json(json!(state.db.delete_one(&db, &coll, payload).await?)))
}
