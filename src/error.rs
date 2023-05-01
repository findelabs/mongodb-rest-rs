//use serde_json::error::Error as SerdeError;
use axum::{
    body::{self},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    //    Forbidden,
    //    Unauthorized,
    ReadOnly,
    Mongo(mongodb::error::Error),
    Bson(bson::document::ValueAccessError),
    DeError(bson::de::Error),
    SerError(bson::ser::Error),
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ReadOnly => f.write_str("{\"error\": \"Readonly cluster\"}"),
            Error::Mongo(ref err) => write!(
                f,
                "{{\"error\": \"{}\"}}",
                err.to_string().replace('"', "\\\"")
            ),
            Error::Bson(ref err) => write!(
                f,
                "{{\"error\": \"{}\"}}",
                err.to_string().replace('"', "\\\"")
            ),
            Error::DeError(ref err) => write!(
                f,
                "{{\"error\": \"{}\"}}",
                err.to_string().replace('"', "\\\"")
            ),
            Error::SerError(ref err) => write!(
                f,
                "{{\"error\": \"{}\"}}",
                err.to_string().replace('"', "\\\"")
            ),
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let payload = self.to_string();
        let body = body::boxed(body::Full::from(payload));

        let status_code = match self {
            Error::ReadOnly => StatusCode::FORBIDDEN,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        Response::builder().status(status_code).body(body).unwrap()
    }
}

impl From<bson::document::ValueAccessError> for Error {
    fn from(err: bson::document::ValueAccessError) -> Error {
        Error::Bson(err)
    }
}

impl From<bson::de::Error> for Error {
    fn from(err: bson::de::Error) -> Error {
        Error::DeError(err)
    }
}

impl From<bson::ser::Error> for Error {
    fn from(err: bson::ser::Error) -> Error {
        Error::SerError(err)
    }
}

impl From<mongodb::error::Error> for Error {
    fn from(err: mongodb::error::Error) -> Error {
        Error::Mongo(err)
    }
}
