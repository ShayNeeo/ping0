use axum::extract::Multipart;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use uuid::Uuid;
use qrcode::QrCode;

pub async fn upload_handler(mut multipart: Multipart) -> impl IntoResponse {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("");
        if name == "file" {
            let _data = field.bytes().await.unwrap_or_default();
            let id = Uuid::new_v4();
            let link = format!("https://0.id.vn/p/{}", id);
            let code = QrCode::new(link.as_bytes()).unwrap();
            let svg = code.render().min_dimensions(200,200).build();
            let body = format!("{{\"id\":\"{}\", \"link\":\"{}\", \"qr_svg\": \"{}\"}}", id, link, svg.replace('\"', "\\\""));
            return (StatusCode::OK, body);
        }
    }
    (StatusCode::BAD_REQUEST, "no file provided".to_string())
}
