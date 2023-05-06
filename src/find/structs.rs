use bson::{Document, doc};
use serde::{Deserialize};

use mongodb::options::{
    FindOneOptions, FindOptions, DistinctOptions
};

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
