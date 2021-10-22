use std::convert::Infallible;
use std::io::SeekFrom;
use std::ops::Bound;
use std::path::PathBuf;

use headers::{AcceptRanges, ContentLength, ContentRange, ContentType, Range};
use hyper::{Body, StatusCode};
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncSeekExt};
use tokio_util::codec::{BytesCodec, FramedRead};

use crate::respond::RespondRequestExt;
use crate::{async_trait, Request, Response};

#[async_trait]
pub trait Responder: Send {
    async fn respond_to(self, req: &mut Request);
}

// TODO: This is useful for endpoints that call set_res (or equivalent) on req
// directly so don't need to return anything. Not sure if it's a good idea though -
// might remove it at some point.
#[async_trait]
impl Responder for () {
    async fn respond_to(self, _: &mut Request) {}
}

macro_rules! impl_responder_for_into_response {
    ($t:ty) => {
        #[async_trait]
        impl Responder for $t {
            async fn respond_to(self, req: &mut Request) {
                req.set_res(self);
            }
        }
    };
}

impl_responder_for_into_response!(StatusCode);
impl_responder_for_into_response!(&'static str);
impl_responder_for_into_response!(String);
impl_responder_for_into_response!(Vec<u8>);
impl_responder_for_into_response!(Body);

#[async_trait]
impl<B: Into<Body> + Send> Responder for (StatusCode, B) {
    async fn respond_to(self, req: &mut Request) {
        req.set_res(self);
    }
}

#[async_trait]
impl Responder for Infallible {
    async fn respond_to(self, _: &mut Request) {
        unreachable!()
    }
}

#[async_trait]
impl<T: Responder, E: Responder> Responder for Result<T, E> {
    async fn respond_to(self, req: &mut Request) {
        match self {
            Ok(res) => res.respond_to(req),
            Err(e) => e.respond_to(req),
        };
    }
}

#[cfg(feature = "eyre")]
#[async_trait]
impl Responder for eyre::Error {
    async fn respond_to(self, req: &mut Request) {
        req.set_ext(self);
    }
}

pub struct Json<T>(pub T);

#[async_trait]
impl<T: Serialize + Send> Responder for Json<T> {
    async fn respond_to(self, req: &mut Request) {
        if let Err(e) = req.ok().json(&self.0) {
            req.set_res((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    }
}

pub struct File {
    inner: tokio::fs::File,
    mime: mime::Mime,
}

impl File {
    pub async fn open(path: impl Into<PathBuf>) -> std::io::Result<File> {
        let path = path.into();
        let mime = mime_guess::from_path(&path).first_or("video/*".parse().unwrap());
        let file = tokio::fs::File::open(path).await?;
        Ok(File { inner: file, mime })
    }
}

#[async_trait]
impl Responder for File {
    async fn respond_to(self, req: &mut Request) {
        async fn respond_file(req: &mut Request, file: File) -> std::io::Result<Response> {
            let File {
                inner: mut file,
                mime,
            } = file;

            let total_length = file.metadata().await?.len();
            let range = req.header::<Range>().and_then(|range| range.iter().next());

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

            Ok(res)
        }

        if let Err(e) = respond_file(req, self).await {
            req.set_res((StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));
        }
    }
}
