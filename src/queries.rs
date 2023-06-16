use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Formats {
    #[serde(rename = "json")]
    Json,
    #[serde(rename = "ejson")]
    Ejson,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExplainVerbosity {
    verbosity: String,
}

impl Default for Formats {
    fn default() -> Self {
        Formats::Json
    }
}

impl Default for QueriesFormat {
    fn default() -> Self {
        QueriesFormat {
            format: Some(Formats::default()),
        }
    }
}

impl Default for ExplainFormat {
    fn default() -> Self {
        ExplainFormat {
            format: Some(Formats::default()),
            verbosity: Some(String::from("allPlansExecution")),
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct ExplainFormat {
    pub format: Option<Formats>,
    pub verbosity: Option<String>,
}

#[derive(Clone, Deserialize, Debug)]
pub struct QueriesFormat {
    pub format: Option<Formats>,
}

#[derive(Deserialize, Debug)]
pub struct QueriesDelete {
    pub name: String,
}
