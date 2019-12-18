
use crate::Result;
use futures_util::stream::StreamExt;

pub trait ResponseExt {
    fn json<T>(self) -> JsonFuture<T> where T: serde::de::DeserializeOwned;
}

impl ResponseExt for hyper::Response<hyper::Body> {
    fn json<T>(self) -> JsonFuture<T> where T: serde::de::DeserializeOwned {
        parse_json(self)
    }
}

pub type JsonFuture<T: serde::de::DeserializeOwned> = impl std::future::Future<Output = Result<T>> + Sized;

async fn parse_json<T: serde::de::DeserializeOwned>(response: hyper::Response<hyper::Body>) -> Result<T> {
    let mut v = Vec::default();

    let body = response.into_body();

    while let Some(bytes_result) = body.next().await {
        let bytes = bytes_result?;
        v.extend(&bytes)
    }

    Ok(serde_json::from_slice(&v)?)
}