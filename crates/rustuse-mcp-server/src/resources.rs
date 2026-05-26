use rmcp::{
    ErrorData as McpError,
    model::{
        AnnotateAble, RawResource, RawResourceTemplate, ReadResourceResult, Resource,
        ResourceContents, ResourceTemplate,
    },
};
use rustuse_mcp_core::{RustUseCatalog, list_prompt_templates};
use serde::Serialize;
use serde_json::json;

use crate::server::RustUseMcpServer;

const MIME_JSON: &str = "application/json";
const MIME_MARKDOWN: &str = "text/markdown";

const DOC_OVERVIEW: &str = include_str!("../../../docs/overview.md");
const DOC_RESOURCES: &str = include_str!("../../../docs/resources.md");
const DOC_TOOLS: &str = include_str!("../../../docs/tools.md");
const DOC_PROMPTS: &str = include_str!("../../../docs/prompts.md");
const DOC_SECURITY: &str = include_str!("../../../docs/security.md");
const DOC_LOCAL_SETUP: &str = include_str!("../../../docs/local-setup.md");

#[must_use]
pub fn list_resources(catalog: &RustUseCatalog) -> Vec<Resource> {
    let mut resources = vec![
        resource(
            "rustuse://catalog",
            "catalog",
            "RustUse catalog",
            "Static v0.1 RustUse set and crate catalog.",
            MIME_JSON,
        ),
        resource(
            "rustuse://rules",
            "rules",
            "RustUse rules",
            "Static RustUse rule set used for planning and validation.",
            MIME_JSON,
        ),
        resource(
            "rustuse://adoption-paths",
            "adoption-paths",
            "RustUse adoption paths",
            "Static RustUse adoption and distribution paths.",
            MIME_JSON,
        ),
        resource(
            "rustuse://sets",
            "sets",
            "RustUse sets",
            "Static list of cataloged RustUse facade sets.",
            MIME_JSON,
        ),
        resource(
            "rustuse://prompts",
            "prompts",
            "RustUse prompts",
            "Markdown index of RustUse MCP prompt templates.",
            MIME_MARKDOWN,
        ),
        resource(
            "rustuse://docs/overview",
            "docs-overview",
            "RustUse MCP overview",
            "Overview of the RustUse MCP server.",
            MIME_MARKDOWN,
        ),
        resource(
            "rustuse://docs/security",
            "docs-security",
            "RustUse MCP security",
            "Security posture of the RustUse MCP server.",
            MIME_MARKDOWN,
        ),
    ];

    resources.extend(catalog.sets.iter().map(|set| {
        resource(
            &format!("rustuse://sets/{}", set.name),
            &set.name,
            &format!("RustUse set {}", set.name),
            &set.description,
            MIME_JSON,
        )
    }));

    resources
}

#[must_use]
pub fn list_resource_templates() -> Vec<ResourceTemplate> {
    vec![
        RawResourceTemplate::new("rustuse://sets/{set_name}", "set")
            .with_title("RustUse set")
            .with_description("Read metadata for a RustUse set by set name.")
            .with_mime_type(MIME_JSON)
            .no_annotation(),
        RawResourceTemplate::new("rustuse://crates/{crate_name}", "crate")
            .with_title("RustUse crate")
            .with_description("Read metadata for a RustUse child crate by crate name.")
            .with_mime_type(MIME_JSON)
            .no_annotation(),
        RawResourceTemplate::new("rustuse://docs/{doc_slug}", "docs")
            .with_title("RustUse MCP docs")
            .with_description("Read RustUse MCP documentation by slug.")
            .with_mime_type(MIME_MARKDOWN)
            .no_annotation(),
    ]
}

