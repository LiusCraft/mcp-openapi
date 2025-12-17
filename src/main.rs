mod handler;
mod models;
mod service;
mod storage;

use anyhow::Result;
use axum::Router;
use clap::{Parser, ValueEnum};
use handler::OpenApiHandler;
use rmcp::ServiceExt;
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
};
use service::OpenApiService;
use std::path::PathBuf;
use std::sync::Arc;
use storage::ApiStorageManager;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// MCP OpenAPI Server - Manage and call HTTP APIs as MCP tools
#[derive(Parser, Debug)]
#[command(name = "mcp-openapi")]
#[command(version, about, long_about = None)]
struct Args {
    /// Transport mode: stdio or http
    #[arg(short, long, default_value = "stdio")]
    transport: TransportMode,

    /// HTTP server host (only for http mode)
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    /// HTTP server port (only for http mode)
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Path to API storage file
    #[arg(short, long, env = "MCP_OPENAPI_STORE")]
    store: Option<PathBuf>,

    /// Disable management tools (add_api, delete_api, etc.)
    #[arg(long)]
    nomg: bool,
}

#[derive(Debug, Clone, ValueEnum)]
enum TransportMode {
    Stdio,
    Http,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let args = Args::parse();

    // 初始化日志
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mcp_openapi=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting MCP OpenAPI server...");

    // 获取存储文件路径
    let storage_path = args.store.unwrap_or_else(|| {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("mcp-openapi")
            .join("apis.json")
    });

    tracing::info!("Using storage file: {}", storage_path.display());

    // 创建存储管理器
    let storage = Arc::new(ApiStorageManager::new(storage_path).await?);

    // 创建服务 (当 nomg 为 true 时禁用管理工具)
    let enable_management = !args.nomg;
    let service = Arc::new(OpenApiService::new(storage, enable_management));

    // 创建 Handler
    let handler = OpenApiHandler::new(service);

    match args.transport {
        TransportMode::Stdio => {
            run_stdio(handler).await?;
        }
        TransportMode::Http => {
            run_http(handler, &args.host, args.port).await?;
        }
    }

    tracing::info!("MCP OpenAPI server stopped");

    Ok(())
}

async fn run_stdio(handler: OpenApiHandler) -> Result<()> {
    tracing::info!("Starting stdio transport...");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let server = handler.serve((stdin, stdout)).await?;
    server.waiting().await?;

    Ok(())
}

async fn run_http(handler: OpenApiHandler, host: &str, port: u16) -> Result<()> {
    let addr = format!("{}:{}", host, port);
    tracing::info!("Starting Streamable HTTP transport on http://{}", addr);

    let ct = CancellationToken::new();
    let config = StreamableHttpServerConfig {
        cancellation_token: ct.clone(),
        ..Default::default()
    };

    let session_manager = Arc::new(LocalSessionManager::default());

    let service = StreamableHttpService::new(move || Ok(handler.clone()), session_manager, config);

    let app = Router::new().route("/mcp", axum::routing::any_service(service));

    let listener = TcpListener::bind(&addr).await?;

    tracing::info!("MCP OpenAPI server listening on http://{}", addr);
    tracing::info!("MCP endpoint: POST http://{}/mcp", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            tokio::signal::ctrl_c().await.ok();
            ct.cancel();
        })
        .await?;

    Ok(())
}
