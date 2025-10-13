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
    fs::create_dir_all("uploads").await?;
    fs::write(&path, &file).await?;
    let link = format!("https://0.id.vn/files/{}", filename_saved);
    let code = QrCode::new(link.as_bytes())?;
    let svg = code.render().min_dimensions(200,200).build();
    Ok(UploadResponse {
        id: id.to_string(),
        link,
        qr_svg: svg,
    })
}

#[server(GenerateQr, "/api")]
pub async fn generate_qr(link: String) -> Result<LinkResponse, ServerFnError> {
    use qrcode::QrCode;

    let code = QrCode::new(link.as_bytes())?;
    let svg = code.render().min_dimensions(200,200).build();
    Ok(LinkResponse {
        link,
        qr_svg: svg,
    })
}

#[component]
pub fn App(cx: Scope) -> impl IntoView {
    let upload_action = create_server_action::<UploadFile>(cx);
    let link_action = create_server_action::<GenerateQr>(cx);
    let (link_input, set_link_input) = create_signal(cx, String::new());

    view! { cx,
        <div>
            <h1>ping0 - Fast Upload & Share</h1>
            <ActionForm action=upload_action>
                <label>
                    "Upload Image: "
                    <input type="file" name="file"/>
                </label>
                <input type="submit" value="Upload"/>
            </ActionForm>
            {move || upload_action.value().get().map(|res| match res {
                Ok(res) => view! { cx,
                    <div>
                        <p>Link: <a href={&res.link}>{&res.link}</a></p>
                        <div inner_html={&res.qr_svg}></div>
                    </div>
                }.into_view(cx),
                Err(e) => view! { cx, <p>"Error: " {e.to_string()}</p> }.into_view(cx),
            })}
            <ActionForm action=link_action>
                <label>
                    "Link: "
                    <input type="text" name="link" on:input=move |ev| set_link_input.set(event_target_value(&ev))/>
                </label>
                <input type="submit" value="Generate QR"/>
            </ActionForm>
            {move || link_action.value().get().map(|res| match res {
                Ok(res) => view! { cx,
                    <div>
                        <p>Link: " { &res.link }</p>
                        <div inner_html={&res.qr_svg}></div>
                    </div>
                }.into_view(cx),
                Err(e) => view! { cx, <p>"Error: " {e.to_string()}</p> }.into_view(cx),
            })}
        </div>
    }
}
