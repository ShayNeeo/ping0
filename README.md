# ping0 (full Rust)

Ping0 — instant file and link sharing with QR generation. This repository uses a full-Rust stack with Leptos (CSR) for the frontend and Axum for the backend.

## Features

- Upload images and get shareable links with QR codes
- Generate QR codes for any link
- Fast sharing between chats

## Getting started (local)

1. Install Rust and wasm32 target

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown
```

2. Build workspace

```bash
cargo build --workspace
```

3. Build frontend

```bash
wasm-pack build app --target web --out-dir ../server/pkg
```

4. Run server

```bash
cd server
cargo run
```

The server listens on http://127.0.0.1:8080 and serves the app.

## Deployment

### Backend

Build and run the Docker container:

```bash
docker build -t ping0 .
docker run -p 8080:8080 ping0
```

Host on deploy.cx or your VPS.

### Frontend

The frontend is built as WASM and can be hosted on Cloudflare Pages.

Build the frontend:

```bash
wasm-pack build app --target web --out-dir dist
```

Then deploy the `dist` folder to Cloudflare Pages.

## Domain

Configured for 0.id.vn

License: MIT
