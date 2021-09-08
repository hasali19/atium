use async_trait::async_trait;

use crate::{Handler, Next, Request};

pub struct State<T>(pub T);

#[async_trait]
impl<T: Clone + Send + Sync + 'static> Handler for State<T> {
    async fn run(&self, mut req: Request, next: &dyn Next) -> Request {
        req.set_ext(self.0.clone());
        next.run(req).await
    }
}
