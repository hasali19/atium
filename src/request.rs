use hyper::{Body, Method, Uri};

use crate::{RequestError, Response};

#[derive(Debug)]
pub struct Request {
    inner: hyper::Request<Body>,
    res: Option<Response>,
}

impl Request {
    pub(crate) fn new(inner: hyper::Request<Body>) -> Self {
        Request { inner, res: None }
    }

    pub fn method(&self) -> &Method {
        self.inner.method()
    }

    pub fn uri(&self) -> &Uri {
        self.inner.uri()
    }

    pub fn path(&self) -> &str {
        self.uri().path()
    }

    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner.extensions().get()
    }

    pub fn set_ext<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.inner.extensions_mut().insert(val)
    }

    pub fn with_ext<T: Send + Sync + 'static>(mut self, val: T) -> Self {
        self.set_ext(val);
        self
    }

    pub fn take_ext<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.inner.extensions_mut().remove()
    }

    pub fn res(&self) -> Option<&Response> {
        self.res.as_ref()
    }

    pub fn set_res(&mut self, res: impl Into<Response>) {
        self.res = Some(res.into());
    }

    pub fn with_res(mut self, res: impl Into<Response>) -> Self {
        self.set_res(res.into());
        self
    }

    pub fn take_res(&mut self) -> Option<Response> {
        self.res.take()
    }

    pub fn into_error(self, e: impl Into<anyhow::Error>) -> RequestError {
        RequestError::new(self, e)
    }
}
