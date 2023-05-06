use bson::{Document};
use serde::{Deserialize};
use mongodb::options::{
    ChangeStreamOptions
};

#[derive(Deserialize, Debug, Clone)]
pub struct Watch {
    pub pipeline: Vec<Document>,
    pub options: Option<ChangeStreamOptions>,
}
