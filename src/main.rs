use axum::{
    extract::Extension,
    middleware,
    routing::{get, post, delete},
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
    aggregate, coll_count, coll_index_stats, coll_indexes, coll_stats, databases, db_colls,
    db_stats, echo, find, find_one, handler_404, health, help, root, rs_conn, rs_log,
    rs_operations, rs_pool, rs_stats, rs_status, rs_top, index_create, index_delete, watch, find_explain, aggregate_explain
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

    /// MongoDB username
    #[arg(short = 'U', long, requires = "password", env = "MONGODB_USERNAME")]
    username: Option<String>,

    /// MongoDB username password
    #[arg(short = 'P', long, requires = "username", env = "MONGODB_PASSWORD")]
    password: Option<String>,

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
        .route("/rs/status", get(rs_status))
        .route("/rs/log", get(rs_log))
        .route("/rs/ops", get(rs_operations))
        .route("/rs/stats", get(rs_stats))
        .route("/rs/dbs", get(databases))
        .route("/rs/top", get(rs_top))
        .route("/rs/conn", get(rs_conn))
        .route("/rs/pool", get(rs_pool))
        .route("/db/:db/_stats", get(db_stats))
        .route("/db/:db/_collections", get(db_colls))
        .route("/db/:db/:coll/_count", get(coll_count))
        .route("/db/:db/:coll/_indexes", get(coll_indexes))
        .route("/db/:db/:coll/_index", post(index_create))
        .route("/db/:db/:coll/_index", delete(index_delete))
        .route("/db/:db/:coll/_index_stats", get(coll_index_stats))
        .route("/db/:db/:coll/_find_one", post(find_one))
        .route("/db/:db/:coll/_find", post(find))
        .route("/db/:db/:coll/_find_explain", post(find_explain))
        .route("/db/:db/:coll/_stats", get(coll_stats))
        .route("/db/:db/:coll/_aggregate", post(aggregate))
        .route("/db/:db/:coll/_aggregate_explain", post(aggregate_explain))
        .route("/db/:db/:coll/_watch", post(watch))
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
