use crate::{http_client::HttpClient, tty, Result};
use futures_util::{
    future::TryFutureExt,
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
};
use std::{path::Path, sync::Arc};

mod requests;
mod types;
pub use types::*;

pub use requests::{Create, Kill, List, Restart, Start, Stop};

/// Interface for docker containers
pub struct Containers {
    http_client: Arc<HttpClient>,
}

impl Containers {
    /// Exports an interface for interacting with docker containers
    pub(super) fn new(http_client: Arc<HttpClient>) -> Self {
        Self { http_client }
    }

    /// Returns a reference to a set of operations available to a specific container instance
    pub fn get<'a>(
        &self,
        id: &'a str,
    ) -> Container<'a> {
        let http_client = Arc::clone(&self.http_client);
        Container::new(http_client, id)
    }

    /// Lists the container instances on the docker host.
    ///
    /// See '[List](List)' for additional options
    ///
    /// # Example
    /// ```no_run
    /// # async {
    /// # use shiplift::Docker;
    /// let containers = Docker::new()
    ///     .containers()
    ///     .list()
    ///     .send()
    ///     .await?;
    /// # }
    /// ```
    pub fn list(&self) -> List {
        List::new(&self.http_client)
    }

    /// Returns a builder interface for creating a new container instance
    pub fn create(&self) -> Create {
        Create::new(&self.http_client)
    }
}

/// Interface for accessing and manipulating a docker container
pub struct Container<'a> {
    http_client: Arc<HttpClient>,
    id: &'a str,
}

impl<'a> Container<'a> {
    /// Exports an interface exposing operations against a container instance
    fn new(
        http_client: Arc<HttpClient>,
        id: &'a str,
    ) -> Self {
        Container { http_client, id }
    }

    /// a getter for the container id
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Inspects the current docker container instance's details
    pub async fn inspect(&self) -> Result<types::ContainerDetails> {
        self.http_client
            .get(&format!("/containers/{}/json", self.id()))
            .into_json()
            .await
    }

    /// Returns a `top` view of information about the container process
    pub async fn top(
        &self,
        psargs: Option<&str>,
    ) -> Result<types::Top> {
        let endpoint = format!("/containers/{}/top", self.id());
        let mut request = self.http_client.get(&endpoint);

        if let Some(psargs) = psargs {
            request = request.query(&[("ps_args", psargs)]);
        }

        request.into_json().await
    }

/*     /// Returns a stream of logs emitted but the container instance
    pub fn logs(
        &'a self,
        opts: &builder::LogsOptions,
    ) -> impl Stream<Item = Result<crate::tty::TtyChunk>> + 'a {
        let mut path = format!("/containers/{}/logs", self.id());
        if let Some(query) = opts.serialize() {
            path = format!("{}?{}", path, query);
        }

        let stream = self.http_client.get(&path).into_stream();

        tty::decode_chunks(stream)
    } */

