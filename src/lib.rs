#![feature(type_alias_impl_trait)]
#![feature(into_future)]
//! Shiplift is a multi-transport utility for maneuvering [docker](https://www.docker.com/) containers
//!
//! # examples
//!
//! ```no_run
//! # async {
//! let docker = shiplift::Docker::new();
//!
//! match docker.images().list().await {
//!     Ok(images) => {
//!         for image in images {
//!             println!("{:?}", image.repo_tags);
//!         }
//!     },
//!     Err(e) => eprintln!("Something bad happened! {}", e),
//! }
//! # };
//! ```

pub mod builder;
mod errors;
pub use errors::Error;
mod clients;
mod http_client;
mod into_future;
pub mod rep;
mod tarball;
pub mod tty;
pub use into_future::IntoFuture;

/// Represents the result of all docker operations
pub type Result<T> = std::result::Result<T, Error>;
