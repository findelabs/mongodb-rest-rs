use axum::{extract::OriginalUri, http::StatusCode, response::IntoResponse, Extension, Json};
use clap::{crate_description, crate_name, crate_version};
use serde_json::json;
use serde_json::Value;
//use axum_macros::debug_handler;

use crate::state::State;

pub async fn health(Extension(state): Extension<State>) -> impl IntoResponse {
    log::info!("{{\"fn\": \"health\", \"method\":\"get\"}}");
    if state.db.databases().await.is_err() {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            "{\"message\": \"Unhealthy\"}",
        )
    } else {
        (StatusCode::NOT_FOUND, "{\"message\": \"Healthy\"}")
    }
}

pub async fn root() -> Json<Value> {
    log::info!("{{\"fn\": \"root\", \"method\":\"get\"}}");
    Json(
        json!({ "version": crate_version!(), "name": crate_name!(), "description": crate_description!()}),
    )
}

pub async fn handler_404(OriginalUri(original_uri): OriginalUri) -> impl IntoResponse {
    let parts = original_uri.into_parts();
    let path_and_query = parts.path_and_query.expect("Missing post path and query");
    log::info!(
        "{{\"fn\": \"handler_404\", \"method\":\"get\", \"path\":\"{}\"}}",
        path_and_query
    );
    (
        StatusCode::NOT_FOUND,
        "{\"error_code\": 404, \"message\": \"HTTP 404 Not Found\"}",
    )
}
