pub mod create {

    use super::super::rep;
    use crate::{http_client::HttpClient, Result};
    use serde::Serialize;
    use std::collections::HashMap;

    pub struct Builder<'a> {
        http_client: &'a HttpClient,
        body: Body<'a>,
    }

    impl<'a> Builder<'a> {
        pub(crate) fn new(http_client: &'a HttpClient) -> Self {
            let body = Body::default();
            Self { http_client, body }
        }

        pub fn name(
            mut self,
            name: &'a str,
        ) -> Self {
            self.body.name = Some(name);
            self
        }

        pub fn label(
            mut self,
            key: &'a str,
            value: &'a str,
        ) -> Self {
            self.body.labels.insert(key, value);
            self
        }

        pub fn labels(
            mut self,
            items: impl IntoIterator<Item = (&'a str, &'a str)>,
        ) -> Self {
            for (key, value) in items {
                self = self.label(key, value);
            }
            self
        }

        pub async fn send(self) -> Result<rep::CreateInfo> {
            self.http_client
                .post("/volumes/create")
                .json_body(self.body)
                .into_json()
                .await
        }
    }

    #[derive(Serialize, Default)]
    #[serde(rename_all = "PascalCase")]
    struct Body<'a> {
        #[serde(skip_serializing_if = "Option::is_none")]
        name: Option<&'a str>,
        #[serde(skip_serializing_if = "Option::is_none")]
        driver: Option<Driver>,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        driver_opts: HashMap<String, String>,
        #[serde(skip_serializing_if = "HashMap::is_empty")]
        labels: HashMap<&'a str, &'a str>,
    }

    #[derive(Serialize)]
    pub enum Driver {
        Local,
    }
}
