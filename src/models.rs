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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ParameterType {
    #[default]
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApiStatus {
    /// 启用
    #[default]
    Enabled,
    /// 禁用
    Disabled,
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Authentication {
    /// 无认证
    #[default]
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
    /// 变量存储（用于环境变量替换）
    #[serde(default)]
    pub variables: HashMap<String, String>,
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
            variables: HashMap::new(),
        }
    }
}

/// 替换字符串中的变量占位符 ${VAR_NAME}
///
/// 支持语法：
/// - ${VAR_NAME} - 替换为 variables 中 VAR_NAME 的值
/// - 如果变量不存在，保留原始占位符
///
/// # 参数
/// - `s` - 包含占位符的字符串
/// - `variables` - 变量名到值的映射
///
/// # 示例
/// ```
/// let mut vars = std::collections::HashMap::new();
/// vars.insert("TEST_VAR".to_string(), "hello");
/// assert_eq!(substitute_vars("prefix_${TEST_VAR}_suffix", &vars), "prefix_hello_suffix");
/// ```
pub fn substitute_vars(s: &str, variables: &HashMap<String, String>) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    let mut in_var = false;
    let mut var_name = String::new();

    while let Some(c) = chars.next() {
        if !in_var {
            if c == '$'
                && let Some(&'{') = chars.peek()
            {
                // 开始变量 ${...
                chars.next(); // 消耗 '{'
                in_var = true;
                var_name.clear();
                continue;
            }
            result.push(c);
        } else if c == '}' {
            // 变量结束
            if let Some(value) = variables.get(&var_name) {
                result.push_str(value);
            } else {
                // 变量不存在，保留原始占位符
                result.push_str("${");
                result.push_str(&var_name);
                result.push('}');
            }
            in_var = false;
            var_name.clear();
        } else {
            var_name.push(c);
        }
    }

    // 处理未闭合的变量（保留原样）
    if in_var {
        result.push_str("${");
        result.push_str(&var_name);
    }

    result
}

/// 对字符串进行递归变量替换
///
/// 允许变量的值中包含其他变量引用
pub fn substitute_vars_recursive(s: &str, variables: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    let mut max_iterations = 10; // 防止无限循环

    loop {
        let new_result = substitute_vars(&result, variables);
        if new_result == result || max_iterations == 0 {
            break;
        }
        result = new_result;
        max_iterations -= 1;
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_substitute_vars() {
        let mut vars = HashMap::new();
        vars.insert("MCP_TEST_VAR".to_string(), "test_value".to_string());
        vars.insert("MCP_NUM".to_string(), "123".to_string());

        assert_eq!(substitute_vars("hello", &vars), "hello");
        assert_eq!(substitute_vars("${MCP_TEST_VAR}", &vars), "test_value");
        assert_eq!(
            substitute_vars("prefix_${MCP_TEST_VAR}_suffix", &vars),
            "prefix_test_value_suffix"
        );
        assert_eq!(
            substitute_vars("${MCP_TEST_VAR}_${MCP_NUM}", &vars),
            "test_value_123"
        );
        assert_eq!(substitute_vars("${NON_EXISTENT}", &vars), "${NON_EXISTENT}");
        assert_eq!(substitute_vars("$", &vars), "$");
        assert_eq!(substitute_vars("${", &vars), "${");
        assert_eq!(substitute_vars("${UNCLOSED", &vars), "${UNCLOSED");
    }

    #[test]
    fn test_substitute_vars_recursive() {
        let mut vars = HashMap::new();
        vars.insert("MCP_OUTER".to_string(), "${MCP_INNER}".to_string());
        vars.insert("MCP_INNER".to_string(), "final_value".to_string());

        assert_eq!(
            substitute_vars_recursive("${MCP_OUTER}", &vars),
            "final_value"
        );
    }
}
