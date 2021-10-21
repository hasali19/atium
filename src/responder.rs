use std::convert::Infallible;

use crate::{Request, Response};

pub trait Responder {
    fn respond_to(self, req: &mut Request);
}

// TODO: This is useful for endpoints that call set_res (or equivalent) on req
// directly so don't need to return anything. Not sure if it's a good idea though -
// might remove it at some point.
impl Responder for () {
    fn respond_to(self, _: &mut Request) {}
}

impl<T: Into<Response>> Responder for T {
    fn respond_to(self, req: &mut Request) {
        req.set_res(self);
    }
}

impl Responder for Infallible {
    fn respond_to(self, _: &mut Request) {
        unreachable!()
    }
}

impl<T: Responder, E: Responder> Responder for Result<T, E> {
    fn respond_to(self, req: &mut Request) {
        match self {
            Ok(res) => res.respond_to(req),
            Err(e) => e.respond_to(req),
        }
    }
}
