# Use the official Rust image for building. Use AS in upper-case to avoid linter warnings.
FROM rustlang/rust:nightly AS builder

ARG REPO_URL=https://github.com/ShayNeeo/ping0.git
ARG GIT_REF=main

WORKDIR /app

# Pull lightweight commit metadata so the cache invalidates whenever the main branch moves.
ADD https://api.github.com/repos/ShayNeeo/ping0/git/refs/heads/main /tmp/git-ref.json

# Fetch the repository contents for online builds.
RUN git clone --branch ${GIT_REF} --depth 1 ${REPO_URL} .

# Build the server (release mode with optimizations)
RUN cargo build --release --package ping0-server

# Production stage - use minimal base image
FROM debian:bullseye-slim

# Install required runtime dependencies and curl for health checks
RUN apt-get update && \
    apt-get install -y ca-certificates curl && \
    rm -rf /var/lib/apt/lists/*

# Create a non-root user for security
RUN useradd -m -u 1000 ping0user

WORKDIR /app

# Copy binaries and assets from builder
COPY --from=builder /app/target/release/ping0-server /app/ping0-server
COPY --from=builder /app/static /app/pkg

# Create uploads directory with proper permissions
RUN mkdir -p /app/uploads && \
    chown -R ping0user:ping0user /app

# Create a wrapper script to configure environment dynamically
RUN cat <<'EOF' > /app/start.sh
#!/bin/sh
set -e

# Set defaults if not provided
export PORT="${PORT:-8080}"
export HOST="${HOST:-0.0.0.0}"
export RUST_LOG="${RUST_LOG:-info}"

# Auto-configure BASE_URL if not set
if [ -z "$BASE_URL" ]; then
    export BASE_URL="http://127.0.0.1:${PORT}"
    echo "▶ Auto-configured BASE_URL: $BASE_URL"
else
    echo "▶ Using provided BASE_URL: $BASE_URL"
fi

echo "▶ Server configuration:"
echo "  - PORT: $PORT"
echo "  - HOST: $HOST"
echo "  - BASE_URL: $BASE_URL"
echo "  - RUST_LOG: $RUST_LOG"
echo "▶ Starting ping0-server..."

# Start the server
exec /app/ping0-server
EOF

RUN chmod +x /app/start.sh

# Switch to non-root user
USER ping0user

# Set default environment variables (all can be overridden at runtime)
ENV PORT=8080
ENV HOST=0.0.0.0
ENV RUST_LOG=info
# BASE_URL will be auto-configured by start.sh if not provided

# Expose default (actual runtime port can be overridden by $PORT)
EXPOSE 8080

# Healthcheck: ensure server answers locally on 127.0.0.1
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD curl -f "http://127.0.0.1:${PORT:-8080}/health" || exit 1

# Run the server via wrapper script
CMD ["/app/start.sh"]