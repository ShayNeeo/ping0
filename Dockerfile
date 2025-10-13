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

# Switch to non-root user
USER ping0user

# Set environment variables with defaults
ENV PORT=8080
ENV HOST=0.0.0.0
ENV BASE_URL=https://0.id.vn
ENV RUST_LOG=info

# Expose the port
EXPOSE 8080

# Add health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Run the server
CMD ["./ping0-server"]