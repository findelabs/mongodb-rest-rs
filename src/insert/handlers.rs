use axum::{
    extract::{Path, Query},
    Extension, Json,
};
use bson::Document;
use serde_json::{json, Value};

use crate::error::Error as RestError;
use crate::insert::structs::{CustomInsertManyOptions, CustomInsertOneOptions};
use crate::scopes::AuthorizeScope;
use crate::State;

pub async fn insert_many(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<CustomInsertManyOptions>,
    Json(body): Json<Vec<Document>>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.write(&db)?;

    log::info!("{{\"fn\": \"insert\", \"db\": \"{}\", \"coll\": \"{}\"}}", &db, &coll);
    Ok(Json(json!(
        state.db.insert_many(&db, &coll, body, queries).await?
    )))
}

pub async fn insert_one(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<CustomInsertOneOptions>,
    Json(body): Json<Document>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.write(&db)?;

    log::info!("{{\"fn\": \"insert_one\", \"db\": \"{}\", \"coll\": \"{}\"}}", &db, &coll);
    Ok(Json(json!(
        state.db.insert_one(&db, &coll, body, queries).await?
    )))
}
