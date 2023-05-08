use bson::Document;
use mongodb::options::DeleteOptions;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct DeleteOne {
    pub filter: Document,
    pub options: Option<DeleteOptions>,
}
