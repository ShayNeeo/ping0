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
    <title>{{ title }}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="canonical" href="{{ page_url }}">
    <meta property="og:type" content="website">
    <meta property="og:site_name" content="0.id.vn">
    <meta property="og:title" content="{{ title }}">
    <meta property="og:description" content="{{ description }}">
    <meta property="og:url" content="{{ page_url }}">
    <meta property="og:image" content="{{ og_image_url }}">
    <meta name="twitter:card" content="summary_large_image">
    <meta name="twitter:title" content="{{ title }}">
    <meta name="twitter:description" content="{{ description }}">
    <meta name="twitter:image" content="{{ og_image_url }}">
  </head>
  <body style="font-family:Courier New,monospace;background:#fff;color:#000;text-align:center">
    <img src="{{ full_image_url }}" alt="{{ title }}" style="max-width:95vw;max-height:90vh">
  </body>
 </html>"#, ext = "html")]
pub struct ImageOgTemplate { pub og_image_url: String, pub full_image_url: String, pub page_url: String, pub title: String, pub description: String }

#[derive(Template)]
#[template(source = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>{{ filename }}</title>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link rel="canonical" href="{{ page_url }}">
    <meta property="og:type" content="website">
    <meta property="og:site_name" content="0.id.vn">
    <meta property="og:title" content="{{ filename }} ({{ mime }})">
    <meta property="og:description" content="Download or preview file">
    <meta property="og:url" content="{{ page_url }}">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{{ filename }} ({{ mime }})">
    <meta name="twitter:description" content="Download or preview file">
    <style>
      body{font-family:Courier New,monospace;background:#fff;color:#000}
      main{max-width:560px;margin:4rem auto;text-align:center}
      a{color:#000}
    </style>
  </head>
  <body>
    <main>
      <h1>{{ filename }}</h1>
      <p>MIME: {{ mime }}</p>
      <p><a href="{{ file_url }}">Download</a></p>
    </main>
  </body>
</html>"#, ext = "html")]
pub struct FileInfoTemplate { pub filename: String, pub file_url: String, pub mime: String, pub page_url: String }

// ---------- Admin Templates ----------

#[derive(Template)]
#[template(source = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Admin Login</title>
    <style>
      body{font-family:Courier New,monospace;background:#fff;color:#000}
      main{max-width:480px;margin:4rem auto;text-align:center}
      label,input,button{display:block;margin:0.6rem auto}
    </style>
  </head>
  <body>
    <main>
      <h1>Admin Login</h1>
      <form action="/admin/login" method="post">
        <label>Username:
          <input type="text" name="username" required>
        </label>
        <label>Password:
          <input type="password" name="password" required>
        </label>
        <button type="submit">Login</button>
      </form>
      <p style="font-size:0.9rem">First login sets the admin username and password.</p>
    </main>
  </body>
</html>"#, ext = "html")]
pub struct AdminLoginTemplate;

#[derive(Template)]
#[template(source = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Admin</title>
    <style>
      body{font-family:Courier New,monospace;background:#fff;color:#000}
      main{max-width:640px;margin:4rem auto;text-align:center}
      a,button{color:#000}
      form{display:inline}
    </style>
  </head>
  <body>
    <main>
      <h1>Admin</h1>
      <p>
        <a href="/admin/items">Items</a>
      </p>
      <form action="/admin/logout" method="post"><button type="submit">Logout</button></form>
    </main>
  </body>
</html>"#, ext = "html")]
pub struct AdminHomeTemplate;

#[derive(Template)]
#[template(source = r#"<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8">
    <title>Items</title>
    <style>
      body{font-family:Courier New,monospace;background:#fff;color:#000}
      main{max-width:900px;margin:2rem auto}
      table{width:100%;border-collapse:collapse}
      th,td{border:1px solid #000;padding:6px;text-align:left}
      a,button{color:#000}
      form{display:inline}
    </style>
  </head>
  <body>
    <main>
      <h1>Items</h1>
      <p>
        <a href="/admin">Home</a>
        <span> · </span>
        <form action="/admin/logout" method="post"><button type="submit">Logout</button></form>
      </p>
      <table>
        <thead>
          <tr><th>Code</th><th>Kind</th><th>Type</th><th>Value</th><th>When</th><th>Actions</th></tr>
        </thead>
        <tbody>
        {% for item in items %}
          <tr>
            <td><a href="/s/{{ item.code }}" target="_blank">{{ item.code }}</a></td>
            <td>{{ item.kind }}</td>
            <td>{{ item.mime.as_deref().unwrap_or("-") }}</td>
            <td style="max-width:420px;word-break:break-all">{{ item.value }}</td>
            <td>{{ item.created_at }}</td>
            <td>
              <form action="/admin/items/{{ item.code }}/delete" method="post" onsubmit="return confirm('Delete {{ item.code }}?')"><button type="submit">Delete</button></form>
            </td>
          </tr>
        {% endfor %}
        </tbody>
      </table>
    </main>
  </body>
</html>"#, ext = "html")]
pub struct AdminItemsTemplate { pub items: Vec<AdminItem> }

pub struct AdminItem { pub code: String, pub kind: String, pub value: String, pub created_at: i64, pub mime: Option<String> }
