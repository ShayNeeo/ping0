# Use the official Rust image for building. Use AS in upper-case to avoid linter warnings.
FROM rustlang/rust:nightly AS builder

# Build-time cache buster: change the value of CACHEBUST when building to force
# the following RUN layer to re-execute (useful to ensure `git clone` runs).
ARG CACHEBUST=1

WORKDIR /app

# Pull the repo from GitHub so online builds use the repository contents. The
# ARG above lets you invalidate this layer without disabling cache for the
# entire build.
RUN echo "CACHEBUST=$CACHEBUST" && git clone https://github.com/ShayNeeo/ping0 .

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