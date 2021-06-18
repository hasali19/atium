use anyhow::anyhow;
use async_trait::async_trait;
use dawn::logger::Logger;
use dawn::router::{RequestExt, Router};
use dawn::{Handler, Next, Request, Response};
use env_logger::Env;
use hyper::StatusCode;

struct ErrorHandler;

#[async_trait]
impl Handler for ErrorHandler {
    async fn run(&self, req: Request, next: &dyn Next) -> dawn::Result {
        match next.run(req).await {
            Ok(req) => Ok(req),
            Err(e) => {
                let (e, req) = e.into_parts();

                log::error!("{:#}", e);

                return Ok(req.with_res(
                    Response::new()
                        .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                        .with_body(format!("{:?}", e)),
                ));
            }
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let router = Router::new().build(|r| {
        r.get("/", |req: Request| async move {
            Ok(req.with_res((StatusCode::OK, "hello, world!")))
        });

        r.get("/:name", |req: Request| async move {
            let name = req.param("name").unwrap();
            let res = format!("hello, {}!", name);
            Ok(req.with_res(res))
        });

        r.get("/error", |req: Request| async move {
            Err(req.into_error(anyhow!("this is an error")))
        });
    });

    let addr = ([127, 0, 0, 1], 8080);
    let handler = dawn::compose!(Logger::default(), ErrorHandler, router);

    dawn::run(addr, handler).await.unwrap();
}
