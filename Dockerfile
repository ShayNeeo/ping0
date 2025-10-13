FROM rust:1.75 as builder

WORKDIR /app

# Pull the repo
RUN git clone https://github.com/ShayNeeo/ping0 .

# Install wasm32 target
RUN rustup target add wasm32-unknown-unknown

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