use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::CoreError;

const RULES_JSON: &str = include_str!("../../../data/rustuse-rules.json");

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RustUseRuleSet {
    pub version: String,
    pub rules: Vec<RustUseRule>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RustUseRule {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub guidance: String,
}

pub fn load_rules() -> Result<RustUseRuleSet, CoreError> {
    serde_json::from_str(RULES_JSON).map_err(|source| CoreError::EmbeddedJson {
        name: "rustuse-rules.json",
        source,
    })
}
