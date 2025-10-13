use axum::routing::post;
use axum::{Router};
use leptos::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use std::net::SocketAddr;

mod handlers;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    // mount the leptos app
    let routes = generate_route_list(|cx| view! { cx, <app::App/> });

    let app = Router::new()
        .route("/upload", post(handlers::upload_handler))
        .leptos_routes(routes);

    let addr = SocketAddr::from(([127,0,0,1], 8080));
    tracing::info!("Server listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}
