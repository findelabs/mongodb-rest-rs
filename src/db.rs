use crate::error::Error as RestError;
use bson::Bson;
use futures::StreamExt;
use mongodb::bson::{doc, document::Document, to_bson, to_document};
use mongodb::options::{FindOneOptions, FindOptions};
use mongodb::{options::ClientOptions, options::ListDatabasesOptions, Client};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::handlers::{Aggregate, Find, FindOne};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Explain {
    pub explain: Document,
    pub verbosity: String,
    pub comment: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregateRaw {
    pub aggregate: String,
    pub pipeline: Vec<Document>,
    pub cursor: Document,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FindRaw {
    pub find: String,
    pub filter: Document,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection: Option<Document>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
}

type Result<T> = std::result::Result<T, RestError>;

impl DB {
    pub async fn init(mut client_options: ClientOptions) -> Result<Self> {
        client_options.app_name = Some("mongodb-rest-rs".to_string());

        Ok(Self {
            client: Client::with_options(client_options)?,
        })
    }

    pub async fn aggregate_explain(
        &self,
        database: &str,
        collection: &str,
        payload: Aggregate,
    ) -> Result<Vec<Value>> {
        // Log which collection this is going into
        log::debug!("Explaining aggregate in {}.{}", database, collection);

        let aggregate_raw = AggregateRaw {
            aggregate: collection.to_string(),
            pipeline: payload.pipeline,
            cursor: doc! {},
        };

        let command = Explain {
            explain: to_document(&aggregate_raw)?,
            verbosity: payload.explain.unwrap(),
            comment: "mongodb-rest-rs explain".to_string(),
        };

        let db = self.client.database(database);

        match db.run_command(to_document(&command)?, None).await {
            Ok(mut c) => {
                log::debug!("Successfully ran explain in {}.{}", database, collection);
                c.remove("$clusterTime");
                c.remove("operationTime");
                let bson = to_bson(&c)?.into_relaxed_extjson();
                Ok(vec![bson])
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn find_explain(
        &self,
        database: &str,
        collection: &str,
        payload: Find,
    ) -> Result<Vec<Value>> {
        // Log which collection this is going into
        log::debug!("Explaining search in {}.{}", database, collection);

        let find_raw = FindRaw {
            find: collection.to_string(),
            filter: payload.filter.clone(),
            sort: payload.sort.clone(),
            projection: payload.projection.clone(),
            limit: payload.limit.clone(),
            skip: payload.skip.clone(),
        };

        let command = Explain {
            explain: to_document(&find_raw)?,
            verbosity: payload.explain.unwrap(),
            comment: "mongodb-rest-rs explain".to_string(),
        };

        let db = self.client.database(database);

        match db.run_command(to_document(&command)?, None).await {
            Ok(mut c) => {
                log::debug!("Successfully ran explain in {}.{}", database, collection);
                c.remove("$clusterTime");
                c.remove("operationTime");
                let bson = to_bson(&c)?.into_relaxed_extjson();
                Ok(vec![bson])
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn aggregate(
        &self,
        database: &str,
        collection: &str,
        payload: Aggregate,
    ) -> Result<Vec<Value>> {
        if payload.explain.is_some() {
            return self.aggregate_explain(database, collection, payload).await;
        }

        let collection = self
            .client
            .database(&database)
            .collection::<Document>(collection);

        let mut cursor = collection.aggregate(payload.pipeline, None).await?;

        let mut result: Vec<Value> = Vec::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(conv) => {
                    let bson = to_bson(&conv)?.into_relaxed_extjson();
                    result.push(bson);
                }
                Err(e) => {
                    log::error!("Caught error, skipping: {}", e);
                    continue;
                }
            }
        }
        let result = result.into_iter().rev().collect();
        Ok(result)
    }

    pub async fn find(
        &self,
        database: &str,
        collection: &str,
        payload: Find,
    ) -> Result<Vec<Value>> {
        if payload.explain.is_some() {
            return self.find_explain(database, collection, payload).await;
        }

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

        let mut result: Vec<Value> = Vec::new();
        while let Some(doc) = cursor.next().await {
            match doc {
                Ok(conv) => {
                    let bson = to_bson(&conv)?.into_relaxed_extjson();
                    result.push(bson);
                }
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
    ) -> Result<Value> {
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
                    let bson = to_bson(&doc)?.into_relaxed_extjson();
                    Ok(bson)
                }
                None => {
                    log::debug!("No results found");
                    Ok(json!({ "msg": "no results found" }))
                }
            },
            Err(e) => {
                log::error!("Error searching mongodb: {}", e);
                return Err(e)?;
            }
        }
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

    pub async fn coll_count(&self, database: &str, collection: &str) -> Result<Value> {
        log::debug!("Getting document count in {}", database);

        let collection = self
            .client
            .database(&database)
            .collection::<Document>(collection);

        match collection.estimated_document_count(None).await {
            Ok(count) => {
                log::debug!("Successfully counted docs in {}", database);
                let result = json!({ "docs": count });
                Ok(result)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn coll_indexes(&self, database: &str, collection: &str) -> Result<Value> {
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
                let bson = to_bson(&results)?.into_relaxed_extjson();
                Ok(bson)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn rs_status(&self) -> Result<Value> {
        log::debug!("Getting replSetGetStatus");

        let database = self.client.database("admin");
        let command = doc! { "replSetGetStatus": 1};

        match database.run_command(command, None).await {
            Ok(mut output) => {
                log::debug!("Successfully got replSetGetStatus");
                output.remove("$clusterTime");
                output.remove("operationTime");
                let bson = to_bson(&output)?.into_relaxed_extjson();
                Ok(bson)
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

    pub async fn rs_stats(&self) -> Result<Value> {
        log::debug!("Getting serverStatus");

        let database = self.client.database("admin");
        let command = doc! { "serverStatus": 1};

        match database.run_command(command, None).await {
            Ok(mut output) => {
                log::debug!("Successfully got serverStatus");
                output.remove("$clusterTime");
                output.remove("operationTime");
                let bson = to_bson(&output)?.into_relaxed_extjson();
                Ok(bson)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn rs_pool(&self) -> Result<Value> {
        log::debug!("Getting connPoolStats");

        let database = self.client.database("admin");
        let command = doc! { "connPoolStats": 1};

        match database.run_command(command, None).await {
            Ok(mut output) => {
                log::debug!("Successfully got connPoolStats");
                output.remove("$clusterTime");
                output.remove("operationTime");
                let bson = to_bson(&output)?.into_relaxed_extjson();
                Ok(bson)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn rs_conn(&self) -> Result<Value> {
        log::debug!("Getting connectionStatus");

        let database = self.client.database("admin");
        let command = doc! { "connectionStatus": 1};

        match database.run_command(command, None).await {
            Ok(mut output) => {
                log::debug!("Successfully got connectionStatus");
                output.remove("$clusterTime");
                output.remove("operationTime");
                let bson = to_bson(&output)?.into_relaxed_extjson();
                Ok(bson)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn coll_stats(&self, database: &str, collection: &str) -> Result<Value> {
        log::debug!("Getting collStats");

        let database = self.client.database(database);
        let command = doc! { "collStats": collection};

        match database.run_command(command, None).await {
            Ok(mut output) => {
                log::debug!("Successfully got collStats");
                output.remove("$clusterTime");
                output.remove("operationTime");
                let bson = to_bson(&output)?.into_relaxed_extjson();
                Ok(bson)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn db_stats(&self, database: &str) -> Result<Value> {
        log::debug!("Getting dbStats");

        let database = self.client.database(database);
        let command = doc! { "dbStats": 1};

        match database.run_command(command, None).await {
            Ok(mut output) => {
                log::debug!("Successfully got dbStats");
                output.remove("$clusterTime");
                output.remove("operationTime");
                let bson = to_bson(&output)?.into_relaxed_extjson();
                Ok(bson)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn rs_operations(&self) -> Result<Vec<Value>> {
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
                let output = results
                    .iter()
                    .map(|x| {
                        let mut doc = to_document(x).expect("Malformed operation doc");
                        doc.remove("$clusterTime");
                        doc.remove("operationTime");
                        let bson = to_bson(&x)
                            .expect("Malformed bson operation doc")
                            .into_relaxed_extjson();
                        bson
                    })
                    .collect();
                Ok(output)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn rs_top(&self) -> Result<Value> {
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
                let bson = to_bson(&results)?.into_relaxed_extjson();
                Ok(bson)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn coll_index_stats(&self, database: &str, collection: &str) -> Result<Vec<Value>> {
        log::debug!("Getting index stats");

        let mut commands = Vec::new();
        let command = doc! { "$indexStats": {}};
        commands.push(command);

        let payload = Aggregate {
            pipeline: commands,
            explain: None,
        };

        match self.aggregate(database, collection, payload).await {
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
