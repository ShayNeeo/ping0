FROM rust:1.72-slim
WORKDIR /usr/src/app
COPY . /usr/src/app
RUN apt-get update && apt-get install -y pkg-config libssl-dev libjpeg-dev libpng-dev
RUN cargo build --release
CMD ["/usr/src/app/target/release/ping0-backend"]
