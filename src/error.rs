use crate::Request;

#[derive(Debug)]
pub struct RequestError {
    inner: anyhow::Error,
    req: Request,
}

impl std::error::Error for RequestError {}

impl std::fmt::Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl RequestError {
    pub fn new(req: Request, e: impl Into<anyhow::Error>) -> Self {
        RequestError {
            inner: e.into(),
            req,
        }
    }

    pub fn error(&self) -> &anyhow::Error {
        &self.inner
    }

    pub fn req(&self) -> &Request {
        &self.req
    }

    pub fn into_parts(self) -> (anyhow::Error, Request) {
        (self.inner, self.req)
    }
}
