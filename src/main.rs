use chrono::Local;
use clap::{crate_version, App, Arg};
use env_logger::{Builder, Target};
use log::LevelFilter;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Server};
use std::io::Write;

use db::DB;
use error::MyError;

mod db;
mod error;
mod server;

type Result<T> = std::result::Result<T, MyError>;

#[tokio::main]
async fn main() -> Result<()> {
    let opts = App::new("json-bucket")
        .version(crate_version!())
        .author("Daniel F. <dan@findelabs.com>")
        .about("Main findereport site generator")
        .arg(
            Arg::with_name("uri")
                .short("u")
                .long("uri")
                .required(true)
                .value_name("URI")
                .env("MONGODB_URI")
                .help("MongoDB URI")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("db")
                .short("d")
                .long("db")
                .required(true)
                .value_name("MONGODB_DB")
                .env("MONGODB_DB")
                .help("MongoDB Database")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("port")
                .short("p")
                .long("port")
                .help("Set port to listen on")
                .required(false)
                .env("LISTEN_PORT")
                .default_value("8080")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("readonly")
                .short("r")
                .long("readonly")
                .help("Only access database read-only")
                .required(false)
        )
        .get_matches();

    // Initialize log Builder
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{{\"date\": \"{}\", \"level\": \"{}\", \"message\": \"{}\"}}",
                Local::now().format("%Y-%m-%dT%H:%M:%S:%f"),
                record.level(),
                record.args()
            )
        })
        .target(Target::Stdout)
        .filter_level(LevelFilter::Error)
        .parse_default_env()
        .init();

    // Read in config file
    let url = &opts.value_of("uri").unwrap();
    let db = &opts.value_of("db").unwrap();
    let port: u16 = opts.value_of("port").unwrap().parse().unwrap_or_else(|_| {
        eprintln!("specified port isn't in a valid range, setting to 8080");
        8080
    });

    let db = DB::init(&url, &db).await?;
    let addr = ([0, 0, 0, 0], port).into();
    let service = make_service_fn(move |_| {
        let opts = opts.clone();
        let db = db.clone();
        async move {
            Ok::<_, hyper::Error>(service_fn(move |req: Request<Body>| {
                server::main_handler(opts.clone(), req, db.clone())
            }))
        }
    });

    let server = Server::bind(&addr).serve(service);

    println!(
        "Starting json-bucket:{} on http://{}",
        crate_version!(),
        addr
    );

    server.await?;
    //    if let Err(e) = server.await {
    //        eprintln!("server error: {}", e);
    //    }

    Ok(())
}
