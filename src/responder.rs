use std::convert::Infallible;

use hyper::{Body, StatusCode};

use crate::Request;

pub trait Responder {
    fn respond_to(self, req: &mut Request);
}

// TODO: This is useful for endpoints that call set_res (or equivalent) on req
// directly so don't need to return anything. Not sure if it's a good idea though -
// might remove it at some point.
impl Responder for () {
    fn respond_to(self, _: &mut Request) {}
}

macro_rules! impl_responder_for_into_response {
    ($t:ty) => {
        impl Responder for $t {
            fn respond_to(self, req: &mut Request) {
                req.set_res(self);
            }
        }
    };
}

impl_responder_for_into_response!(StatusCode);
impl_responder_for_into_response!(&'static str);
impl_responder_for_into_response!(String);
impl_responder_for_into_response!(Vec<u8>);
impl_responder_for_into_response!(Body);

impl<B: Into<Body>> Responder for (StatusCode, B) {
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

#[cfg(feature = "eyre")]
impl Responder for eyre::Error {
    fn respond_to(self, req: &mut Request) {
        req.set_ext(self);
    }
}

