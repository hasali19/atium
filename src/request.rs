use headers::{Header, HeaderMapExt};
use hyper::body::Buf;
use hyper::{Body, HeaderMap, Method, Uri};
use serde::de::DeserializeOwned;

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

    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    pub fn header<H: Header>(&self) -> Option<H> {
        self.inner.headers().typed_get()
    }

    pub async fn body_bytes(&mut self) -> Result<hyper::body::Bytes, hyper::Error> {
        hyper::body::to_bytes(std::mem::take(self.inner.body_mut())).await
    }

    pub async fn body_json<T: DeserializeOwned>(&mut self) -> serde_json::Result<T> {
        let body = hyper::body::aggregate(std::mem::take(self.inner.body_mut()))
            .await
            .unwrap();

        serde_json::from_reader(body.reader())
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

    pub fn res_mut(&mut self) -> &mut Option<Response> {
        &mut self.res
    }

    pub fn set_res(&mut self, res: impl Into<Response>) -> &mut Response {
        self.res.insert(res.into())
    }

    pub fn take_res(&mut self) -> Option<Response> {
        self.res.take()
    }
}
