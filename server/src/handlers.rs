use axum::extract::{Form, Multipart};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;
use qrcode::QrCode;
use serde::Deserialize;
use qrcode::render::svg::Color;

// Maximum file size: 10MB
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

// Allowed file extensions for uploads
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "pdf", "txt"];

#[derive(Deserialize)]
pub struct LinkRequest {
    pub link: String,
}

fn get_base_url() -> String {
    std::env::var("BASE_URL").unwrap_or_else(|_| "https://0.id.vn".to_string())
}

fn is_allowed_extension(ext: &str) -> bool {
    ALLOWED_EXTENSIONS.iter().any(|&allowed| allowed.eq_ignore_ascii_case(ext))
}

pub async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");
        if name == "file" {
            let filename = field.file_name().unwrap_or("file").to_string();
            
            // Validate file extension
            let ext = Path::new(&filename)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("bin");
            
            if !is_allowed_extension(ext) {
                tracing::warn!("Rejected file with extension: {}", ext);
                return (StatusCode::BAD_REQUEST, format!("File type '.{}' not allowed", ext));
            }
            
            // Read bytes with size limit
            let data = match field.bytes().await {
                Ok(b) => {
                    if b.len() > MAX_FILE_SIZE {
                        tracing::warn!("File too large: {} bytes", b.len());
                        return (StatusCode::PAYLOAD_TOO_LARGE, format!("File too large. Max size: {}MB", MAX_FILE_SIZE / 1024 / 1024));
                    }
                    b.to_vec()
                },
                Err(e) => {
                    tracing::error!("Failed to read file data: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file data".to_string());
                },
            };
            
            let id = Uuid::new_v4();
            let filename_saved = format!("{}.{}", id, ext);
            let path = format!("uploads/{}", filename_saved);
            
            if let Err(e) = fs::create_dir_all("uploads").await {
                tracing::error!("Failed to create uploads dir: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create uploads directory".to_string());
            }
            
            if let Err(e) = fs::write(&path, &data).await {
                tracing::error!("Failed to save file: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file".to_string());
            }
            
            let base_url = get_base_url();
            let link = format!("{}/files/{}", base_url, filename_saved);
            
            let code = match QrCode::new(link.as_bytes()) {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Failed to generate QR code: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate QR code".to_string());
                }
            };
            
            let svg: String = code.render::<Color>().min_dimensions(200, 200).build();
            let body = format!("{{\"id\":\"{}\", \"link\":\"{}\", \"qr_svg\": \"{}\"}}", id, link, svg.replace('\"', "\\\""));
            
            tracing::info!("File uploaded successfully: {} ({} bytes)", filename_saved, data.len());
            return (StatusCode::OK, body);
        }
    }
    (StatusCode::BAD_REQUEST, "No file provided".to_string())
}

pub async fn link_handler(Form(req): Form<LinkRequest>) -> impl IntoResponse {
    if req.link.is_empty() {
        return (StatusCode::BAD_REQUEST, "No link provided".to_string());
    }
    
    // Basic URL validation
    if !req.link.starts_with("http://") && !req.link.starts_with("https://") {
        return (StatusCode::BAD_REQUEST, "Invalid URL format. Must start with http:// or https://".to_string());
    }
    
    let code = match QrCode::new(req.link.as_bytes()) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to generate QR code: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to generate QR code".to_string());
        }
    };
    
    let svg: String = code.render::<Color>().min_dimensions(200, 200).build();
    let body = format!("{{\"link\":\"{}\", \"qr_svg\": \"{}\"}}", req.link, svg.replace('\"', "\\\""));
    
    tracing::info!("QR code generated for link: {}", req.link);
    (StatusCode::OK, body)
}
