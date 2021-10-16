use serde::de::DeserializeOwned;

use crate::Request;

pub trait QueryRequestExt {
    fn query<T: DeserializeOwned>(&self) -> Result<T, serde_qs::Error>;
}

impl QueryRequestExt for Request {
    fn query<T: DeserializeOwned>(&self) -> Result<T, serde_qs::Error> {
        serde_qs::from_str(self.uri().query().unwrap_or(""))
    }
}
