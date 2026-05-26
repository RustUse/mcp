use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::CoreError;

const BRAINSTORM_SET: &str = include_str!("../../../prompts/brainstorm-set.md");
const CREATE_SET: &str = include_str!("../../../prompts/create-set-v0.1.md");
const CREATE_CHILD_CRATE: &str = include_str!("../../../prompts/create-child-crate.md");
const VALIDATE_SET_PLAN: &str = include_str!("../../../prompts/validate-set-plan.md");
const GENERATE_COPILOT_PLAN: &str = include_str!("../../../prompts/generate-copilot-plan.md");
const AUDIT_OVERLAP: &str = include_str!("../../../prompts/audit-overlap.md");
const GENERATE_DOCS_PAGE: &str = include_str!("../../../prompts/generate-docs-page.md");

#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct PromptRenderRequest {
    pub prompt_name: String,
    pub set_name: Option<String>,
    pub crate_name: Option<String>,
    pub description: Option<String>,
    pub minimum_children: Option<usize>,
    pub proposed_children: Option<Vec<String>>,
    pub parent_set: Option<String>,
    pub include_docs: Option<bool>,
    pub include_ci: Option<bool>,
    pub include_examples: Option<bool>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct PromptRenderResult {
    pub prompt_name: String,
    pub title: String,
    pub text: String,
    pub assumptions: Vec<String>,
    pub referenced_rules: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct GeneratedCopilotPrompt {
    pub prompt: String,
    pub assumptions: Vec<String>,
    pub referenced_rules: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct PromptTemplate {
    pub name: String,
    pub title: String,
    pub description: String,
    pub arguments: Vec<PromptArgument>,
    #[serde(skip)]
    #[schemars(skip)]
    pub template: &'static str,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
}

#[must_use]
pub fn list_prompt_templates() -> Vec<PromptTemplate> {
    vec![
        template(
            "rustuse_brainstorm_set",
            "Brainstorm RustUse Set",
            "Brainstorm a new RustUse facade set and candidate child crates.",
            BRAINSTORM_SET,
            &["set_name"],
        ),
        template(
            "rustuse_create_set_v0_1",
            "Create RustUse Set v0.1",
            "Generate an implementation prompt for a minimal RustUse facade set.",
            CREATE_SET,
            &["set_name"],
        ),
        template(
            "rustuse_create_child_crate",
            "Create RustUse Child Crate",
            "Generate an implementation prompt for one focused RustUse child crate.",
            CREATE_CHILD_CRATE,
            &["set_name", "crate_name"],
        ),
        template(
            "rustuse_validate_facade",
            "Validate RustUse Facade",
            "Validate a RustUse facade plan against project rules.",
            VALIDATE_SET_PLAN,
            &["set_name"],
        ),
        template(
            "rustuse_audit_overlap",
            "Audit RustUse Overlap",
            "Audit a proposed set or crate for taxonomy overlap.",
            AUDIT_OVERLAP,
            &["set_name"],
        ),
        template(
            "rustuse_generate_copilot_plan",
            "Generate Copilot Plan",
            "Generate a ready-to-paste GitHub Copilot Plan Mode prompt.",
            GENERATE_COPILOT_PLAN,
            &["set_name"],
        ),
        template(
            "rustuse_generate_docs_page",
            "Generate RustUse Docs Page",
            "Generate concise RustUse documentation for a set or crate.",
            GENERATE_DOCS_PAGE,
            &["set_name"],
        ),
    ]
}

pub fn render_prompt(request: &PromptRenderRequest) -> Result<PromptRenderResult, CoreError> {
    let template = list_prompt_templates()
        .into_iter()
        .find(|candidate| candidate.name == request.prompt_name)
        .ok_or_else(|| CoreError::UnknownPrompt(request.prompt_name.clone()))?;

    for argument in &template.arguments {
        if argument.required && argument_value(request, &argument.name).is_none() {
            return Err(CoreError::MissingPromptArgument(argument.name.clone()));
        }
    }

    let values = prompt_values(request);
    let rendered = values
        .iter()
        .fold(template.template.to_owned(), |text, (key, value)| {
            text.replace(&format!("{{{{{key}}}}}"), value)
        });

    Ok(PromptRenderResult {
        prompt_name: template.name,
        title: template.title,
        text: rendered,
        assumptions: assumptions(request),
        referenced_rules: referenced_rules(),
    })
}

pub fn generate_copilot_prompt(
    set_name: &str,
    description: Option<&str>,
    minimum_children: Option<usize>,
    proposed_children: &[String],
    include_docs: Option<bool>,
    include_ci: Option<bool>,
) -> Result<GeneratedCopilotPrompt, CoreError> {
    let request = PromptRenderRequest {
        prompt_name: "rustuse_generate_copilot_plan".to_owned(),
        set_name: Some(set_name.to_owned()),
        description: description.map(ToOwned::to_owned),
        minimum_children,
        proposed_children: Some(proposed_children.to_vec()),
        include_docs,
        include_ci,
        ..PromptRenderRequest::default()
    };
    let result = render_prompt(&request)?;

    Ok(GeneratedCopilotPrompt {
        prompt: result.text,
        assumptions: result.assumptions,
        referenced_rules: result.referenced_rules,
    })
}

fn template(
    name: &str,
    title: &str,
    description: &str,
    template_text: &'static str,
    required_arguments: &[&str],
) -> PromptTemplate {
    let known_arguments = [
        ("set_name", "RustUse set name"),
        ("crate_name", "RustUse child crate name"),
        ("description", "Short description"),
        ("minimum_children", "Minimum number of child crates"),
        ("proposed_children", "Proposed child crate names"),
        ("parent_set", "Parent set name"),
        ("include_docs", "Whether docs should be included"),
        ("include_ci", "Whether CI should be included"),
        ("include_examples", "Whether examples should be included"),
    ];

    PromptTemplate {
        name: name.to_owned(),
        title: title.to_owned(),
        description: description.to_owned(),
        arguments: known_arguments
            .iter()
            .map(|(argument_name, argument_description)| PromptArgument {
                name: (*argument_name).to_owned(),
                description: (*argument_description).to_owned(),
                required: required_arguments.contains(argument_name),
            })
            .collect(),
        template: template_text,
    }
}

fn prompt_values(request: &PromptRenderRequest) -> BTreeMap<&'static str, String> {
    BTreeMap::from([
        (
            "set_name",
            request
                .set_name
                .clone()
                .unwrap_or_else(|| "use-example".to_owned()),
        ),
        (
            "crate_name",
            request
                .crate_name
                .clone()
                .unwrap_or_else(|| "use-example-child".to_owned()),
        ),
        (
            "description",
            request
                .description
                .clone()
                .unwrap_or_else(|| "A focused RustUse primitive set.".to_owned()),
        ),
        (
            "minimum_children",
            request.minimum_children.unwrap_or(10).to_string(),
        ),
        (
            "proposed_children",
            request
                .proposed_children
                .as_ref()
                .filter(|children| !children.is_empty())
                .map_or_else(|| "not provided".to_owned(), |children| children.join(", ")),
        ),
        (
            "parent_set",
            request
                .parent_set
                .clone()
                .unwrap_or_else(|| "not provided".to_owned()),
        ),
        (
            "include_docs",
            request.include_docs.unwrap_or(true).to_string(),
        ),
        ("include_ci", request.include_ci.unwrap_or(true).to_string()),
        (
            "include_examples",
            request.include_examples.unwrap_or(true).to_string(),
        ),
    ])
}

fn argument_value(request: &PromptRenderRequest, argument_name: &str) -> Option<String> {
    match argument_name {
        "set_name" => request.set_name.clone(),
        "crate_name" => request.crate_name.clone(),
        "description" => request.description.clone(),
        "minimum_children" => request.minimum_children.map(|value| value.to_string()),
        "proposed_children" => request
            .proposed_children
            .as_ref()
            .map(|value| value.join(", ")),
        "parent_set" => request.parent_set.clone(),
        "include_docs" => request.include_docs.map(|value| value.to_string()),
        "include_ci" => request.include_ci.map(|value| value.to_string()),
        "include_examples" => request.include_examples.map(|value| value.to_string()),
        _ => None,
    }
}

fn assumptions(request: &PromptRenderRequest) -> Vec<String> {
    let mut assumptions = Vec::new();
    if request.description.is_none() {
        assumptions.push(
            "Description was not provided; prompt uses a generic RustUse primitive boundary."
                .to_owned(),
        );
    }
    if request.minimum_children.is_none() {
        assumptions.push("Minimum child crate target defaults to 10.".to_owned());
    }
    if request.include_docs.is_none() {
        assumptions.push("Documentation is included by default.".to_owned());
    }
    if request.include_ci.is_none() {
        assumptions.push("CI is included by default.".to_owned());
    }
    assumptions
}

fn referenced_rules() -> Vec<String> {
    vec![
        "rust-edition-2024".to_owned(),
        "facade-reexports".to_owned(),
        "child-implementation".to_owned(),
        "primitive-utilities".to_owned(),
        "few-dependencies".to_owned(),
        "dual-license".to_owned(),
        "taxonomy-hygiene".to_owned(),
    ]
}
