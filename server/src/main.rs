use axum::routing::{get, post};
use axum::{Router, Json};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tower_http::cors::{CorsLayer, Any};
use axum::http::Method;
use axum::routing::get_service;
use axum::http::StatusCode;
use serde_json::json;
use rusqlite::Connection;

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

    // Initialize SQLite (file-based)
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "data/ping0.db".to_string());
    std::fs::create_dir_all("data").ok();
    let conn = Connection::open(&db_path)?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS items (
            code TEXT PRIMARY KEY,
            kind TEXT NOT NULL,        -- 'url' | 'file'
            value TEXT NOT NULL,       -- url or 'file:filename'
            created_at INTEGER NOT NULL
        );
        "#,
    )?;

    let base_url = std::env::var("BASE_URL")
        .unwrap_or_else(|_| format!("http://{}:{}", host, port));

    let app_state = handlers::AppState { db_path: db_path.clone(), base_url: base_url.clone() };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/", get(handlers::index_handler))
        .route("/submit", post(handlers::submit_handler))
        .route("/upload", post(handlers::upload_handler))
        .route("/link", post(handlers::link_handler))
        .route("/api/upload", post(handlers::api_upload))
        .route("/r/:code", get(handlers::result_handler))
        .route("/s/:code", get(handlers::short_handler))
        .with_state(app_state)
        // CORS: allow requests from the frontend hosted on Cloudflare Pages (https://0.id.vn)
        // Adjust the allowed origin to your Pages domain(s) if different.
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any)
        )
        .nest_service("/files", get_service(ServeDir::new("uploads")).handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "IO Error") }));

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("🚀 Server listening on {}", addr);
    
    // Use Axum 0.6 API
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
