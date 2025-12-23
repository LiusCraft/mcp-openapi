# 构建阶段
FROM rust:1.85-slim as builder

# 安装构建依赖
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# 创建工作目录
WORKDIR /app

# 复制 Cargo 配置文件
COPY Cargo.toml Cargo.lock ./

# 复制源代码
COPY src ./src

# 构建发布版本
RUN cargo build --release

# 运行阶段
FROM debian:bookworm-slim

# 安装运行时依赖
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# 创建非 root 用户
RUN useradd -m -u 1000 mcpuser

# 创建配置目录
RUN mkdir -p /home/mcpuser/.config/mcp-openapi && \
    chown -R mcpuser:mcpuser /home/mcpuser/.config

# 复制二进制文件
COPY --from=builder /app/target/release/mcp-openapi /usr/local/bin/mcp-openapi

# 切换到非 root 用户
USER mcpuser

# 设置工作目录
WORKDIR /home/mcpuser

# 暴露默认端口
EXPOSE 3000

# 设置环境变量
ENV RUST_LOG=info
ENV MCP_OPENAPI_STORE=/home/mcpuser/.config/mcp-openapi/apis.json

# 默认使用 HTTP 模式，监听 0.0.0.0 以便容器外部访问
CMD ["mcp-openapi", "--transport", "http", "--host", "0.0.0.0", "--port", "3000"]
