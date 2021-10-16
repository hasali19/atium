use std::io::SeekFrom;
use std::ops::Bound;
use std::path::Path;

use async_trait::async_trait;
use headers::{AcceptRanges, ContentLength, ContentRange, ContentType, Header, Range};
use hyper::{Body, StatusCode};
use serde::Serialize;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::{Request, Response};

#[async_trait]
pub trait RespondRequestExt {
    fn ok(&mut self) -> Respond;
    fn respond<R: Into<Response>>(&mut self, res: R) -> Respond;

    async fn respond_file(
        &mut self,
        path: impl AsRef<Path> + Send + 'async_trait,
    ) -> std::io::Result<Respond>;
}

#[async_trait]
impl RespondRequestExt for Request {
    fn ok(&mut self) -> Respond {
        self.respond(StatusCode::OK)
    }

    fn respond<R: Into<Response>>(&mut self, res: R) -> Respond {
        Respond(self.set_res(res.into()))
    }

    async fn respond_file(
        &mut self,
        path: impl AsRef<Path> + Send + 'async_trait,
    ) -> std::io::Result<Respond> {
        let mime = mime_guess::from_path(&path).first_or("video/*".parse().unwrap());
        let mut file = File::open(path).await?;

        let total_length = file.metadata().await?.len();
        let range = self.header::<Range>().and_then(|range| range.iter().next());

        let mut res = Response::ok()
            .with_header(AcceptRanges::bytes())
            .with_header(ContentType::from(mime));

        match range {
            Some((from, to)) => {
                let from = match from {
                    Bound::Included(n) => n,
                    Bound::Excluded(n) => n + 1,
                    Bound::Unbounded => 0,
                };

                let to = match to {
                    Bound::Included(n) => n,
                    Bound::Excluded(n) => n - 1,
                    Bound::Unbounded => total_length - 1,
                };

                file.seek(SeekFrom::Start(from)).await?;

                let read_length = u64::min(total_length - from, to - from + 1);
                let reader = file.take(read_length);
                let stream = FramedRead::new(reader, BytesCodec::new());
                let body = Body::wrap_stream(stream);
                let range = from..=from + read_length - 1;

                res.set_status(StatusCode::PARTIAL_CONTENT);
                res.set_header(ContentRange::bytes(range, total_length).unwrap());
                res.set_header(ContentLength(read_length));
                res.set_body(body);
            }
            None => {
                let stream = FramedRead::new(file, BytesCodec::new());
                let body = Body::wrap_stream(stream);

                res.set_header(ContentLength(total_length));
                res.set_body(body);
            }
        }

        Ok(self.respond(res))
    }
}

pub struct Respond<'a>(&'a mut Response);

impl<'a> Respond<'a> {
    pub fn status(self, status: StatusCode) -> Self {
        self.0.set_status(status);
        self
    }

    pub fn header(self, header: impl Header) -> Self {
        self.0.set_header(header);
        self
    }

    pub fn body(self, body: impl Into<Body>) -> Self {
        self.0.set_body(body);
        self
    }

    pub fn json<T: Serialize>(self, val: &T) -> serde_json::Result<Self> {
        Ok(self
            .header(ContentType::json())
            .body(serde_json::to_vec(val)?))
    }
}
