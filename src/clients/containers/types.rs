use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CreateInfo {
    pub id: String,
    pub warnings: Option<Vec<String>>,
}
