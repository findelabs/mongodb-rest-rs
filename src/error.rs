use serde_json::error::Error as SerdeError;
use std::fmt;
use std::str::Utf8Error;

#[derive(Debug)]
pub enum MyError {
    JsonError,
    HyperError,
    UtfError,
    MongodbError,
    UrlParseError,
    BsonError,
//    ReadOnly,
    MissingDatabase,
    MissingCollection,
    MissingOperation
}

impl std::error::Error for MyError {}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MyError::JsonError => f.write_str("Error converting data to json"),
            MyError::HyperError => f.write_str("Hyper Error"),
            MyError::UtfError => f.write_str("Utf conversion Error"),
            MyError::MongodbError => f.write_str("MongoDB Error"),
            MyError::UrlParseError => f.write_str("Failed to parse url Error"),
            MyError::BsonError => f.write_str("Could not parse as bson doc"),
//            MyError::ReadOnly=> f.write_str("Running in read only mode"),
            MyError::MissingDatabase=> f.write_str("Missing database in uri"),
            MyError::MissingCollection=> f.write_str("Missing collection in uri"),
            MyError::MissingOperation=> f.write_str("Missing operation in uri"),
        }
    }
}

impl From<SerdeError> for MyError {
    fn from(e: SerdeError) -> Self {
        match e {
            _ => MyError::JsonError,
        }
    }
}

impl From<hyper::Error> for MyError {
    fn from(e: hyper::Error) -> Self {
        match e {
            _ => MyError::HyperError,
        }
    }
}

impl From<Utf8Error> for MyError {
    fn from(e: Utf8Error) -> Self {
        match e {
            _ => MyError::UtfError,
        }
    }
}

impl From<mongodb::error::Error> for MyError {
    fn from(e: mongodb::error::Error) -> Self {
        match e {
            _ => MyError::MongodbError,
        }
    }
}

impl From<url::ParseError> for MyError {
    fn from(e: url::ParseError) -> Self {
        match e {
            _ => MyError::UrlParseError,
        }
    }
}

impl From<bson::ser::Error> for MyError {
    fn from(e: bson::ser::Error) -> Self {
        match e {
            _ => MyError::BsonError,
        }
    }
}
