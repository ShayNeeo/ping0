use axum::extract::{Form, Multipart, Path as AxumPath, Query, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{Html, IntoResponse, Redirect};
use axum::Json;
use mime_guess::from_path as mime_from_path;
use nanoid::nanoid;
use qrcode::render::svg::Color;
use qrcode::QrCode;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;
use askama::Template;
use ping0::templates::{IndexTemplate, ResultTemplate, ImageOgTemplate, FileInfoTemplate, AdminLoginTemplate, AdminHomeTemplate, AdminItemsTemplate};
use sha2::{Digest, Sha256};
use rand::{distributions::Alphanumeric, Rng};

// Maximum file size: 1 GiB
const MAX_FILE_SIZE: usize = 1024 * 1024 * 1024;

// Allowed file extensions for uploads
const ALLOWED_EXTENSIONS: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "svg",
    "pdf", "txt", "md", "csv", "json",
    "doc", "docx", "xls", "xlsx", "ppt", "pptx",
    "zip", "tar", "gz", "rar", "7z",
    "mp3", "mp4", "mov", "webm"
];

#[derive(Clone)]
pub struct AppState { pub db_path: String, pub base_url: String }

#[derive(Deserialize)]
pub struct LinkRequest { pub link: String, pub qr: Option<String> }

fn get_base_url() -> String { std::env::var("BASE_URL").unwrap_or_else(|_| "https://0.id.vn".to_string()) }

fn is_allowed_extension(ext: &str) -> bool {
    ALLOWED_EXTENSIONS.iter().any(|&allowed| allowed.eq_ignore_ascii_case(ext))
}

fn ensure_absolute(base: &str, url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("{}/{}", base.trim_end_matches('/'), url.trim_start_matches('/'))
    }
}

pub async fn upload_handler(State(state): State<AppState>, mut multipart: Multipart) -> impl IntoResponse {
    while let Ok(Some(mut field)) = multipart.next_field().await {
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
            
            // Stream to disk with size limit
            let id = Uuid::new_v4();
            let filename_saved = format!("{}.{}", id, ext);
            let path = format!("uploads/{}", filename_saved);
            if let Err(e) = fs::create_dir_all("uploads").await {
                tracing::error!("Failed to create uploads dir: {}", e);
                return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to create uploads directory".to_string());
            }
            let mut out = match tokio::fs::File::create(&path).await {
                Ok(f) => f,
                Err(e) => {
                    tracing::error!("Failed to create file: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file".to_string());
                }
            };
            let mut written: usize = 0;
            while let Ok(Some(chunk)) = field.chunk().await {
                written = written.saturating_add(chunk.len());
                if written > MAX_FILE_SIZE {
                    let _ = tokio::fs::remove_file(&path).await;
                    return (StatusCode::PAYLOAD_TOO_LARGE, format!("File too large. Max size: {}MB", MAX_FILE_SIZE / 1024 / 1024));
                }
                if let Err(e) = out.write_all(&chunk).await {
                    tracing::error!("Write error: {}", e);
                    let _ = tokio::fs::remove_file(&path).await;
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file".to_string());
                }
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
            let qr_target = ensure_absolute(&state.base_url, &short_link);
            let qr_svg = QrCode::new(qr_target.as_bytes())
                .map(|c| c
                    .render::<Color>()
                    .min_dimensions(320, 320)
                    .quiet_zone(true)
                    .dark_color(Color("#000000"))
                    .light_color(Color("#ffffff"))
                    .build())
                .unwrap_or_default();

            let body = format!(
                "{{\"code\":\"{}\", \"short\":\"{}\", \"file\":\"{}\", \"qr_svg\": \"{}\"}}",
                short_code,
                short_link,
                filename_saved,
                qr_svg.replace('\"', "\\\""),
            );

            tracing::info!("File uploaded successfully: {}", filename_saved);
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
        let qr_target = ensure_absolute(&state.base_url, &short_link);
        QrCode::new(qr_target.as_bytes())
            .map(|c| c
                .render::<Color>()
                .min_dimensions(320, 320)
                .quiet_zone(true)
                .dark_color(Color("#000000"))
                .light_color(Color("#ffffff"))
                .build())
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
        let qr_target = ensure_absolute(&state.base_url, &short_link);
        QrCode::new(qr_target.as_bytes())
            .map(|c| c
                .render::<Color>()
                .min_dimensions(320,320)
                .quiet_zone(true)
                .dark_color(Color("#000000"))
                .light_color(Color("#ffffff"))
                .build())
            .unwrap_or_default()
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
            if let Some(ext) = Path::new(filename).extension().and_then(|e| e.to_str()) {
                let mime = mime_from_path(filename).first_or_octet_stream();
                if ["jpg","jpeg","png","gif","webp","svg"].iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    // Render OG meta page for image
                    let image_url = format!("{}/files/{}", state.base_url, filename);
                    let tpl = ImageOgTemplate { image_url };
                    return Html(tpl.render().unwrap_or_else(|_| "Template error".to_string())).into_response();
                }
                // Non-image: render info page with download link
                let filename_display = Path::new(filename).file_name().and_then(|f| f.to_str()).unwrap_or(filename).to_string();
                let file_url = format!("{}/files/{}", state.base_url, filename);
                let tpl = FileInfoTemplate { filename: filename_display, file_url, mime: mime.to_string() };
                return Html(tpl.render().unwrap_or_else(|_| "Template error".to_string())).into_response();
            }
            (StatusCode::NOT_FOUND, "File not found").into_response()
        }
        _ => (StatusCode::NOT_FOUND, "Not found").into_response(),
    }
}

