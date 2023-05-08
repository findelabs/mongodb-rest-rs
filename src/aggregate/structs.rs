use bson::{Document, doc};
use serde::{Serialize, Deserialize};

use mongodb::options::{
    AggregateOptions
};

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
