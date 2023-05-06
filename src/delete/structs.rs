use bson::Document;
use serde::{Deserialize};
use mongodb::options::{
    DeleteOptions
};

#[derive(Deserialize, Debug, Clone)]
pub struct DeleteOne {
    pub filter: Document,
    pub options: Option<DeleteOptions>,
}

