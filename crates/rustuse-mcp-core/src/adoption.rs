use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::CoreError;

const ADOPTION_PATHS_JSON: &str = include_str!("../../../data/rustuse-adoption-paths.json");

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RustUseAdoptionPaths {
    pub version: String,
    pub paths: Vec<RustUseAdoptionPath>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RustUseAdoptionPath {
    pub name: String,
    pub title: String,
    pub description: String,
    pub maintenance_notes: Vec<String>,
}

pub fn load_adoption_paths() -> Result<RustUseAdoptionPaths, CoreError> {
    serde_json::from_str(ADOPTION_PATHS_JSON).map_err(|source| CoreError::EmbeddedJson {
        name: "rustuse-adoption-paths.json",
        source,
    })
}
