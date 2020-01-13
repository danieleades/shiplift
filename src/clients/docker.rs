use crate::{
    clients::{Containers, Images, Networks, Volumes},
    http_client::HttpClient,
    rep, Result,
};
use futures_util::stream::Stream;
use hyper::Uri;
use std::{env, path::PathBuf, sync::Arc};

mod requests;
mod types;

/// Entrypoint interface for communicating with docker daemon
#[derive(Clone)]
pub struct Docker {
    http_client: Arc<HttpClient>,
}

impl Default for Docker {
    fn default() -> Self {
        Self::new()
    }
}

// https://docs.docker.com/reference/api/docker_remote_api_v1.17/
impl Docker {
    /// constructs a new Docker instance for a docker host listening at a url specified by an env var `DOCKER_HOST`,
    /// falling back on unix:///var/run/docker.sock
    pub fn new() -> Docker {
        match env::var("DOCKER_HOST").ok() {
            Some(host) => {
                let host = host.parse().expect("invalid url");
                Self::host(host)
            }
            #[cfg(feature = "unix-socket")]
            None => Self::unix(PathBuf::from("/var/run/docker.sock")),
            #[cfg(not(feature = "unix-socket"))]
            None => panic!("Unix socket support is disabled"),
        }
    }

    /// Creates a new docker instance for a docker host
    /// listening on a given Unix socket.
    #[cfg(feature = "unix-socket")]
    pub fn unix(socket_path: impl Into<PathBuf>) -> Docker {
        let http_client = Arc::new(HttpClient::unix(socket_path));
        Self { http_client }
    }

    /// constructs a new Docker instance for docker host listening at the given host url
    pub fn host(host: Uri) -> Docker {
        let _tcp_host_str = format!(
            "{}://{}:{}",
            host.scheme_str().unwrap(),
            host.host().unwrap().to_owned(),
            host.port_u16().unwrap_or(80)
        );

        match host.scheme_str() {
            #[cfg(feature = "unix-socket")]
            Some("unix") => Self::unix(host.path().to_owned()),

            #[cfg(not(feature = "unix-socket"))]
            Some("unix") => panic!("Unix socket support is disabled"),

            #[cfg(feature = "tls")]
            _ => {
                let tcp_host_str = format!(
                    "{}://{}:{}",
                    host.scheme_str().unwrap(),
                    host.host().unwrap().to_owned(),
                    host.port_u16().unwrap_or(80)
                );
                Self::tls(tcp_host_str)
            }

            #[cfg(not(feature = "tls"))]
            _ => {
                let tcp_host_str = format!(
                    "{}://{}:{}",
                    host.scheme_str().unwrap(),
                    host.host().unwrap().to_owned(),
                    host.port_u16().unwrap_or(80)
                );
                Self::tcp(tcp_host_str)
            }
        }
    }

    #[cfg(not(feature = "tls"))]
    fn tcp(host: String) -> Docker {
        let http_client = Arc::new(HttpClient::tcp(host));
        Self { http_client }
    }

    #[cfg(feature = "tls")]
    fn tls(host: String) -> Docker {
        let http_client = Arc::new(HttpClient::tls(host));
        Self { http_client }
    }

    /// Exports an interface for interacting with docker images
    pub fn images(&self) -> Images {
        let http_client = Arc::clone(&self.http_client);
        Images::new(http_client)
    }

    /// Exports an interface for interacting with docker containers
    pub fn containers(&self) -> Containers {
        let http_client = Arc::clone(&self.http_client);
        Containers::new(http_client)
    }

    pub fn networks(&self) -> Networks {
        let http_client = Arc::clone(&self.http_client);

        Networks::new(http_client)
    }

    pub fn volumes(&self) -> Volumes {
        let http_client = Arc::clone(&self.http_client);

        Volumes::new(http_client)
    }

    /// Returns version information associated with the docker daemon
    pub async fn version(&self) -> Result<rep::Version> {
        self.http_client.get("/version").into_json().await
    }

    /// Returns information associated with the docker daemon
    pub async fn info(&self) -> Result<crate::rep::Info> {
        self.http_client.get("/info").into_json().await
    }

    /// Returns a simple ping response indicating the docker daemon is accessible
    pub async fn ping(&self) -> Result<String> {
        self.http_client.get("/_ping").into_string().await
    }

    /// Returns a stream of docker events
    pub fn events<'a>(
        &'a self,
        opts: &'a crate::builder::EventsOptions,
    ) -> impl Stream<Item = Result<crate::rep::Event>> + 'a {
        let mut path = "/events".to_string();
        if let Some(query) = opts.serialize() {
            path = format!("{}?{}", path, query);
        }

        let codec = tokio_util::codec::LinesCodec::new();

        self.http_client.get(&path).decode(codec)
    }
}