// ---------- Admin auth and DB management ----------

fn hash_with_salt(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

fn generate_token(len: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(len)
        .map(char::from)
        .collect()
}

fn get_cookie_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(axum::http::header::COOKIE)?.to_str().ok()?;
    for part in cookie_header.split(';') {
        let kv = part.trim();
        if let Some(v) = kv.strip_prefix("ping0_admin=") {
            return Some(v.to_string());
        }
    }
    None
}

async fn require_admin(db_path: &str, headers: &HeaderMap) -> bool {
    if let Some(token) = get_cookie_token(headers) {
        if let Ok(conn) = Connection::open(db_path) {
            if let Ok(mut stmt) = conn.prepare("SELECT 1 FROM sessions WHERE token = ?1") {
                let exists: Result<i32, _> = stmt.query_row(params![token], |r| r.get(0));
                return exists.is_ok();
            }
        }
    }
    false
}

pub async fn admin_login_get() -> Html<String> {
    Html(AdminLoginTemplate.render().unwrap_or_else(|_| "Template error".to_string()))
}

#[derive(Deserialize)]
pub struct AdminLoginForm { pub username: String, pub password: String }

pub async fn admin_login_post(State(state): State<AppState>, Form(f): Form<AdminLoginForm>) -> impl IntoResponse {
    let conn = Connection::open(&state.db_path).unwrap();
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM admin", [], |r| r.get(0)).unwrap_or(0);
    if count == 0 {
        // First-time setup: create admin
        let salt = generate_token(16);
        let hash = hash_with_salt(&f.password, &salt);
        let _ = conn.execute("INSERT INTO admin (id, username, password_hash, salt) VALUES (1, ?1, ?2, ?3)", params![f.username, hash, salt]);
    }
    // Authenticate
    let row = conn
        .prepare("SELECT password_hash, salt FROM admin WHERE username = ?1")
        .and_then(|mut s| s.query_row(params![f.username], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))));
    let (hash, salt) = match row { Ok(v) => v, Err(_) => return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response() };
    if hash != hash_with_salt(&f.password, &salt) { return (StatusCode::UNAUTHORIZED, "Invalid credentials").into_response(); }

    // Issue session
    let token = generate_token(48);
    let _ = conn.execute("INSERT INTO sessions (token, created_at) VALUES (?1, strftime('%s','now'))", params![token.clone()]);
    let mut headers = HeaderMap::new();
    let cookie = format!("ping0_admin={}; HttpOnly; SameSite=Lax; Path=/; Max-Age=2592000", token);
    headers.insert(axum::http::header::SET_COOKIE, HeaderValue::from_str(&cookie).unwrap());
    (headers, Redirect::to("/admin")).into_response()
}

