use bson::{Document, doc};
use serde::{Serialize, Deserialize};

use mongodb::options::{
    FindOneOptions, FindOptions, DistinctOptions, Collation
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Explain {
    pub explain: Document,
    pub verbosity: String,
    pub comment: String,
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

#[derive(Deserialize, Debug, Clone)]
pub struct FindOne {
    pub filter: Document,
    pub options: Option<FindOneOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Find {
    pub filter: Document,
    pub options: Option<FindOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Distinct {
    pub field_name: String,
    pub filter: Option<Document>,
    pub options: Option<DistinctOptions>,
}
