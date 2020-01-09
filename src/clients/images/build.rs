use crate::http_client::HttpClient;
use std::path::Path;

pub struct Request<'a> {
    http_client: &'a HttpClient,
    path: &'a Path,
}

impl<'a> Request<'a> {
    pub(crate) fn new(
        http_client: &'a HttpClient,
        path: &'a Path,
    ) -> Self {
        Self { http_client, path }
    }
}
