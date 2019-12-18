use hex::FromHex;
use hyper::{
    client::connect::{Connected, Connection},
    service::Service,
    Body, Uri,
};
use std::{
    future::Future,
    io,
    path::{Path, PathBuf},
    pin::Pin,
};
use tokio::net::UnixStream;
use super::InnerTransport;

pub struct Transport {
    path: PathBuf,
    client: hyper::Client<UnixConnector, hyper::Body>,
}

impl Transport {
    pub fn new(path: PathBuf) -> Self {
        let unix_connector = UnixConnector;
        let client: hyper::Client<UnixConnector, Body> =
            hyper::Client::builder().build(unix_connector);
        Self { path, client }
    }
}

impl InnerTransport for Transport {
    fn uri(&self, endpoint: impl AsRef<str>) -> String {
        format!("unix://{}:0{}", self.path.to_string_lossy(), endpoint.as_ref())
    }
    fn send_request_inner(
        &self,
        req: hyper::Request<hyper::Body>,
    ) -> hyper::client::ResponseFuture {
        self.client.request(req)
    }
}

#[derive(Clone, Copy)]
pub struct UnixConnector;

impl Unpin for UnixConnector {}

impl Service<Uri> for UnixConnector {
    type Response = UnixStream;
    type Error = std::io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + Sync>>;
    fn call(
        &mut self,
        req: Uri,
    ) -> Self::Future {
        let fut = async move {
            let path = parse_socket_path(req)?;
            UnixStream::connect(path).await
        };

        Box::pin(fut)
    }
}

impl Connection for UnixStream {
    fn connected(&self) -> Connected {
        Connected::new()
    }
}

fn unix_uri(
    socket: impl AsRef<Path>,
    path: &str,
) -> Uri {
    let host = hex::encode(socket.as_ref().to_string_lossy().as_bytes());
    let host_str = format!("unix://{}:0{}", host, path);
    host_str.parse().unwrap()
}

fn parse_socket_path(uri: Uri) -> Result<std::path::PathBuf, io::Error> {
    if uri.scheme_str() != Some("unix") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid URL, scheme must be unix",
        ));
    }

    if let Some(host) = uri.host() {
        let bytes = Vec::from_hex(host).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid URL, host must be a hex-encoded path",
            )
        })?;

        Ok(PathBuf::from(String::from_utf8_lossy(&bytes).into_owned()))
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid URL, host must be present",
        ))
    }
}
