use axum::{extract::Path, Extension, Json};
use serde_json::{json, Value};

use crate::error::Error as RestError;
use crate::scopes::AuthorizeScope;
use crate::update::structs::Update;
use crate::State;

pub async fn update_one(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<Update>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.write(&db)?;

    log::info!("{{\"fn\": \"update_one\", \"db\": \"{}\", \"coll\": \"{}\"}}", &db, &coll);
    Ok(Json(json!(state.db.update_one(&db, &coll, payload).await?)))
}

pub async fn update_many(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<Update>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.write(&db)?;

    log::info!("{{\"fn\": \"update_many\", \"db\": \"{}\", \"coll\": \"{}\"}}", &db, &coll);
    Ok(Json(json!(
        state.db.update_many(&db, &coll, payload).await?
    )))
}
