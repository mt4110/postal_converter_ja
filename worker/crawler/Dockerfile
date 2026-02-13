# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.91.1

FROM rust:${RUST_VERSION}-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    clang \
    libclang-dev \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /workspace/worker
COPY worker ./worker

WORKDIR /workspace/worker/worker
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --locked -p crawler_service --bin crawler

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 10001 --create-home appuser

WORKDIR /app
COPY --from=builder /workspace/worker/worker/target/release/crawler /usr/local/bin/crawler
COPY --from=builder /workspace/worker/worker/common /app/common
RUN mkdir -p /app/temp_assets && chown -R appuser:appuser /app

ENV ZIP_CODE_URL=https://www.post.japanpost.jp/zipcode/dl/kogaki/zip/ken_all.zip
ENV CRAWLER_INTERVAL_SECONDS=86400
ENV CRAWLER_RUN_ONCE=false
ENV DATABASE_TYPE=postgres

USER appuser
CMD ["crawler"]
