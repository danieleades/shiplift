use super::{BodyType, ShipliftOption};
use hyper::Method;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Options<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<&'a str>,

    driver: Option<Driver>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    driver_opts: HashMap<&'a str, &'a str>,

    #[serde(skip_serializing_if = "HashMap::is_empty")]
    labels: HashMap<&'a str, &'a str>,
}

impl<'a> Options<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(
        mut self,
        name: &'a str,
    ) -> Self {
        self.name = Some(name);
        self
    }

    pub fn label(
        mut self,
        key: &'a str,
        value: &'a str,
    ) -> Self {
        self.labels.insert(key, value);
        self
    }
}

impl<'a> ShipliftOption for Options<'a> {
    fn method(&self) -> Method {
        Method::POST
    }
    fn endpoint(&self) -> String {
        "/volumes/create".to_string()
    }
    fn body(&self) -> Option<BodyType> {
        let body = serde_json::to_vec(self).unwrap().into();

        Some(BodyType::Json(body))
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Driver {
    Local,
}
