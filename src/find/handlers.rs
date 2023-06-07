use axum::{
    body::Bytes,
    body::StreamBody,
    extract::{Path, Query},
    Extension, Json,
};
use bson::{doc, to_document};
use futures::Stream;
use serde_json::{json, Value};

use mongodb::options::FindOptions;

use crate::error::Error as RestError;
use crate::find::structs::{Distinct, Explain, Find, FindOne, FindRaw, Count};
use crate::queries::{ExplainFormat, QueriesFormat};
use crate::scopes::AuthorizeScope;
use crate::State;

pub async fn find_explain(
    Extension(state): Extension<State>,
    Extension(scopes): Extension<AuthorizeScope>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<ExplainFormat>,
    Json(payload): Json<Find>,
) -> Result<Json<Value>, RestError> {
    log::info!("{{\"fn\": \"find_explain\", \"method\":\"post\"}}");

    // Validate that the client has access to this database
    scopes.read(&db)?;

    let find_raw = FindRaw {
        find: coll.to_string(),
        filter: payload.filter.clone(),
        sort: payload.options.clone().map_or(None, |x| x.sort.clone()),
        projection: payload
            .options
            .clone()
            .map_or(None, |x| x.projection.clone()),
        limit: payload.options.clone().map_or(None, |x| x.limit.clone()),
        skip: payload.options.clone().map_or(None, |x| x.skip.clone()),
        collation: payload
            .options
            .clone()
            .map_or(None, |x| x.collation.clone()),
    };

    let payload = Explain {
        explain: to_document(&find_raw)?,
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

pub async fn find_latest_ten(
    Extension(scopes): Extension<AuthorizeScope>,
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    // Validate that the client has access to this database
    scopes.read(&db)?;

    log::info!("{{\"fn\": \"find\", \"method\":\"get\"}}");
    let payload = Find {
        filter: doc! {},
        options: Some(
            FindOptions::builder()
                .limit(10)
                .sort(doc! {"_id": -1})
                .build(),
        ),
    };
    state.db.find(&db, &coll, payload.into(), queries).await
}

pub async fn find_latest_one(
    Extension(scopes): Extension<AuthorizeScope>,
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    // Validate that the client has access to this database
    scopes.read(&db)?;

    log::info!("{{\"fn\": \"find\", \"method\":\"get\"}}");
    let payload = Find {
        filter: doc! {},
        options: Some(
            FindOptions::builder()
                .limit(1)
                .sort(doc! {"_id": -1})
                .build(),
        ),
    };
    state.db.find(&db, &coll, payload.into(), queries).await
}

pub async fn find(
    Extension(scopes): Extension<AuthorizeScope>,
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Find>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    // Validate that the client has access to this database
    scopes.read(&db)?;

    log::info!("{{\"fn\": \"find\", \"method\":\"post\"}}");
    state.db.find(&db, &coll, payload, queries).await
}

pub async fn count(
    Extension(scopes): Extension<AuthorizeScope>,
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    Json(payload): Json<Count>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.read(&db)?;

    log::info!("{{\"fn\": \"find_one\", \"method\":\"post\"}}");
    
    Ok(Json(json!(
        state.db.count(&db, &coll, payload).await?
    )))
}

pub async fn find_one(
    Extension(scopes): Extension<AuthorizeScope>,
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<FindOne>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.read(&db)?;

    log::info!("{{\"fn\": \"find_one\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.find_one(&db, &coll, payload, &queries).await?
    )))
}

pub async fn distinct(
    Extension(scopes): Extension<AuthorizeScope>,
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Distinct>,
) -> Result<Json<Value>, RestError> {
    // Validate that the client has access to this database
    scopes.read(&db)?;

    log::info!("{{\"fn\": \"distinct\", \"method\":\"post\"}}");
    Ok(Json(json!(
        state.db.distinct(&db, &coll, payload, &queries).await?
    )))
}
