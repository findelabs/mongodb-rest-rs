use chrono::prelude::*;
use mongodb::bson::{doc, document::Document};
//use mongodb::{options::ClientOptions, options::FindOptions, Client, Collection};
use crate::error::MyError;
use mongodb::{options::ClientOptions, options::FindOneOptions, options::FindOptions, options::UpdateOptions, Client};
//use serde::{Deserialize, Serialize};
use futures::StreamExt;
use clap::ArgMatches;
use std::collections::HashMap;
use bson::Bson;

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
    pub db: String,
}

type Result<T> = std::result::Result<T, MyError>;

impl DB {
    pub async fn init(url: &str, db: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(url).await?;
        client_options.app_name = Some("json-bucket".to_string());
        Ok(Self {
            client: Client::with_options(client_options)?,
            db: db.to_owned(),
        })
    }

    pub async fn findone(&self, collection: &str, query: Document, projection: Option<Document>) -> Result<Document> {
        // Log which collection this is going into
        log::debug!("Searching {}.{}", self.db, collection);

        let project = match projection {
            Some(project) => Some(project),
            None => Some(doc! {"_id": 0})
        };

        let find_one_options = FindOneOptions::builder()
            .sort(doc! { "_id": -1 })
            .projection(project)
            .build();

        let collection = self.client.database(&self.db).collection(collection);

        match collection.find_one(query, find_one_options).await {
            Ok(result) => match result {
                Some(doc) => {
                    log::debug!("Found a result");
                    Ok(doc)
                }
                None => {
                    log::debug!("No results found");
                    Ok(doc! { "msg": "no results found" })
                }
            },
            Err(e) => {
                log::error!("Error searching mongodb: {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn find(&self, collection: &str, query: Document, projection: Option<Document>) -> Result<Vec<Document>> {
        // Log which collection this is going into
        log::debug!("Searching {}.{}", self.db, collection);

        let project = match projection {
            Some(project) => Some(project),
            None => Some(doc! {"_id": 0})
        };

        let find_options = FindOptions::builder()
            .sort(doc! { "_id": -1 })
            .projection(project)
            .limit(100)
            .build();

        let collection = self.client.database(&self.db).collection(collection);
        let mut cursor = collection.find(query, find_options).await?;

        let mut result: Vec<Document> = Vec::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(converted) => result.push(converted),
                Err(e) => {
                    log::error!("Caught error, skipping: {}", e);
                    continue;
                }
            }
        }
        let result = result.into_iter().rev().collect();
        Ok(result)
    }

    pub async fn insert(&self, opts: ArgMatches<'_>, collection: &str, mut mongodoc: Document) -> Result<String> {
        match opts.is_present("readonly") {
            true => {
                log::error!("Rejecting post, as we are in readonly mode");
                return Err(MyError::ReadOnly)
            }
            _ => {
                // Log which collection this is going into
                log::debug!("Inserting doc into {}.{}", self.db, collection);
            }
        };

        let now = Utc::now();
        mongodoc.insert("_time", now);
        let collection = self.client.database(&self.db).collection(collection);
        match collection.insert_one(mongodoc, None).await {
            Ok(id) => Ok(id.inserted_id.to_string()),
            Err(e) => {
                log::error!("Error inserting into mongodb: {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn insert_many(&self, opts: ArgMatches<'_>, collection: &str, mut mongodocs: Vec<Document>) -> Result<HashMap<usize, Bson>> {
        match opts.is_present("readonly") {
            true => {
                log::error!("Rejecting post, as we are in readonly mode");
                return Err(MyError::ReadOnly)
            }
            _ => {
                // Log which collection this is going into
                log::debug!("Inserting doc into {}.{}", self.db, collection);
            }
        };

        let now = Utc::now();
        for mongodoc in mongodocs.iter_mut() {
            mongodoc.insert("_time", now);
        };

        let collection = self.client.database(&self.db).collection(collection);
        match collection.insert_many(mongodocs, None).await {
            Ok(id) => Ok(id.inserted_ids),
            Err(e) => {
                log::error!("Error inserting into mongodb: {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn update_one(&self, opts: ArgMatches<'_>, collection: &str, mongodocs: Vec<Document>) -> Result<String> {
        match opts.is_present("readonly") {
            true => {
                log::error!("Rejecting post, as we are in readonly mode");
                return Err(MyError::ReadOnly)
            }
            _ => {
                // Log which collection this is going into
                log::debug!("Inserting doc into {}.{}", self.db, collection);
            }
        };

        let now = Utc::now();

        let filter = mongodocs[0].clone();
        let mut mongodoc = mongodocs[1].clone();
        mongodoc.insert("_time", now);

        let update_options = UpdateOptions::builder()
            .upsert(true)
            .build();

        let collection = self.client.database(&self.db).collection(collection);
        match collection.update_one(filter, mongodoc, update_options).await {
            Ok(result) => {
                match result.upserted_id {
                    Some(_) => Ok("Created new doc".to_owned()),
                    None => Ok("Updated existing doc".to_owned())
                }
            },
            Err(e) => {
                log::error!("Error inserting into mongodb: {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn aggregate(&self, collection: &str, pipeline: Vec<Document>) -> Result<Vec<Document>> {
        let collection = self.client.database(&self.db).collection(collection);
        let mut cursor = collection.aggregate(pipeline, None).await?;

        let mut result: Vec<Document> = Vec::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(converted) => result.push(converted),
                Err(e) => {
                    log::error!("Caught error, skipping: {}", e);
                    continue;
                }
            }
        }
        let result = result.into_iter().rev().collect();
        Ok(result)
    }
    pub async fn collections(&self) -> Result<Vec<String>> {
        // Log that we are trying to list collections
        log::debug!("Getting collections in {}", self.db);

        match self
            .client
            .database(&self.db)
            .list_collection_names(None)
            .await
        {
            Ok(collections) => {
                log::debug!("Success listing collections in {}", self.db);
                Ok(collections)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn count(&self, collection: &str) -> Result<Document> {
        // Log that we are trying to list collections
        log::debug!("Getting document count in {}", self.db);

        let collection = self.client.database(&self.db).collection(collection);

        match collection.estimated_document_count(None).await {
            Ok(count) => {
                log::debug!("Successfully counted docs in {}", self.db);
                let result = doc! {"docs" : count};
                Ok(result)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn get_indexes(&self, collection: &str) -> Result<Document> {
        // Log that we are trying to list collections
        log::debug!("Getting indexes in {}", self.db);

        let database = self.client.database(&self.db);
        let command = doc! { "listIndexes": collection };

        match database.run_command(command, None).await {
            Ok(indexes) => {
                log::debug!("Successfully got indexes in {}.{}", self.db, collection);
                let results = indexes.get_document("cursor").expect("Successfully got indexes, but failed to extract cursor").clone();
                Ok(results)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn rs_status(&self) -> Result<Document> {
        // Log that we are trying to list collections
        log::debug!("Getting replSetGetStatus");

        let database = self.client.database("admin");
        let command = doc! { "replSetGetStatus": 1};

        match database.run_command(command, None).await {
            Ok(output) => {
                log::debug!("Successfully got replSetGetStatus");
                Ok(output)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn get_log(&self) -> Result<Vec<Bson>> {
        // Log that we are trying to list collections
        log::debug!("Getting getLog");

        let database = self.client.database("admin");
        let command = doc! { "getLog": "global"};

        match database.run_command(command, None).await {
            Ok(output) => {
                let results = output.get_array("log").expect("Failed to get log field").clone();
                log::debug!("Successfully got getLog");
                Ok(results)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn server_status(&self) -> Result<Document> {
        // Log that we are trying to list collections
        log::debug!("Getting serverStatus");

        let database = self.client.database("admin");
        let command = doc! { "serverStatus": 1};

        match database.run_command(command, None).await {
            Ok(output) => {
                log::debug!("Successfully got serverStatus");
                Ok(output)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn inprog(&self) -> Result<Vec<Bson>> {
        log::debug!("Getting inprog");

        let database = self.client.database("admin");
        let command = doc! { "currentOp": 1};

        match database.run_command(command, None).await {
            Ok(output) => {
                log::debug!("Successfully got inprog");
                let results = output.get_array("inprog").expect("Failed to get log field").clone();
                Ok(results)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn top(&self) -> Result<Document> {
        log::debug!("Getting top");

        let database = self.client.database("admin");
        let command = doc! { "top": 1};

        match database.run_command(command, None).await {
            Ok(output) => {
                log::debug!("Successfully got top");
                let results = output.get_document("totals").expect("Failed to get log field").clone();
                Ok(results)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn index_stats(&self, collection: &str) -> Result<Vec<Document>> {
        log::debug!("Getting index stats");

        let mut commands = Vec::new();

        let command = doc! { "$indexStats": {}};
        commands.push(command);

        match self.aggregate(collection, commands).await {
            Ok(output) => {
                log::debug!("Successfully got IndexStats");
                Ok(output)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }
}
