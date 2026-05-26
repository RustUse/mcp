use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        GetPromptRequestParams, GetPromptResult, Implementation, ListPromptsResult,
        ListResourceTemplatesResult, ListResourcesResult, ListToolsResult,
        ReadResourceRequestParams, ReadResourceResult, ServerCapabilities, ServerInfo,
    },
    service::{NotificationContext, RequestContext},
};
use rustuse_mcp_core::{
    CoreError, RustUseAdoptionPaths, RustUseCatalog, RustUseRuleSet, load_adoption_paths,
    load_catalog, load_rules,
};

use crate::{config, prompts, resources, tools};

#[derive(Clone, Debug)]
pub struct RustUseMcpServer {
    pub catalog: RustUseCatalog,
    pub rules: RustUseRuleSet,
    pub adoption_paths: RustUseAdoptionPaths,
}

impl RustUseMcpServer {
    pub fn new() -> Result<Self, CoreError> {
        Ok(Self {
            catalog: load_catalog()?,
            rules: load_rules()?,
            adoption_paths: load_adoption_paths()?,
        })
    }
}

impl ServerHandler for RustUseMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_resources()
                .enable_tools()
                .enable_prompts()
                .build(),
        )
        .with_server_info(
            Implementation::new(config::SERVER_NAME, env!("CARGO_PKG_VERSION"))
                .with_title(config::SERVER_TITLE)
                .with_description(config::SERVER_DESCRIPTION)
                .with_website_url(config::SERVER_WEBSITE),
        )
        .with_instructions(config::SERVER_INSTRUCTIONS)
    }

    async fn list_resources(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult::with_all_items(
            resources::list_resources(&self.catalog),
        ))
    }

    async fn list_resource_templates(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult::with_all_items(
            resources::list_resource_templates(),
        ))
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        resources::read_resource(self, &request.uri)
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult::with_all_items(tools::list_tools()))
    }

    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::CallToolResult, McpError> {
        tools::call_tool(self, request.name.as_ref(), request.arguments)
    }

    async fn list_prompts(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult::with_all_items(prompts::list_prompts()))
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        prompts::get_prompt(&request.name, request.arguments)
    }

    async fn on_initialized(&self, _context: NotificationContext<RoleServer>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_info_declares_only_v0_1_capabilities() -> Result<(), CoreError> {
        let server = RustUseMcpServer::new()?;
        let info = server.get_info();

        assert!(info.capabilities.resources.is_some());
        assert!(info.capabilities.tools.is_some());
        assert!(info.capabilities.prompts.is_some());
        assert!(info.capabilities.logging.is_none());
        assert!(info.capabilities.completions.is_none());
        assert!(info.capabilities.tasks.is_none());

        if let Some(resources) = info.capabilities.resources {
            assert_eq!(resources.subscribe, None);
            assert_eq!(resources.list_changed, None);
        }
        if let Some(tools) = info.capabilities.tools {
            assert_eq!(tools.list_changed, None);
        }
        if let Some(prompts) = info.capabilities.prompts {
            assert_eq!(prompts.list_changed, None);
        }

        Ok(())
    }
}
