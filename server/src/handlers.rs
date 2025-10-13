use axum::extract::{Form, Multipart, Path as AxumPath, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect};
use mime_guess::from_path as mime_from_path;
use nanoid::nanoid;
use qrcode::render::svg::Color;
use qrcode::QrCode;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;
use askama::Template;
use ping0::templates::{IndexTemplate, ResultTemplate, ImageOgTemplate};

// Maximum file size: 10MB
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

// Allowed file extensions for uploads
const ALLOWED_EXTENSIONS: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "pdf", "txt"];

#[derive(Clone)]
pub struct AppState { pub db_path: String, pub base_url: String }

#[derive(Deserialize)]
pub struct LinkRequest { pub link: String, pub qr: Option<String> }

fn get_base_url() -> String { std::env::var("BASE_URL").unwrap_or_else(|_| "https://0.id.vn".to_string()) }

fn is_allowed_extension(ext: &str) -> bool {
    ALLOWED_EXTENSIONS.iter().any(|&allowed| allowed.eq_ignore_ascii_case(ext))
}

pub async fn upload_handler(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
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
            
            // Save mapping in DB with a fresh shortcode
            let short_code = nanoid!(8);
            let original = format!("file:{}", filename_saved);
            let conn = Connection::open(&state.db_path).unwrap();
            conn.execute(
                "INSERT INTO items(code, kind, value, created_at) VALUES (?1, ?2, ?3, strftime('%s','now'))",
                params![short_code, "file", original],
            ).ok();

            let short_link = format!("{}/s/{}", state.base_url, short_code);
            let qr_svg = QrCode::new(short_link.as_bytes())
                .map(|c| c.render::<Color>().min_dimensions(200, 200).build())
                .unwrap_or_default();

            let body = format!(
                "{{\"code\":\"{}\", \"short\":\"{}\", \"file\":\"{}\", \"qr_svg\": \"{}\"}}",
                short_code,
                short_link,
                filename_saved,
                qr_svg.replace('\"', "\\\""),
            );

            tracing::info!("File uploaded successfully: {} ({} bytes)", filename_saved, data.len());
            return (StatusCode::OK, body);
        }
    }
    (StatusCode::BAD_REQUEST, "No file provided".to_string())
}

