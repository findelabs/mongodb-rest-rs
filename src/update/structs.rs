use bson::{Document};
use serde::{Deserialize};
use mongodb::options::{
    UpdateOptions, UpdateModifications
};

#[derive(Deserialize, Debug, Clone)]
pub struct Update {
    pub filter: Document,
    pub update: UpdateModifications,
    pub options: Option<UpdateOptions>,
}
