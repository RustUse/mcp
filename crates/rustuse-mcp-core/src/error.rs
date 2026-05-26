use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("failed to parse embedded {name}: {source}")]
    EmbeddedJson {
        name: &'static str,
        #[source]
        source: serde_json::Error,
    },
    #[error("unknown prompt template: {0}")]
    UnknownPrompt(String),
    #[error("missing required prompt argument: {0}")]
    MissingPromptArgument(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
}
