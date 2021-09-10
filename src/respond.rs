use headers::{ContentType, Header};
use hyper::{Body, StatusCode};
use serde::Serialize;

use crate::{Request, Response};

pub trait RespondRequestExt {
    fn ok(&mut self) -> Respond;
    fn respond<R: Into<Response>>(&mut self, res: R) -> Respond;
}

impl RespondRequestExt for Request {
    fn ok(&mut self) -> Respond {
        Respond(self.res_or_default_mut())
    }

    fn respond<R: Into<Response>>(&mut self, res: R) -> Respond {
        Respond(self.set_res(res.into()))
    }
}

pub struct Respond<'a>(&'a mut Response);

impl<'a> Respond<'a> {
    pub fn status(self, status: StatusCode) -> Self {
        self.0.set_status(status);
        self
    }

    pub fn header(self, header: impl Header) -> Self {
        self.0.set_header(header);
        self
    }

    pub fn body(self, body: impl Into<Body>) -> Self {
        self.0.set_body(body);
        self
    }

    pub fn json<T: Serialize>(self, val: &T) -> serde_json::Result<Self> {
        Ok(self
            .header(ContentType::json())
            .body(serde_json::to_vec(val)?))
    }
}
