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

### Quick Deploy to Production

The project is production-ready and configured for deploy.cx:

```bash
# 1. Push to GitHub (already done)
git push origin main

# 2. Connect repository to deploy.cx
# 3. Set environment variables:
#    - PORT=8080
#    - HOST=0.0.0.0
#    - BASE_URL=https://0.id.vn
#    - RUST_LOG=info

# 4. Configure persistent storage for /app/uploads
# 5. Deploy!
```

**📖 See [DEPLOYMENT.md](DEPLOYMENT.md) for complete deployment guide**

### Local Docker Testing

Build and run the Docker container:

```bash
docker build -t ping0 .
docker run -p 8080:8080 \
  -e BASE_URL=http://localhost:8080 \
  -v $(pwd)/uploads:/app/uploads \
  ping0
```

Or use docker-compose:

```bash
docker-compose up
```

### Health Check

Verify the service is running:

```bash
curl http://localhost:8080/health
# Response: {"status":"healthy","service":"ping0"}
```

## Production Features

✅ **Security**
- Non-root user in Docker
- File size limits (10MB max)
- File type validation
- Proper error handling
- Environment-based configuration

✅ **Reliability**
- Health check endpoint
- Graceful error handling
- Persistent volume support
- Modern Axum/Tokio API

✅ **Performance**
- Release mode builds
- Minimal Docker image
- Optimized WASM frontend

## Domain

Configured for **0.id.vn**

## License

MIT
