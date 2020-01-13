//! Transports for communicating with the docker daemon

use hyper::Method;
use std::{path::PathBuf};

mod transport;
use transport::Transport;

mod request;
use request::RequestBuilder;

pub(crate) enum HttpClient {
    Tcp(transport::Tcp),
    Tls(transport::Tls),
    Uds(transport::Uds),
}

impl HttpClient {
    pub fn tcp(host: impl Into<String>) -> Self {
        let transport = transport::Tcp::new(host);
        Self::Tcp(transport)
    }

    #[cfg(feature = "unix-socket")]
    pub fn unix(path: impl Into<PathBuf>) -> Self {
        let transport = transport::Uds::new(path);
        Self::Uds(transport)
    }

    #[cfg(feature = "tls")]
    pub fn tls(host: impl Into<String>) -> Self {
        let transport = transport::Tls::new(host);
        Self::Tls(transport)
    }

    pub fn request(
        &self,
        endpoint: impl AsRef<str>,
    ) -> RequestBuilder {
        RequestBuilder::new(self, endpoint)
    }
    pub fn get(
        &self,
        endpoint: impl AsRef<str>,
    ) -> RequestBuilder {
        RequestBuilder::new(self, endpoint).method(Method::GET)
    }
    pub fn post(
        &self,
        endpoint: impl AsRef<str>,
    ) -> RequestBuilder {
        RequestBuilder::new(self, endpoint).method(Method::POST)
    }
    pub fn put(
        &self,
        endpoint: impl AsRef<str>,
    ) -> RequestBuilder {
        RequestBuilder::new(&self, endpoint).method(Method::PUT)
    }
    pub fn delete<'a>(
        &'a self,
        endpoint: &'a str,
    ) -> RequestBuilder<'a> {
        RequestBuilder::new(self, endpoint).method(Method::DELETE)
    }

    fn transport(&self) -> &dyn Transport {
        match self {
            Self::Tcp(transport) => transport,
            #[cfg(feature = "tls")]
            Self::Tls(transport) => transport,
            #[cfg(feature = "unix-socket")]
            Self::Uds(transport) => transport,
        }
    }

    fn uri(
        &self,
        endpoint: impl AsRef<str>,
    ) -> String {
        self.transport().uri(endpoint.as_ref())
    }

    fn send_request(
        &self,
        req: hyper::Request<hyper::Body>,
    ) -> hyper::client::ResponseFuture {
        self.transport().send_request(req)
    }
}

pub enum BodyType {
    Json(Vec<u8>),
    Tar(Vec<u8>),
}

impl BodyType {
    fn json(data: Vec<u8>) -> Self {
        Self::Json(data)
    }

    fn tar(data: Vec<u8>) -> Self {
        Self::Tar(data)
    }

    fn mime(&self) -> String {
        match self {
            Self::Json(_) => mime::APPLICATION_JSON.to_string(),
            Self::Tar(_) => "application/x-tar".to_string(),
        }
    }

    fn into_data(self) -> Vec<u8> {
        match self {
            Self::Json(data) | Self::Tar(data) => data,
        }
    }
}

