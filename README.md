# MCP OpenAPI

一个基于 Rust 开发的 MCP (Model Context Protocol) 服务，用于将 HTTP API 封装为 MCP 工具。

## 功能特性

- **多种传输方式**: 支持 stdio 和 Streamable HTTP 两种传输模式
- **API 管理工具**: 内置工具用于查询、新增、删除、启用/禁用 API
- **动态 API 工具**: 注册的 API 会自动成为可调用的 MCP 工具
- **持久化存储**: API 定义保存在 JSON 文件中（格式类似 OpenAPI 规范）
- **多种认证方式**: 支持 API Key、Bearer Token、Basic Auth
- **参数支持**: 支持 Path、Query、Header、Body 参数

## 安装

```bash
cargo build --release
```

## 使用方法

### 命令行参数

```
Usage: mcp-openapi [OPTIONS]

Options:
  -t, --transport <TRANSPORT>  传输模式: stdio 或 http [默认: stdio]
      --host <HOST>            HTTP 服务器地址 (仅 http 模式) [默认: 127.0.0.1]
  -p, --port <PORT>            HTTP 服务器端口 (仅 http 模式) [默认: 3000]
  -s, --store <STORE>          API 存储文件路径 [环境变量: MCP_OPENAPI_STORE]
  -h, --help                   显示帮助信息
  -V, --version                显示版本信息
```

### 启动服务

```bash
# stdio 模式 (默认)
./target/release/mcp-openapi

# Streamable HTTP 模式
./target/release/mcp-openapi -t http -p 3000

# 指定存储文件路径
./target/release/mcp-openapi -s /path/to/apis.json

# HTTP 模式 + 自定义端口和存储
./target/release/mcp-openapi -t http -p 8080 -s /path/to/apis.json
```

### 配置 Claude Desktop (stdio 模式)

在 Claude Desktop 配置文件中添加：

```json
{
  "mcpServers": {
    "openapi": {
      "command": "/path/to/mcp-openapi",
      "env": {
        "MCP_OPENAPI_STORE": "/path/to/apis.json"
      }
    }
  }
}
```

## 内置工具

### list_apis

列出所有已注册的 API。

参数：
- `status` (可选): 筛选状态，可选值: `all`, `enabled`, `disabled`
- `tag` (可选): 按标签筛选

### add_api

添加新的 API 定义。

参数：
- `name` (必需): API 名称，将作为工具名称
- `description` (必需): API 描述，将作为工具描述
- `base_url` (必需): API 基础 URL
- `path` (必需): API 路径，支持路径参数如 `/users/{id}`
- `method` (必需): HTTP 方法
- `parameters` (可选): 参数列表
- `request_body` (可选): 请求体定义
- `authentication` (可选): 认证配置
- `headers` (可选): 默认请求头
- `tags` (可选): 标签列表

### delete_api

删除 API。

参数：
- `id` 或 `name`: API ID 或名称

### enable_api

启用已禁用的 API。

参数：
- `id` 或 `name`: API ID 或名称

### disable_api

禁用 API（保留但不显示为工具）。

参数：
- `id` 或 `name`: API ID 或名称

## API 定义格式

```json
{
  "id": "unique-id",
  "name": "api_name",
  "description": "API description for the tool",
  "base_url": "https://api.example.com",
  "path": "/users/{id}",
  "method": "GET",
  "parameters": [
    {
      "name": "id",
      "description": "User ID",
      "in": "path",
      "required": true,
      "type": "string"
    },
    {
      "name": "include",
      "description": "Fields to include",
      "in": "query",
      "required": false,
      "type": "string"
    }
  ],
  "request_body": {
    "content_type": "application/json",
    "required": true,
    "description": "Request body description",
    "schema": {}
  },
  "authentication": {
    "type": "bearer",
    "token": "your-token"
  },
  "headers": {
    "Accept": "application/json"
  },
  "status": "enabled",
  "tags": ["user", "internal"]
}
```

## 认证类型

### 无认证
```json
{
  "type": "none"
}
```

### API Key
```json
{
  "type": "api_key",
  "header_name": "X-API-Key",
  "api_key": "your-api-key"
}
```

### Bearer Token
```json
{
  "type": "bearer",
  "token": "your-token"
}
```

### Basic Auth
```json
{
  "type": "basic",
  "username": "user",
  "password": "pass"
}
```

## 示例

### 添加一个 GET API

```json
{
  "name": "get_weather",
  "description": "获取指定城市的天气信息",
  "base_url": "https://api.weather.com",
  "path": "/v1/weather",
  "method": "GET",
  "parameters": [
    {
      "name": "city",
      "description": "城市名称",
      "in": "query",
      "required": true,
      "type": "string"
    }
  ],
  "authentication": {
    "type": "api_key",
    "header_name": "X-API-Key",
    "api_key": "your-key"
  }
}
```

### 添加一个 POST API

```json
{
  "name": "create_user",
  "description": "创建新用户",
  "base_url": "https://api.example.com",
  "path": "/users",
  "method": "POST",
  "request_body": {
    "content_type": "application/json",
    "required": true,
    "description": "用户信息"
  },
  "authentication": {
    "type": "bearer",
    "token": "your-token"
  }
}
```

## 许可证

MIT
