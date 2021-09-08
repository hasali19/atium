use async_trait::async_trait;
use dawn::logger::Logger;
use dawn::respond::RespondRequestExt;
use dawn::router::{Router, RouterRequestExt};
use dawn::{endpoint, Handler, Next, Request, Response};
use env_logger::Env;
use hyper::StatusCode;

struct MyError;

struct ErrorHandler;

#[async_trait]
impl Handler for ErrorHandler {
    async fn run(&self, req: Request, next: &dyn Next) -> Request {
        let mut req = next.run(req).await;
        let error = req.take_ext::<MyError>();

        if error.is_some() {
            log::error!("got an error!");
            req.set_res(
                Response::new()
                    .with_status(StatusCode::INTERNAL_SERVER_ERROR)
                    .with_body("got an error!"),
            );
        }

        req
    }
}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));

    let router = Router::new().build(|r| {
        r.get("/", index);
        r.get("/error", error);
        r.get("/hello/:name", hello);
    });

    let addr = ([127, 0, 0, 1], 8080);
    let handler = dawn::compose!(Logger::default(), ErrorHandler, router, fallback);

    dawn::run(addr, handler).await.unwrap();
}

#[endpoint]
async fn index(req: &mut Request) -> Result<(), MyError> {
    req.ok().body("hello, world!");
    Ok(())
}

#[endpoint]
async fn hello(req: &mut Request) -> Result<(), MyError> {
    let name = req.param("name").expect("missing parameter: name");
    let message = format!("hello, {}!", name);
    req.respond(message);
    Ok(())
}

#[endpoint]
async fn error(_: &mut Request) -> Result<(), MyError> {
    Err(MyError)
}

#[endpoint]
async fn fallback(req: &mut Request) -> Result<(), MyError> {
    req.ok().body("this is the fallback route");
    Ok(())
}
