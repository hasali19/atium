use serde::de::DeserializeOwned;

use crate::Request;

pub type QueryError = serde_qs::Error;

pub trait QueryRequestExt {
    fn query<T: DeserializeOwned>(&self) -> Result<T, QueryError>;
}

impl QueryRequestExt for Request {
    fn query<T: DeserializeOwned>(&self) -> Result<T, QueryError> {
        serde_qs::from_str(self.uri().query().unwrap_or(""))
    }
}
