use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateInfo {
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Volumes {
    pub volumes: Option<Vec<Volume>>,
    pub warnings: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Volume {
    #[cfg(feature = "chrono")]
    pub created_at: DateTime<Utc>,
    #[cfg(not(feature = "chrono"))]
    pub created_at: String,
    pub driver: String,
    pub labels: Option<HashMap<String, String>>,
    pub name: String,
    pub mountpoint: String,
    pub options: Option<HashMap<String, String>>,
    pub scope: String,
}
