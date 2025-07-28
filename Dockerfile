FROM rust:1.88-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y libssl-dev pkg-config curl
RUN cargo install sqlx-cli --no-default-features --features postgres
COPY . .
ENV SQLX_OFFLINE=true
RUN cargo build --release

FROM debian:stable-slim
WORKDIR /app
RUN apt-get update && apt-get install -y libssl3 ca-certificates curl file && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/axum-restful-api .
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/sqlx
COPY --from=builder /app/migrations ./migrations
COPY --from=builder /app/.env.example .env
ENV RUST_LOG=info
CMD ["/app/axum-restful-api"]
