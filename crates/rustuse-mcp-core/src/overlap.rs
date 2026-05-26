use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::catalog::{name_tokens, normalize_name, RustUseCatalog};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct OverlapReport {
    pub proposed_name: String,
    pub likely_overlaps: Vec<String>,
    pub severity: String,
    pub findings: Vec<OverlapFinding>,
    pub recommendation: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct OverlapFinding {
    pub target: String,
    pub reason: String,
    pub score: u16,
}

#[must_use]
pub fn find_overlap(
    catalog: &RustUseCatalog,
    proposed_name: &str,
    description: Option<&str>,
    proposed_children: &[String],
) -> OverlapReport {
    let normalized_name = normalize_name(proposed_name);
    let proposed_tokens = name_tokens(proposed_name);
    let description_text = description.unwrap_or_default().to_ascii_lowercase();
    let normalized_children: Vec<String> = proposed_children
        .iter()
        .map(|child_name| normalize_name(child_name))
        .collect();
    let mut findings = Vec::new();

    for set in &catalog.sets {
        let normalized_set = normalize_name(&set.name);
        if normalized_set == normalized_name {
            findings.push(OverlapFinding {
                target: set.name.clone(),
                reason: "Exact or hyphen/underscore-normalized set name collision.".to_owned(),
                score: 100,
            });
        }

        let set_tokens = name_tokens(&set.name);
        let shared_tokens = shared_token_count(&proposed_tokens, &set_tokens);
        if shared_tokens > 0 {
            findings.push(OverlapFinding {
                target: set.name.clone(),
                reason: format!("Shares {shared_tokens} naming token(s) with an existing set."),
                score: 25 + score_bonus(shared_tokens, 10),
            });
        }

        if !description_text.is_empty()
            && set_tokens
                .iter()
                .any(|token| token.len() > 2 && description_text.contains(token))
        {
            findings.push(OverlapFinding {
                target: set.name.clone(),
                reason: "Description references an existing set boundary.".to_owned(),
                score: 35,
            });
        }

        let child_overlap = set
            .children
            .iter()
            .filter(|child| normalized_children.contains(&normalize_name(&child.name)))
            .count();
        if child_overlap > 0 {
            findings.push(OverlapFinding {
                target: set.name.clone(),
                reason: format!("Shares {child_overlap} proposed child crate name(s)."),
                score: 50 + score_bonus(child_overlap, 10),
            });
        }
    }

    findings.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.target.cmp(&right.target))
    });

    let likely_overlaps = findings
        .iter()
        .filter(|finding| finding.score >= 45)
        .map(|finding| finding.target.clone())
        .fold(Vec::new(), |mut unique, target| {
            if !unique.contains(&target) {
                unique.push(target);
            }
            unique
        });

    let max_score = findings.first().map_or(0, |finding| finding.score);
    let severity = match max_score {
        90.. => "high",
        50..=89 => "medium",
        1..=49 => "low",
        _ => "none",
    }
    .to_owned();

    let recommendation = match severity.as_str() {
        "high" => "Do not proceed until the name or ownership boundary is changed.",
        "medium" => "Revise the plan and document the boundary against likely overlaps.",
        "low" => "Proceed with a short taxonomy note explaining the distinction.",
        _ => "No obvious overlap in the static v0.1 catalog.",
    }
    .to_owned();

    OverlapReport {
        proposed_name: proposed_name.to_owned(),
        likely_overlaps,
        severity,
        findings,
        recommendation,
    }
}

fn shared_token_count(left: &[String], right: &[String]) -> usize {
    left.iter()
        .filter(|left_token| right.iter().any(|right_token| right_token == *left_token))
        .count()
}

fn score_bonus(count: usize, multiplier: u16) -> u16 {
    u16::try_from(count).map_or(u16::MAX, |value| value.saturating_mul(multiplier))
}
