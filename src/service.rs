use crate::models::{
    ApiDefinition, ApiParameter, ApiStatus, Authentication, HttpMethod, ParameterIn, ParameterType,
    RequestBody,
};
use crate::storage::ApiStorageManager;
use anyhow::Result;
use rmcp::model::{CallToolResult, Content, Tool};
use std::collections::HashMap;
use std::sync::Arc;

/// MCP OpenAPI 服务
pub struct OpenApiService {
    storage: Arc<ApiStorageManager>,
    http_client: reqwest::Client,
    enable_management: bool,
}

impl OpenApiService {
    pub fn new(storage: Arc<ApiStorageManager>, enable_management: bool) -> Self {
        Self {
            storage,
            http_client: reqwest::Client::new(),
            enable_management,
        }
    }

    /// 获取所有工具（包括管理工具和动态 API 工具）
    pub async fn get_all_tools(&self) -> Vec<Tool> {
        let mut tools = self.get_management_tools();

        // 添加所有启用的 API 作为工具
        let apis = self.storage.list_enabled_apis().await;
        for api in apis {
            tools.push(self.api_to_tool(&api));
        }

        tools
    }

    /// 获取管理工具列表
    fn get_management_tools(&self) -> Vec<Tool> {
        let mut tools = vec![
            // 查询类工具 - 总是可用
            Tool::new(
                "list_apis",
                "List all registered APIs. Returns a list of all API definitions including their status (enabled/disabled).",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": {
                            "type": "string",
                            "enum": ["all", "enabled", "disabled"],
                            "description": "Filter APIs by status. Default is 'all'."
                        },
                        "tag": {
                            "type": "string",
                            "description": "Filter APIs by tag."
                        }
                    },
                    "required": []
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            Tool::new(
                "get_api",
                "Get detailed information about a specific API by its ID or name.",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "API ID to get"
                        },
                        "name": {
                            "type": "string",
                            "description": "API name to get (used if id is not provided)"
                        }
                    },
                    "required": []
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
            Tool::new(
                "list_apis_by_tag",
                "List all APIs that have a specific tag.",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "tag": {
                            "type": "string",
                            "description": "Tag to filter by"
                        }
                    },
                    "required": ["tag"]
                })
                .as_object()
                .unwrap()
                .clone(),
            ),
        ];

        // 修改类工具 - 只在启用管理功能时添加
        if self.enable_management {
            tools.extend(vec![
            Tool::new(
                "add_api",
                "Add a new API definition. The API will be registered as a new tool that can be called through MCP.",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Unique name for the API (will be used as tool name)"
                        },
                        "description": {
                            "type": "string",
                            "description": "Detailed description of what the API does (will be used as tool description)"
                        },
                        "base_url": {
                            "type": "string",
                            "description": "Base URL of the API (e.g., https://api.example.com)"
                        },
                        "path": {
                            "type": "string",
                            "description": "API path with optional path parameters (e.g., /users/{id})"
                        },
                        "method": {
                            "type": "string",
                            "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
                            "description": "HTTP method"
                        },
                        "parameters": {
                            "type": "array",
                            "description": "List of API parameters",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "description": {"type": "string"},
                                    "in": {"type": "string", "enum": ["query", "header", "path", "body"]},
                                    "required": {"type": "boolean"},
                                    "type": {"type": "string", "enum": ["string", "integer", "number", "boolean", "array", "object"]}
                                },
                                "required": ["name", "in"]
                            }
                        },
                        "request_body": {
                            "type": "object",
                            "description": "Request body definition",
                            "properties": {
                                "content_type": {"type": "string"},
                                "schema": {"type": "object"},
                                "required": {"type": "boolean"},
                                "description": {"type": "string"}
                            }
                        },
                        "authentication": {
                            "type": "object",
                            "description": "Authentication configuration",
                            "properties": {
                                "type": {"type": "string", "enum": ["none", "api_key", "bearer", "basic"]},
                                "header_name": {"type": "string"},
                                "api_key": {"type": "string"},
                                "token": {"type": "string"},
                                "username": {"type": "string"},
                                "password": {"type": "string"}
                            }
                        },
                        "headers": {
                            "type": "object",
                            "description": "Default headers to include in requests",
                            "additionalProperties": {"type": "string"}
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Tags for categorizing the API"
                        }
                    },
                    "required": ["name", "description", "base_url", "path", "method"]
                }).as_object().unwrap().clone(),
            ),
            Tool::new(
                "delete_api",
                "Delete an API by its ID or name. The API tool will be removed from the available tools.",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "API ID to delete"
                        },
                        "name": {
                            "type": "string",
                            "description": "API name to delete (used if id is not provided)"
                        }
                    },
                    "required": []
                }).as_object().unwrap().clone(),
            ),
            Tool::new(
                "enable_api",
                "Enable a disabled API. The API will appear as an available tool.",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "API ID to enable"
                        },
                        "name": {
                            "type": "string",
                            "description": "API name to enable (used if id is not provided)"
                        }
                    },
                    "required": []
                }).as_object().unwrap().clone(),
            ),
            Tool::new(
                "disable_api",
                "Disable an API. The API will not appear as an available tool but will be preserved.",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "API ID to disable"
                        },
                        "name": {
                            "type": "string",
                            "description": "API name to disable (used if id is not provided)"
                        }
                    },
                    "required": []
                }).as_object().unwrap().clone(),
            ),
            Tool::new(
                "update_api",
                "Update an existing API definition. Only provided fields will be updated.",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string",
                            "description": "API ID to update (required if name not provided)"
                        },
                        "name": {
                            "type": "string",
                            "description": "API name to update (used to find API if id not provided, or new name if updating)"
                        },
                        "new_name": {
                            "type": "string",
                            "description": "New name for the API"
                        },
                        "description": {
                            "type": "string",
                            "description": "New description"
                        },
                        "base_url": {
                            "type": "string",
                            "description": "New base URL"
                        },
                        "path": {
                            "type": "string",
                            "description": "New path"
                        },
                        "method": {
                            "type": "string",
                            "enum": ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"],
                            "description": "New HTTP method"
                        },
                        "parameters": {
                            "type": "array",
                            "description": "New parameters list (replaces existing)",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "description": {"type": "string"},
                                    "in": {"type": "string", "enum": ["query", "header", "path", "body"]},
                                    "required": {"type": "boolean"},
                                    "type": {"type": "string", "enum": ["string", "integer", "number", "boolean", "array", "object"]}
                                },
                                "required": ["name", "in"]
                            }
                        },
                        "authentication": {
                            "type": "object",
                            "description": "New authentication configuration",
                            "properties": {
                                "type": {"type": "string", "enum": ["none", "api_key", "bearer", "basic"]},
                                "header_name": {"type": "string"},
                                "api_key": {"type": "string"},
                                "token": {"type": "string"},
                                "username": {"type": "string"},
                                "password": {"type": "string"}
                            }
                        },
                        "headers": {
                            "type": "object",
                            "description": "New default headers",
                            "additionalProperties": {"type": "string"}
                        },
                        "tags": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "New tags"
                        }
                    },
                    "required": []
                }).as_object().unwrap().clone(),
            ),
            ]);
        }

        tools
    }

    /// 将 API 定义转换为 MCP Tool
    fn api_to_tool(&self, api: &ApiDefinition) -> Tool {
        Tool::new(
            api.name.clone(),
            api.description.clone(),
            api.to_tool_input_schema().as_object().unwrap().clone(),
        )
    }

    /// 处理工具调用
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        match name {
            // 查询类工具 - 总是允许
            "list_apis" => self.handle_list_apis(arguments).await,
            "get_api" => self.handle_get_api(arguments).await,
            "list_apis_by_tag" => self.handle_list_apis_by_tag(arguments).await,

            // 修改类工具
            "add_api" | "delete_api" | "enable_api" | "disable_api" | "update_api"
                if !self.enable_management =>
            {
                Err(anyhow::anyhow!(
                    "Management tool '{}' is disabled. Start without --nomg flag to enable it.",
                    name
                ))
            }
            "add_api" => self.handle_add_api(arguments).await,
            "delete_api" => self.handle_delete_api(arguments).await,
            "enable_api" => self.handle_enable_api(arguments).await,
            "disable_api" => self.handle_disable_api(arguments).await,
            "update_api" => self.handle_update_api(arguments).await,

            // 动态 API 工具调用
            _ => self.handle_api_call(name, arguments).await,
        }
    }

    async fn handle_list_apis(&self, arguments: serde_json::Value) -> Result<CallToolResult> {
        let status_filter = arguments
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("all");
        let tag_filter = arguments.get("tag").and_then(|v| v.as_str());

        let apis = match status_filter {
            "enabled" => self.storage.list_enabled_apis().await,
            "disabled" => self
                .storage
                .list_apis()
                .await
                .into_iter()
                .filter(|api| api.status == ApiStatus::Disabled)
                .collect(),
            _ => self.storage.list_apis().await,
        };

        let apis: Vec<_> = if let Some(tag) = tag_filter {
            apis.into_iter()
                .filter(|api| api.tags.contains(&tag.to_string()))
                .collect()
        } else {
            apis
        };

        let summary: Vec<serde_json::Value> = apis
            .iter()
            .map(|api| {
                serde_json::json!({
                    "id": api.id,
                    "name": api.name,
                    "description": api.description,
                    "method": api.method,
                    "base_url": api.base_url,
                    "path": api.path,
                    "status": api.status,
                    "tags": api.tags
                })
            })
            .collect();

        Ok(CallToolResult {
            content: vec![Content::text(serde_json::to_string_pretty(&summary)?)],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    async fn handle_add_api(&self, arguments: serde_json::Value) -> Result<CallToolResult> {
        let name = arguments
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("name is required"))?;
        let description = arguments
            .get("description")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("description is required"))?;
        let base_url = arguments
            .get("base_url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("base_url is required"))?;
        let path = arguments
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("path is required"))?;
        let method_str = arguments
            .get("method")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("method is required"))?;

        let method = match method_str.to_uppercase().as_str() {
            "GET" => HttpMethod::Get,
            "POST" => HttpMethod::Post,
            "PUT" => HttpMethod::Put,
            "DELETE" => HttpMethod::Delete,
            "PATCH" => HttpMethod::Patch,
            "HEAD" => HttpMethod::Head,
            "OPTIONS" => HttpMethod::Options,
            _ => return Err(anyhow::anyhow!("Invalid HTTP method: {}", method_str)),
        };

        let mut api = ApiDefinition::new(
            name.to_string(),
            description.to_string(),
            base_url.to_string(),
            path.to_string(),
            method,
        );

        // 解析参数
        if let Some(params) = arguments.get("parameters").and_then(|v| v.as_array()) {
            for param in params {
                let param_name = param
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let param_in = param.get("in").and_then(|v| v.as_str()).unwrap_or("query");
                let param_desc = param
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let param_required = param
                    .get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let param_type = param
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("string");

                api.parameters.push(ApiParameter {
                    name: param_name.to_string(),
                    description: param_desc.to_string(),
                    location: match param_in {
                        "header" => ParameterIn::Header,
                        "path" => ParameterIn::Path,
                        "body" => ParameterIn::Body,
                        _ => ParameterIn::Query,
                    },
                    required: param_required,
                    param_type: match param_type {
                        "integer" => ParameterType::Integer,
                        "number" => ParameterType::Number,
                        "boolean" => ParameterType::Boolean,
                        "array" => ParameterType::Array,
                        "object" => ParameterType::Object,
                        _ => ParameterType::String,
                    },
                    default: param.get("default").cloned(),
                    enum_values: param.get("enum").and_then(|v| v.as_array()).cloned(),
                });
            }
        }

        // 解析请求体
        if let Some(body) = arguments.get("request_body") {
            api.request_body = Some(RequestBody {
                content_type: body
                    .get("content_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("application/json")
                    .to_string(),
                schema: body.get("schema").cloned(),
                required: body
                    .get("required")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                description: body
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
            });
        }

        // 解析认证
        if let Some(auth) = arguments.get("authentication") {
            let auth_type = auth.get("type").and_then(|v| v.as_str()).unwrap_or("none");
            api.authentication = match auth_type {
                "api_key" => Authentication::ApiKey {
                    header_name: auth
                        .get("header_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("X-API-Key")
                        .to_string(),
                    api_key: auth
                        .get("api_key")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                },
                "bearer" => Authentication::Bearer {
                    token: auth
                        .get("token")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                },
                "basic" => Authentication::Basic {
                    username: auth
                        .get("username")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    password: auth
                        .get("password")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string(),
                },
                _ => Authentication::None,
            };
        }

        // 解析默认头
        if let Some(headers) = arguments.get("headers").and_then(|v| v.as_object()) {
            for (key, value) in headers {
                if let Some(v) = value.as_str() {
                    api.headers.insert(key.clone(), v.to_string());
                }
            }
        }

        // 解析标签
        if let Some(tags) = arguments.get("tags").and_then(|v| v.as_array()) {
            api.tags = tags
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }

        let api = self.storage.add_api(api).await?;

        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "API '{}' added successfully with ID: {}",
                api.name, api.id
            ))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    async fn handle_delete_api(&self, arguments: serde_json::Value) -> Result<CallToolResult> {
        let id = if let Some(id) = arguments.get("id").and_then(|v| v.as_str()) {
            id.to_string()
        } else if let Some(name) = arguments.get("name").and_then(|v| v.as_str()) {
            self.storage
                .get_api_by_name(name)
                .await
                .ok_or_else(|| anyhow::anyhow!("API with name '{}' not found", name))?
                .id
        } else {
            return Err(anyhow::anyhow!("Either id or name must be provided"));
        };

        let api = self.storage.delete_api(&id).await?;

        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "API '{}' deleted successfully",
                api.name
            ))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    async fn handle_enable_api(&self, arguments: serde_json::Value) -> Result<CallToolResult> {
        let id = if let Some(id) = arguments.get("id").and_then(|v| v.as_str()) {
            id.to_string()
        } else if let Some(name) = arguments.get("name").and_then(|v| v.as_str()) {
            self.storage
                .get_api_by_name(name)
                .await
                .ok_or_else(|| anyhow::anyhow!("API with name '{}' not found", name))?
                .id
        } else {
            return Err(anyhow::anyhow!("Either id or name must be provided"));
        };

        let api = self.storage.enable_api(&id).await?;

        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "API '{}' enabled successfully",
                api.name
            ))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    async fn handle_disable_api(&self, arguments: serde_json::Value) -> Result<CallToolResult> {
        let id = if let Some(id) = arguments.get("id").and_then(|v| v.as_str()) {
            id.to_string()
        } else if let Some(name) = arguments.get("name").and_then(|v| v.as_str()) {
            self.storage
                .get_api_by_name(name)
                .await
                .ok_or_else(|| anyhow::anyhow!("API with name '{}' not found", name))?
                .id
        } else {
            return Err(anyhow::anyhow!("Either id or name must be provided"));
        };

        let api = self.storage.disable_api(&id).await?;

        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "API '{}' disabled successfully",
                api.name
            ))],
            is_error: Some(false),
            meta: None,
            structured_content: None,
        })
    }

    async fn handle_api_call(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        let api = self
            .storage
            .get_api_by_name(name)
            .await
            .ok_or_else(|| anyhow::anyhow!("API '{}' not found", name))?;

        if api.status != ApiStatus::Enabled {
            return Err(anyhow::anyhow!("API '{}' is disabled", name));
        }

        // 构建请求
        let mut path_params = HashMap::new();
        let mut query_params = Vec::new();
        let mut headers = api.headers.clone();

        // 处理参数
        for param in &api.parameters {
            let value = arguments.get(&param.name);

            match param.location {
                ParameterIn::Path => {
                    if let Some(v) = value {
                        path_params.insert(
                            param.name.clone(),
                            v.to_string().trim_matches('"').to_string(),
                        );
                    } else if param.required {
                        return Err(anyhow::anyhow!(
                            "Required path parameter '{}' is missing",
                            param.name
                        ));
                    }
                }
                ParameterIn::Query => {
                    if let Some(v) = value {
                        query_params.push((
                            param.name.clone(),
                            v.to_string().trim_matches('"').to_string(),
                        ));
                    } else if param.required {
                        return Err(anyhow::anyhow!(
                            "Required query parameter '{}' is missing",
                            param.name
                        ));
                    }
                }
                ParameterIn::Header => {
                    if let Some(v) = value {
                        headers.insert(
                            param.name.clone(),
                            v.to_string().trim_matches('"').to_string(),
                        );
                    } else if param.required {
                        return Err(anyhow::anyhow!(
                            "Required header parameter '{}' is missing",
                            param.name
                        ));
                    }
                }
                ParameterIn::Body => {
                    // Body 参数将在后面处理
                }
            }
        }

        // 构建 URL
        let url = api.build_url(&path_params);

        // 创建请求
        let mut request = match api.method {
            HttpMethod::Get => self.http_client.get(&url),
            HttpMethod::Post => self.http_client.post(&url),
            HttpMethod::Put => self.http_client.put(&url),
            HttpMethod::Delete => self.http_client.delete(&url),
            HttpMethod::Patch => self.http_client.patch(&url),
            HttpMethod::Head => self.http_client.head(&url),
            HttpMethod::Options => self.http_client.request(reqwest::Method::OPTIONS, &url),
        };

        // 添加查询参数
        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        // 添加头
        for (key, value) in &headers {
            request = request.header(key, value);
        }

        // 添加认证
        match &api.authentication {
            Authentication::ApiKey {
                header_name,
                api_key,
            } => {
                request = request.header(header_name, api_key);
            }
            Authentication::Bearer { token } => {
                request = request.header("Authorization", format!("Bearer {}", token));
            }
            Authentication::Basic { username, password } => {
                request = request.basic_auth(username, Some(password));
            }
            Authentication::None => {}
        }

        // 添加请求体
        if let Some(body) = arguments.get("body") {
            request = request.json(body);
        }

        // 发送请求
        let response = request.send().await?;
        let status = response.status();
        let body = response.text().await?;

        // 尝试格式化 JSON 响应
        let formatted_body = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
            serde_json::to_string_pretty(&json).unwrap_or(body)
        } else {
            body
        };

        Ok(CallToolResult {
            content: vec![Content::text(format!(
                "Status: {}\n\nResponse:\n{}",
                status, formatted_body
            ))],
            is_error: Some(!status.is_success()),
            meta: None,
            structured_content: None,
        })
    }

    /// 处理获取单个 API 详情
    async fn handle_get_api(&self, arguments: serde_json::Value) -> Result<CallToolResult> {
        let api_id = arguments
            .get("api_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing api_id parameter"))?;

        match self.storage.get_api(api_id).await {
            Some(api) => {
                let api_json = serde_json::to_string_pretty(&api)?;
                Ok(CallToolResult {
                    content: vec![Content::text(format!("API Details:\n{}", api_json))],
                    is_error: Some(false),
                    meta: None,
                    structured_content: None,
                })
            }
            None => Ok(CallToolResult {
                content: vec![Content::text(format!("API with id '{}' not found", api_id))],
                is_error: Some(true),
                meta: None,
                structured_content: None,
            }),
        }
    }

    /// 处理更新 API
    async fn handle_update_api(&self, arguments: serde_json::Value) -> Result<CallToolResult> {
        let api_id = arguments
            .get("api_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing api_id parameter"))?;

        // 首先获取现有的 API
        let existing_api = self.storage.get_api(api_id).await;

        match existing_api {
            Some(mut api) => {
                // 更新各个字段（如果提供了新值）
                if let Some(name) = arguments.get("name").and_then(|v| v.as_str()) {
                    api.name = name.to_string();
                }
                if let Some(description) = arguments.get("description").and_then(|v| v.as_str()) {
                    api.description = description.to_string();
                }
                if let Some(base_url) = arguments.get("base_url").and_then(|v| v.as_str()) {
                    api.base_url = base_url.to_string();
                }
                if let Some(path) = arguments.get("path").and_then(|v| v.as_str()) {
                    api.path = path.to_string();
                }
                if let Some(method) = arguments.get("method").and_then(|v| v.as_str()) {
                    api.method = serde_json::from_value(serde_json::json!(method))?;
                }
                if let Some(tags) = arguments.get("tags").and_then(|v| v.as_array()) {
                    api.tags = tags
                        .iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect();
                }
                if let Some(params) = arguments.get("parameters") {
                    api.parameters = serde_json::from_value(params.clone())?;
                }
                if let Some(body) = arguments.get("request_body") {
                    api.request_body = serde_json::from_value(body.clone())?;
                }
                if let Some(auth) = arguments.get("authentication") {
                    api.authentication = serde_json::from_value(auth.clone())?;
                }
                if let Some(headers) = arguments.get("headers").and_then(|v| v.as_object()) {
                    api.headers = headers
                        .iter()
                        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                        .collect();
                }

                // 更新时间戳
                api.updated_at = chrono::Utc::now().to_rfc3339();

                // 保存更新
                match self.storage.update_api(api_id, api).await {
                    Ok(_) => Ok(CallToolResult {
                        content: vec![Content::text(format!(
                            "API '{}' updated successfully",
                            api_id
                        ))],
                        is_error: Some(false),
                        meta: None,
                        structured_content: None,
                    }),
                    Err(e) => Ok(CallToolResult {
                        content: vec![Content::text(format!("Failed to update API: {}", e))],
                        is_error: Some(true),
                        meta: None,
                        structured_content: None,
                    }),
                }
            }
            None => Ok(CallToolResult {
                content: vec![Content::text(format!("API with id '{}' not found", api_id))],
                is_error: Some(true),
                meta: None,
                structured_content: None,
            }),
        }
    }

    /// 处理按标签列出 API
    async fn handle_list_apis_by_tag(
        &self,
        arguments: serde_json::Value,
    ) -> Result<CallToolResult> {
        let tag = arguments
            .get("tag")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing tag parameter"))?;

        let apis = self.storage.list_apis_by_tag(tag).await;

        if apis.is_empty() {
            Ok(CallToolResult {
                content: vec![Content::text(format!("No APIs found with tag '{}'", tag))],
                is_error: Some(false),
                meta: None,
                structured_content: None,
            })
        } else {
            let api_list: Vec<serde_json::Value> = apis
                .iter()
                .map(|api| {
                    serde_json::json!({
                        "id": api.id,
                        "name": api.name,
                        "description": api.description,
                        "method": api.method,
                        "path": api.path,
                        "status": api.status,
                        "tags": api.tags
                    })
                })
                .collect();

            Ok(CallToolResult {
                content: vec![Content::text(format!(
                    "APIs with tag '{}':\n{}",
                    tag,
                    serde_json::to_string_pretty(&api_list)?
                ))],
                is_error: Some(false),
                meta: None,
                structured_content: None,
            })
        }
    }
}
