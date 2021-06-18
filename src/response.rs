use hyper::{Body, StatusCode};

#[derive(Debug, Default)]
pub struct Response(hyper::Response<Body>);

impl Response {
    pub fn new() -> Self {
        Response::default()
    }

    pub fn ok() -> Self {
        Response::default()
    }

    pub fn status(&self) -> StatusCode {
        self.0.status()
    }

    pub fn set_status(&mut self, status: StatusCode) {
        *self.0.status_mut() = status;
    }

    pub fn with_status(mut self, status: StatusCode) -> Self {
        self.set_status(status);
        self
    }

    pub fn body(&self) -> &Body {
        self.0.body()
    }

    pub fn set_body(&mut self, body: impl Into<Body>) {
        *self.0.body_mut() = body.into();
    }

    pub fn with_body(mut self, body: impl Into<Body>) -> Self {
        self.set_body(body);
        self
    }

    pub(crate) fn into_inner(self) -> hyper::Response<Body> {
        self.0
    }
}
