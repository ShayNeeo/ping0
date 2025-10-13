use axum::routing::{get, post};
use axum::{Router};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes, LeptosOptions, handle_server_fns};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use axum::routing::get_service;
use axum::http::StatusCode;

mod handlers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // mount the leptos app
    let routes = generate_route_list(|cx| view! { cx, <ping0_app::App/> });

    let state = LeptosOptions::builder().build();

    let app = Router::new()
        .route("/upload", post(handlers::upload_handler))
        .route("/link", post(handlers::link_handler))
        .nest_service("/files", get_service(ServeDir::new("uploads")).handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "IO Error") }))
        .leptos_routes(routes, &state, vec![])
        .route("/api/*fn_name", get(handle_server_fns).post(handle_server_fns))
        .nest_service("/", get_service(ServeDir::new("pkg")).handle_error(|_| async { (StatusCode::INTERNAL_SERVER_ERROR, "IO Error") }));

    let addr = SocketAddr::from(([127,0,0,1], 8080));
    tracing::info!("Server listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
