use askama::Template;

// Simple monochrome templates (no external CSS/JS)

#[derive(Template)]
#[template(source = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>ping0</title>
    <style>
      body{font-family:Courier New,monospace;background:#fff;color:#000}
      main{max-width:560px;margin:4rem auto;text-align:center}
      label,input,button{display:block;margin:0.6rem auto}
    </style>
  </head>
  <body>
    <main>
      <h1>ping0</h1>
      <form action="/submit" method="post" enctype="multipart/form-data">
        <label>URL:
          <input type="text" name="link">
        </label>
        <label>File:
          <input type="file" name="file">
        </label>
        <label>
          <input type="checkbox" name="qr"> Generate QR Code
        </label>
        <button type="submit">Create</button>
      </form>
    </main>
  </body>
 </html>"#, ext = "html")]
pub struct IndexTemplate;

#[derive(Template)]
#[template(source = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>ping0 result</title>
    <style>
      body{font-family:Courier New,monospace;background:#fff;color:#000}
      main{max-width:560px;margin:4rem auto;text-align:center}
      a{color:#000}
      .qr{margin-top:1rem}
    </style>
  </head>
  <body>
    <main>
      <h1>Short link created</h1>
      <p><strong>{{ code }}</strong></p>
      <p><a href="{{ short_link }}">{{ short_link }}</a></p>
      {% if qr_svg.is_some() %}
      <div class="qr">{{ qr_svg.as_ref().unwrap()|safe }}</div>
      {% endif %}
    </main>
  </body>
 </html>"#, ext = "html")]
pub struct ResultTemplate { pub code: String, pub short_link: String, pub qr_svg: Option<String> }

#[derive(Template)]
#[template(source = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Shared Image</title>
    <meta property="og:image" content="{{ image_url }}">
  </head>
  <body style="font-family:Courier New,monospace;background:#fff;color:#000;text-align:center">
    <img src="{{ image_url }}" style="max-width:95vw;max-height:90vh">
  </body>
 </html>"#, ext = "html")]
pub struct ImageOgTemplate { pub image_url: String }


