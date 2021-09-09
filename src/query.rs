use serde::de::DeserializeOwned;

use crate::Request;

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("missing query string")]
    NotFound,
    #[error("failed to parse query string: {0}")]
    ParseError(serde_urlencoded::de::Error),
}

pub trait QueryRequestExt {
    fn query<T: DeserializeOwned>(&self) -> Result<T, QueryError>;
}

impl QueryRequestExt for Request {
    fn query<T: DeserializeOwned>(&self) -> Result<T, QueryError> {
        serde_urlencoded::from_str(self.uri().query().ok_or(QueryError::NotFound)?)
            .map_err(QueryError::ParseError)
    }
}
