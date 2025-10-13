use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UploadResponse {
    pub id: String,
    pub link: String,
    pub qr_svg: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkResponse {
    pub link: String,
    pub qr_svg: String,
}

#[cfg(feature = "ssr")]
#[server(UploadFile, "/api")]
pub async fn upload_file(file: Vec<u8>, filename: String) -> Result<UploadResponse, ServerFnError> {
    use std::path::Path;
    use tokio::fs;
    use uuid::Uuid;
    use qrcode::QrCode;

    let id = Uuid::new_v4();
    let ext = Path::new(&filename).extension().and_then(|e| e.to_str()).unwrap_or("bin");
    let filename_saved = format!("{}.{}", id, ext);
    let path = format!("uploads/{}", filename_saved);
    fs::create_dir_all("uploads").await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    fs::write(&path, &file).await.map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let link = format!("https://0.id.vn/files/{}", filename_saved);
    let code = QrCode::new(link.as_bytes()).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let svg = code.render::<qrcode::render::svg::Color>().min_dimensions(200,200).build();
    Ok(UploadResponse {
        id: id.to_string(),
        link,
        qr_svg: svg,
    })
}

#[cfg(feature = "ssr")]
#[server(GenerateQr, "/api")]
pub async fn generate_qr(link: String) -> Result<LinkResponse, ServerFnError> {
    use qrcode::QrCode;

    let code = QrCode::new(link.as_bytes()).map_err(|e| ServerFnError::ServerError(e.to_string()))?;
    let svg = code.render::<qrcode::render::svg::Color>().min_dimensions(200,200).build();
    Ok(LinkResponse {
        link,
        qr_svg: svg,
    })
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    #[cfg(feature = "csr")]
    let (_link_input, set_link_input) = create_signal(cx, String::new());
    // CSR-only forms (compile-time gated so SSR builds don't reference ActionForm)
    #[cfg(feature = "csr")]
    let upload_form = {
        view! { cx,
            <form action="/upload" method="post" enctype="multipart/form-data">
                <label>
                    "Upload Image: "
                    <input type="file" name="file"/>
                </label>
                <input type="submit" value="Upload"/>
            </form>
        }.into_view(cx)
    };

    #[cfg(not(feature = "csr"))]
    let upload_form = view! { cx, <div/> }.into_view(cx);

    #[cfg(feature = "csr")]
    let link_form = {
        view! { cx,
            <form action="/link" method="post">
                <label>
                    "Link: "
                    <input type="text" name="link" on:input=move |ev| set_link_input.set(event_target_value(&ev))/>
                </label>
                <input type="submit" value="Generate QR"/>
            </form>
        }.into_view(cx)
    };

    #[cfg(not(feature = "csr"))]
    let link_form = view! { cx, <div/> }.into_view(cx);

    view! { cx,
        <div>
            <h1>"ping0 - Fast Upload & Share"</h1>
            {upload_form}
            {link_form}
        </div>
    }
}
