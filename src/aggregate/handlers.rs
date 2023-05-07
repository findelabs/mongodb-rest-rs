use axum::{
    body::Bytes,
    body::StreamBody,
    extract::{Path, Query},
    Extension,
    Json,
};
use bson::{to_document, doc};
use futures::Stream;
use serde_json::{json, Value};

use crate::error::Error as RestError;
use crate::queries::{ExplainFormat, QueriesFormat};
use crate::aggregate::structs::Aggregate;
use crate::State;

pub async fn aggregate(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<QueriesFormat>,
    Json(payload): Json<Aggregate>,
) -> Result<StreamBody<impl Stream<Item = Result<Bytes, RestError>>>, RestError> {
    log::info!("{{\"fn\": \"aggregate\", \"method\":\"post\"}}");
    state.db.aggregate(&db, &coll, payload, queries).await
}

pub async fn aggregate_explain(
    Extension(state): Extension<State>,
    Path((db, coll)): Path<(String, String)>,
    queries: Query<ExplainFormat>,
    Json(payload): Json<Aggregate>,
) -> Result<Json<Value>, RestError> {
    use crate::db::AggregateRaw;
    use crate::db::Explain;

    log::info!("{{\"fn\": \"aggregate_explain\", \"method\":\"post\"}}");

    let aggregate_raw = AggregateRaw {
        aggregate: coll.to_string(),
        pipeline: payload.pipeline,
        cursor: doc! {},
    };

   let payload = Explain {
        explain: to_document(&aggregate_raw)?,
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

