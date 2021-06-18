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

impl From<StatusCode> for Response {
    fn from(val: StatusCode) -> Self {
        Response::new().with_status(val)
    }
}

impl From<&'static str> for Response {
    fn from(val: &'static str) -> Self {
        Response::new().with_body(val)
    }
}

impl From<String> for Response {
    fn from(val: String) -> Self {
        Response::new().with_body(val)
    }
}

impl From<Vec<u8>> for Response {
    fn from(val: Vec<u8>) -> Self {
        Response::new().with_body(val)
    }
}

impl From<Body> for Response {
    fn from(val: Body) -> Self {
        Response::new().with_body(val)
    }
}

impl<B: Into<Body>> From<(StatusCode, B)> for Response {
    fn from((status, body): (StatusCode, B)) -> Self {
        Response::new().with_status(status).with_body(body)
    }
}