pub async fn link_handler(State(state): State<AppState>, Form(req): Form<LinkRequest>) -> impl IntoResponse {
    if req.link.is_empty() {
        return (StatusCode::BAD_REQUEST, "No link provided".to_string());
    }
    
    // Basic URL validation
    if !req.link.starts_with("http://") && !req.link.starts_with("https://") {
        return (StatusCode::BAD_REQUEST, "Invalid URL format. Must start with http:// or https://".to_string());
    }

    let short_code = nanoid!(8);
    let conn = Connection::open(&state.db_path).unwrap();
    conn.execute(
        "INSERT INTO items(code, kind, value, created_at) VALUES (?1, ?2, ?3, strftime('%s','now'))",
        params![short_code, "url", req.link],
    ).ok();

    let short_link = format!("{}/s/{}", state.base_url, short_code);
    let qr_svg = if matches!(req.qr.as_deref(), Some("on")) {
        QrCode::new(short_link.as_bytes())
            .map(|c| c.render::<Color>().min_dimensions(200, 200).build())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let body = format!(
        "{{\"code\":\"{}\", \"short\":\"{}\", \"qr_svg\": \"{}\"}}",
        short_code,
        short_link,
        qr_svg.replace('\"', "\\\""),
    );

    tracing::info!("Short link created for URL: {} -> {}", req.link, short_link);
    (StatusCode::OK, body)
}

pub async fn index_handler() -> Html<String> { Html(IndexTemplate.render().unwrap_or_else(|_| "Template error".to_string())) }

#[derive(Deserialize)]
pub struct SubmitForm { pub link: Option<String>, pub qr: Option<String> }

pub async fn submit_handler(State(state): State<AppState>, mut multipart: Multipart) -> axum::response::Response {
    // Try to parse multipart fields manually to support both link and file in one form
    let mut link_value: Option<String> = None;
    let mut file_bytes: Option<(String, Vec<u8>)> = None;
    let mut want_qr: bool = false;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "link" => {
                if let Ok(text) = field.text().await { if !text.trim().is_empty() { link_value = Some(text.trim().to_string()); } }
            }
            "file" => {
                if let Some(fname) = field.file_name().map(|s| s.to_string()) {
                    if let Ok(bytes) = field.bytes().await { file_bytes = Some((fname, bytes.to_vec())); }
                }
            }
            "qr" => { want_qr = true; }
            _ => {}
        }
    }

    // If file present, prioritize file upload mapping
    if let Some((filename, data)) = file_bytes {
        let ext = Path::new(&filename).extension().and_then(|e| e.to_str()).unwrap_or("bin");
        if !is_allowed_extension(ext) { return (StatusCode::BAD_REQUEST, "File type not allowed".to_string()).into_response(); }
        if data.len() > MAX_FILE_SIZE { return (StatusCode::PAYLOAD_TOO_LARGE, "File too large".to_string()).into_response(); }

        let id = Uuid::new_v4();
        let filename_saved = format!("{}.{}", id, ext);
        let path = format!("uploads/{}", filename_saved);
        if let Err(e) = fs::create_dir_all("uploads").await { tracing::error!("create uploads dir: {}", e); return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create uploads directory".to_string()).into_response(); }
        if let Err(e) = fs::write(&path, &data).await { tracing::error!("save file: {}", e); return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file".to_string()).into_response(); }

        let short_code = nanoid!(8);
        let original = format!("file:{}", filename_saved);
        let conn = Connection::open(&state.db_path).unwrap();
        conn.execute("INSERT INTO items(code, kind, value, created_at) VALUES (?1, ?2, ?3, strftime('%s','now'))", params![short_code, "file", original]).ok();
        let redirect_to = format!("/r/{}?qr={}", short_code, if want_qr {"1"} else {"0"});
        return Redirect::to(&redirect_to).into_response();
    }

    // Else if link provided, validate and create mapping
    if let Some(link) = link_value {
        if !link.starts_with("http://") && !link.starts_with("https://") { return (StatusCode::BAD_REQUEST, "Invalid URL format".to_string()).into_response(); }
        let short_code = nanoid!(8);
        let conn = Connection::open(&state.db_path).unwrap();
        conn.execute("INSERT INTO items(code, kind, value, created_at) VALUES (?1, ?2, ?3, strftime('%s','now'))", params![short_code, "url", link]).ok();
        let redirect_to = format!("/r/{}?qr={}", short_code, if want_qr {"1"} else {"0"});
        return Redirect::to(&redirect_to).into_response();
    }

    (StatusCode::BAD_REQUEST, "Provide a URL or a file".to_string()).into_response()
}

pub async fn result_handler(State(state): State<AppState>, AxumPath(code): AxumPath<String>, Query(q): Query<std::collections::HashMap<String,String>>) -> Html<String> {
    let conn = Connection::open(&state.db_path).unwrap();
    let mut stmt = conn.prepare("SELECT kind, value FROM items WHERE code = ?1").unwrap();
    let row = stmt.query_row(params![code.clone()], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)));
    let (_kind, _value) = match row { Ok(v) => v, Err(_) => return Html("<h1>Not found</h1>".to_string()) };
    let short_link = format!("{}/s/{}", state.base_url, code);
    let qr_svg = if q.get("qr").map(|v| v=="1").unwrap_or(false) {
        QrCode::new(short_link.as_bytes()).map(|c| c.render::<Color>().min_dimensions(200,200).build()).unwrap_or_default()
    } else { String::new() };
    let tpl = ResultTemplate { code, short_link, qr_svg: if qr_svg.is_empty() { None } else { Some(qr_svg) } };
    Html(tpl.render().unwrap_or_else(|_| "Template error".to_string()))
}

pub async fn short_handler(State(state): State<AppState>, AxumPath(code): AxumPath<String>) -> axum::response::Response {
    // Query DB in a separate scope so non-Send types are dropped before any await
    let (kind, value) = {
        let conn = Connection::open(&state.db_path).unwrap();
        let mut stmt = conn.prepare("SELECT kind, value FROM items WHERE code = ?1").unwrap();
        let row = stmt.query_row(params![code.clone()], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)));
        match row { Ok(v) => v, Err(_) => return (StatusCode::NOT_FOUND, "Not found").into_response() }
    };

    match kind.as_str() {
        "url" => Redirect::permanent(&value).into_response(),
        "file" => {
            let filename = value.strip_prefix("file:").unwrap_or(&value);
            let file_path = PathBuf::from("uploads").join(filename);
            if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
                let mime = mime_from_path(filename).first_or_octet_stream();
                if ["jpg","jpeg","png","gif","webp"].iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    // Render OG meta page for image
                    let image_url = format!("{}/files/{}", state.base_url, filename);
                    let tpl = ImageOgTemplate { image_url };
                    return Html(tpl.render().unwrap_or_else(|_| "Template error".to_string())).into_response();
                }
                // Non-image: serve file directly
                if let Ok(bytes) = tokio::fs::read(&file_path).await {
                    let mut headers = HeaderMap::new();
                    headers.insert(axum::http::header::CONTENT_TYPE, HeaderValue::from_str(mime.as_ref()).unwrap_or(HeaderValue::from_static("application/octet-stream")));
                    return (headers, bytes).into_response();
                }
            }
            (StatusCode::NOT_FOUND, "File not found").into_response()
        }
        _ => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}
