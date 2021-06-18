use std::marker::PhantomData;

use async_trait::async_trait;
use hyper::StatusCode;

use crate::handler::Next;
use crate::{Handler, Request, Result};

#[derive(Default)]
pub struct Logger(PhantomData<()>);

#[async_trait]
impl Handler for Logger {
    async fn run(&self, req: Request, next: &dyn Next) -> Result {
        let req = next.run(req).await?;
        let status = req
            .res()
            .map(|res| res.status())
            .unwrap_or(StatusCode::NOT_FOUND);

        if status.is_success() {
            log::info!("{} {} -> {}", req.method(), req.uri(), status);
        } else {
            log::error!("{} {} -> {}", req.method(), req.uri(), status);
        }

        Ok(req)
    }
}
