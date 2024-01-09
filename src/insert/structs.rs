use bson::Bson;
use core::time::Duration;
use mongodb::options::{Acknowledgment, InsertManyOptions, InsertOneOptions, WriteConcern};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomInsertManyOptions {
    pub bypass_document_validation: Option<bool>,
    pub ordered: Option<bool>,
    pub w: Option<Acknowledgment>,
    pub n: Option<u32>,
    pub w_timeout: Option<Duration>,
    pub journal: Option<bool>,
    pub comment: Option<Bson>,
    pub inject_time_field: Option<String>
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct CustomInsertOneOptions {
    pub bypass_document_validation: Option<bool>,
    pub w: Option<Acknowledgment>,
    pub n: Option<u32>,
    pub w_timeout: Option<Duration>,
    pub journal: Option<bool>,
    pub comment: Option<Bson>,
    pub inject_time_field: Option<String>
}

impl From<CustomInsertManyOptions> for InsertManyOptions {
    fn from(item: CustomInsertManyOptions) -> Self {
        let w_concern = if let Some(w) = item.w {
            Some(w.into())
        } else if let Some(n) = item.n {
            Some(n.into())
        } else {
            None
        };

        let write_concern = WriteConcern::builder()
            .w(w_concern)
            .w_timeout(item.w_timeout)
            .journal(item.journal)
            .build();

        InsertManyOptions::builder()
            .bypass_document_validation(item.bypass_document_validation)
            .ordered(item.ordered)
            .write_concern(write_concern)
            .comment(item.comment)
            .build()
    }
}

impl From<CustomInsertOneOptions> for InsertOneOptions {
    fn from(item: CustomInsertOneOptions) -> Self {
        let w_concern = if let Some(w) = item.w {
            Some(w.into())
        } else if let Some(n) = item.n {
            Some(n.into())
        } else {
            None
        };

        let write_concern = WriteConcern::builder()
            .w(w_concern)
            .w_timeout(item.w_timeout)
            .journal(item.journal)
            .build();

        InsertOneOptions::builder()
            .bypass_document_validation(item.bypass_document_validation)
            .write_concern(write_concern)
            .comment(item.comment)
            .build()
    }
}
