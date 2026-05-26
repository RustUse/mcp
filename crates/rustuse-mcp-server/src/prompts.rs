use rmcp::{
    ErrorData as McpError,
    model::{
        GetPromptResult, JsonObject, Prompt, PromptArgument as McpPromptArgument, PromptMessage,
        PromptMessageRole,
    },
};
use rustuse_mcp_core::{PromptRenderRequest, list_prompt_templates, render_prompt};
use serde_json::{Value, json};

#[must_use]
pub fn list_prompts() -> Vec<Prompt> {
    list_prompt_templates()
        .into_iter()
        .map(|template| {
            let arguments = template
                .arguments
                .into_iter()
                .map(|argument| {
                    McpPromptArgument::new(argument.name)
                        .with_description(argument.description)
                        .with_required(argument.required)
                })
                .collect();

            Prompt::new(template.name, Some(template.description), Some(arguments))
                .with_title(template.title)
        })
        .collect()
}

pub fn get_prompt(name: &str, arguments: Option<JsonObject>) -> Result<GetPromptResult, McpError> {
    let request = render_request(name, arguments)?;
    let rendered = render_prompt(&request).map_err(|source| {
        McpError::invalid_params(source.to_string(), Some(json!({ "prompt_name": name })))
    })?;

    Ok(GetPromptResult::new(vec![PromptMessage::new_text(
        PromptMessageRole::User,
        rendered.text,
    )])
    .with_description(rendered.title))
}

fn render_request(
    name: &str,
    arguments: Option<JsonObject>,
) -> Result<PromptRenderRequest, McpError> {
    let arguments = arguments.unwrap_or_default();
    Ok(PromptRenderRequest {
        prompt_name: name.to_owned(),
        set_name: optional_string(&arguments, "set_name")?,
        crate_name: optional_string(&arguments, "crate_name")?,
        description: optional_string(&arguments, "description")?,
        minimum_children: optional_usize(&arguments, "minimum_children")?,
        proposed_children: optional_string_vec(&arguments, "proposed_children")?,
        parent_set: optional_string(&arguments, "parent_set")?,
        include_docs: optional_bool(&arguments, "include_docs")?,
        include_ci: optional_bool(&arguments, "include_ci")?,
        include_examples: optional_bool(&arguments, "include_examples")?,
    })
}

fn optional_string(arguments: &JsonObject, key: &str) -> Result<Option<String>, McpError> {
    match arguments.get(key) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(_) => Err(invalid_argument_type(key, "string")),
        None => Ok(None),
    }
}

fn optional_bool(arguments: &JsonObject, key: &str) -> Result<Option<bool>, McpError> {
    match arguments.get(key) {
        Some(Value::Bool(value)) => Ok(Some(*value)),
        Some(_) => Err(invalid_argument_type(key, "boolean")),
        None => Ok(None),
    }
}

fn optional_usize(arguments: &JsonObject, key: &str) -> Result<Option<usize>, McpError> {
    match arguments.get(key) {
        Some(Value::Number(value)) => {
            let Some(raw) = value.as_u64() else {
                return Err(invalid_argument_type(key, "positive integer"));
            };
            usize::try_from(raw)
                .map(Some)
                .map_err(|_| invalid_argument_type(key, "positive integer"))
        }
        Some(_) => Err(invalid_argument_type(key, "positive integer")),
        None => Ok(None),
    }
}

fn optional_string_vec(arguments: &JsonObject, key: &str) -> Result<Option<Vec<String>>, McpError> {
    match arguments.get(key) {
        Some(Value::Array(values)) => values
            .iter()
            .map(|value| match value {
                Value::String(text) => Ok(text.clone()),
                _ => Err(invalid_argument_type(key, "array of strings")),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(Value::String(value)) => Ok(Some(
            value
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        )),
        Some(_) => Err(invalid_argument_type(key, "array of strings")),
        None => Ok(None),
    }
}

fn invalid_argument_type(key: &str, expected: &str) -> McpError {
    McpError::invalid_params(
        format!("Prompt argument {key} must be a {expected}."),
        Some(json!({ "argument": key, "expected": expected })),
    )
}

#[cfg(test)]
mod tests {
    use rmcp::model::object;
    use serde_json::json;

    use super::*;

    #[test]
    fn list_prompts_includes_required_prompts() {
        let prompts = list_prompts();
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "rustuse_brainstorm_set")
        );
        assert!(
            prompts
                .iter()
                .any(|prompt| prompt.name == "rustuse_generate_docs_page")
        );
    }

    #[test]
    fn get_prompt_renders_text() -> Result<(), Box<dyn std::error::Error>> {
        let result = get_prompt(
            "rustuse_brainstorm_set",
            Some(object(
                json!({ "set_name": "use-example", "description": "Example primitives" }),
            )),
        )
        .map_err(to_error)?;
        assert_eq!(result.messages.len(), 1);
        Ok(())
    }

    fn to_error(error: McpError) -> std::io::Error {
        let message = format!("{error:?}");
        drop(error);
        std::io::Error::other(message)
    }
}
