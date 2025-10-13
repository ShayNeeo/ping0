use axum::routing::{get, post};
use axum::{Router, Json};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes, handle_server_fns, leptos_config::LeptosOptions};
use ping0_app::App;
use std::net::SocketAddr;
use tower_http::services::ServeDir;
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

    // mount the leptos app
    let routes = generate_route_list(|cx| view! { cx, <ping0_app::App/> });

    let state = LeptosOptions::builder()
        .output_name("ping0-app")
        .site_root("")
        .site_pkg_dir("pkg")
        .env(leptos_config::Env::PROD)
        .site_addr(([0, 0, 0, 0], port).into())
        .reload_port(3001)
        .build();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/upload", post(handlers::upload_handler))
        .route("/link", post(handlers::link_handler))
        .nest_service("/files", get_service(ServeDir::new("uploads")).handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "IO Error") }))
        .leptos_routes(routes.clone(), &state, |cx| view! { cx, <ping0_app::App/> })
        .route("/api/*fn_name", get(handle_server_fns).post(handle_server_fns))
        .nest_service("/", get_service(ServeDir::new("pkg")).handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "IO Error") }));

    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("🚀 Server listening on {}", addr);
    
    // Use modern tokio API instead of deprecated axum::Server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
