use super::Transport;
use hyperlocal::{UnixClientExt, UnixConnector};
use std::path::PathBuf;

pub struct Uds {
    path: PathBuf,
    client: hyper::Client<UnixConnector, hyper::Body>,
}

impl Uds {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let client = hyper::Client::unix();
        Self { path, client }
    }
}

impl Transport for Uds {
    fn uri(
        &self,
        endpoint: &str,
    ) -> String {
        format!("unix://{}:0{}", self.path.to_string_lossy(), endpoint)
    }
    fn send_request(
        &self,
        req: hyper::Request<hyper::Body>,
    ) -> hyper::client::ResponseFuture {
        self.client.request(req)
    }
}
