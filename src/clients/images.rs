use crate::{builder, http_client::HttpClient, rep, Result};
use futures_util::{
    future::TryFutureExt,
    io::{AsyncRead, AsyncReadExt},
    stream::Stream,
};
use serde_json::Value as JsonValue;
use std::{path::Path, sync::Arc};

mod requests;

/// Interface for docker images
pub struct Images {
    http_client: Arc<HttpClient>,
}

impl Images {
    /// Exports an interface for interacting with docker images
    pub(super) fn new(http_client: Arc<HttpClient>) -> Self {
        Self { http_client }
    }

    /// Builds a new image build by reading a Dockerfile in a target directory
    pub fn build<'a>(
        &'a self,
        path: &'a Path,
    ) -> requests::Build<'a> {
        requests::Build::new(&self.http_client, path)
    }

    /// Lists the docker images on the current docker host
    pub async fn list(
        &self,
        opts: &builder::ImageListOptions,
    ) -> Result<Vec<rep::Image>> {
        let mut path = "/images/json".to_string();
        if let Some(query) = opts.serialize() {
            path = format!("{}?{}", path, query);
        }

        self.http_client.get(&path).into_json().await
    }

    /// Returns a reference to a set of operations available for a named image
    pub fn get<'a>(
        &self,
        name: &'a str,
    ) -> Image<'a> {
        Image::new(Arc::clone(&self.http_client), name)
    }

    /// Search for docker images by term
    pub async fn search(
        &self,
        term: &str,
    ) -> Result<Vec<rep::SearchResult>> {
        self.http_client
            .get("/images/search")
            .query(&[("term", term)])
            .into_json()
            .await
    }

    /*     /// Pull and create a new docker images from an existing image
    pub fn pull(
        &self,
        opts: &builder::PullOptions,
    ) -> impl Stream<Item = Result<JsonValue>> {
        let mut path = vec!["/images/create".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        let headers = opts
            .auth_header()
            .map(|a| std::iter::once(("X-Registry-Auth", a)));

            self.docker
                .stream_post(path.join("?"), None, headers)
                .and_then(move |chunk| {
                    // todo: give this a proper enum type
                    futures_util::future::ready(serde_json::from_slice(&chunk).map_err(crate::Error::from))
                }),
        )
    } */

    /// exports a collection of named images,
    /// either by name, name:tag, or image id, into a tarball
    pub fn export<'a>(
        &'a self,
        names: Vec<String>,
    ) -> impl Stream<Item = Result<Vec<u8>>> + 'a {
        let query = [("names", names.join(","))];

        self.http_client
            .get("images/get")
            .query(query)
            .into_stream_json()
    }

    /// imports an image or set of images from a given tarball source
    /// source can be uncompressed on compressed via gzip, bzip2 or xz
    pub fn import<'a>(
        &'a self,
        mut tarball: impl AsyncRead + Unpin + 'a,
    ) -> impl Stream<Item = Result<JsonValue>> + 'a {
        async move {
            let mut bytes = Vec::default();
            tarball.read_to_end(&mut bytes).await?;

            Ok(self
                .http_client
                .post("/images/load")
                .tar_body(bytes)
                .into_stream_json())
        }
        .try_flatten_stream()
    }
}

/// Interface for accessing and manipulating a named docker image
pub struct Image<'a> {
    transport: Arc<HttpClient>,
    name: &'a str,
}

impl<'a> Image<'a> {
    /// Exports an interface for operations that may be performed against a named image
    pub(crate) fn new(
        transport: Arc<HttpClient>,
        name: &'a str,
    ) -> Self {
        Self { transport, name }
    }

    /// Inspects a named image's details
    pub async fn inspect(&self) -> Result<rep::ImageDetails> {
        self.transport
            .get(&format!("/images/{}/json", self.name))
            .into_json()
            .await
    }

    /// Lists the history of the images set of changes
    pub async fn history(&self) -> Result<Vec<rep::History>> {
        self.transport
            .get(&format!("/images/{}/history", self.name))
            .into_json()
            .await
    }

    /// Deletes an image
    pub async fn delete(&self) -> Result<Vec<rep::Status>> {
        self.transport
            .delete(&format!("/images/{}", self.name))
            .into_json()
            .await
    }

    /// Export this image to a tarball
    pub fn export(&'a self) -> impl Stream<Item = Result<Vec<u8>>> + 'a {
        self.transport
            .get(&format!("/images/{}/get", self.name))
            .into_stream()
    }

    /// Adds a tag to an image
    pub async fn tag(
        &self,
        opts: &builder::TagOptions,
    ) -> Result<()> {
        let mut path = format!("/images/{}/tag", self.name);
        if let Some(query) = opts.serialize() {
            path = format!("{}?{}", path, query);
        }
        self.transport.post(&path).into_response().await?;
        Ok(())
    }
}
