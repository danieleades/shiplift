use super::{BodyType, ShipliftOption};
use hyper::{Method};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Interface for creating new docker network
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct Options<'a> {
    name: &'a str,
    check_duplicate: bool,
    driver: Driver,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    labels: HashMap<&'a str, &'a str>,
}

impl<'a> Options<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            check_duplicate: false,
            driver: Driver::Bridge,
            labels: HashMap::default(),
        }
    }

    pub fn label(
        mut self,
        key: &'a str,
        value: &'a str,
    ) -> Self {
        self.labels.insert(key, value);
        self
    }

    pub fn driver(
        mut self,
        driver: Driver,
    ) -> Self {
        self.driver = driver;
        self
    }
}

impl<'a> ShipliftOption for Options<'a> {
    fn method(&self) -> Method {
        Method::POST
    }
    fn endpoint(&self) -> String {
        "/networks/create".to_string()
    }
    fn body(&self) -> Option<BodyType> {
        let body = serde_json::to_vec(self).unwrap().into();

        Some(BodyType::Json(body))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Driver {
    Bridge,
}
