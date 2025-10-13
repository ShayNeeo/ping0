use axum::routing::{get, post};
use axum::{Router, Json};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use axum::http::{Method, HeaderValue};
use axum::routing::get_service;
use axum::http::StatusCode;
use serde_json::json;

mod handlers;

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "ping0"
    }))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing with better formatting for production
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // Get configuration from environment variables
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    
    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    
    // Set base URL for the application
    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| format!("http://{}:{}", host, port));
    
    tracing::info!("Base URL: {}", base_url);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/upload", post(handlers::upload_handler))
        .route("/link", post(handlers::link_handler))
        // CORS: allow requests from the frontend hosted on Cloudflare Pages (https://0.id.vn)
        // Adjust the allowed origin to your Pages domain(s) if different.
        .layer(
            CorsLayer::new()
                .allow_origin(HeaderValue::from_static("https://0.id.vn"))
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any)
        )
        .nest_service("/files", get_service(ServeDir::new("uploads")).handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "IO Error") }))
        .nest_service("/", get_service(ServeDir::new("pkg")).handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "IO Error") }));

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("🚀 Server listening on {}", addr);
    
    // Use Axum 0.6 API
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
