use std::convert::Infallible;

use async_trait::async_trait;
use atium::logger::Logger;
use atium::respond::RespondRequestExt;
use atium::responder::Responder;
use atium::router::{Router, RouterRequestExt};
use atium::{endpoint, Handler, Next, Request, Response, StatusCode};
use env_logger::Env;

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

    let router = Router::new().with(|r| {
        r.route("/").get(index);
        r.route("/error").get(error);
        r.route("/hello/:name").get(hello);
    });

    let addr = ([127, 0, 0, 1], 8080);
    let handler = atium::compose!(Logger::default(), ErrorHandler, router, fallback);

    atium::run(addr, handler).await.unwrap();
}

impl Responder for MyError {
    fn respond_to(self, req: &mut Request) {
        req.set_ext(self);
    }
}

#[endpoint]
async fn index(_: &mut Request) -> Result<impl Responder, MyError> {
    Ok("hello, world!")
}

#[endpoint]
async fn hello(req: &mut Request) -> Result<impl Responder, MyError> {
    let name = req.param_str("name").expect("missing parameter: name");
    let message = format!("hello, {}!", name);
    req.respond(message);
    Ok(())
}

#[endpoint]
async fn error(_: &mut Request) -> Result<Infallible, MyError> {
    Err(MyError)
}

#[endpoint]
async fn fallback(_: &mut Request) -> impl Responder {
    "this is the fallback route"
}