/* pub struct Transport {
    inner_transport: Box<dyn InnerTransport>,
}pub struct Transport {
    inner_transport: Box<dyn InnerTransport>,
}

impl Transport {

    #[cfg(feature = "unix-socket")]
    pub fn unix<S>(socket_path: S) -> Self
    where
        S: Into<String>,
    {
        let inner_transport = Box::new(UdsTransport::new(socket_path));
        Self {
            inner_transport
        }
    }

    #[cfg(not(feature = "tls"))]
    fn tcp(host: String) -> Self {
    let inner_transport = Box::new(TcpTransport::new(host));
        Self {
            inner_transport
        }
    }

    #[cfg(feature = "tls")]
    fn tls(host: String) -> Self {
    let inner_transport = TlsTransport::new(host).unwrap_or(TcpTransport::new(host));
    Self { inner_transport}
    }

    pub async fn send_request(
        &self,
        endpoint: impl AsRef<str>,
        method: hyper::Method,
        body: Option<BodyType>,
    ) -> Result<Response<Body>> {
        let headers = std::iter::empty();
        self.send_request_with_headers(
            endpoint, method, body, headers,
        ).await
    }

    pub async fn send_request_with_headers(
        &self,
        endpoint: impl AsRef<str>,
        method: hyper::Method,
        body: Option<BodyType>,
        headers: impl IntoIterator<Item = (&'static str, String)>,
    ) -> Result<Response<Body>> {
        let uri = self.inner_transport.uri(endpoint.as_ref());

        let request = build_request(
            uri,
            method,
            body,
            headers,
        );

        let response = (&self.inner_transport).send_request(request).await?;

        Ok(response)
    }

    pub async fn send_request_upgraded(
        &self,
        endpoint: impl AsRef<str>,
        method: hyper::Method,
        body: Option<BodyType>,
    ) -> Result<hyper::upgrade::Upgraded> {
        let response = self.send_request(endpoint, method, body).await?;

        match response.status() {
            hyper::StatusCode::SWITCHING_PROTOCOLS => Ok(response.into_body().on_upgrade().await?),
            _ => Err(Error::ConnectionNotUpgraded),
        }
    }

    // Convenience methods

    async fn get(&self, endpoint: impl AsRef<str>) -> Result<Vec<u8>> {
        let method = hyper::Method::GET;
        let body = None;

        let body = self.send_request(endpoint, method, body).await?.into_body();

        let mut v = Vec::default();

        while let Some(bytes_result) = body.next().await {
            let bytes = bytes_result?;
            v.extend(&bytes)
        }

        Ok(v)
    }

    pub async fn get_string(&self, endpoint: impl AsRef<str>) -> Result<String> {
        let bytes = self.get(endpoint).await?;

        Ok(String::from_utf8(bytes)?)
    }

    pub async fn get_json<T>(&self, endpoint: impl AsRef<str>) -> Result<T> where T: serde::de::DeserializeOwned {
        let bytes = self.get(endpoint).await?;

        Ok(serde_json::from_slice(&bytes)?)
    }
}

fn build_request(
    uri: String,
    method: hyper::Method,
    body: Option<BodyType>,
    headers: impl IntoIterator<Item = (&'static str, String)>,
) -> hyper::Request<hyper::Body> {
    unimplemented!()
}


/* /// Transports are types which define the means of communication
/// with the docker daemon
#[derive(Clone)]
pub enum Transport {
    /// A network tcp interface
    Tcp(tcp::Transport),
    /// TCP/TLS
    #[cfg(feature = "tls")]
    EncryptedTcp(tls::Transport),
    /// A Unix domain socket
    #[cfg(feature = "unix-socket")]
    Unix(uds::Transport),
}

impl Transport {
    /// Make a request and return the whole response in a `String`
    pub async fn request<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<String>
    where
        B: Into<Body>,
    {
        let body = self
            .get_body(method, endpoint, body, None::<iter::Empty<_>>)
            .await?;
        let bytes = concat_chunks(body).await?;

        let string = String::from_utf8(bytes)?;

        Ok(string)
    }

    async fn get_body<B, H>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> Result<Body>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        let req = self
            .build_request(method, endpoint, body, headers, |_| ())
            .expect("Failed to build request!");

        let response = self.send_request(req).await?;

        let status = response.status();

        match status {
            // Success case: pass on the response
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::SWITCHING_PROTOCOLS
            | StatusCode::NO_CONTENT => Ok(response.into_body()),
            // Error case: parse the text
            _ => {
                let bytes = concat_chunks(response.into_body()).await?;

                let message_body = String::from_utf8(bytes)?;

                Err(Error::Fault {
                    code: status,
                    message: Self::get_error_message(&message_body).unwrap_or_else(|| {
                        status
                            .canonical_reason()
                            .unwrap_or_else(|| "unknown error code")
                            .to_owned()
                    }),
                })
            }
        }
    }

    pub fn stream_bytes<'a, H, B>(
        &'a self,
        method: Method,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = Result<Bytes>> + 'a
    where
        H: IntoIterator<Item = (&'static str, String)> + 'a,
        B: Into<Body> + 'a,
    {
        async move {
            let body = self.get_body(method, endpoint, body, headers).await?;

            Ok(stream_body(body))
        }
        .try_flatten_stream()
    }

    /// Builds an HTTP request.
    fn build_request<B, H>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
        headers: Option<H>,
        f: impl FnOnce(&mut ::http::request::Builder),
    ) -> Result<Request<Body>>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        let mut builder = Request::builder();
        f(&mut builder);

        let req = match *self {
            Transport::Tcp { ref host, .. } => {
                builder
                    .method(method)
                    .uri(&format!("{}{}", host, endpoint.as_ref()))
            }
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref host, .. } => {
                builder
                    .method(method)
                    .uri(&format!("{}{}", host, endpoint.as_ref()))
            }
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref path, .. } => {
                let uri: hyper::Uri = DomainUri::new(&path, endpoint.as_ref())?.into();
                builder.method(method).uri(&uri.to_string())
            }
        };
        let req = req.header(header::HOST, "");

        if let Some(h) = headers {
            for (k, v) in h.into_iter() {
                req.header(k, v);
            }
        }

        match body {
            Some((b, c)) => Ok(req
                .header(header::CONTENT_TYPE, &c.to_string()[..])
                .body(b.into())?),
            _ => Ok(req.body(Body::empty())?),
        }
    }

    /// Send the given request to the docker daemon and return a Future of the response.
    async fn send_request(
        &self,
        req: Request<hyper::Body>,
    ) -> Result<hyper::Response<Body>> {
        match self {
            Transport::Tcp { ref client, .. } => Ok(client.request(req).await?),
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref client, .. } => Ok(client.request(req).await?),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref client, .. } => Ok(client.request(req).await?),
        }
    }

    /// Makes an HTTP request, upgrading the connection to a TCP
    /// stream on success.
    ///
    /// This method can be used for operations such as viewing
    /// docker container logs interactively.
    async fn stream_upgrade_tokio<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<hyper::upgrade::Upgraded>
    where
        B: Into<Body>,
    {
        match self {
            Transport::Tcp { .. } => (),
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { .. } => (),
            #[cfg(feature = "unix-socket")]use super::Options;
            Transport::Unix { .. } => panic!("connection streaming is only supported over TCP"),
        };

        let req = self
            .build_request(method, endpoint, body, None::<iter::Empty<_>>, |builder| {
                builder
                    .header(header::CONNECTION, "Upgrade")
                    .header(header::UPGRADE, "tcp");
            })
            .expect("Failed to build request!");

        let response = self.send_request(req).await?;

        match response.status() {
            StatusCode::SWITCHING_PROTOCOLS => Ok(response.into_body().on_upgrade().await?),
            _ => Err(Error::ConnectionNotUpgraded),
        }
    }

    pub async fn stream_upgrade<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<impl AsyncRead + AsyncWrite>
    where
        B: Into<Body>,
    {
        let tokio_multiplexer = self.stream_upgrade_tokio(method, endpoint, body).await?;

        Ok(Compat { tokio_multiplexer })
    }

    /// Extract the error message content from an HTTP response that
    /// contains a Docker JSON error structure.
    fn get_error_message(body: &str) -> Option<String> {
        serde_json::from_str::<ErrorResponse>(body)
            .map(|e| e.message)
            .ok()
    }
}

impl fmt::Debug for Transport {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match *self {
            Transport::Tcp { ref host, .. } => write!(f, "Tcp({})", host),
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref host, .. } => write!(f, "EncryptedTcp({})", host),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref path, .. } => write!(f, "Unix({})", path),
        }
    }
}

#[pin_project]
struct Compat<S> {
    #[pin]
    tokio_multiplexer: S,
}

impl<S> AsyncRead for Compat<S>
where
    S: tokio::io::AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().tokio_multiplexer.poll_read(cx, buf)
    }
}

impl<S> AsyncWrite for Compat<S>
where
    S: tokio::io::AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().tokio_multiplexer.poll_write(cx, buf)
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().tokio_multiplexer.poll_flush(cx)
    }
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().tokio_multiplexer.poll_shutdown(cx)
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    message: String,
}

fn stream_body(body: Body) -> impl Stream<Item = Result<Bytes>> {
    async fn unfold(mut body: Body) -> Option<(Result<Bytes>, Body)> {
        let chunk_result = body.next().await?.map_err(Error::from);

        Some((chunk_result, body))
    }

    futures_util::stream::unfold(body, unfold)
}

async fn concat_chunks(body: Body) -> Result<Vec<u8>> {
    let mut v = Vec::default();
    while let Some(bytes_result) = body.next().await {
        let bytes = bytes_result?;
        v.extend(bytes);
    }
    Ok(v)
}
 */

