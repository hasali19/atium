use hyper::{Body, StatusCode};

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

    pub fn body(self, body: impl Into<Body>) -> Self {
        self.0.set_body(body);
        self
    }
}
