use std::marker::PhantomData;

use async_trait::async_trait;

use crate::handler::Next;
use crate::{Handler, Request};

#[derive(Default)]
pub struct Logger(PhantomData<()>);

#[async_trait]
impl Handler for Logger {
    async fn run(&self, req: Request, next: &dyn Next) -> Request {
        let req = next.run(req).await;
        let status = req.res().map(|res| res.status());

        match status {
            None => log::error!("{} {} -> no response", req.method(), req.uri()),
            Some(status) => {
                if status.is_client_error() {
                    log::warn!("{} {} -> {}", req.method(), req.uri(), status);
                } else if status.is_server_error() {
                    log::error!("{} {} -> {}", req.method(), req.uri(), status);
                } else {
                    log::info!("{} {} -> {}", req.method(), req.uri(), status);
                }
            }
        }

        req
    }
}