pub fn read_resource(server: &RustUseMcpServer, uri: &str) -> Result<ReadResourceResult, McpError> {
    let (content, mime_type) = match uri {
        "rustuse://catalog" => (json_text(&server.catalog)?, MIME_JSON),
        "rustuse://rules" => (json_text(&server.rules)?, MIME_JSON),
        "rustuse://adoption-paths" => (json_text(&server.adoption_paths)?, MIME_JSON),
        "rustuse://sets" => (json_text(&server.catalog.sets)?, MIME_JSON),
        "rustuse://prompts" => (prompt_index(), MIME_MARKDOWN),
        "rustuse://docs/overview" => (DOC_OVERVIEW.to_owned(), MIME_MARKDOWN),
        "rustuse://docs/resources" => (DOC_RESOURCES.to_owned(), MIME_MARKDOWN),
        "rustuse://docs/tools" => (DOC_TOOLS.to_owned(), MIME_MARKDOWN),
        "rustuse://docs/prompts" => (DOC_PROMPTS.to_owned(), MIME_MARKDOWN),
        "rustuse://docs/security" => (DOC_SECURITY.to_owned(), MIME_MARKDOWN),
        "rustuse://docs/local-setup" => (DOC_LOCAL_SETUP.to_owned(), MIME_MARKDOWN),
        _ if uri.starts_with("rustuse://sets/") => {
            let set_name = uri.trim_start_matches("rustuse://sets/");
            let Some(set) = server.catalog.set(set_name) else {
                return Err(resource_not_found(uri));
            };
            (json_text(set)?, MIME_JSON)
        }
        _ if uri.starts_with("rustuse://crates/") => {
            let crate_name = uri.trim_start_matches("rustuse://crates/");
            let lookup = server.catalog.crate_lookup(crate_name);
            if !lookup.found {
                return Err(resource_not_found(uri));
            }
            (json_text(&lookup)?, MIME_JSON)
        }
        _ if uri.starts_with("rustuse://docs/") => return Err(resource_not_found(uri)),
        _ => return Err(resource_not_found(uri)),
    };

    Ok(ReadResourceResult::new(vec![
        ResourceContents::text(content, uri).with_mime_type(mime_type),
    ]))
}

fn resource(uri: &str, name: &str, title: &str, description: &str, mime_type: &str) -> Resource {
    RawResource::new(uri, name)
        .with_title(title)
        .with_description(description)
        .with_mime_type(mime_type)
        .no_annotation()
}

fn json_text<T: Serialize>(value: &T) -> Result<String, McpError> {
    serde_json::to_string_pretty(value).map_err(|source| {
        McpError::internal_error(
            "failed to serialize RustUse resource",
            Some(json!({ "error": source.to_string() })),
        )
    })
}

fn prompt_index() -> String {
    let mut lines = vec!["# RustUse MCP Prompts".to_owned(), String::new()];
    for prompt in list_prompt_templates() {
        lines.push(format!("- `{}` - {}", prompt.name, prompt.description));
    }
    lines.push(String::new());
    lines.join("\n")
}

fn resource_not_found(uri: &str) -> McpError {
    McpError::resource_not_found("resource not found", Some(json!({ "uri": uri })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::RustUseMcpServer;

    #[test]
    fn list_resources_includes_required_entries() -> Result<(), rustuse_mcp_core::CoreError> {
        let server = RustUseMcpServer::new()?;
        let resources = list_resources(&server.catalog);
        assert!(
            resources
                .iter()
                .any(|item| item.raw.uri == "rustuse://catalog")
        );
        assert!(
            resources
                .iter()
                .any(|item| item.raw.uri == "rustuse://sets/use-math")
        );
        assert!(
            resources
                .iter()
                .any(|item| item.raw.uri == "rustuse://docs/security")
        );
        Ok(())
    }

    #[test]
    fn read_resource_returns_json_catalog() -> Result<(), Box<dyn std::error::Error>> {
        let server = RustUseMcpServer::new()?;
        let result = read_resource(&server, "rustuse://catalog").map_err(to_error)?;
        assert_eq!(result.contents.len(), 1);
        Ok(())
    }

    fn to_error(error: McpError) -> std::io::Error {
        let message = format!("{error:?}");
        drop(error);
        std::io::Error::other(message)
    }
}
