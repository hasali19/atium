use async_trait::async_trait;
use dawn::logger::Logger;
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

async fn hello_world(req: Request) -> dawn::Result {
    if req.uri().path() != "/" {
        return Err(req.into_error(anyhow::anyhow!("this is an error")));
    }

    Ok(req.with_res(Response::new().with_body("Hello, world!")))
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let addr = ([127, 0, 0, 1], 8080);
    let handler = dawn::compose!(Logger::default(), ErrorHandler, hello_world);

    dawn::run(addr, handler).await.unwrap();
}
