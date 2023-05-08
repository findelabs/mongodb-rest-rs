use bson::{doc, Document};
use mongodb::options::{
    Collation, CollationAlternate, CollationCaseFirst, CollationMaxVariable, CollationStrength,
    TextIndexVersion,
};
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Index {
    pub keys: Document,
    pub options: Option<IndexCreateOptions>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IndexCreateOptions {
    pub unique: Option<bool>,
    pub name: Option<String>,
    pub partial_filter_expression: Option<Document>,
    pub sparse: Option<bool>,
    pub expire_after: Option<u64>,
    pub hidden: Option<bool>,
    pub collation: Option<Collation>,
    pub weights: Option<Document>,
    pub default_language: Option<String>,
    pub language_override: Option<String>,
    pub text_index_version: Option<TextIndexVersion>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IndexCollation {
    pub locale: Option<String>,
    pub case_level: Option<bool>,
    pub case_first: Option<CollationCaseFirst>,
    pub strength: Option<CollationStrength>,
    pub numeric_ordering: Option<bool>,
    pub alternate: Option<CollationAlternate>,
    pub max_variable: Option<CollationMaxVariable>,
    pub backwards: Option<bool>,
}
