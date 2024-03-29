use axum::{
    body::Bytes,
    body::StreamBody,
    extract::{Path, Query},
    Extension, Json,
};
use bson::doc;
use futures::Stream;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::error::Error as RestError;
use crate::find::structs::Find;
use crate::queries::QueriesFormat;
use crate::scopes::AuthorizeScope;
use crate::State;

#[derive(Deserialize)]
pub struct QueriesName {
    pub name: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRole {
    #[serde(alias = "name")]
    pub create_role: String,
    pub privileges: Vec<Privileges>,
    pub roles: Vec<Roles>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authentication_restrictions: Option<Vec<Restrictions>>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Privileges {
    pub resource: String,
    pub actions: Vec<String>,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Roles {
    pub role: String,
    pub db: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Restrictions {
    pub client_source: Vec<String>,
    pub server_address: Vec<String>,
}

pub async fn get_roles(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path(db): Path<String>,
    queries: Query<QueriesFormat>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    // Validate that the client has access to this database
    scopes.dbadmin(&db)?;

    log::info!("{{\"fn\": \"get_roles\", \"db\": \"{}\"}}", &db);
    let payload = Find {
        filter: doc! {},
        options: None,
    };

    state.db.find(&db, &"system.roles", payload, queries).await
}

pub async fn create_role(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path(db): Path<String>,
    //    queries: Query<CustomInsertOneOptions>,
    Json(payload): Json<CreateRole>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.dbadmin(&db)?;

    log::info!("{{\"fn\": \"create_role\", \"db\": \"{}\"}}", &db);
    Ok(Json(json!(state.db.run_command(&db, payload, true).await?)))
}

pub async fn drop_role(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, role)): Path<(String, String)>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.dbadmin(&db)?;

    log::info!("{{\"fn\": \"drop_role\", \"db\": \"{}\", \"role\": \"{}\"}}", &db, &role);

    let payload = doc! {"dropRole": role};

    Ok(Json(json!(state.db.run_command(&db, payload, true).await?)))
}

pub async fn get_role(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, name)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    // Validate that the client has access to this database
    scopes.dbadmin(&db)?;

    log::info!("{{\"fn\": \"get_role\", \"db\": \"{}\", \"role\": \"{}\"}}", &db, &name);
    let payload = Find {
        filter: doc! {"role": &name},
        options: None,
    };

    log::debug!("Searching for roles with {:?}", payload);

    state.db.find(&db, &"system.roles", payload, queries).await
}
