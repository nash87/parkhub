# Build stage - Web Frontend
FROM node:22-alpine AS web-builder
WORKDIR /app/web
COPY parkhub-web/package*.json ./
RUN npm ci
COPY parkhub-web/ ./
COPY VERSION ../VERSION
RUN npm run build

# Build stage - Rust Server
FROM rust:1.83-alpine AS rust-builder
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig cmake make perl clang
WORKDIR /app
COPY Cargo.toml Cargo.lock VERSION ./
COPY parkhub-common/ ./parkhub-common/
COPY parkhub-server/ ./parkhub-server/
COPY parkhub-client/ ./parkhub-client/
COPY --from=web-builder /app/web/dist ./parkhub-web/dist/
RUN cargo build --release --package parkhub-server --no-default-features --features headless

# Runtime stage
FROM alpine:3.20
RUN apk add --no-cache ca-certificates tzdata
WORKDIR /app
COPY --from=rust-builder /app/target/release/parkhub-server /app/parkhub-server
RUN mkdir -p /data
ENV PARKHUB_DATA_DIR=/data
ENV PARKHUB_HOST=0.0.0.0
ENV PARKHUB_PORT=8080
ENV RUST_LOG=info
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8080/health || exit 1
ENTRYPOINT ["/app/parkhub-server", "--headless"]
