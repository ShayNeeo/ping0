use axum::{extract::Multipart, routing::post, Router, response::IntoResponse, http::StatusCode};
use std::net::SocketAddr;
use uuid::Uuid;
use qrcode::QrCode;
use qrcode::render::svg;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let app = Router::new().route("/upload", post(upload_handler));

    let addr = SocketAddr::from(([127,0,0,1], 8080));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr).serve(app.into_make_service()).await?;

    Ok(())
}

async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    // Basic example: accept a file field named "file" and return an id + QR (svg)
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("");
        if name == "file" {
            let data = field.bytes().await.unwrap_or_default();
            let id = Uuid::new_v4();
            // TODO: persist the file to disk or object store; here we skip
            let link = format!("https://0.id.vn/p/{}", id);
            let code = QrCode::new(link.as_bytes()).unwrap();
            let svg = code.render().min_dimensions(200,200).build();
            let body = format!("{{\"id\":\"{}\", \"link\":\"{}\", \"qr_svg\": \"{}\"}}", id, link, svg.replace('\"', "\\\""));
            return (StatusCode::OK, body);
        }
    }
    (StatusCode::BAD_REQUEST, "no file provided".to_string())
}
