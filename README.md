# ping0 — Full-Stack Rust link & file sharer (no JS)

Minimal, monochrome web app that lets you paste a URL or upload a file, producing a short link `0.id.vn/s/<code>` and optionally a QR code. Implemented entirely in Rust using `axum` + `askama` + `rusqlite`.

### Features
- Short links for URLs and uploaded files
- Optional QR on result page
- For images, `GET /s/<code>` returns an HTML page with Open Graph `og:image` so chat apps show previews (not a redirect)
- Direct file serving under `/files/<filename>`

---

## 1) Setup on a VPS (Debian/Ubuntu)

Run these on a fresh Debian/Ubuntu VPS with sudo access.

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libsqlite3-dev curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source $HOME/.cargo/env
```

Clone and build:

```bash
git clone https://github.com/ShayNeeo/ping0.git
cd ping0
cargo build --release
```

Binary path: `server/target/release/ping0` (workspace member `server` named `ping0`).

Create runtime dirs:

```bash
mkdir -p server/uploads server/data
```

## 2) Configuration

You can configure via environment variables:

- `HOST` (default `0.0.0.0`)
- `PORT` (default `8080`)
- `BASE_URL` (default `http://<HOST>:<PORT>`, set to `https://0.id.vn` in prod)
- `DATABASE_PATH` (default `server/data/ping0.db` if you run from repo root; otherwise set an absolute path)

Example (production):

```bash
export HOST=0.0.0.0
export PORT=8080
export BASE_URL=https://0.id.vn
export DATABASE_PATH=/var/lib/ping0/ping0.db
```

## 3) Run

From repo root:

```bash
./server/target/release/ping0
```

Or from `server/` directory after build:

```bash
cd server
./target/release/ping0
```

Open in browser: `https://0.id.vn` once DNS/HTTPS is set. Locally: `http://127.0.0.1:8080`.

## 4) Usage

- Home `GET /`: form with URL field, file input, and "Generate QR Code" checkbox.
- Submit `POST /submit` (multipart). The app stores either the file or URL, creates a short code, and redirects to `/r/<code>`.
- Result `GET /r/<code>`: shows the short link `0.id.vn/s/<code>`, and if requested, an inline QR SVG.
- Short redirect `GET /s/<code>`:
  - If short maps to a URL: 301 redirect to target URL.
  - If short maps to a file and it is an image: returns HTML with `og:image` pointing to `/files/<file>` and an inline `<img>` for previews.
  - If short maps to a non-image file: serves file bytes directly.

## 5) Notes

- Max file size: 10 MB (configurable at compile time within the code constant).
- Allowed extensions: `jpg,jpeg,png,gif,webp,pdf,txt`.
- No JavaScript used; all HTML rendered server-side via `askama`.

---

## Production hardening (optional)

- Put behind Nginx/Caddy and terminate TLS; set `BASE_URL=https://0.id.vn`.
- Run as a dedicated user; place uploads and DB under `/var/lib/ping0/` with restricted permissions.
- Use a process manager (systemd):

```ini
[Unit]
Description=ping0
After=network.target

[Service]
Environment=HOST=0.0.0.0
Environment=PORT=8080
Environment=BASE_URL=https://0.id.vn
Environment=DATABASE_PATH=/var/lib/ping0/ping0.db
WorkingDirectory=/opt/ping0
ExecStart=/opt/ping0/ping0
User=ping0
Group=ping0
Restart=always

[Install]
WantedBy=multi-user.target
```

---

## Build & Run (recap)

- Build: `cargo build --release`
- Run: `./server/target/release/ping0`

---

License: MIT
