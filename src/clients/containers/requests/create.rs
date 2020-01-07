use crate::http_client::HttpClient;

pub struct Builder<'a> {
    http_client: &'a HttpClient,
}

impl<'a> Builder<'a> {
    pub(crate) fn new(http_client: &'a HttpClient) -> Self {
        Self { http_client }
    }
}
