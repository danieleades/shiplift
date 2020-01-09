use crate::{http_client::HttpClient, Result};
use serde::Serialize;
use futures_util::future::BoxFuture;
use std::future::Future;
use std::pin::Pin;
use std::task::Context;
use std::task::Poll;

pub struct Builder<'a> {
    http_client: &'a HttpClient,
    query: Query,
    future: Option<BoxFuture<'a, Result<Vec<rep::Container>>>>,
}

type Response = Result<Vec<rep::Container>>;

impl<'a> Builder<'a> {
    pub(crate) fn new(http_client: &'a HttpClient) -> Self {
        let query = Query::default();
        let future = None;

        Self { http_client, query, future }
    }
}

impl<'a> Future for Builder<'a> {
    type Output = Response;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Construct a future once, and store it internally.
        if self.future.is_none() {
            self.future = Some(Box::pin(
                self.http_client
            .get("/containers/json")
            .query(&self.query)
            .into_json()
            ))
        }

        // Now that we have an inner future, we can forward all poll calls to
        // it until it resolves.
        self.future.as_mut().unwrap().as_mut().poll(cx)
    }
}

#[derive(Default, Serialize)]
struct Query {
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    all: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<i32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    since: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    before: Option<String>,

    #[serde(skip_serializing_if = "std::ops::Not::not")]
    size: bool,
    //filters: Filters,
}

mod rep {
    use chrono::{serde::ts_seconds::deserialize as from_ts, DateTime, Utc};
    use serde::Deserialize;
    use std::collections::HashMap;

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct Container {
        #[cfg(feature = "chrono")]
        #[serde(deserialize_with = "from_ts")]
        pub created: DateTime<Utc>,
        #[cfg(not(feature = "chrono"))]
        pub created: u64,
        pub command: String,
        pub id: String,
        pub image: String,
        pub labels: HashMap<String, String>,
        pub names: Vec<String>,
        pub ports: Vec<Port>,
        pub status: String,
        pub size_rw: Option<u64>,
        pub size_root_fs: Option<u64>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub struct Port {
        pub ip: Option<String>,
        pub private_port: u64,
        pub public_port: Option<u64>,
        pub r#type: String,
    }
}

/* #[derive(Default, Serialize)]
pub struct Filters {
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    exited: HashSet<u32>,

    #[serde(skip_serializing_if = "HashSet::is_empty")]
    status: HashSet<Status>,

    #[serde(skip_serializing_if = "HashSet::is_empty")]
    label: HashSet<String>,

    #[serde(skip_serializing_if = "HashSet::is_empty")]
    isolation: HashSet<Isolation>,

    #[serde(skip_serializing_if = "Option::is_none")]
    before: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<String>,
}

impl Filters {
    fn insert(&mut self, filter: Filter) {
        match filter {
            Filter::Exited(exit_code) => self.exited.insert(exit_code),
            Filter::Status(status) => self.status.insert(status),
            //Filter::Label
            Filter::Isolation(isolation) => self.isolation.insert(isolation),
            Filter::Before(id) => self.before = Some(id),
            Filter::After(id) => self.after = Some(id),
        }
    }
}

/// Filter options for container listings
pub enum Filter {
    Exited(u32),
    Status(Status),
    Label(String),
    Isolation(Isolation),
    Before(String),
    After(String)
}

impl Filter {
    pub fn exit_code(code: u32) -> Self {
        Self::Exited(code)
    }
}

#[derive(Serialize)]
#[serde(untagged, rename_all="lowercase")]
pub enum Status {
    Created,
    Restarting,
    Running,
    Paused,
    Exited,
    Dead,
}

#[derive(Serialize)]
#[serde(untagged, rename_all="lowercase")]
pub enum Isolation {
    Default,
    Process,
    #[cfg(windows)]
    Hyperv
} */
