use hyper::{Body, Method, Uri};

use crate::Response;

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

    pub fn ext<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.inner.extensions().get()
    }

    pub fn set_ext<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.inner.extensions_mut().insert(val)
    }

    pub fn take_ext<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.inner.extensions_mut().remove()
    }

    pub fn res(&self) -> Option<&Response> {
        self.res.as_ref()
    }

    pub fn res_or_default_mut(&mut self) -> &mut Response {
        self.res.get_or_insert_with(Response::default)
    }

    pub fn set_res(&mut self, res: impl Into<Response>) -> &mut Response {
        self.res.insert(res.into())
    }

    pub fn take_res(&mut self) -> Option<Response> {
        self.res.take()
    }
}
