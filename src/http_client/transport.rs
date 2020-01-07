mod tcp;
pub use tcp::Tcp;

#[cfg(feature = "unix-socket")]
mod uds;
#[cfg(feature = "unix-socket")]
pub use uds::Uds;

#[cfg(feature = "tls")]
mod tls;
#[cfg(feature = "tls")]
pub use tls::Tls;

pub trait Transport {
    fn uri(
        &self,
        endpoint: &str,
    ) -> String;

    fn send_request(
        &self,
        req: hyper::Request<hyper::Body>,
    ) -> hyper::client::ResponseFuture;
}
