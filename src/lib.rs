#![feature(type_alias_impl_trait)]
//#![feature(into_future)]
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

mod errors;
pub use errors::{Error, Result};
pub mod clients;
pub use clients::Docker;
mod compat;
mod http_client;
pub mod tty;
use compat::Compat;