pub async fn admin_logout(State(state): State<AppState>, headers_in: HeaderMap) -> impl IntoResponse {
    if let Some(token) = get_cookie_token(&headers_in) {
        if let Ok(conn) = Connection::open(&state.db_path) { let _ = conn.execute("DELETE FROM sessions WHERE token = ?1", params![token]); }
    }
    let mut headers = HeaderMap::new();
    headers.insert(axum::http::header::SET_COOKIE, HeaderValue::from_static("ping0_admin=; Max-Age=0; Path=/"));
    (headers, Redirect::to("/admin/login")).into_response()
}

pub async fn admin_home(State(state): State<AppState>, headers: HeaderMap) -> axum::response::Response {
    if !require_admin(&state.db_path, &headers).await { return Redirect::to("/admin/login").into_response(); }
    Html(AdminHomeTemplate.render().unwrap_or_else(|_| "Template error".to_string())).into_response()
}

pub async fn admin_items(State(state): State<AppState>, headers: HeaderMap) -> axum::response::Response {
    if !require_admin(&state.db_path, &headers).await { return Redirect::to("/admin/login").into_response(); }
    let conn = Connection::open(&state.db_path).unwrap();
    let mut stmt = conn.prepare("SELECT code, kind, value, created_at FROM items ORDER BY created_at DESC LIMIT 500").unwrap();
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i64>(3)?))).unwrap();
    let mut items: Vec<(String, String, String, i64)> = Vec::new();
    for row in rows { if let Ok(rec) = row { items.push(rec); } }
    Html(AdminItemsTemplate { items }.render().unwrap_or_else(|_| "Template error".to_string())).into_response()
}

pub async fn admin_delete_item(State(state): State<AppState>, AxumPath(code): AxumPath<String>, headers: HeaderMap) -> axum::response::Response {
    if !require_admin(&state.db_path, &headers).await { return Redirect::to("/admin/login").into_response(); }
    let conn = Connection::open(&state.db_path).unwrap();
    if let Ok((kind, value)) = conn.query_row("SELECT kind, value FROM items WHERE code = ?1", params![code.clone()], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
        if kind == "file" { if let Some(fname) = value.strip_prefix("file:") { let _ = tokio::fs::remove_file(PathBuf::from("uploads").join(fname)).await; } }
    }
    let _ = conn.execute("DELETE FROM items WHERE code = ?1", params![code]);
    Redirect::to("/admin/items").into_response()
}

