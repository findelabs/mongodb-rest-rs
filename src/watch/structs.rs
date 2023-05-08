use bson::Document;
use mongodb::options::ChangeStreamOptions;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Watch {
    pub pipeline: Vec<Document>,
    pub options: Option<ChangeStreamOptions>,
}
