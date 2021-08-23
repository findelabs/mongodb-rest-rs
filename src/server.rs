use hyper::{Body, Method, Request, Response, StatusCode};
use std::str::from_utf8;
//use rust_tools::http::queries;
use bson::doc;
use rust_tools::bson::{to_doc, to_doc_vec};
use rust_tools::strings::get_root_path;
use std::error::Error;
use clap::ArgMatches;
use bson::Document;
use crate::db;

type BoxResult<T> = Result<T,Box<dyn Error + Send + Sync>>;

// This is the main handler, to catch any failures in the echo fn
pub async fn main_handler(
    opts: ArgMatches<'_>,
    req: Request<Body>,
    db: db::DB,
) -> BoxResult<Response<Body>> {
    match echo(opts, req, db).await {
        Ok(s) => {
            log::debug!("Handler got success");
            Ok(s)
        }
        Err(e) => {
            log::error!("Handler caught error: {}", e);
            let mut response = Response::new(Body::from(format!("{{\"error\" : \"{}\"}}", e)));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            Ok(response)
        }
    }
}

// This is our service handler. It receives a Request, routes on its
// path, and returns a Future of a Response.
async fn echo(opts: ArgMatches<'_>, req: Request<Body>, db: db::DB) -> BoxResult<Response<Body>> {

    // Check if first folder in path is _cat
    // Get first segment in uri path, looking for _cat (for now)
    let chunks: Vec<&str> = req.uri().path().split("/").collect();
    let first = chunks.get(1).unwrap_or_else(|| &"na");

    // Get path
    let path = &req.uri().path();

    // Match on first folder in path. Currently we just are looking for _cat, but there will be more in the future.
    match first {
        &"_cat" => {
            match (req.method(), path) {
                (&Method::GET, &"/_cat/collections") => {
                    let path = req.uri().path();
                    log::info!("Received GET to {}", &path);
        
                    match db.collections().await {
                        Ok(collections) => {
                            let json_doc = serde_json::to_string(&collections)
                                .expect("failed converting collection bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                },
                (&Method::GET, &"/_cat/rs/status") => {
                    let path = req.uri().path();
                    log::info!("Received GET to {}", &path);
        
                    match db.rs_status().await {
                        Ok(results) => {
                            let json_doc = serde_json::to_string(&results)
                                .expect("failed converting collection bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                },
                (&Method::GET, &"/_cat/rs/log") => {
                    let path = req.uri().path();
                    log::info!("Received GET to {}", &path);
        
                    match db.get_log().await {
                        Ok(results) => {
                            let json_doc = serde_json::to_string(&results)
                                .expect("failed converting collection bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                },
                (&Method::GET, &"/_cat/rs/stats") => {
                    let path = req.uri().path();
                    log::info!("Received GET to {}", &path);
        
                    match db.server_status().await {
                        Ok(results) => {
                            let json_doc = serde_json::to_string(&results)
                                .expect("failed converting collection bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                },
                (&Method::GET, &"/_cat/rs/operations") => {
                    let path = req.uri().path();
                    log::info!("Received GET to {}", &path);
        
                    match db.inprog().await {
                        Ok(results) => {
                            let json_doc = serde_json::to_string(&results)
                                .expect("failed converting collection bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                },
                (&Method::GET, &"/_cat/rs/top") => {
                    let path = req.uri().path();
                    log::info!("Received GET to {}", &path);
        
                    match db.top().await {
                        Ok(results) => {
                            let json_doc = serde_json::to_string(&results)
                                .expect("failed converting collection bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                _ => Ok(Response::new(Body::from(format!(
                    "{{ \"msg\" : \"{} is not a known path under /_cat\" }}",
                    path
                )))),
            }
        },
        // Here, since _cat has been skipped, match Method and Last folder as action
        _ => {
            // Get last segment in uri path
            let last = &req.uri().path().split("/").last().unwrap_or_else(|| "na");

            match (req.method(), last) {
                (&Method::POST, &"_insert") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson(req).await?;

                    match db.insert(opts, &collection, data).await {
                        Ok(_) => {
                            let mut response = Response::new(Body::from(format!(
                                "{{\"msg\" : \"Successfully saved\" }}"
                            )));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::debug!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                (&Method::POST, &"_update_one") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson_vec(req).await?;

                    match db.update_one(opts, &collection, data).await {
                        Ok(body) => {
                            let mut response = Response::new(Body::from(format!(
                                "{{\"msg\" : \"{}\" }}", body
                            )));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                (&Method::POST, &"_insert_many") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson_vec(req).await?;

                    match db.insert_many(opts, &collection, data).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                (&Method::POST, &"_find_one") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson(req).await?;

                    match db.findone(&collection, data, None).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                (&Method::POST, &"_find_one_project") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, mut data) = data_to_bson_vec(req).await?;

                    // Get query
                    let query = match data.len(){
                        0 => doc! {},
                        _ => data.swap_remove(0)
                    };

                    // Get projection
                    let projection = match data.len(){
                        0 => None,
                        _ => Some(data.swap_remove(0))
                    };

                    match db.findone(&collection, query, projection).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                (&Method::POST, &"_find") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, query) = data_to_bson(req).await?;

                    match db.find(&collection, query, None).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                (&Method::POST, &"_find_project") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, mut data) = data_to_bson_vec(req).await?;

                    // Get query
                    let query = match data.len(){
                        0 => doc! {},
                        _ => data.swap_remove(0)
                    };

                    // Get projection
                    let projection = match data.len(){
                        0 => None,
                        _ => Some(data.swap_remove(0))
                    };

                    match db.find(&collection, query, projection).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                (&Method::POST, &"_aggregate") => {
                    let path = req.uri().path();
                    log::info!("Received POST to {}", &path);

                    // Get data and collection
                    let (collection, data) = data_to_bson_vec(req).await?;

                    match db.aggregate(&collection, data).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                }
                (&Method::GET, &"_count") => {
                    log::info!("Received GET to {}", req.uri().path());

                    // Get short root path (the collection name)
                    let (parts, _body) = req.into_parts();
                    let collection = get_root_path(&parts);

                    match db.count(&collection).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                },
                (&Method::GET, &"_indexes") => {
                    log::info!("Received GET to {}", req.uri().path());

                    // Get short root path (the collection name)
                    let (parts, _body) = req.into_parts();
                    let collection = get_root_path(&parts);

                    match db.get_indexes(&collection).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                },
                (&Method::GET, &"_index_stats") => {
                    log::info!("Received GET to {}", req.uri().path());

                    // Get short root path (the collection name)
                    let (parts, _body) = req.into_parts();
                    let collection = get_root_path(&parts);

                    match db.index_stats(&collection).await {
                        Ok(doc) => {
                            let json_doc = serde_json::to_string(&doc)
                                .expect("failed converting bson to json");
                            let mut response = Response::new(Body::from(json_doc));
                            *response.status_mut() = StatusCode::OK;
                            Ok(response)
                        }
                        Err(e) => {
                            log::error!("Got error {}", e);
                            Err(Box::new(e))
                        }
                    }
                },
                _ => Ok(Response::new(Body::from(format!(
                    "{{ \"msg\" : \"{} {} is not a recognized action\" }}",
                    req.method(),
                    last)
                ))),
            }
        }
    }
}

pub async fn get_data(req: Request<Body>) -> BoxResult<(String, String)> {
    // Split apart request
    let (parts, body) = req.into_parts();

    // Get short root path
    let collection = get_root_path(&parts);

    // Create queriable hashmap from queries
    // let _queries = queries(&parts).expect("Failed to generate hashmap of queries");

    // Convert body to utf8 string
    let whole_body = hyper::body::to_bytes(body).await?;
    let whole_body_vec = whole_body.iter().cloned().collect::<Vec<u8>>();
    let value = from_utf8(&whole_body_vec).to_owned()?;
    Ok((collection, value.to_owned()))
}

pub async fn data_to_bson(req: Request<Body>) -> BoxResult<(String, Document)> {

    let (collection, value) = get_data(req).await?;

    // Convert string to bson
    let data = match to_doc(&value) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };

    // Print out converted bson doc
    log::debug!("Converted json into bson doc: {}", data);

    Ok((collection, data))
}

pub async fn data_to_bson_vec(req: Request<Body>) -> BoxResult<(String, Vec<Document>)> {

    let (collection, value) = get_data(req).await?;

    // Convert string to bson
    let data = match to_doc_vec(&value) {
        Ok(d) => d,
        Err(e) => return Err(e),
    };

    // Print out converted bson doc
    log::debug!("Converted json into bson doc: {:?}", data);

    Ok((collection, data))
}
