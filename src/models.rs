use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// HTTP 方法
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

/// API 参数位置
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterIn {
    Query,
    Header,
    Path,
    Body,
}

/// 参数类型
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
}

impl Default for ParameterType {
    fn default() -> Self {
        ParameterType::String
    }
}

/// API 参数定义
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiParameter {
    /// 参数名称
    pub name: String,
    /// 参数描述
    #[serde(default)]
    pub description: String,
    /// 参数位置
    #[serde(rename = "in")]
    pub location: ParameterIn,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    /// 参数类型
    #[serde(rename = "type", default)]
    pub param_type: ParameterType,
    /// 默认值
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,
    /// 枚举值
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "enum")]
    pub enum_values: Option<Vec<serde_json::Value>>,
}

/// API 状态
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
    /// 启用
    Enabled,
    /// 禁用
    Disabled,
}

impl Default for ApiStatus {
    fn default() -> Self {
        ApiStatus::Enabled
    }
}

/// 请求体定义
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RequestBody {
    /// 内容类型
    #[serde(default = "default_content_type")]
    pub content_type: String,
    /// JSON Schema 定义
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
    /// 是否必需
    #[serde(default)]
    pub required: bool,
    /// 描述
    #[serde(default)]
    pub description: String,
}

fn default_content_type() -> String {
    "application/json".to_string()
}

/// 响应定义
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ApiResponse {
    /// 状态码
    pub status_code: u16,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// JSON Schema 定义
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}

/// 认证类型
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Authentication {
    /// 无认证
    None,
    /// API Key 认证
    ApiKey {
        /// Header 名称
        header_name: String,
        /// API Key 值
        api_key: String,
    },
    /// Bearer Token 认证
    Bearer {
        /// Token 值
        token: String,
    },
    /// Basic 认证
    Basic {
        /// 用户名
        username: String,
        /// 密码
        password: String,
    },
}

impl Default for Authentication {
    fn default() -> Self {
        Authentication::None
    }
}

/// API 定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiDefinition {
    /// 唯一标识符
    pub id: String,
    /// API 名称 (用作工具名称)
    pub name: String,
    /// API 描述 (用作工具描述)
    pub description: String,
    /// 基础 URL
    pub base_url: String,
    /// 路径
    pub path: String,
    /// HTTP 方法
    pub method: HttpMethod,
    /// 参数列表
    #[serde(default)]
    pub parameters: Vec<ApiParameter>,
    /// 请求体
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_body: Option<RequestBody>,
    /// 响应定义
    #[serde(default)]
    pub responses: Vec<ApiResponse>,
    /// 认证配置
    #[serde(default)]
    pub authentication: Authentication,
    /// 默认请求头
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// API 状态
    #[serde(default)]
    pub status: ApiStatus,
    /// 标签/分类
    #[serde(default)]
    pub tags: Vec<String>,
    /// 创建时间
    #[serde(default = "default_now")]
    pub created_at: String,
    /// 更新时间
    #[serde(default = "default_now")]
    pub updated_at: String,
}

fn default_now() -> String {
    chrono::Utc::now().to_rfc3339()
}

impl ApiDefinition {
    pub fn new(
        name: String,
        description: String,
        base_url: String,
        path: String,
        method: HttpMethod,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            base_url,
            path,
            method,
            parameters: Vec::new(),
            request_body: None,
            responses: Vec::new(),
            authentication: Authentication::None,
            headers: HashMap::new(),
            status: ApiStatus::Enabled,
            tags: Vec::new(),
            created_at: now.clone(),
            updated_at: now,
        }
    }

    /// 构建完整的请求 URL
    pub fn build_url(&self, path_params: &HashMap<String, String>) -> String {
        let mut url = format!("{}{}", self.base_url.trim_end_matches('/'), self.path);

        // 替换路径参数
        for (key, value) in path_params {
            url = url.replace(&format!("{{{}}}", key), value);
        }

        url
    }

    /// 生成工具的 JSON Schema
    pub fn to_tool_input_schema(&self) -> serde_json::Value {
        let mut properties = serde_json::Map::new();
        let mut required = Vec::new();

        // 添加参数
        for param in &self.parameters {
            let mut prop = serde_json::Map::new();
            prop.insert(
                "type".to_string(),
                serde_json::json!(match param.param_type {
                    ParameterType::String => "string",
                    ParameterType::Integer => "integer",
                    ParameterType::Number => "number",
                    ParameterType::Boolean => "boolean",
                    ParameterType::Array => "array",
                    ParameterType::Object => "object",
                }),
            );
            prop.insert(
                "description".to_string(),
                serde_json::json!(param.description),
            );

            if let Some(ref default) = param.default {
                prop.insert("default".to_string(), default.clone());
            }
            if let Some(ref enum_vals) = param.enum_values {
                prop.insert("enum".to_string(), serde_json::json!(enum_vals));
            }

            properties.insert(param.name.clone(), serde_json::Value::Object(prop));

            if param.required {
                required.push(param.name.clone());
            }
        }

        // 如果有请求体，添加 body 参数
        if let Some(ref body) = self.request_body {
            let body_prop = if let Some(ref schema) = body.schema {
                // 如果 schema 是完整的对象定义，直接使用
                if let Some(obj) = schema.as_object() {
                    let mut prop = obj.clone();
                    // 确保有 description
                    if !body.description.is_empty() && !prop.contains_key("description") {
                        prop.insert(
                            "description".to_string(),
                            serde_json::json!(body.description),
                        );
                    }
                    serde_json::Value::Object(prop)
                } else {
                    // schema 是 properties 对象
                    serde_json::json!({
                        "type": "object",
                        "description": body.description,
                        "properties": schema
                    })
                }
            } else {
                serde_json::json!({
                    "type": "object",
                    "description": body.description
                })
            };
            properties.insert("body".to_string(), body_prop);

            if body.required {
                required.push("body".to_string());
            }
        }

        serde_json::json!({
            "type": "object",
            "properties": properties,
            "required": required
        })
    }
}

/// API 存储文件格式 (类似 OpenAPI 规范)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStore {
    /// 版本
    pub version: String,
    /// 信息
    pub info: ApiStoreInfo,
    /// API 列表
    pub apis: Vec<ApiDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiStoreInfo {
    /// 标题
    pub title: String,
    /// 描述
    #[serde(default)]
    pub description: String,
    /// 版本
    pub version: String,
}

impl Default for ApiStore {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            info: ApiStoreInfo {
                title: "MCP OpenAPI Store".to_string(),
                description: "API definitions for MCP tools".to_string(),
                version: "1.0.0".to_string(),
            },
            apis: Vec::new(),
        }
    }
}
