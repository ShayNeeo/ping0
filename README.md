# ping0 (full Rust)

Ping0 — instant file and link sharing with QR generation. This repository uses a full-Rust stack with Leptos (SSR + hydration) for the frontend and Axum for the backend.

Getting started (local)

1. Build workspace

```bash
cd /home/shayneeo/Documents/Coding/ping0
cargo build --workspace
```

2. Run server

```bash
cd server
cargo run
```

The server listens on http://127.0.0.1:8080 and serves the Leptos app and the `/upload` endpoint.

License: MIT
