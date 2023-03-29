use axum::{
    extract::Extension,
    middleware,
    routing::{get, post},
    Router,
};
use chrono::Local;
use clap::Parser;
use env_logger::{Builder, Target};
use log::LevelFilter;
use std::future::ready;
use std::io::Write;
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;

mod db;
mod error;
mod handlers;
mod metrics;
mod state;

use crate::metrics::{setup_metrics_recorder, track_metrics};
use handlers::{
    aggregate, coll_count, coll_index_stats, coll_indexes, databases, db_colls, echo, find,
    find_one, handler_404, health, help, root, rs_log, rs_operations, rs_stats, rs_status, rs_top,
};
use state::State;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Port to listen on
    #[arg(short, long, default_value_t = 8080, env = "API_PORT")]
    port: u16,

    /// Default connection uri
    #[arg(short, long, env = "MONGODB_URI")]
    uri: String,

    /// Should connection be readonly?
    #[arg(short, long, env = "MONGODB_READONLY")]
    readonly: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    // Initialize log Builder
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{{\"date\": \"{}\", \"level\": \"{}\", \"log\": {}}}",
                Local::now().format("%Y-%m-%dT%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .target(Target::Stdout)
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();

    // Create state for axum
    let state = State::new(args.clone()).await?;

    // Create prometheus handle
    let recorder_handle = setup_metrics_recorder();

    let base = Router::new()
        .route("/_cat/rs/status", get(rs_status))
        .route("/_cat/rs/log", get(rs_log))
        .route("/_cat/rs/stats", get(rs_stats))
        .route("/_cat/rs/operations", get(rs_operations))
        .route("/_cat/rs/top", get(rs_top))
        .route("/_cat/dbs", get(databases))
        .route("/:db/_collections", get(db_colls))
        .route("/:db/:coll/_count", get(coll_count))
        .route("/:db/:coll/_indexes", get(coll_indexes))
        .route("/:db/:coll/_index_stats", get(coll_index_stats))
        .route("/:db/:coll/_find_one", post(find_one))
        .route("/:db/:coll/_find", post(find))
        .route("/:db/:coll/_aggregate", post(aggregate))
        .route("/", get(root));

    // These should NOT be authenticated
    let standard = Router::new()
        .route("/health", get(health))
        .route("/echo", post(echo))
        .route("/help", get(help))
        .route("/metrics", get(move || ready(recorder_handle.render())));

    let app = Router::new()
        .merge(base)
        .merge(standard)
        .layer(TraceLayer::new_for_http())
        .route_layer(middleware::from_fn(track_metrics))
        .fallback(handler_404)
        .layer(Extension(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port as u16));
    log::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
