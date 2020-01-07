pub use build::Builder as Build;

mod build {

    use crate::{http_client::HttpClient, tarball, Result};
    use futures_util::stream::Stream;
    use serde::Serialize;
    use std::path::{Path, PathBuf};

    pub struct Builder<'a> {
        http_client: &'a HttpClient,
        path: &'a Path,
        query: Query,
    }

    impl<'a> Builder<'a> {
        pub(crate) fn new(
            http_client: &'a HttpClient,
            path: &'a Path,
        ) -> Self {
            let query = Query::default();

            Self {
                http_client,
                path,
                query,
            }
        }

        pub async fn send(self) -> impl Stream<Item = Result<serde_json::Value>> + 'a {
            let mut bytes = Vec::default();

            tarball::dir(&mut bytes, &self.path.to_string_lossy()).unwrap();

            self.http_client
                .post("/build")
                .query(self.query)
                .tar_body(bytes)
                .into_stream_json()
        }
    }

    #[derive(Default, Serialize)]
    struct Query {
        dockerfile: Option<PathBuf>,
        t: Vec<String>,
        remote: Option<String>,
        q: bool,
        nocache: bool,
        pull: bool,
        rm: bool,
        forcerm: bool,
        //memory,
        // memswap,
        // cpushares
        // cpusetcpus
        // cpuperiod
        // cpuquota
        // buildargs
        // shmsize
        // labels
    }
}
