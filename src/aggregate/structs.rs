use bson::{doc, Document};
use serde::{Deserialize, Serialize};

use mongodb::options::AggregateOptions;

#[derive(Deserialize, Debug, Clone)]
pub struct Aggregate {
    pub pipeline: Vec<Document>,
    pub options: Option<AggregateOptions>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AggregateRaw {
    pub aggregate: String,
    pub pipeline: Vec<Document>,
    pub cursor: Document,
}