// SPA contract: POST /api/upload (multipart/form-data)
// fields: content (URL string or file), qr_required ("true"|"false")
// returns: { success: bool, short_url?: string, qr_code_data?: string|null, error?: string }
pub async fn api_upload(State(state): State<AppState>, mut multipart: Multipart) -> axum::response::Response {
    let mut link_value: Option<String> = None;
    let mut saved_filename: Option<String> = None;
    let mut qr_required: bool = false;

    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("");
        match name {
            "content" => {
                if let Some(fname) = field.file_name().map(|s| s.to_string()) {
                    // file upload: stream to disk with limit
                    let ext = Path::new(&fname).extension().and_then(|e| e.to_str()).unwrap_or("bin");
                    if !is_allowed_extension(ext) { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "File type not allowed"}))).into_response(); }
                    if let Err(e) = fs::create_dir_all("uploads").await { tracing::error!("create uploads dir: {}", e); return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Server error"}))).into_response(); }
                    let id = Uuid::new_v4();
                    let filename_saved = format!("{}.{}", id, ext);
                    let path = format!("uploads/{}", filename_saved);
                    let mut out = match tokio::fs::File::create(&path).await { Ok(f) => f, Err(e) => { tracing::error!("create file: {}", e); return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Server error"}))).into_response(); } };
                    let mut written: usize = 0;
                    while let Ok(Some(chunk)) = field.chunk().await {
                        written = written.saturating_add(chunk.len());
                        if written > MAX_FILE_SIZE { let _ = tokio::fs::remove_file(&path).await; return (StatusCode::PAYLOAD_TOO_LARGE, Json(serde_json::json!({"success": false, "error": "File too large"}))).into_response(); }
                        if let Err(e) = out.write_all(&chunk).await { tracing::error!("write: {}", e); let _ = tokio::fs::remove_file(&path).await; return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({"success": false, "error": "Server error"}))).into_response(); }
                    }
                    saved_filename = Some(filename_saved);
                } else if let Ok(text) = field.text().await {
                    if !text.trim().is_empty() { link_value = Some(text.trim().to_string()); }
                }
            }
            "qr_required" => {
                if let Ok(v) = field.text().await { qr_required = v.trim().eq_ignore_ascii_case("true"); }
            }
            _ => {}
        }
    }

    if let Some(filename_saved) = saved_filename {
        let short_code = nanoid!(8);
        {
            let conn = Connection::open(&state.db_path).unwrap();
            let original = format!("file:{}", filename_saved);
            conn.execute("INSERT INTO items(code, kind, value, created_at) VALUES (?1, ?2, ?3, strftime('%s','now'))", params![short_code, "file", original]).ok();
        }
        let short_url = format!("{}/s/{}", state.base_url, short_code);
        let qr_code_data = if qr_required {
            let qr_target = ensure_absolute(&state.base_url, &short_url);
            match QrCode::new(qr_target.as_bytes()) {
                Ok(c) => {
                    let image = c
                        .render::<qrcode::render::svg::Color>()
                        .min_dimensions(320,320)
                        .quiet_zone(true)
                        .dark_color(Color("#000000"))
                        .light_color(Color("#ffffff"))
                        .build();
                    let data_url = format!("data:image/svg+xml;utf8,{}", urlencoding::encode(&image));
                    Some(data_url)
                }
                Err(_) => None,
            }
        } else { None };
        return Json(serde_json::json!({"success": true, "short_url": short_url, "qr_code_data": qr_code_data})).into_response();
    }

    if let Some(link) = link_value {
        if !link.starts_with("http://") && !link.starts_with("https://") { return (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "Invalid URL"}))).into_response(); }
        let short_code = nanoid!(8);
        {
            let conn = Connection::open(&state.db_path).unwrap();
            conn.execute("INSERT INTO items(code, kind, value, created_at) VALUES (?1, ?2, ?3, strftime('%s','now'))", params![short_code, "url", link]).ok();
        }
        let short_url = format!("{}/s/{}", state.base_url, short_code);
        let qr_code_data = if qr_required {
            let qr_target = ensure_absolute(&state.base_url, &short_url);
            match QrCode::new(qr_target.as_bytes()) {
                Ok(c) => {
                    let image = c
                        .render::<qrcode::render::svg::Color>()
                        .min_dimensions(320,320)
                        .quiet_zone(true)
                        .dark_color(Color("#000000"))
                        .light_color(Color("#ffffff"))
                        .build();
                    let data_url = format!("data:image/svg+xml;utf8,{}", urlencoding::encode(&image));
                    Some(data_url)
                }
                Err(_) => None,
            }
        } else { None };
        return Json(serde_json::json!({"success": true, "short_url": short_url, "qr_code_data": qr_code_data})).into_response();
    }

    (StatusCode::BAD_REQUEST, Json(serde_json::json!({"success": false, "error": "Provide content or file"}))).into_response()
}
