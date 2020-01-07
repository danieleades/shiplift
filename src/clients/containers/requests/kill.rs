use crate::{http_client::HttpClient, Result};
use serde::Serialize;

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

    pub fn signal(
        mut self,
        signal: String,
    ) -> Self {
        self.query.signal = Some(signal);
        self
    }

    pub async fn send(self) -> Result<()> {
        self.http_client
            .post(&format!("/containers/{}/kill", self.id))
            .query(self.query)
            .into_response()
            .await?;
        Ok(())
    }
}

#[derive(Default, Serialize)]
struct Query {
    #[serde(skip_serializing_if = "Option::is_none")]
    signal: Option<String>,
}
