use async_trait::async_trait;

use crate::Handler;

pub struct State<T>(pub T);

#[async_trait]
impl<T: Clone + Send + Sync + 'static> Handler for State<T> {
    async fn run(&self, req: crate::Request, next: &dyn crate::Next) -> crate::Result {
        next.run(req.with_ext(self.0.clone())).await
    }
}
