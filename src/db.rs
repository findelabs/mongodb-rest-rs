use crate::error::Error as RestError;
use bson::Bson;
use futures::StreamExt;
use mongodb::bson::{doc, document::Document};
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{options::ClientOptions, options::ListDatabasesOptions, Client};

use crate::handlers::{Find, FindOne};

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
}

type Result<T> = std::result::Result<T, RestError>;

impl DB {
    pub async fn init(url: &str) -> Result<Self> {
        let mut client_options = ClientOptions::parse(url).await?;
        client_options.app_name = Some("json-bucket".to_string());

        Ok(Self {
            client: Client::with_options(client_options)?,
        })
    }

    pub async fn find(
        &self,
        database: &str,
        collection: &str,
        payload: Find,
    ) -> Result<Vec<Document>> {
        // Log which collection this is going into
        log::debug!("Searching {}.{}", database, collection);

        let mut find_options = FindOptions::builder().build();

        find_options.projection = payload.projection;
        find_options.sort = payload.sort;
        find_options.limit = payload.limit;
        find_options.skip = payload.skip;

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);
        let mut cursor = collection.find(payload.filter, find_options).await?;

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

    pub async fn find_one(
        &self,
        database: &str,
        collection: &str,
        payload: FindOne,
    ) -> Result<Document> {
        log::debug!("Searching {}.{}", database, collection);

        let find_one_options = match payload.projection {
            Some(p) => Some(FindOneOptions::builder().projection(p).build()),
            None => None,
        };

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection.find_one(payload.filter, find_one_options).await {
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
                return Err(e)?;
            }
        }
    }

    pub async fn aggregate(
        &self,
        database: &str,
        collection: &str,
        pipeline: Vec<Document>,
    ) -> Result<Vec<Document>> {
        let collection = self
            .client
            .database(&database)
            .collection::<Document>(collection);
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
            Err(e) => return Err(e)?,
        }
    }

    pub async fn coll_count(&self, database: &str, collection: &str) -> Result<Document> {
        log::debug!("Getting document count in {}", database);

        let collection = self
            .client
            .database(&database)
            .collection::<Document>(collection);

        match collection.estimated_document_count(None).await {
            Ok(count) => {
                log::debug!("Successfully counted docs in {}", database);
                let result = doc! {"docs" : count.to_string()};
                Ok(result)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn coll_indexes(&self, database: &str, collection: &str) -> Result<Document> {
        log::debug!("Getting indexes in {}", database);

        let db = self.client.database(&database);
        let command = doc! { "listIndexes": collection };

        match db.run_command(command, None).await {
            Ok(indexes) => {
                log::debug!("Successfully got indexes in {}.{}", database, collection);
                let results = indexes
                    .get_document("cursor")
                    .expect("Successfully got indexes, but failed to extract cursor")
                    .clone();
                Ok(results)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn rs_status(&self) -> Result<Document> {
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
                return Err(e)?;
            }
        }
    }

    pub async fn rs_log(&self) -> Result<Vec<Bson>> {
        log::debug!("Getting getLog");

        let database = self.client.database("admin");
        let command = doc! { "getLog": "global"};

        match database.run_command(command, None).await {
            Ok(output) => {
                let results = output.get_array("log")?.to_vec();
                log::debug!("Successfully got getLog");
                Ok(results)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn rs_stats(&self) -> Result<Document> {
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
                return Err(e)?;
            }
        }
    }

    pub async fn rs_operations(&self) -> Result<Vec<Bson>> {
        log::debug!("Getting inprog");

        let database = self.client.database("admin");
        let command = doc! { "currentOp": 1};

        match database.run_command(command, None).await {
            Ok(output) => {
                log::debug!("Successfully got inprog");
                let results = output
                    .get_array("inprog")
                    .expect("Failed to get log field")
                    .clone();
                Ok(results)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn rs_top(&self) -> Result<Document> {
        log::debug!("Getting top");

        let database = self.client.database("admin");
        let command = doc! { "top": 1};

        match database.run_command(command, None).await {
            Ok(output) => {
                log::debug!("Successfully got top");
                let results = output
                    .get_document("totals")
                    .expect("Failed to get log field")
                    .clone();
                Ok(results)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn coll_index_stats(
        &self,
        database: &str,
        collection: &str,
    ) -> Result<Vec<Document>> {
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
                return Err(e)?;
            }
        }
    }

    pub async fn databases(&self) -> Result<Vec<String>> {
        log::debug!("Getting databases");
        let options = ListDatabasesOptions::builder()
            .authorized_databases(Some(false))
            .build();

        match self.client.list_database_names(None, options).await {
            Ok(output) => {
                log::debug!("Successfully got databases");
                Ok(output)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }
}
