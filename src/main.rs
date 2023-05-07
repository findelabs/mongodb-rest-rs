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
mod roles;
mod find;
mod queries;
mod index;
mod delete;
mod insert;
mod update;
mod watch;
mod aggregate;
mod database;

use crate::metrics::{setup_metrics_recorder, track_metrics};
use handlers::{
    root, handler_404, health, 
};

use database::handlers::{databases, db_colls, db_stats,rs_conn, rs_log, rs_operations, rs_pool, rs_stats, rs_status, rs_top, coll_count, coll_stats};
use roles::handlers::{get_roles, create_role, drop_role, get_role};
use find::handlers::{find_explain, find_latest_ten, find_latest_one, find, find_one, distinct};
use index::handlers::{index_create, index_delete, indexes, index_stats};
use delete::handlers::{delete_one, delete_many};
use insert::handlers::{insert_one, insert_many};
use update::handlers::{update_one, update_many};
use watch::handlers::{watch, watch_latest};
use aggregate::handlers::{aggregate, aggregate_explain};
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

    let v1 = Router::new()
        .route("/rs/status", get(rs_status))
        .route("/rs/log", get(rs_log))
        .route("/rs/ops", get(rs_operations))
        .route("/rs/stats", get(rs_stats))
        .route("/rs/dbs", get(databases))
        .route("/rs/top", get(rs_top))
        .route("/rs/conn", get(rs_conn))
        .route("/rs/pool", get(rs_pool))
        .route("/db/:db", get(db_colls))
        .route("/db/:db/_stats", get(db_stats))
        .route("/db/:db/_roles", get(get_roles))
        .route("/db/:db/_roles", post(create_role))
        .route("/db/:db/_roles/:role", get(get_role).delete(drop_role))
        .route("/db/:db/collection/:coll", get(find_latest_ten))
        .route("/db/:db/collection/:coll/_aggregate", post(aggregate))
        .route("/db/:db/collection/:coll/_aggregate/explain", post(aggregate_explain))
        .route("/db/:db/collection/:coll/_count", get(coll_count))
        .route("/db/:db/collection/:coll/_delete_one", post(delete_one))
        .route("/db/:db/collection/:coll/_delete_many", post(delete_many))
        .route("/db/:db/collection/:coll/_distinct", post(distinct))
        .route("/db/:db/collection/:coll/_find", post(find).get(find_latest_ten))
        .route(
            "/db/:db/collection/:coll/_find_one",
            post(find_one).get(find_latest_one),
        )
        .route("/db/:db/collection/:coll/_find/explain", post(find_explain))
        .route("/db/:db/collection/:coll/_indexes", get(indexes))
        .route("/db/:db/collection/:coll/_indexes", post(index_create))
        .route("/db/:db/collection/:coll/_indexes", delete(index_delete))
        .route("/db/:db/collection/:coll/_indexes/stats", get(index_stats))
        .route("/db/:db/collection/:coll/_insert", post(insert_one))
        .route("/db/:db/collection/:coll/_insert_many", post(insert_many))
        .route("/db/:db/collection/:coll/_update", post(update_many))
        .route("/db/:db/collection/:coll/_update_one", post(update_one))
        .route("/db/:db/collection/:coll/_watch", post(watch).get(watch_latest))
        .route("/db/:db/collection/:coll/_stats", get(coll_stats))
        .route("/", get(root));

    // These should NOT be authenticated
    let standard = Router::new()
        .route("/health", get(health))
        .route("/metrics", get(move || ready(recorder_handle.render())));

    let app = Router::new()
        .merge(v1.clone())
        .merge(standard)
        .nest("/api/beta", v1)
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
