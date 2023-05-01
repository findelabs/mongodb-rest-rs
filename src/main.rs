use axum::{
    extract::{DefaultBodyLimit, Extension},
    middleware,
    routing::{delete, get, post},
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
    aggregate, aggregate_explain, coll_count, coll_index_stats, coll_indexes, coll_stats,
    databases, db_colls, db_stats, delete_many, delete_one, distinct, echo, find, find_explain,
    find_latest_one, find_latest_ten, find_one, handler_404, health, help, index_create,
    index_delete, insert_many, insert_one, root, rs_conn, rs_log, rs_operations, rs_pool, rs_stats,
    rs_status, rs_top, update_many, update_one, watch, watch_latest,
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
    #[arg(short, long, env = "MONGODB_READONLY", default_value = "false")]
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
        .route("/db/:db", get(db_colls))
        .route("/db/:db/:coll", get(find_latest_ten))
        .route("/db/:db/:coll/_aggregate", post(aggregate))
        .route("/db/:db/:coll/_aggregate/explain", post(aggregate_explain))
        .route("/db/:db/:coll/_count", get(coll_count))
        .route("/db/:db/:coll/_delete_one", post(delete_one))
        .route("/db/:db/:coll/_delete_many", post(delete_many))
        .route("/db/:db/:coll/_distinct", post(distinct))
        .route("/db/:db/:coll/_find", post(find).get(find_latest_ten))
        .route(
            "/db/:db/:coll/_find_one",
            post(find_one).get(find_latest_one),
        )
        .route("/db/:db/:coll/_find/explain", post(find_explain))
        .route("/db/:db/:coll/_indexes", get(coll_indexes))
        .route("/db/:db/:coll/_indexes", post(index_create))
        .route("/db/:db/:coll/_indexes", delete(index_delete))
        .route("/db/:db/:coll/_indexes/stats", get(coll_index_stats))
        .route("/db/:db/:coll/_insert", post(insert_one))
        .route("/db/:db/:coll/_insert_many", post(insert_many))
        .route("/db/:db/:coll/_update", post(update_many))
        .route("/db/:db/:coll/_update_one", post(update_one))
        .route("/db/:db/:coll/_watch", post(watch).get(watch_latest))
        .route("/db/:db/:coll/_stats", get(coll_stats))
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
        .layer(DefaultBodyLimit::max(16777216))
        .layer(Extension(state));

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port as u16));
    log::info!("Listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
