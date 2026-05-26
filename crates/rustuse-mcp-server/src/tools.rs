use rmcp::{
    ErrorData as McpError,
    handler::server::tool::schema_for_type,
    model::{CallToolResult, Content, JsonObject, Tool, ToolAnnotations},
};
use rustuse_mcp_core::{NameCollisionKind, generate_copilot_prompt, validate_set_plan};
use schemars::JsonSchema;
use serde::{Deserialize, de::DeserializeOwned};
use serde_json::{Map, Value, json};

use crate::server::RustUseMcpServer;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CatalogSearchInput {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetSetInput {
    pub set_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCrateInput {
    pub crate_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListChildrenInput {
    pub set_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NameCollisionInput {
    pub name: String,
    pub kind: Option<NameCollisionKind>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindOverlapInput {
    pub proposed_name: String,
    pub description: Option<String>,
    pub proposed_children: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateSetPlanInput {
    pub set_name: String,
    pub description: String,
    pub proposed_children: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GenerateCopilotPromptInput {
    pub set_name: String,
    pub description: Option<String>,
    pub minimum_children: Option<usize>,
    pub proposed_children: Option<Vec<String>>,
    pub include_docs: Option<bool>,
    pub include_ci: Option<bool>,
}

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    vec![
        tool::<CatalogSearchInput>(
            "rustuse_catalog_search",
            "RustUse catalog search",
            "Search RustUse sets and child crates in the static v0.1 catalog.",
        ),
        tool::<GetSetInput>(
            "rustuse_get_set",
            "Get RustUse set",
            "Get metadata, children, and notes for a RustUse set.",
        ),
        tool::<GetCrateInput>(
            "rustuse_get_crate",
            "Get RustUse crate",
            "Get child crate metadata and its parent set if known.",
        ),
        tool::<ListChildrenInput>(
            "rustuse_list_children",
            "List RustUse children",
            "List child crates for a RustUse set.",
        ),
        tool::<NameCollisionInput>(
            "rustuse_check_name_collision",
            "Check RustUse name collision",
            "Check exact and similar name collisions against the static catalog.",
        ),
        tool::<FindOverlapInput>(
            "rustuse_find_overlap",
            "Find RustUse overlap",
            "Find likely set or crate taxonomy overlap for a proposal.",
        ),
        tool::<ValidateSetPlanInput>(
            "rustuse_validate_set_plan",
            "Validate RustUse set plan",
            "Validate a proposed RustUse facade set plan against project rules.",
        ),
        tool::<GenerateCopilotPromptInput>(
            "rustuse_generate_copilot_prompt",
            "Generate Copilot prompt",
            "Generate a ready-to-paste GitHub Copilot Plan Mode prompt for RustUse work.",
        ),
    ]
}

pub fn call_tool(
    server: &RustUseMcpServer,
    name: &str,
    arguments: Option<JsonObject>,
) -> Result<CallToolResult, McpError> {
    match name {
        "rustuse_catalog_search" => catalog_search(server, arguments),
        "rustuse_get_set" => get_set(server, arguments),
        "rustuse_get_crate" => get_crate(server, arguments),
        "rustuse_list_children" => list_children(server, arguments),
        "rustuse_check_name_collision" => check_name_collision(server, arguments),
        "rustuse_find_overlap" => find_overlap(server, arguments),
        "rustuse_validate_set_plan" => validate_plan(server, arguments),
        "rustuse_generate_copilot_prompt" => generate_prompt(arguments),
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: {name}"),
            None,
        )),
    }
}

fn catalog_search(
    server: &RustUseMcpServer,
    arguments: Option<JsonObject>,
) -> Result<CallToolResult, McpError> {
    let input = parse_input::<CatalogSearchInput>(arguments)?;
    if input.query.trim().is_empty() {
        return Ok(tool_error(
            "Catalog search query must not be empty.",
            "empty_query",
            json!({ "query": input.query }),
        ));
    }
    let limit = input.limit.unwrap_or(10);
    if !(1..=50).contains(&limit) {
        return Ok(tool_error(
            "Catalog search limit must be between 1 and 50.",
            "invalid_limit",
            json!({ "limit": limit }),
        ));
    }

    let result = server.catalog.search(&input.query, limit);
    Ok(tool_success(&result.summary, json!(result)))
}

fn get_set(
    server: &RustUseMcpServer,
    arguments: Option<JsonObject>,
) -> Result<CallToolResult, McpError> {
    let input = parse_input::<GetSetInput>(arguments)?;
    let Some(set) = server.catalog.set(&input.set_name) else {
        return Ok(tool_error(
            &format!(
                "No RustUse set named {} is present in the static v0.1 catalog.",
                input.set_name
            ),
            "set_not_found",
            json!({ "set_name": input.set_name }),
        ));
    };

    Ok(tool_success(
        &format!(
            "Found set {} with {} child crates.",
            set.name,
            set.children.len()
        ),
        json!({
            "found": true,
            "set": set,
            "children": set.children,
            "notes": set.notes,
        }),
    ))
}

fn get_crate(
    server: &RustUseMcpServer,
    arguments: Option<JsonObject>,
) -> Result<CallToolResult, McpError> {
    let input = parse_input::<GetCrateInput>(arguments)?;
    let result = server.catalog.crate_lookup(&input.crate_name);
    if !result.found {
        return Ok(tool_error(
            &result.message,
            "crate_not_found",
            json!(result),
        ));
    }
    Ok(tool_success(&result.message, json!(result)))
}

fn list_children(
    server: &RustUseMcpServer,
    arguments: Option<JsonObject>,
) -> Result<CallToolResult, McpError> {
    let input = parse_input::<ListChildrenInput>(arguments)?;
    let result = server.catalog.list_children(&input.set_name);
    if !result.found {
        return Ok(tool_error(&result.message, "set_not_found", json!(result)));
    }
    Ok(tool_success(&result.message, json!(result)))
}

fn check_name_collision(
    server: &RustUseMcpServer,
    arguments: Option<JsonObject>,
) -> Result<CallToolResult, McpError> {
    let input = parse_input::<NameCollisionInput>(arguments)?;
    if input.name.trim().is_empty() {
        return Ok(tool_error(
            "Name collision checks require a non-empty name.",
            "empty_name",
            json!({ "name": input.name }),
        ));
    }
    let result = server
        .catalog
        .check_name_collision(&input.name, input.kind.unwrap_or_default());
    Ok(tool_success(&result.recommendation, json!(result)))
}

fn find_overlap(
    server: &RustUseMcpServer,
    arguments: Option<JsonObject>,
) -> Result<CallToolResult, McpError> {
    let input = parse_input::<FindOverlapInput>(arguments)?;
    if input.proposed_name.trim().is_empty() {
        return Ok(tool_error(
            "Overlap checks require a non-empty proposed_name.",
            "empty_proposed_name",
            json!({ "proposed_name": input.proposed_name }),
        ));
    }
    let children = input.proposed_children.unwrap_or_default();
    let result = rustuse_mcp_core::find_overlap(
        &server.catalog,
        &input.proposed_name,
        input.description.as_deref(),
        &children,
    );
    Ok(tool_success(&result.recommendation, json!(result)))
}

fn validate_plan(
    server: &RustUseMcpServer,
    arguments: Option<JsonObject>,
) -> Result<CallToolResult, McpError> {
    let input = parse_input::<ValidateSetPlanInput>(arguments)?;
    let result = validate_set_plan(
        &server.catalog,
        &server.rules,
        &input.set_name,
        &input.description,
        &input.proposed_children,
    );
    let summary = if result.valid {
        format!("Set plan is valid with score {}.", result.score)
    } else {
        format!(
            "Set plan is not valid; severity {} with score {}.",
            result.severity, result.score
        )
    };
    Ok(tool_success(&summary, json!(result)))
}

fn generate_prompt(arguments: Option<JsonObject>) -> Result<CallToolResult, McpError> {
    let input = parse_input::<GenerateCopilotPromptInput>(arguments)?;
    if input.set_name.trim().is_empty() {
        return Ok(tool_error(
            "Copilot prompt generation requires a non-empty set_name.",
            "empty_set_name",
            json!({ "set_name": input.set_name }),
        ));
    }
    if matches!(input.minimum_children, Some(0)) {
        return Ok(tool_error(
            "minimum_children must be at least 1 when provided.",
            "invalid_minimum_children",
            json!({ "minimum_children": input.minimum_children }),
        ));
    }
    let proposed_children = input.proposed_children.unwrap_or_default();
    let result = generate_copilot_prompt(
        &input.set_name,
        input.description.as_deref(),
        input.minimum_children,
        &proposed_children,
        input.include_docs,
        input.include_ci,
    )
    .map_err(|source| McpError::internal_error(source.to_string(), None))?;

    Ok(tool_success(
        "Generated a ready-to-paste GitHub Copilot Plan Mode prompt.",
        json!(result),
    ))
}

fn tool<T: JsonSchema + 'static>(
    name: &'static str,
    title: &str,
    description: &'static str,
) -> Tool {
    Tool::new(name, description, schema_for_type::<T>())
        .with_title(title)
        .with_annotations(
            ToolAnnotations::new()
                .read_only(true)
                .destructive(false)
                .idempotent(true)
                .open_world(false),
        )
}

fn parse_input<T: DeserializeOwned>(arguments: Option<JsonObject>) -> Result<T, McpError> {
    serde_json::from_value(Value::Object(arguments.unwrap_or_default())).map_err(|source| {
        McpError::invalid_params(
            "invalid tool arguments",
            Some(json!({ "error": source.to_string() })),
        )
    })
}

fn tool_success(summary: &str, value: Value) -> CallToolResult {
    let mut result = CallToolResult::structured(value);
    result.content.insert(0, Content::text(summary.to_owned()));
    result
}

fn tool_error(summary: &str, code: &str, details: Value) -> CallToolResult {
    let mut error = Map::new();
    error.insert("code".to_owned(), Value::String(code.to_owned()));
    error.insert("message".to_owned(), Value::String(summary.to_owned()));

    let mut payload = Map::new();
    payload.insert("error".to_owned(), Value::Object(error));
    payload.insert("details".to_owned(), details);

    let mut result = CallToolResult::structured_error(Value::Object(payload));
    result.content.insert(0, Content::text(summary.to_owned()));
    result
}

#[cfg(test)]
mod tests {
    use rmcp::model::object;
    use serde_json::json;

    use super::*;
    use crate::server::RustUseMcpServer;

    #[test]
    fn list_tools_includes_required_tools() {
        let tools = list_tools();
        assert!(
            tools
                .iter()
                .any(|tool| tool.name == "rustuse_catalog_search")
        );
        assert!(
            tools
                .iter()
                .any(|tool| tool.name == "rustuse_generate_copilot_prompt")
        );
    }

    #[test]
    fn call_tool_returns_catalog_matches() -> Result<(), Box<dyn std::error::Error>> {
        let server = RustUseMcpServer::new()?;
        let result = call_tool(
            &server,
            "rustuse_catalog_search",
            Some(object(json!({ "query": "geometry", "limit": 5 }))),
        )
        .map_err(to_error)?;
        assert_eq!(result.is_error, Some(false));
        assert!(result.structured_content.is_some());
        Ok(())
    }

    #[test]
    fn bad_limit_returns_tool_error() -> Result<(), Box<dyn std::error::Error>> {
        let server = RustUseMcpServer::new()?;
        let result = call_tool(
            &server,
            "rustuse_catalog_search",
            Some(object(json!({ "query": "math", "limit": 0 }))),
        )
        .map_err(to_error)?;
        assert_eq!(result.is_error, Some(true));
        Ok(())
    }

    fn to_error(error: McpError) -> std::io::Error {
        let message = format!("{error:?}");
        drop(error);
        std::io::Error::other(message)
    }
}
