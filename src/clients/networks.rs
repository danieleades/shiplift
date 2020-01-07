use crate::{builder, http_client::HttpClient, rep, Result};
use std::sync::Arc;

/// Interface for docker network
pub struct Networks {
    http_client: Arc<HttpClient>,
}

impl Networks {
    /// Exports an interface for interacting with docker Networks
    pub(super) fn new(http_client: Arc<HttpClient>) -> Self {
        Self { http_client }
    }

    /// List the docker networks on the current docker host
    pub async fn list(
        &self,
        opts: &builder::NetworkListOptions,
    ) -> Result<Vec<rep::Network>> {
        let mut path = vec!["/networks".to_owned()];
        if let Some(query) = opts.serialize() {
            path.push(query);
        }
        self.http_client.get(&path.join("?")).into_json().await
    }

    /// Returns a reference to a set of operations available to a specific network instance
    pub fn get<'a>(
        &self,
        id: &'a str,
    ) -> Network<'a> {
        let http_client = Arc::clone(&self.http_client);
        Network::new(http_client, id)
    }

    /// Create a new Network instance
    pub async fn create(
        &self,
        opts: &builder::NetworkCreateOptions,
    ) -> Result<rep::NetworkCreateInfo> {
        let body = opts.serialize()?;
        self.http_client
            .post("/networks/create")
            .json_body(body)
            .into_json()
            .await
    }
}

/// Interface for accessing and manipulating a docker network
pub struct Network<'a> {
    http_client: Arc<HttpClient>,
    id: &'a str,
}

impl<'a> Network<'a> {
    /// Exports an interface exposing operations against a network instance
    pub(crate) fn new(
        http_client: Arc<HttpClient>,
        id: &'a str,
    ) -> Self {
        Self { http_client, id }
    }

    /// a getter for the Network id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Inspects the current docker network instance's details
    pub async fn inspect(&self) -> Result<rep::Network> {
        self.http_client
            .get(&format!("/networks/{}", self.id))
            .into_json()
            .await
    }

    /// Delete the network instance
    pub async fn delete(&self) -> Result<()> {
        self.http_client
            .delete(&format!("/networks/{}", self.id))
            .into_response()
            .await?;
        Ok(())
    }

    /// Connect container to network
    pub async fn connect(
        &self,
        opts: &builder::ContainerConnectionOptions,
    ) -> Result<()> {
        self.do_connection("connect", opts).await
    }

    /// Disconnect container to network
    pub async fn disconnect(
        &self,
        opts: &builder::ContainerConnectionOptions,
    ) -> Result<()> {
        self.do_connection("disconnect", opts).await
    }

    async fn do_connection(
        &self,
        segment: &str,
        opts: &builder::ContainerConnectionOptions,
    ) -> Result<()> {
        let body = opts.serialize()?;

        self.http_client
            .post(&format!("/networks/{}/{}", self.id, segment))
            .json_body(body)
            .into_response()
            .await?;

        Ok(())
    }
}