impl Transport {

    #[cfg(feature = "unix-socket")]
    pub fn unix<S>(socket_path: S) -> Self
    where
        S: Into<String>,
    {
        let inner_transport = Box::new(UdsTransport::new(socket_path));
        Self {
            inner_transport
        }
    }

    #[cfg(not(feature = "tls"))]
    fn tcp(host: String) -> Self {
    let inner_transport = Box::new(TcpTransport::new(host));
        Self {
            inner_transport
        }
    }

    #[cfg(feature = "tls")]
    fn tls(host: String) -> Self {
    let inner_transport = TlsTransport::new(host).unwrap_or(TcpTransport::new(host));
    Self { inner_transport}
    }

    pub async fn send_request(
        &self,
        endpoint: impl AsRef<str>,
        method: hyper::Method,
        body: Option<BodyType>,
    ) -> Result<Response<Body>> {
        let headers = std::iter::empty();
        self.send_request_with_headers(
            endpoint, method, body, headers,
        ).await
    }

    pub async fn send_request_with_headers(
        &self,
        endpoint: impl AsRef<str>,
        method: hyper::Method,
        body: Option<BodyType>,
        headers: impl IntoIterator<Item = (&'static str, String)>,
    ) -> Result<Response<Body>> {
        let uri = self.inner_transport.uri(endpoint.as_ref());

        let request = build_request(
            uri,
            method,
            body,
            headers,
        );

        let response = (&self.inner_transport).send_request(request).await?;

        Ok(response)
    }

    pub async fn send_request_upgraded(
        &self,
        endpoint: impl AsRef<str>,
        method: hyper::Method,
        body: Option<BodyType>,
    ) -> Result<hyper::upgrade::Upgraded> {
        let response = self.send_request(endpoint, method, body).await?;

        match response.status() {
            hyper::StatusCode::SWITCHING_PROTOCOLS => Ok(response.into_body().on_upgrade().await?),
            _ => Err(Error::ConnectionNotUpgraded),
        }
    }

    // Convenience methods

    async fn get(&self, endpoint: impl AsRef<str>) -> Result<Vec<u8>> {
        let method = hyper::Method::GET;
        let body = None;

        let body = self.send_request(endpoint, method, body).await?.into_body();

        let mut v = Vec::default();

        while let Some(bytes_result) = body.next().await {
            let bytes = bytes_result?;
            v.extend(&bytes)
        }

        Ok(v)
    }

    pub async fn get_string(&self, endpoint: impl AsRef<str>) -> Result<String> {
        let bytes = self.get(endpoint).await?;

        Ok(String::from_utf8(bytes)?)
    }

    pub async fn get_json<T>(&self, endpoint: impl AsRef<str>) -> Result<T> where T: serde::de::DeserializeOwned {
        let bytes = self.get(endpoint).await?;

        Ok(serde_json::from_slice(&bytes)?)
    }
}

fn build_request(
    uri: String,
    method: hyper::Method,
    body: Option<BodyType>,
    headers: impl IntoIterator<Item = (&'static str, String)>,
) -> hyper::Request<hyper::Body> {
    unimplemented!()
}

pub enum BodyType {
    Json(Vec<u8>),
    Tar(Vec<u8>),
}

impl BodyType {
    fn json(data: Vec<u8>) -> Self {
        Self::Json(data)
    }

    fn tar(data: Vec<u8>) -> Self {
        Self::Tar(data)
    }

    fn mime(&self) -> String {
        match self {
            Self::Json(_) => mime::APPLICATION_JSON.to_string(),
            Self::Tar(_) => "application/x-tar".to_string(),
        }
    }

    fn into_data(self) -> Vec<u8> {
        match self {
            Self::Json(data) | Self::Tar(data) => data
        }
    }
}

/* /// Transports are types which define the means of communication
/// with the docker daemon
#[derive(Clone)]
pub enum Transport {
    /// A network tcp interface
    Tcp(tcp::Transport),
    /// TCP/TLS
    #[cfg(feature = "tls")]
    EncryptedTcp(tls::Transport),
    /// A Unix domain socket
    #[cfg(feature = "unix-socket")]
    Unix(uds::Transport),
}

impl Transport {
    /// Make a request and return the whole response in a `String`
    pub async fn request<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<String>
    where
        B: Into<Body>,
    {
        let body = self
            .get_body(method, endpoint, body, None::<iter::Empty<_>>)
            .await?;
        let bytes = concat_chunks(body).await?;

        let string = String::from_utf8(bytes)?;

        Ok(string)
    }

    async fn get_body<B, H>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> Result<Body>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        let req = self
            .build_request(method, endpoint, body, headers, |_| ())
            .expect("Failed to build request!");

        let response = self.send_request(req).await?;

        let status = response.status();

        match status {
            // Success case: pass on the response
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::SWITCHING_PROTOCOLS
            | StatusCode::NO_CONTENT => Ok(response.into_body()),
            // Error case: parse the text
            _ => {
                let bytes = concat_chunks(response.into_body()).await?;

                let message_body = String::from_utf8(bytes)?;

                Err(Error::Fault {
                    code: status,
                    message: Self::get_error_message(&message_body).unwrap_or_else(|| {
                        status
                            .canonical_reason()
                            .unwrap_or_else(|| "unknown error code")
                            .to_owned()
                    }),
                })
            }
        }
    }

    pub fn stream_bytes<'a, H, B>(
        &'a self,
        method: Method,
        endpoint: impl AsRef<str> + 'a,
        body: Option<(B, Mime)>,
        headers: Option<H>,
    ) -> impl Stream<Item = Result<Bytes>> + 'a
    where
        H: IntoIterator<Item = (&'static str, String)> + 'a,
        B: Into<Body> + 'a,
    {
        async move {
            let body = self.get_body(method, endpoint, body, headers).await?;

            Ok(stream_body(body))
        }
        .try_flatten_stream()
    }

    /// Builds an HTTP request.
    fn build_request<B, H>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
        headers: Option<H>,
        f: impl FnOnce(&mut ::http::request::Builder),
    ) -> Result<Request<Body>>
    where
        B: Into<Body>,
        H: IntoIterator<Item = (&'static str, String)>,
    {
        let mut builder = Request::builder();
        f(&mut builder);

        let req = match *self {
            Transport::Tcp { ref host, .. } => {
                builder
                    .method(method)
                    .uri(&format!("{}{}", host, endpoint.as_ref()))
            }
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref host, .. } => {
                builder
                    .method(method)
                    .uri(&format!("{}{}", host, endpoint.as_ref()))
            }
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref path, .. } => {
                let uri: hyper::Uri = DomainUri::new(&path, endpoint.as_ref())?.into();
                builder.method(method).uri(&uri.to_string())
            }
        };
        let req = req.header(header::HOST, "");

        if let Some(h) = headers {
            for (k, v) in h.into_iter() {
                req.header(k, v);
            }
        }

        match body {
            Some((b, c)) => Ok(req
                .header(header::CONTENT_TYPE, &c.to_string()[..])
                .body(b.into())?),
            _ => Ok(req.body(Body::empty())?),
        }
    }

    /// Send the given request to the docker daemon and return a Future of the response.
    async fn send_request(
        &self,
        req: Request<hyper::Body>,
    ) -> Result<hyper::Response<Body>> {
        match self {
            Transport::Tcp { ref client, .. } => Ok(client.request(req).await?),
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref client, .. } => Ok(client.request(req).await?),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref client, .. } => Ok(client.request(req).await?),
        }
    }

    /// Makes an HTTP request, upgrading the connection to a TCP
    /// stream on success.
    ///
    /// This method can be used for operations such as viewing
    /// docker container logs interactively.
    async fn stream_upgrade_tokio<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<hyper::upgrade::Upgraded>
    where
        B: Into<Body>,
    {
        match self {
            Transport::Tcp { .. } => (),
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { .. } => (),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { .. } => panic!("connection streaming is only supported over TCP"),
        };

        let req = self
            .build_request(method, endpoint, body, None::<iter::Empty<_>>, |builder| {
                builder
                    .header(header::CONNECTION, "Upgrade")
                    .header(header::UPGRADE, "tcp");
            })
            .expect("Failed to build request!");

        let response = self.send_request(req).await?;

        match response.status() {
            StatusCode::SWITCHING_PROTOCOLS => Ok(response.into_body().on_upgrade().await?),
            _ => Err(Error::ConnectionNotUpgraded),
        }
    }

    pub async fn stream_upgrade<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Option<(B, Mime)>,
    ) -> Result<impl AsyncRead + AsyncWrite>
    where
        B: Into<Body>,
    {
        let tokio_multiplexer = self.stream_upgrade_tokio(method, endpoint, body).await?;

        Ok(Compat { tokio_multiplexer })
    }

    /// Extract the error message content from an HTTP response that
    /// contains a Docker JSON error structure.
    fn get_error_message(body: &str) -> Option<String> {
        serde_json::from_str::<ErrorResponse>(body)
            .map(|e| e.message)
            .ok()
    }
}

impl fmt::Debug for Transport {
    fn fmt(
        &self,
        f: &mut fmt::Formatter,
    ) -> fmt::Result {
        match *self {
            Transport::Tcp { ref host, .. } => write!(f, "Tcp({})", host),
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref host, .. } => write!(f, "EncryptedTcp({})", host),
            #[cfg(feature = "unix-socket")]
            Transport::Unix { ref path, .. } => write!(f, "Unix({})", path),
        }
    }
}

#[pin_project]
struct Compat<S> {
    #[pin]
    tokio_multiplexer: S,
}

impl<S> AsyncRead for Compat<S>
where
    S: tokio::io::AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        self.project().tokio_multiplexer.poll_read(cx, buf)
    }
}

impl<S> AsyncWrite for Compat<S>
where
    S: tokio::io::AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().tokio_multiplexer.poll_write(cx, buf)
    }
    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().tokio_multiplexer.poll_flush(cx)
    }
    fn poll_close(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<io::Result<()>> {
        self.project().tokio_multiplexer.poll_shutdown(cx)
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    message: String,
}

fn stream_body(body: Body) -> impl Stream<Item = Result<Bytes>> {
    async fn unfold(mut body: Body) -> Option<(Result<Bytes>, Body)> {
        let chunk_result = body.next().await?.map_err(Error::from);

        Some((chunk_result, body))
    }

    futures_util::stream::unfold(body, unfold)
}

async fn concat_chunks(body: Body) -> Result<Vec<u8>> {
    let mut v = Vec::default();
    while let Some(bytes_result) = body.next().await {
        let bytes = bytes_result?;
        v.extend(bytes);
    }
    Ok(v)
}
 */ */
