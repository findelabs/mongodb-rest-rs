use crate::error::Error as RestError;
use crate::queries::QueriesFormat;
use crate::watch::structs::Watch;
use crate::State;
use axum::{
    body::Bytes,
    body::StreamBody,
    extract::{Path, Query},
    Extension, Json,
};
use bson::doc;
use futures::Stream;

pub async fn watch(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Watch>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    log::info!("{{\"fn\": \"watch\", \"method\":\"post\"}}");
    state.db.watch(&db, &coll, payload, queries).await
}

pub async fn watch_latest(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    log::info!("{{\"fn\": \"watch\", \"method\":\"get\"}}");
    let payload = Watch {
        pipeline: vec![doc! {"$match":{}}],
        options: None,
    };
    state.db.watch(&db, &coll, payload, queries).await
}
