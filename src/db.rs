use crate::error::Error as RestError;
use core::time::Duration;
use axum::body::Bytes;
use axum::body::StreamBody;
use axum::extract::Query;
use bson::Bson;
use futures::stream::StreamExt;
use futures::Stream;
use mongodb::bson::{doc, document::Document, to_bson, to_document};
use mongodb::options::{Collation, IndexOptions};
use mongodb::IndexModel;
use mongodb::{
    options::ClientOptions, options::InsertManyOptions, options::InsertOneOptions,
    options::ListDatabasesOptions, Client,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::handlers::{
    Aggregate, CustomInsertManyOptions, CustomInsertOneOptions, DeleteOne, Distinct, ExplainFormat,
    Find, FindOne, Formats, Index, QueriesDelete, QueriesFormat, UpdateOne, Watch,
};

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collation: Option<Collation>,
}

#[derive(Clone, Debug)]
pub struct DB {
    pub client: Client,
    pub readonly: bool,
}

type Result<T> = std::result::Result<T, RestError>;

impl DB {
    pub async fn init(mut client_options: ClientOptions, readonly: bool) -> Result<Self> {
        client_options.app_name = Some("mongodb-rest-rs".to_string());

        Ok(Self {
            client: Client::with_options(client_options)?,
            readonly,
        })
    }

