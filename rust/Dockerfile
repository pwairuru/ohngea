# Minimal multi-stage build for pbx-api (MSRV 1.80+).
# Build context is ./rust (see rust/docker-compose.rust.yml).
# 16k header tolerance is enforced at the edge proxy in production:
# nginx `large_client_header_buffers 4 16k;` (axum 0.8 serve exposes no
# max_buf_size hook; see pbx-api src/main.rs + MAX_HEADER_BYTES).
FROM rust:1.80-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p pbx-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/pbx-api /usr/local/bin/pbx-api
EXPOSE 8000
# Migrate on boot: pbx-api runs `pbx_db::migration::Migrator::up` at startup
# (see pbx-api src/main.rs + routing_db::init_db) before serving, so Postgres
# schema/RLS/views are current even on fresh `db` volumes.
CMD ["pbx-api"]
