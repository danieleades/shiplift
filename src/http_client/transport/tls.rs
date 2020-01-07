use super::Transport;
use hyper::{client::HttpConnector, Body, Client};
use hyper_openssl::HttpsConnector;
use openssl::ssl::{SslConnector, SslConnectorBuilder, SslFiletype, SslMethod};
use std::{env, path::Path};

pub struct Tls {
    host: String,
    client: Client<HttpsConnector<HttpConnector>, Body>,
}

impl Tls {
    pub fn new(host: impl Into<String>) -> Self {
        let host = host.into();
        let http_connector = get_http_connector();
        let ssl_connector_builder = get_ssl_connector_builder().unwrap();

        let https_connector =
            HttpsConnector::with_connector(http_connector, ssl_connector_builder).unwrap();

        let client = Client::builder().build(https_connector);
        Self { host, client }
    }
}

impl Transport for Tls {
    fn uri(
        &self,
        endpoint: &str,
    ) -> String {
        format!("{}{}", self.host, endpoint)
    }
    fn send_request(
        &self,
        req: hyper::Request<hyper::Body>,
    ) -> hyper::client::ResponseFuture {
        self.client.request(req)
    }
}

fn get_http_connector() -> HttpConnector {
    let mut http = HttpConnector::new();
    http.enforce_http(false);

    http
}

fn get_ssl_connector_builder() -> Result<SslConnectorBuilder, ()> {
    let certs = env::var("DOCKER_CERT_PATH").map_err(|_| ())?;
    // fixme: don't unwrap before you know what's in the box
    // https://github.com/hyperium/hyper/blob/master/src/net.rs#L427-L428
    let mut ssl_connector_builder = SslConnector::builder(SslMethod::tls()).unwrap();

    ssl_connector_builder.set_cipher_list("DEFAULT").unwrap();

    let cert = &format!("{}/cert.pem", certs);
    let key = &format!("{}/key.pem", certs);

    ssl_connector_builder
        .set_certificate_file(&Path::new(cert), SslFiletype::PEM)
        .unwrap();

    ssl_connector_builder
        .set_private_key_file(&Path::new(key), SslFiletype::PEM)
        .unwrap();

    if env::var("DOCKER_TLS_VERIFY").is_ok() {
        let ca = &format!("{}/ca.pem", certs);
        ssl_connector_builder.set_ca_file(&Path::new(ca)).unwrap();
    }

    Ok(ssl_connector_builder)
}
