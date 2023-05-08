use bson::Document;
use mongodb::options::{UpdateModifications, UpdateOptions};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Update {
    pub filter: Document,
    pub update: UpdateModifications,
    pub options: Option<UpdateOptions>,
}
