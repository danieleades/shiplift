use hyper::{client::HttpConnector, Body, Client};
use super::InnerTransport;

pub struct Transport {
    host: String,
    client: Client<HttpConnector, Body>,
}

impl Transport {
    pub fn new(host: String) -> Self {
        let client = Client::new();
        Self { host, client }
    }
}

impl InnerTransport for Transport {
    fn uri(&self, endpoint: impl AsRef<str>) -> String {
        format!("{}{}", self.host, endpoint.as_ref())
    }
    fn send_request_inner(
        &self,
        req: hyper::Request<hyper::Body>,
    ) -> hyper::client::ResponseFuture {self.client.request(req)}
    
}