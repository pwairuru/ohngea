# Minimal multi-stage build for pbx-api.
# Build context is ./rust (see rust/docker-compose.rust.yml).
# Builder must stay >= the Cargo.lock's minimum (edition2024 deps need
# cargo 1.85+; rust:1.80 predates that and fails parsing the lockfile).
# 16k header tolerance is enforced at the edge proxy in production:
# nginx `large_client_header_buffers 4 16k;` (axum 0.8 serve exposes no
# max_buf_size hook; see pbx-api src/main.rs + MAX_HEADER_BYTES).
FROM rust:1.99-bookworm AS builder
# NOTE: build context must be the repo ROOT (not ./rust): provision modules
# `include_str!("../../../../../templates/...")` resolve to /templates, so
# the root templates/ dir has to be copied into the builder image.
COPY rust/Cargo.toml rust/Cargo.lock /app/
COPY rust/crates /app/crates
COPY templates /templates
COPY plugins /plugins
WORKDIR /app
# BuildKit cache mounts: without them every `up --build` re-downloads the
# registry and recompiles all ~300 deps from scratch in release mode (max
# LLVM opts). With them only changed crates rebuild — same incrementality
# as local `cargo build`. The binary must be copied out of the cached
# target dir within the same RUN (cache mounts are not image layers).
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release -p pbx-api && cp /app/target/release/pbx-api /app/pbx-api

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/pbx-api /usr/local/bin/pbx-api
EXPOSE 8000
# Migrate on boot: pbx-api runs `pbx_db::migration::Migrator::up` at startup
# (see pbx-api src/main.rs + routing_db::init_db) before serving, so Postgres
# schema/RLS/views are current even on fresh `db` volumes.
CMD ["pbx-api"]
