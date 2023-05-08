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

mod aggregate;
mod database;
mod db;
mod delete;
mod error;
mod find;
mod handlers;
mod index;
mod insert;
mod metrics;
mod queries;
mod roles;
mod state;
mod update;
mod watch;

use crate::metrics::{setup_metrics_recorder, track_metrics};
use handlers::{handler_404, health, root};

use aggregate::handlers::{aggregate, aggregate_explain};
use database::handlers::{
    coll_count, coll_stats, databases, db_colls, db_stats, rs_conn, rs_log, rs_operations, rs_pool,
    rs_stats, rs_status, rs_top,
};
use delete::handlers::{delete_many, delete_one};
use find::handlers::{distinct, find, find_explain, find_latest_one, find_latest_ten, find_one};
use index::handlers::{index_create, index_delete, index_stats, indexes};
use insert::handlers::{insert_many, insert_one};
use roles::handlers::{create_role, drop_role, get_role, get_roles};
use state::State;
use update::handlers::{update_many, update_one};
use watch::handlers::{watch, watch_latest};

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
        .route(
            "/db/:db/collection/:coll/_aggregate/explain",
            post(aggregate_explain),
        )
        .route("/db/:db/collection/:coll/_count", get(coll_count))
        .route("/db/:db/collection/:coll/_delete_one", post(delete_one))
        .route("/db/:db/collection/:coll/_delete_many", post(delete_many))
        .route("/db/:db/collection/:coll/_distinct", post(distinct))
        .route(
            "/db/:db/collection/:coll/_find",
            post(find).get(find_latest_ten),
        )
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
        .route(
            "/db/:db/collection/:coll/_watch",
            post(watch).get(watch_latest),
        )
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
