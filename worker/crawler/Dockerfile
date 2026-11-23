# syntax=docker/dockerfile:1.7

ARG RUST_VERSION=1.100.0

FROM rust:${RUST_VERSION}-alpine AS rust-base
RUN apk upgrade --no-cache

FROM rust-base AS builder

RUN apk add --no-cache build-base pkgconf clang-dev openssl-dev ca-certificates

WORKDIR /workspace/worker
COPY worker ./worker

WORKDIR /workspace/worker/worker
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    cargo build --release --locked -p crawler_service --bin crawler

FROM alpine:3.24 AS runtime-base
RUN apk upgrade --no-cache

FROM runtime-base AS runtime

RUN apk add --no-cache ca-certificates libssl3 libgcc tzdata && adduser -D -u 10001 appuser

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
