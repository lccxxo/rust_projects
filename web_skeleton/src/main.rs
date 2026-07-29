//! 程序入口：加载配置、初始化日志、构建应用、启动服务。

mod app;
mod config;
mod error;
mod extract;
mod middleware;
mod models;
mod repository;
mod response;
mod routes;
mod services;
mod state;

use std::net::SocketAddr;

use crate::config::AppConfig;
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 加载 .env（存在则加载，不存在也不报错）
    dotenvy::dotenv().ok();

    // 2. 加载配置
    let config = AppConfig::from_env()?;

    // 3. 初始化日志（RUST_LOG 环境变量可覆盖，默认取配置里的等级）
    init_tracing(&config.log_level);

    // 4. 构建共享状态
    let state = AppState::new(config.clone());

    // 5. 组装路由与中间件
    let app = app::build_router(state);

    // 6. 启动服务
    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    tracing::info!("🚀 服务已启动: http://{}", addr);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;

    Ok(())
}

/// 初始化结构化日志订阅器。
fn init_tracing(default_level: &str) {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};

    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(default_level));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}
