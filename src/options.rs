use hyper::{Body, Method};

mod network_create;
pub use network_create::{Driver as NetworkDriver, Options as NetworkCreateOptions};

mod volume_create;
pub use volume_create::Options as VolumeCreateOptions;

pub(crate) trait ShipliftOption {
    fn method(&self) -> Method;
    fn endpoint(&self) -> String;
    fn body(&self) -> Option<BodyType>;
}

#[derive(Debug)]
pub(crate) enum BodyType {
    Json(Body),
    Tar(Body),
}
