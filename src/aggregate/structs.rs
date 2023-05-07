use bson::{Document, doc};
use serde::{Deserialize};

use mongodb::options::{
    AggregateOptions
};

#[derive(Deserialize, Debug, Clone)]
pub struct Aggregate {
    pub pipeline: Vec<Document>,
    pub options: Option<AggregateOptions>,
}
