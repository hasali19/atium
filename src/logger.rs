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
                if status.is_success() {
                    log::info!("{} {} -> {}", req.method(), req.uri(), status);
                } else {
                    log::error!("{} {} -> {}", req.method(), req.uri(), status);
                }
            }
        }

        req
    }
}
