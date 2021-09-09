use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::Arc;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Server};

use crate::handler::NextFn;
use crate::{Handler, Request};

pub use hyper::Error;

#[derive(Debug)]
struct NoResponse;

impl Display for NoResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "no response")
    }
}

impl std::error::Error for NoResponse {}

async fn service(
    req: Request,
    handler: Arc<impl Handler>,
) -> std::result::Result<hyper::Response<Body>, NoResponse> {
    let mut req = handler
        .run(req, &NextFn(|req: Request| async move { req }))
        .await;

    let res = match req.take_res() {
        Some(res) => res,
        None => return Err(NoResponse),
    };

    Ok(res.into_inner())
}

pub async fn run(addr: impl Into<SocketAddr>, handler: impl Handler) -> Result<(), Error> {
    let addr = addr.into();
    let handler = Arc::new(handler);

    let make_svc = make_service_fn(|_| {
        let handler = handler.clone();
        async {
            Ok::<_, NoResponse>(service_fn(move |req| {
                service(Request::new(req), handler.clone())
            }))
        }
    });

    log::info!("running server at http://{}", addr);

    Server::bind(&addr).serve(make_svc).await
}