    pub async fn index_delete(
        &self,
        database: &str,
        collection: &str,
        queries: &QueriesDelete,
    ) -> Result<Value> {
        log::debug!(
            "Deleting index {} on {}.{}",
            queries.name,
            database,
            collection
        );

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection.drop_index(queries.name.to_string(), None).await {
            Ok(_) => {
                log::debug!("Deleted index");
                Ok(json!({"message":"deleted index", "name": queries.name}))
            }
            Err(e) => {
                log::error!("Error deleting index: {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn index_create(
        &self,
        database: &str,
        collection: &str,
        payload: Index,
    ) -> Result<Value> {
        log::debug!("Creating index on {}.{}", database, collection);

        let mut index_options = IndexOptions::builder().build();
        if let Some(options) = payload.options {
            index_options.unique = options.unique;
            index_options.name = options.name;
            index_options.partial_filter_expression = options.partial_filter_expression;
            index_options.sparse = options.sparse;
            index_options.expire_after = options.expire_after.map(|e| Duration::from_secs(e));
            index_options.hidden = options.hidden;
            index_options.collation = options.collation;
            index_options.weights = options.weights;
            index_options.default_language = options.default_language;
            index_options.language_override = options.language_override;
            index_options.text_index_version = options.text_index_version;
        }

        let index_model = IndexModel::builder()
            .keys(payload.keys)
            .options(Some(index_options))
            .build();

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection.create_index(index_model, None).await {
            Ok(doc) => {
                log::debug!("Created index");
                Ok(json!({"message":"Created index", "name": doc.index_name}))
            }
            Err(e) => {
                log::error!("Error creating index: {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn aggregate_explain(
        &self,
        database: &str,
        collection: &str,
        payload: Aggregate,
        queries: Query<ExplainFormat>,
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
            verbosity: queries
                .verbosity
                .as_ref()
                .unwrap_or(&"allPlansExecution".to_string())
                .clone(),
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
        queries: Query<ExplainFormat>,
    ) -> Result<Value> {
        // Log which collection this is accessing
        log::debug!("Explaining search in {}.{}", database, collection);

        let find_raw = FindRaw {
            find: collection.to_string(),
            filter: payload.filter.clone(),
            sort: payload.options.clone().map_or(None, |x| x.sort.clone()),
            projection: payload
                .options
                .clone()
                .map_or(None, |x| x.projection.clone()),
            limit: payload.options.clone().map_or(None, |x| x.limit.clone()),
            skip: payload.options.clone().map_or(None, |x| x.skip.clone()),
            collation: payload
                .options
                .clone()
                .map_or(None, |x| x.collation.clone()),
        };

        let command = Explain {
            explain: to_document(&find_raw)?,
            verbosity: queries
                .verbosity
                .as_ref()
                .unwrap_or(&"allPlansExecution".to_string())
                .clone(),
            comment: "mongodb-rest-rs explain".to_string(),
        };

        let db = self.client.database(database);

        match db.run_command(to_document(&command)?, None).await {
            Ok(mut c) => {
                log::debug!("Successfully ran explain in {}.{}", database, collection);
                c.remove("$clusterTime");
                c.remove("operationTime");
                let bson = to_bson(&c)?.into_relaxed_extjson();
                Ok(bson)
            }
            Err(e) => {
                log::error!("Got error {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn watch(
        &self,
        database: &str,
        collection: &str,
        payload: Watch,
        queries: Query<QueriesFormat>,
    ) -> Result<StreamBody<impl Stream<Item = Result<Bytes>>>> {
        let collection = self
            .client
            .database(&database)
            .collection::<Document>(collection);

        let cursor = collection.watch(payload.pipeline, payload.options).await?;

        // This was the simplest way to get this to work
        // Trying to map the items to Bytes did not work, and would cause the connection to drop
        // However, get'ing a single field from the ChangeStream doc would work, only if the var was to_owned()
        // However, I couldn't get the full document to persist

        Ok(StreamBody::new(cursor.map(move |d| match d {
            Ok(o) => {
                let bson = match queries.clone().format {
                    None | Some(Formats::Json) => to_bson(&o)?.into_relaxed_extjson(),
                    Some(Formats::Ejson) => to_bson(&o)?.into_canonical_extjson(),
                };
                log::debug!("Change stream event: {:?}", bson);
                let string = format!("{}\n", bson);
                Ok(string.into())
            }
            Err(e) => {
                log::error!("Error in change stream: {}", e);
                Ok(format!("{{\"error\":\"{}\"}}", e.to_string().replace('"', "\\\"")).into())
            }
        })))
    }

    pub async fn aggregate(
        &self,
        database: &str,
        collection: &str,
        payload: Aggregate,
        queries: Query<QueriesFormat>,
    ) -> Result<StreamBody<impl Stream<Item = Result<Bytes>>>> {
        let collection = self
            .client
            .database(&database)
            .collection::<Document>(collection);

        let cursor = collection
            .aggregate(payload.pipeline, payload.options)
            .await?;

        let stream = cursor.map(move |d| match d {
            Ok(o) => {
                let bson = match queries.clone().format {
                    None | Some(Formats::Json) => to_bson(&o)?.into_relaxed_extjson(),
                    Some(Formats::Ejson) => to_bson(&o)?.into_canonical_extjson(),
                };
                log::debug!("Found doc: {:?}", bson);
                let string = format!("{}\n", bson);
                Ok(string.into())
            }
            Err(e) => Err(e)?,
        });

        Ok(StreamBody::new(stream))
    }

    pub async fn find(
        &self,
        database: &str,
        collection: &str,
        payload: Find,
        queries: Query<QueriesFormat>,
    ) -> Result<StreamBody<impl Stream<Item = Result<Bytes>>>> {
        // Log which collection this is going into
        log::debug!("Searching {}.{}", database, collection);

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        let cursor = collection.find(payload.filter, payload.options).await?;

        let stream = cursor.map(move |d| match d {
            Ok(o) => {
                let bson = match queries.clone().format {
                    None | Some(Formats::Json) => to_bson(&o)?.into_relaxed_extjson(),
                    Some(Formats::Ejson) => to_bson(&o)?.into_canonical_extjson(),
                };
                log::debug!("Found doc: {:?}", bson);
                let string = format!("{}\n", bson);
                Ok(string.into())
            }
            Err(e) => Err(e)?,
        });

        Ok(StreamBody::new(stream))
    }

    pub async fn insert_many(
        &self,
        database: &str,
        collection: &str,
        body: Vec<Bson>,
        queries: Query<CustomInsertManyOptions>,
    ) -> Result<Value> {
        if self.readonly {
            return Err(RestError::ReadOnly);
        }

        log::debug!("Inserting many to {}.{}", database, collection);

        let collection = self
            .client
            .database(database)
            .collection::<Bson>(collection);

        let options: InsertManyOptions = queries.0.into();

        match collection.insert_many(body, options).await {
            Ok(id) => {
                log::debug!("Successfully inserted doc");
                let response = json!({"Inserted": id.inserted_ids});
                Ok(response)
            }
            Err(e) => {
                log::error!("Error inserting into mongo: {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn insert_one(
        &self,
        database: &str,
        collection: &str,
        body: Bson,
        queries: Query<CustomInsertOneOptions>,
    ) -> Result<Value> {
        if self.readonly {
            return Err(RestError::ReadOnly);
        }

        log::debug!("Inserting into {}.{}", database, collection);

        let collection = self
            .client
            .database(database)
            .collection::<Bson>(collection);

        let options: InsertOneOptions = queries.0.into();

        match collection.insert_one(body, options).await {
            Ok(id) => {
                log::debug!("Successfully inserted doc");
                let response = json!({"Inserted": id.inserted_id});
                Ok(response)
            }
            Err(e) => {
                log::error!("Error inserting into mongo: {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn delete_many(
        &self,
        database: &str,
        collection: &str,
        payload: DeleteOne,
    ) -> Result<Value> {
        if self.readonly {
            return Err(RestError::ReadOnly);
        }

        log::debug!("Deleting many from {}.{}", database, collection);

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection
            .delete_many(payload.filter, payload.options)
            .await
        {
            Ok(result) => {
                log::debug!("Successfully deleted docs");
                let response = json!({"Deleted": result.deleted_count});
                Ok(response)
            }
            Err(e) => {
                log::error!("Error deleting from mongo: {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn delete_one(
        &self,
        database: &str,
        collection: &str,
        payload: DeleteOne,
    ) -> Result<Value> {
        if self.readonly {
            return Err(RestError::ReadOnly);
        }

        log::debug!("Deleting one from {}.{}", database, collection);

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection.delete_one(payload.filter, payload.options).await {
            Ok(result) => {
                log::debug!("Successfully deleted doc");
                let response = json!({"Deleted": result.deleted_count});
                Ok(response)
            }
            Err(e) => {
                log::error!("Error deleting from mongo: {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn update_one(
        &self,
        database: &str,
        collection: &str,
        payload: UpdateOne,
    ) -> Result<Value> {
        if self.readonly {
            return Err(RestError::ReadOnly);
        }

        log::debug!("Updating one from {}.{}", database, collection);

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection
            .update_one(payload.filter, payload.update, payload.options)
            .await
        {
            Ok(result) => {
                log::debug!("Successfully updated doc");
                let response = json!(result);
                Ok(response)
            }
            Err(e) => {
                log::error!("Error updating in mongo: {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn update_many(
        &self,
        database: &str,
        collection: &str,
        payload: UpdateOne,
    ) -> Result<Value> {
        if self.readonly {
            return Err(RestError::ReadOnly);
        }

        log::debug!("Updating many from {}.{}", database, collection);

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection
            .update_many(payload.filter, payload.update, payload.options)
            .await
        {
            Ok(result) => {
                log::debug!("Successfully updated docs");
                let response = json!(result);
                Ok(response)
            }
            Err(e) => {
                log::error!("Error updating in mongo: {}", e);
                return Err(e)?;
            }
        }
    }

    pub async fn distinct(
        &self,
        database: &str,
        collection: &str,
        payload: Distinct,
        queries: &QueriesFormat,
    ) -> Result<Value> {
        log::debug!(
            "Searching for distinct values in {}.{}",
            database,
            collection
        );

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection
            .distinct(payload.field_name, payload.filter, payload.options)
            .await
        {
            Ok(doc) => {
                log::debug!("Found a result");
                let bson = match &queries.format {
                    None | Some(Formats::Json) => to_bson(&doc)?.into_relaxed_extjson(),
                    Some(Formats::Ejson) => to_bson(&doc)?.into_canonical_extjson(),
                };
                Ok(bson)
            }
            Err(e) => {
                log::error!("Error searching mongodb: {}", e);
                return Err(e)?;
            }
        }
    }
    pub async fn find_one(
        &self,
        database: &str,
        collection: &str,
        payload: FindOne,
        queries: &QueriesFormat,
    ) -> Result<Value> {
        log::debug!("Searching {}.{}", database, collection);

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        match collection.find_one(payload.filter, payload.options).await {
            Ok(result) => match result {
                Some(doc) => {
                    log::debug!("Found a result");
                    let bson = match &queries.format {
                        None | Some(Formats::Json) => to_bson(&doc)?.into_relaxed_extjson(),
                        Some(Formats::Ejson) => to_bson(&doc)?.into_canonical_extjson(),
                    };
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

    pub async fn coll_indexes(
        &self,
        database: &str,
        collection: &str,
        queries: &QueriesFormat,
    ) -> Result<Value> {
        log::debug!("Getting indexes in {}", database);

        let collection = self
            .client
            .database(database)
            .collection::<Document>(collection);

        let mut cursor = collection.list_indexes(None).await?;

        let mut result: Vec<Value> = Vec::new();
        while let Some(next) = cursor.next().await {
            match next {
                Ok(doc) => {
                    let bson = match &queries.format {
                        None | Some(Formats::Json) => to_bson(&doc)?.into_relaxed_extjson(),
                        Some(Formats::Ejson) => to_bson(&doc)?.into_canonical_extjson(),
                    };
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

    pub async fn run_command<T: Serialize>(&self, db: &str, payload: T, makes_changes: bool) -> Result<Value> {
        if makes_changes && self.readonly {
            return Err(RestError::ReadOnly);
        }
        log::debug!("Running command against database");

        let database = self.client.database(&db);

        match database.run_command(to_document(&payload)?, None).await {
            Ok(mut output) => {
                log::debug!("Successfully ran command against database");
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

    pub async fn coll_index_stats(
        &self,
        database: &str,
        collection: &str,
    ) -> Result<StreamBody<impl Stream<Item = Result<Bytes>>>> {
        log::debug!("Getting index stats");

        let mut commands = Vec::new();
        let command = doc! { "$indexStats": {}};
        commands.push(command);

        let payload = Aggregate {
            pipeline: commands,
            options: None,
        };

        let queries = QueriesFormat::default();

        match self
            .aggregate(database, collection, payload, axum::extract::Query(queries))
            .await
        {
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
