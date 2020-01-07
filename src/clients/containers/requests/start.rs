use crate::{http_client::HttpClient, Result};
use serde::Serialize;
use std::time::Duration;

pub struct Builder<'a> {
    http_client: &'a HttpClient,
    id: &'a str,
    query: Query,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(
        http_client: &'a HttpClient,
        id: &'a str,
    ) -> Self {
        let query = Query::default();
        Self {
            http_client,
            id,
            query,
        }
    }

    pub fn wait(
        mut self,
        wait: Duration,
    ) -> Self {
        let seconds = wait.as_secs();
        self.query.t = Some(seconds);
        self
    }

    pub async fn send(self) -> Result<()> {
        self.http_client
            .post(&format!("/containers/{}/start", self.id))
            .query(self.query)
            .into_response()
            .await?;
        Ok(())
    }
}

#[derive(Serialize, Default)]
struct Query {
    #[serde(skip_serializing_if = "Option::is_none")]
    t: Option<u64>,
}
