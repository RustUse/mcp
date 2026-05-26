use std::collections::HashSet;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    catalog::{NameCollisionKind, RustUseCatalog, normalize_name},
    overlap::find_overlap,
    rules::RustUseRuleSet,
};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ValidationReport {
    pub valid: bool,
    pub severity: String,
    pub score: u8,
    pub findings: Vec<ValidationFinding>,
    pub recommendations: Vec<String>,
    pub rule_checks: Vec<RuleCheck>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct ValidationFinding {
    pub rule_id: String,
    pub severity: String,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct RuleCheck {
    pub rule_id: String,
    pub passed: bool,
    pub message: String,
}

#[must_use]
#[allow(clippy::too_many_lines)]
pub fn validate_set_plan(
    catalog: &RustUseCatalog,
    rules: &RustUseRuleSet,
    set_name: &str,
    description: &str,
    proposed_children: &[String],
) -> ValidationReport {
    let mut findings = Vec::new();
    let mut recommendations = Vec::new();
    let mut rule_checks = Vec::new();

    push_rule_check(
        &mut rule_checks,
        "rust-edition-2024",
        true,
        "Rust edition 2024 is required for any implementation plan.",
    );

    if !is_rustuse_name(set_name) {
        findings.push(ValidationFinding {
            rule_id: "taxonomy-hygiene".to_owned(),
            severity: "error".to_owned(),
            message: "Set names should be ASCII-safe, lowercase, hyphenated, and start with use-."
                .to_owned(),
        });
    }

    let collision = catalog.check_name_collision(set_name, NameCollisionKind::Any);
    if collision.collision {
        findings.push(ValidationFinding {
            rule_id: "taxonomy-hygiene".to_owned(),
            severity: "error".to_owned(),
            message: format!(
                "Name collides with existing RustUse catalog entries: {}.",
                collision.exact_matches.join(", ")
            ),
        });
    } else if !collision.similar_names.is_empty() {
        findings.push(ValidationFinding {
            rule_id: "taxonomy-hygiene".to_owned(),
            severity: "warning".to_owned(),
            message: format!(
                "Review similar RustUse names: {}.",
                collision.similar_names.join(", ")
            ),
        });
    }

    if description.trim().len() < 20 {
        findings.push(ValidationFinding {
            rule_id: "primitive-utilities".to_owned(),
            severity: "warning".to_owned(),
            message: "Description is too short to establish a clear primitive boundary.".to_owned(),
        });
    }

    if proposed_children.is_empty() {
        findings.push(ValidationFinding {
            rule_id: "child-implementation".to_owned(),
            severity: "error".to_owned(),
            message: "A facade set plan needs focused child crates.".to_owned(),
        });
    } else if proposed_children.len() < 3 {
        findings.push(ValidationFinding {
            rule_id: "child-implementation".to_owned(),
            severity: "warning".to_owned(),
            message: "Consider whether the set has enough focused primitives to justify a facade."
                .to_owned(),
        });
    }

    let duplicate_children = duplicate_normalized_names(proposed_children);
    if !duplicate_children.is_empty() {
        findings.push(ValidationFinding {
            rule_id: "taxonomy-hygiene".to_owned(),
            severity: "error".to_owned(),
            message: format!(
                "Duplicate proposed child crate names: {}.",
                duplicate_children.join(", ")
            ),
        });
    }

    let non_rustuse_children: Vec<String> = proposed_children
        .iter()
        .filter(|child_name| !is_rustuse_name(child_name))
        .cloned()
        .collect();
    if !non_rustuse_children.is_empty() {
        findings.push(ValidationFinding {
            rule_id: "taxonomy-hygiene".to_owned(),
            severity: "warning".to_owned(),
            message: format!(
                "Child crate names should be lowercase use-* names: {}.",
                non_rustuse_children.join(", ")
            ),
        });
    }

    let overlap = find_overlap(catalog, set_name, Some(description), proposed_children);
    if overlap.severity == "high" {
        findings.push(ValidationFinding {
            rule_id: "taxonomy-hygiene".to_owned(),
            severity: "error".to_owned(),
            message: format!("High overlap risk: {}", overlap.recommendation),
        });
    } else if overlap.severity == "medium" {
        findings.push(ValidationFinding {
            rule_id: "taxonomy-hygiene".to_owned(),
            severity: "warning".to_owned(),
            message: format!("Medium overlap risk: {}", overlap.recommendation),
        });
    }

    recommendations.push(
        "Keep implementation in child crates and facade logic limited to re-exports.".to_owned(),
    );
    recommendations.push(
        "Add README, CHANGELOG, tests, docs, and examples proportional to the public API."
            .to_owned(),
    );
    recommendations.push(
        "Document crates.io, copy-and-own, and future CLI-assisted adoption paths.".to_owned(),
    );

    for rule in &rules.rules {
        if !rule_checks.iter().any(|check| check.rule_id == rule.id) {
            push_rule_check(&mut rule_checks, &rule.id, true, &rule.guidance);
        }
    }

    let error_count = findings
        .iter()
        .filter(|finding| finding.severity == "error")
        .count();
    let warning_count = findings
        .iter()
        .filter(|finding| finding.severity == "warning")
        .count();
    let score_penalty = (error_count * 30 + warning_count * 10).min(100);
    let score = 100_u8.saturating_sub(u8::try_from(score_penalty).unwrap_or(100));
    let severity = if error_count > 0 {
        "error"
    } else if warning_count > 0 {
        "warning"
    } else {
        "ok"
    }
    .to_owned();

    ValidationReport {
        valid: error_count == 0,
        severity,
        score,
        findings,
        recommendations,
        rule_checks,
    }
}

fn is_rustuse_name(name: &str) -> bool {
    let trimmed = name.trim();
    trimmed.starts_with("use-")
        && trimmed.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

fn duplicate_normalized_names(names: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut duplicate_names = Vec::new();
    for name in names {
        let normalized = normalize_name(name);
        if !seen.insert(normalized) {
            duplicate_names.push(name.clone());
        }
    }
    duplicate_names.sort();
    duplicate_names.dedup();
    duplicate_names
}

fn push_rule_check(rule_checks: &mut Vec<RuleCheck>, rule_id: &str, passed: bool, message: &str) {
    rule_checks.push(RuleCheck {
        rule_id: rule_id.to_owned(),
        passed,
        message: message.to_owned(),
    });
}
