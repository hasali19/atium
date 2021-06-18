mod request;
mod response;

pub mod error;
pub mod handler;
pub mod logger;
pub mod server;

pub use error::RequestError;
pub use handler::{Handler, Next};
pub use request::Request;
pub use response::Response;
pub use server::run;

pub use hyper::Body;

pub type Result = std::result::Result<Request, RequestError>;