    /// Attaches a multiplexed TCP stream to the container that can be used to read Stdout, Stderr and write Stdin.
    async fn attach_raw(&self) -> Result<impl AsyncRead + AsyncWrite + 'a> {
        self.http_client
            .post(&format!("/containers/{}/attach", self.id()))
            .query(&[
                ("stream", true),
                ("stdout", true),
                ("stderr", true),
                ("stdin", true),
            ])
            .upgrade()
            .await
    }

    /*     /// Attaches a `[TtyMultiplexer]` to the container.
    ///
    /// The `[TtyMultiplexer]` implements Stream for returning Stdout and Stderr chunks. It also implements `[AsyncWrite]` for writing to Stdin.
    ///
    /// The multiplexer can be split into its read and write halves with the `[split](TtyMultiplexer::split)` method
    pub async fn attach(&self) -> Result<tty::Multiplexer<'a>> {
        let tcp_stream = self.attach_raw().await?;

        Ok(tty::Multiplexer::new(tcp_stream))
    } */

    /// Returns a set of changes made to the container instance
    pub async fn changes(&self) -> Result<Vec<types::Change>> {
        self.http_client
            .get(&format!("/containers/{}/changes", self.id))
            .into_json()
            .await
    }

    /// Exports the current docker container into a tarball
    pub fn export(&'a self) -> impl Stream<Item = Result<Vec<u8>>> + 'a {
        self.http_client
            .get(&format!("/containers/{}/export", self.id))
            .into_stream()
    }

    /// Returns a stream of stats specific to this container instance
    pub fn stats(&'a self) -> impl Stream<Item = Result<types::Stats>> + 'a {
        let codec = tokio_util::codec::LinesCodec::new();

        self.http_client
            .get(&format!("/containers/{}/stats", self.id))
            .decode(codec)
    }

    /// Start the container instance
    ///
    /// Optionally set the wait time before starting the container
    ///
    /// # Example
    /// ```no_run
    /// # let container = shiplift::Docker::new()
    /// #    .containers()
    /// #    .get("some_id");
    /// #
    /// # async move {
    /// #
    /// let t = std::time::Duration::from_secs(10);
    /// container.start()
    ///     .wait(t)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn start(&self) -> Start {
        Start::new(&self.http_client, self.id)
    }

    /// Stop the container instance
    ///
    /// Optionally set the wait time before killing the container
    ///
    /// # Example
    /// ```no_run
    /// # let container = shiplift::Docker::new()
    /// #    .containers()
    /// #    .get("some_id");
    /// #
    /// # async move {
    /// #
    /// let t = std::time::Duration::from_secs(10);
    /// container.stop()
    ///     .wait(t)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn stop(&self) -> Stop {
        Stop::new(&self.http_client, self.id)
    }

    /// Restart the container instance
    ///
    /// Optionally set the wait time before killing the container
    ///
    /// # Example
    /// ```no_run
    /// # let container = shiplift::Docker::new()
    /// #    .containers()
    /// #    .get("some_id");
    /// #
    /// # async move {
    /// #
    /// let t = std::time::Duration::from_secs(10);
    /// container.restart()
    ///     .wait(t)
    ///     .await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn restart(&self) -> Restart {
        Restart::new(&self.http_client, self.id)
    }

    /// Kill the container instance
    pub fn kill(&self) -> Kill {
        Kill::new(&self.http_client, self.id())
    }

    /// Rename the container instance
    pub async fn rename(
        &self,
        name: &str,
    ) -> Result<()> {
        self.http_client
            .post(&format!("/containers/{}/rename", self.id))
            .query(&[("name", name)])
            .into_response()
            .await?;

        Ok(())
    }

    /// Pause the container instance
    pub async fn pause(&self) -> Result<()> {
        self.http_client
            .post(&format!("/containers/{}/pause", self.id))
            .into_response()
            .await?;
        Ok(())
    }

    /// Unpause the container instance
    pub async fn unpause(&self) -> Result<()> {
        self.http_client
            .post(&format!("/containers/{}/unpause", self.id))
            .into_response()
            .await?;
        Ok(())
    }

    /// Wait until the container stops
    pub async fn wait(&self) -> Result<types::Exit> {
        self.http_client
            .post(&format!("/containers/{}/wait", self.id))
            .into_json()
            .await
    }

    /// Delete the container instance
    ///
    /// Use remove instead to use the force/v options.
    pub async fn delete(&self) -> Result<()> {
        self.http_client
            .delete(&format!("/containers/{}", self.id))
            .into_response()
            .await?;
        Ok(())
    }

    /// Delete the container instance (todo: force/v)
    pub async fn remove(
        &self,
        opts: builder::RmContainerOptions,
    ) -> Result<()> {
        let mut path = vec![format!("/containers/{}", self.id)];
        if let Some(query) = opts.serialize() {
            path.push(query)
        }
        self.http_client
            .delete(&path.join("?"))
            .into_response()
            .await?;
        Ok(())
    }

    async fn exec_create(
        &self,
        opts: &builder::ExecContainerOptions,
    ) -> Result<String> {
        #[derive(serde::Deserialize)]
        #[serde(rename_all = "PascalCase")]
        struct Response {
            id: String,
        }

        let body = opts.serialize()?;

        let Response { id } = self
            .http_client
            .post(&format!("/containers/{}/exec", self.id))
            .json_body(body)
            .into_json()
            .await?;

        Ok(id)
    }

    fn exec_start(
        &'a self,
        id: String,
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + 'a {
        let _bytes: &[u8] = b"{}";

        let stream = self
            .http_client
            .post(&format!("/exec/{}/start", id))
            .json_body(())
            .into_stream();

        tty::decode_chunks(stream)
    }

    pub fn exec(
        &'a self,
        opts: &'a builder::ExecContainerOptions,
    ) -> impl Stream<Item = Result<tty::TtyChunk>> + Unpin + 'a {
        Box::pin(
            async move {
                let id = self.exec_create(opts).await?;
                Ok(self.exec_start(id))
            }
            .try_flatten_stream(),
        )
    }

    /// Copy a file/folder from the container.  The resulting stream is a tarball of the extracted
    /// files.
    ///
    /// If `path` is not an absolute path, it is relative to the container’s root directory. The
    /// resource specified by `path` must exist. To assert that the resource is expected to be a
    /// directory, `path` should end in `/` or `/`. (assuming a path separator of `/`). If `path`
    /// ends in `/.`  then this indicates that only the contents of the path directory should be
    /// copied.  A symlink is always resolved to its target.
    pub fn copy_from(
        &'a self,
        path: &'a Path,
    ) -> impl Stream<Item = Result<Vec<u8>>> + 'a {
        let path_arg = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("path", &path.to_string_lossy())
            .finish();

        let endpoint = format!("/containers/{}/archive?{}", self.id, path_arg);
        self.http_client.get(&endpoint).into_stream()
    }

    /// Copy a byte slice as file into (see `bytes`) the container.
    ///
    /// The file will be copied at the given location (see `path`) and will be owned by root
    /// with access mask 644.
    pub async fn copy_file_into<P: AsRef<Path>>(
        &self,
        path: P,
        bytes: &[u8],
    ) -> Result<()> {
        let path = path.as_ref();

        let mut ar = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o0644);
        ar.append_data(
            &mut header,
            path.to_path_buf()
                .iter()
                .skip(1)
                .collect::<std::path::PathBuf>(),
            bytes,
        )
        .unwrap();
        let data = ar.into_inner().unwrap();

        self.http_client
            .put(&format!("/containers/{}/archive", self.id))
            .query(&[("path", "/")])
            .tar_body(data)
            .into_response()
            .await?;

        Ok(())
    }
}
