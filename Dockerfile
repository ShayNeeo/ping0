# Use the official Rust image for building. Use AS in upper-case to avoid linter warnings.
FROM rustlang/rust:nightly AS builder

ARG REPO_URL=https://github.com/ShayNeeo/ping0.git
ARG GIT_REF=main

WORKDIR /app

# Pull lightweight commit metadata so the cache invalidates whenever the main branch moves.
ADD https://api.github.com/repos/ShayNeeo/ping0/git/refs/heads/main /tmp/git-ref.json

# Fetch the repository contents for online builds.
RUN git clone --branch ${GIT_REF} --depth 1 ${REPO_URL} .

# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown

# Install wasm-bindgen CLI for packaging wasm frontend
RUN cargo install -f wasm-bindgen-cli

# Build the server
RUN cargo build --release --package ping0-server

# Build the frontend
RUN cargo build --release --package ping0-app --target wasm32-unknown-unknown
RUN wasm-bindgen --out-dir ./pkg --target web ./target/wasm32-unknown-unknown/release/ping0_app.wasm

FROM debian:bullseye-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ping0-server /app/ping0-server
COPY --from=builder /app/pkg /app/pkg

EXPOSE 8080

CMD ["./ping0-server"]