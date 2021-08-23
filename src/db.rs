//use chrono::prelude::*;
use mongodb::bson::{doc, document::Document};
//use mongodb::{options::ClientOptions, options::FindOptions, Client, Collection};
use crate::error::MyError;
use mongodb::{options::ClientOptions, Client};
//use serde::{Deserialize, Serialize};
use futures::StreamExt;
//use clap::ArgMatches;
//use std::collections::HashMap;
use bson::Bson;

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client
}

type Result<T> = std::result::Result<T, MyError>;

impl DB {
    pub async fn init(url: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(url).await?;
        client_options.app_name = Some("json-bucket".to_string());

        Ok(Self {
            client: Client::with_options(client_options)?
        })
    }

    pub async fn aggregate(&self, database: &str, collection: &str, pipeline: Vec<Document>) -> Result<Vec<Document>> {
        let collection = self.client.database(&database).collection(collection);
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

    pub async fn collections(&self, database: &str) -> Result<Vec<String>> {
        // Log that we are trying to list collections
        log::debug!("Getting collections in {}", database);

        match self
            .client
            .database(&database)
            .list_collection_names(None)
            .await
        {
            Ok(collections) => {
                log::debug!("Success listing collections in {}", database);
                Ok(collections)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn count(&self, database: &str, collection: &str) -> Result<Document> {
        // Log that we are trying to list collections
        log::debug!("Getting document count in {}", database);

        let collection = self.client.database(&database).collection(collection);

        match collection.estimated_document_count(None).await {
            Ok(count) => {
                log::debug!("Successfully counted docs in {}", database);
                let result = doc! {"docs" : count};
                Ok(result)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                Err(MyError::MongodbError)
            }
        }
    }

    pub async fn get_indexes(&self, database: &str, collection: &str) -> Result<Document> {
        // Log that we are trying to list collections
        log::debug!("Getting indexes in {}", database);

        let db = self.client.database(&database);
        let command = doc! { "listIndexes": collection };

        match db.run_command(command, None).await {
            Ok(indexes) => {
                log::debug!("Successfully got indexes in {}.{}", database, collection);
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

    pub async fn index_stats(&self, database: &str, collection: &str) -> Result<Vec<Document>> {
        log::debug!("Getting index stats");

        let mut commands = Vec::new();

        let command = doc! { "$indexStats": {}};
        commands.push(command);

        match self.aggregate(database, collection, commands).await {
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
