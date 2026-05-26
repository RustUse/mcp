use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::CoreError;

const CATALOG_JSON: &str = include_str!("../../../data/rustuse-catalog.json");

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RustUseCatalog {
    pub version: String,
    pub sets: Vec<RustUseSet>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RustUseSet {
    pub name: String,
    pub kind: String,
    pub status: String,
    pub description: String,
    pub repo_path: String,
    pub children: Vec<RustUseCrate>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RustUseCrate {
    pub name: String,
    pub kind: String,
    pub description: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchMatch {
    pub name: String,
    pub item_type: String,
    pub parent_set: Option<String>,
    pub description: String,
    pub score: u16,
    pub resource_uri: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct CatalogSearchResult {
    pub matches: Vec<SearchMatch>,
    pub total: usize,
    pub summary: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct CrateLookupResult {
    pub found: bool,
    pub crate_metadata: Option<RustUseCrate>,
    pub parent_set: Option<String>,
    pub resource_uri: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ChildrenResult {
    pub found: bool,
    pub set_name: String,
    pub children: Vec<RustUseCrate>,
    pub count: usize,
    pub message: String,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NameCollisionKind {
    Set,
    Crate,
    #[default]
    Any,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct NameCollisionReport {
    pub name: String,
    pub kind: NameCollisionKind,
    pub collision: bool,
    pub exact_matches: Vec<String>,
    pub similar_names: Vec<String>,
    pub recommendation: String,
}

pub fn load_catalog() -> Result<RustUseCatalog, CoreError> {
    serde_json::from_str(CATALOG_JSON).map_err(|source| CoreError::EmbeddedJson {
        name: "rustuse-catalog.json",
        source,
    })
}

impl RustUseCatalog {
    #[must_use]
    pub fn set(&self, set_name: &str) -> Option<&RustUseSet> {
        let normalized = normalize_name(set_name);
        self.sets
            .iter()
            .find(|set| normalize_name(&set.name) == normalized)
    }

    #[must_use]
    pub fn set_names(&self) -> Vec<String> {
        self.sets.iter().map(|set| set.name.clone()).collect()
    }

    #[must_use]
    pub fn crate_lookup(&self, crate_name: &str) -> CrateLookupResult {
        let normalized = normalize_name(crate_name);
        for set in &self.sets {
            if let Some(crate_metadata) = set
                .children
                .iter()
                .find(|child| normalize_name(&child.name) == normalized)
            {
                return CrateLookupResult {
                    found: true,
                    crate_metadata: Some(crate_metadata.clone()),
                    parent_set: Some(set.name.clone()),
                    resource_uri: Some(format!("rustuse://crates/{}", crate_metadata.name)),
                    message: format!("Found crate {} in set {}.", crate_metadata.name, set.name),
                };
            }
        }

        CrateLookupResult {
            found: false,
            crate_metadata: None,
            parent_set: None,
            resource_uri: None,
            message: format!("No crate named {crate_name} is present in the static v0.1 catalog."),
        }
    }

    #[must_use]
    pub fn list_children(&self, set_name: &str) -> ChildrenResult {
        self.set(set_name).map_or_else(
            || ChildrenResult {
                found: false,
                set_name: set_name.to_owned(),
                children: Vec::new(),
                count: 0,
                message: format!("No set named {set_name} is present in the static v0.1 catalog."),
            },
            |set| ChildrenResult {
                found: true,
                set_name: set.name.clone(),
                children: set.children.clone(),
                count: set.children.len(),
                message: format!("{} has {} child crates.", set.name, set.children.len()),
            },
        )
    }

    #[must_use]
    pub fn search(&self, query: &str, limit: usize) -> CatalogSearchResult {
        let normalized_query = normalize_query(query);
        let effective_limit = limit.clamp(1, 50);
        let mut matches = Vec::new();

        for set in &self.sets {
            if let Some(score) =
                score_match(&normalized_query, &set.name, &set.description, &set.notes)
            {
                matches.push(SearchMatch {
                    name: set.name.clone(),
                    item_type: "set".to_owned(),
                    parent_set: None,
                    description: set.description.clone(),
                    score,
                    resource_uri: format!("rustuse://sets/{}", set.name),
                });
            }

            for child in &set.children {
                if let Some(score) =
                    score_match(&normalized_query, &child.name, &child.description, &[])
                {
                    matches.push(SearchMatch {
                        name: child.name.clone(),
                        item_type: "crate".to_owned(),
                        parent_set: Some(set.name.clone()),
                        description: child.description.clone(),
                        score,
                        resource_uri: format!("rustuse://crates/{}", child.name),
                    });
                }
            }
        }

        matches.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.name.cmp(&right.name))
        });

        let total = matches.len();
        matches.truncate(effective_limit);

        CatalogSearchResult {
            summary: if total == 0 {
                format!("No RustUse catalog matches found for '{query}'.")
            } else {
                format!("Found {total} RustUse catalog matches for '{query}'.")
            },
            matches,
            total,
        }
    }

    #[must_use]
    pub fn check_name_collision(&self, name: &str, kind: NameCollisionKind) -> NameCollisionReport {
        let normalized = normalize_name(name);
        let mut exact_matches = Vec::new();
        let mut similar_names = Vec::new();

        if matches_kind(kind, NameCollisionKind::Set) {
            for set in &self.sets {
                collect_name_match(
                    &set.name,
                    &normalized,
                    &mut exact_matches,
                    &mut similar_names,
                );
            }
        }

        if matches_kind(kind, NameCollisionKind::Crate) {
            for set in &self.sets {
                for child in &set.children {
                    collect_name_match(
                        &child.name,
                        &normalized,
                        &mut exact_matches,
                        &mut similar_names,
                    );
                }
            }
        }

        exact_matches.sort();
        exact_matches.dedup();
        similar_names.sort();
        similar_names.dedup();

        let collision = !exact_matches.is_empty();
        let recommendation = if collision {
            "Choose a different name; exact or normalized RustUse catalog collisions should be avoided."
        } else if similar_names.is_empty() {
            "No obvious static catalog collision found for v0.1."
        } else {
            "Review similar names before proceeding to avoid taxonomy overlap."
        }
        .to_owned();

        NameCollisionReport {
            name: name.to_owned(),
            kind,
            collision,
            exact_matches,
            similar_names,
            recommendation,
        }
    }
}

#[must_use]
pub fn normalize_name(name: &str) -> String {
    name.trim().to_ascii_lowercase().replace('_', "-")
}

#[must_use]
pub fn name_tokens(name: &str) -> Vec<String> {
    let normalized = normalize_name(name);
    let without_prefix = normalized.strip_prefix("use-").unwrap_or(&normalized);
    without_prefix
        .split('-')
        .filter(|token| !token.is_empty() && *token != "use")
        .map(ToOwned::to_owned)
        .collect()
}

fn normalize_query(query: &str) -> String {
    query.trim().to_ascii_lowercase().replace('_', "-")
}

fn score_match(query: &str, name: &str, description: &str, notes: &[String]) -> Option<u16> {
    if query.is_empty() {
        return Some(1);
    }

    let normalized_name = normalize_name(name);
    let normalized_description = description.to_ascii_lowercase();

    if normalized_name == query {
        return Some(100);
    }
    if normalized_name.contains(query) {
        return Some(80);
    }
    if normalized_description.contains(query) {
        return Some(50);
    }
    if notes
        .iter()
        .any(|note| note.to_ascii_lowercase().contains(query))
    {
        return Some(25);
    }

    None
}

fn matches_kind(requested: NameCollisionKind, candidate: NameCollisionKind) -> bool {
    requested == NameCollisionKind::Any || requested == candidate
}

fn collect_name_match(
    catalog_name: &str,
    normalized_input: &str,
    exact_matches: &mut Vec<String>,
    similar_names: &mut Vec<String>,
) {
    let normalized_catalog_name = normalize_name(catalog_name);
    if normalized_catalog_name == normalized_input {
        exact_matches.push(catalog_name.to_owned());
        return;
    }

    if normalized_catalog_name.contains(normalized_input)
        || normalized_input.contains(&normalized_catalog_name)
        || shares_name_token(&normalized_catalog_name, normalized_input)
    {
        similar_names.push(catalog_name.to_owned());
    }
}

fn shares_name_token(left: &str, right: &str) -> bool {
    let left_tokens = name_tokens(left);
    let right_tokens = name_tokens(right);
    left_tokens.iter().any(|left_token| {
        right_tokens
            .iter()
            .any(|right_token| left_token == right_token)
    })
}
