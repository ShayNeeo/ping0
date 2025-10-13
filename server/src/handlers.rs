use axum::extract::{Multipart, Form};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;
use qrcode::QrCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct LinkRequest {
    pub link: String,
}

pub async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("");
        if name == "file" {
            let filename = field.file_name().unwrap_or("file").to_string();
            let data = field.bytes().await.unwrap_or_default();
            let id = Uuid::new_v4();
            let ext = Path::new(&filename).extension().and_then(|e| e.to_str()).unwrap_or("bin");
            let filename_saved = format!("{}.{}", id, ext);
            let path = format!("uploads/{}", filename_saved);
            if let Err(_) = fs::create_dir_all("uploads").await {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create uploads dir".to_string());
            }
            if let Err(_) = fs::write(&path, &data).await {
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file".to_string());
            }
            let link = format!("https://0.id.vn/files/{}", filename_saved);
            let code = QrCode::new(link.as_bytes()).unwrap();
            let svg = code.render().min_dimensions(200,200).build();
            let body = format!("{{\"id\":\"{}\", \"link\":\"{}\", \"qr_svg\": \"{}\"}}", id, link, svg.replace('\"', "\\\""));
            return (StatusCode::OK, body);
        }
    }
    (StatusCode::BAD_REQUEST, "no file provided".to_string())
}

pub async fn link_handler(Form(req): Form<LinkRequest>) -> impl IntoResponse {
    if req.link.is_empty() {
        return (StatusCode::BAD_REQUEST, "no link provided".to_string());
    }
    let code = QrCode::new(req.link.as_bytes()).unwrap();
    let svg = code.render().min_dimensions(200,200).build();
    let body = format!("{{\"link\":\"{}\", \"qr_svg\": \"{}\"}}", req.link, svg.replace('\"', "\\\""));
    (StatusCode::OK, body)
}
