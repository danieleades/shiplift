use crate::{http_client::HttpClient, Result};
use std::sync::Arc;

mod rep;
mod requests;
use requests::create::Builder as Create;

/// Interface for docker volumes
pub struct Volumes {
    http_client: Arc<HttpClient>,
}

impl Volumes {
    /// Exports an interface for interacting with docker volumes
    pub(super) fn new(http_client: Arc<HttpClient>) -> Self {
        Self { http_client }
    }

    /// Create a new Docker volume.
    ///
    /// # Example
    /// ```no_run
    /// # async move {
    /// # use shiplift::Docker;
    /// #
    /// let volumes = Docker::new().volumes();
    /// let create_info = volumes.create()
    ///     .name("new_volume")
    ///     .label("com.example.some-label", "some-value")
    ///     .await?
    /// # Ok(())
    /// # }
    /// ```
    pub fn create(&self) -> Create {
        Create::new(&self.http_client)
    }

    /// Lists the docker volumes on the current docker host
    pub async fn list(&self) -> Result<Vec<rep::Volume>> {
        let volumes: rep::Volumes = self.http_client.get("/volumes").into_json().await?;

        Ok(volumes.volumes.unwrap_or_default())
    }

    /// Returns a reference to a set of operations available for a named volume
    pub fn get<'a>(
        &self,
        name: &'a str,
    ) -> Volume<'a> {
        Volume::new(Arc::clone(&self.http_client), name)
    }
}

/// Interface for accessing and manipulating a named docker volume
pub struct Volume<'a> {
    http_client: Arc<HttpClient>,
    name: &'a str,
}

impl<'a> Volume<'a> {
    /// Exports an interface for operations that may be performed against a named volume
    pub(crate) fn new(
        http_client: Arc<HttpClient>,
        name: &'a str,
    ) -> Self {
        Self { http_client, name }
    }

    /// Deletes a volume
    pub async fn delete(&self) -> Result<()> {
        self.http_client
            .delete(&format!("/volumes/{}", self.name))
            .into_response()
            .await?;
        Ok(())
    }
}
