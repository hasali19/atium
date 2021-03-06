use std::future::Future;

use async_trait::async_trait;

use crate::Request;

#[async_trait]
pub trait Next: Sync {
    async fn run(&self, req: Request) -> Request;
}

pub struct NextFn<F>(pub F);

#[async_trait]
impl<F, Fut> Next for NextFn<F>
where
    F: Send + Sync,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = Request> + Send,
{
    async fn run(&self, req: Request) -> Request {
        self.0(req).await
    }
}

#[async_trait]
pub trait Handler: Send + Sync + 'static {
    async fn run(&self, req: Request, next: &dyn Next) -> Request;

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

#[async_trait]
impl<A: Handler, B: Handler> Handler for (A, B) {
    async fn run(&self, req: Request, next: &dyn Next) -> Request {
        let (a, b) = self;
        a.run(req, &NextFn(|req| b.run(req, next))).await
    }
}

#[async_trait]
impl Handler for Box<dyn Handler> {
    async fn run(&self, req: Request, next: &dyn Next) -> Request {
        self.as_ref().run(req, next).await
    }

    fn name(&self) -> &str {
        self.as_ref().name()
    }
}

#[async_trait]
impl<H: Handler> Handler for Vec<H> {
    async fn run(&self, req: Request, next: &dyn Next) -> Request {
        struct NextImpl<'a, H> {
            rest: &'a [H],
            next: &'a dyn Next,
        }

        #[async_trait]
        impl<H: Handler> Next for NextImpl<'_, H> {
            async fn run(&self, req: Request) -> Request {
                run(self.rest, req, self.next).await
            }
        }

        async fn run<H: Handler>(slice: &[H], req: Request, next: &dyn Next) -> Request {
            match slice.split_first() {
                Some((v, rest)) => v.run(req, &NextImpl { rest, next }).await,
                None => next.run(req).await,
            }
        }

        run(self, req, next).await
    }
}

#[async_trait]
impl<F, Fut> Handler for F
where
    F: Send + Sync + 'static + Fn(Request) -> Fut,
    Fut: Future<Output = Request> + Send,
{
    async fn run(&self, req: Request, _: &dyn Next) -> Request {
        (self)(req).await
    }
}

#[macro_export]
macro_rules! compose {
    ($e:expr) => {
        $e
    };

    ($e:expr, $($es:expr),+) => {
        ($e, $crate::compose!($($es),+))
    }
}
