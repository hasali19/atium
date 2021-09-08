use async_trait::async_trait;

use crate::{Handler, Next, Request};

pub struct State<T>(pub T);

#[async_trait]
impl<T: Clone + Send + Sync + 'static> Handler for State<T> {
    async fn run(&self, req: Request, next: &dyn Next) -> Request {
        next.run(req.with_ext(self.0.clone())).await
    }
}
