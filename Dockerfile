FROM node:22-alpine AS web-builder
WORKDIR /app/web
COPY parkhub-web/package*.json ./
RUN npm ci
COPY parkhub-web/ ./
COPY VERSION ../VERSION
RUN npm run build

FROM rust:latest AS rust-builder
RUN apt-get update && apt-get install -y --no-install-recommends build-essential pkg-config libssl-dev cmake perl clang musl-tools && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl
WORKDIR /app
COPY Cargo.toml Cargo.lock VERSION ./
COPY parkhub-common/ ./parkhub-common/
COPY parkhub-server/ ./parkhub-server/
COPY parkhub-client/ ./parkhub-client/
COPY --from=web-builder /app/web/dist ./parkhub-web/dist/
ENV OPENSSL_STATIC=1 OPENSSL_DIR=/usr
RUN cargo build --release --package parkhub-server --no-default-features --features headless --target x86_64-unknown-linux-musl

FROM alpine:3.20
RUN apk add --no-cache ca-certificates wget
WORKDIR /app
COPY --from=rust-builder /app/target/x86_64-unknown-linux-musl/release/parkhub-server /app/parkhub-server
RUN mkdir -p /data
ENV PARKHUB_DATA_DIR=/data PARKHUB_HOST=0.0.0.0 PARKHUB_PORT=8080 RUST_LOG=info
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1
ENTRYPOINT ["/app/parkhub-server", "--headless"]
