use std::fmt::Display;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use futures::future::{self, Either};
use futures::FutureExt;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Server};
use tokio::sync::Notify;

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

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("server error: {0}")]
    Server(hyper::Error),
    #[error("server shut down unexpectedly: {0}")]
    UnexpectedShutdown(hyper::Error),
    #[error("server was shut down forcefully")]
    ForcedShutdown,
}

pub async fn run(addr: impl Into<SocketAddr>, handler: impl Handler) -> Result<(), ServerError> {
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

    let graceful_shutdown = Notify::new();

    let server = Server::bind(&addr)
        .serve(make_svc)
        .with_graceful_shutdown(graceful_shutdown.notified());

    let ctrl_c =
        tokio::signal::ctrl_c().map(|r| r.expect("failed to install ctrl+c signal handler"));

    tokio::pin!(server, ctrl_c);

    // Wait for either the server to shutdown on its own (due to an error) or a ctrl+c signal.
    match future::select(server, ctrl_c).await {
        // Server shut down due to error
        Either::Left((res, _)) => Err(ServerError::UnexpectedShutdown(res.unwrap_err())),
        // Ctrl+c was detected
        Either::Right((_, server)) => {
            // Notify server to shutdown
            graceful_shutdown.notify_waiters();

            log::info!("shutting down server ...");

            // Wait until the server actually shuts down, with a timeout
            match tokio::time::timeout(Duration::from_secs(5), server).await {
                Ok(res) => res.map_err(ServerError::Server),
                Err(_) => {
                    // There are probably active long-lived connections so force shutdown by
                    // dropping the server
                    Err(ServerError::ForcedShutdown)
                }
            }
        }
    }
}
