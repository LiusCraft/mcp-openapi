use crate::service::OpenApiService;
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Implementation, ListToolsResult,
        PaginatedRequestParam, ServerCapabilities, ServerInfo, ToolsCapability,
    },
    service::RequestContext,
};
use std::sync::Arc;

/// MCP Handler 实现
#[derive(Clone)]
pub struct OpenApiHandler {
    service: Arc<OpenApiService>,
}

impl OpenApiHandler {
    pub fn new(service: Arc<OpenApiService>) -> Self {
        Self { service }
    }
}

impl ServerHandler for OpenApiHandler {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities {
                tools: Some(ToolsCapability {
                    list_changed: Some(true),
                }),
                ..Default::default()
            },
            server_info: Implementation {
                name: "mcp-openapi".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                title: None,
                icons: None,
                website_url: None,
            },
            instructions: Some(
                "This MCP server allows you to manage and call HTTP APIs as tools. \
                Use 'list_apis' to see available APIs, 'add_api' to register new APIs, \
                and call APIs directly by their registered names."
                    .to_string(),
            ),
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        let tools = self.service.get_all_tools().await;
        Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let name = request.name.as_ref();
        let arguments = request
            .arguments
            .map(serde_json::Value::Object)
            .unwrap_or(serde_json::Value::Null);

        match self.service.call_tool(name, arguments).await {
            Ok(result) => Ok(result),
            Err(e) => Ok(CallToolResult {
                content: vec![rmcp::model::Content::text(format!("Error: {}", e))],
                is_error: Some(true),
                meta: None,
                structured_content: None,
            }),
        }
    }
}
