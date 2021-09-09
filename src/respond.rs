use std::convert::TryInto;

use hyper::header::{HeaderValue, IntoHeaderName, CONTENT_TYPE};
use hyper::{Body, StatusCode};
use mime::APPLICATION_JSON;
use serde::Serialize;

use crate::{Request, Response};

pub trait RespondRequestExt {
    fn ok(&mut self) -> Respond;
    fn respond(&mut self, res: impl Into<Response>) -> Respond;
}

impl RespondRequestExt for Request {
    fn ok(&mut self) -> Respond {
        Respond(self.res_or_default_mut())
    }

    fn respond(&mut self, res: impl Into<Response>) -> Respond {
        Respond(self.set_res(res.into()))
    }
}

pub struct Respond<'a>(&'a mut Response);

impl<'a> Respond<'a> {
    pub fn status(self, status: StatusCode) -> Self {
        self.0.set_status(status);
        self
    }

    pub fn header<V: TryInto<HeaderValue>>(
        self,
        name: impl IntoHeaderName,
        value: V,
    ) -> Result<Self, V::Error> {
        let value = value.try_into()?;
        self.0.set_header(name, value);
        Ok(self)
    }

    pub fn body(self, body: impl Into<Body>) -> Self {
        self.0.set_body(body);
        self
    }

    pub fn json<T: Serialize>(self, val: &T) -> serde_json::Result<Self> {
        Ok(self
            .header(CONTENT_TYPE, APPLICATION_JSON.essence_str())
            .unwrap()
            .body(serde_json::to_vec(val)?))
    }
}
