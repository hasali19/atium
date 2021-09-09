use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use hyper::Method;
use routefinder::Captures;

use crate::{Handler, Request};

#[derive(Default)]
pub struct Router {
    method_map: HashMap<Method, routefinder::Router<Arc<dyn Handler>>>,
}

struct MatchedPath(usize);

#[async_trait]
impl Handler for Router {
    async fn run(&self, mut req: crate::Request, next: &dyn crate::Next) -> Request {
        // If this is a nested router, we should skip the part of the path that has
        // already been matched.
        let offset = match req.take_ext::<MatchedPath>() {
            Some(MatchedPath(n)) => n,
            None => 0,
        };

        let path = req.uri().path();
        let m = self
            .method_map
            .get(req.method())
            .and_then(|r| r.best_match(&path[offset..]));

        let (handler, params) = match m {
            Some(val) => (val.handler(), val.captures()),
            None => return next.run(req).await,
        };

        // Calculate how much of the path has been matched.
        // If this is a wildcard route, calculate the length of the matched part using
        // some simple pointer arithmetic.
        // Otherwise it's just the length of the path, since the whole thing was matched.
        let start = match params.wildcard() {
            Some(wildcard) => offset + wildcard.as_ptr() as usize - path.as_ptr() as usize,
            None => path.len(),
        };

        let params = params.into_owned();

        req.set_ext(MatchedPath(start));
        req.set_ext(params);

        handler.run(req, next).await
    }
}

impl Router {
    pub fn new() -> Self {
        Router {
            method_map: Default::default(),
        }
    }

    pub fn with(mut self, builder: impl Fn(&mut Router)) -> Self {
        builder(&mut self);
        self
    }

    pub fn route<'a, 'b>(&'a mut self, path: &'b str) -> Route<'a, 'b> {
        Route(self, path)
    }
}

macro_rules! method_fn {
    ($name:ident, $method:ident) => {
        pub fn $name(self, handler: impl Handler) -> Self {
            self.0
                .method_map
                .entry(Method::$method)
                .or_insert_with(Default::default)
                .add(self.1, Arc::new(handler))
                .expect("invalid path");
            self
        }
    };
}

pub struct Route<'a, 'b>(&'a mut Router, &'b str);

impl<'a, 'b> Route<'a, 'b> {
    method_fn!(connect, CONNECT);
    method_fn!(delete, DELETE);
    method_fn!(get, GET);
    method_fn!(head, HEAD);
    method_fn!(options, OPTIONS);
    method_fn!(patch, PATCH);
    method_fn!(post, POST);
    method_fn!(put, PUT);
    method_fn!(trace, TRACE);

    pub fn any(self, handler: impl Handler) -> Self {
        let handler = Arc::new(handler);
        let methods = [
            Method::CONNECT,
            Method::DELETE,
            Method::GET,
            Method::HEAD,
            Method::OPTIONS,
            Method::PATCH,
            Method::POST,
            Method::PUT,
            Method::TRACE,
        ];

        for method in methods {
            self.0
                .method_map
                .entry(method.clone())
                .or_insert_with(Default::default)
                .add(self.1, handler.clone())
                .expect("invalid path");
        }

        self
    }
}

pub trait RouterRequestExt {
    fn param(&self, name: &str) -> Option<&str>;
}

impl RouterRequestExt for Request {
    fn param(&self, name: &str) -> Option<&str> {
        self.ext::<Captures>().and_then(|params| params.get(name))
    }
}
