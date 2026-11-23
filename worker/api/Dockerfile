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
    cargo build --release --locked -p api_service --bin api

FROM alpine:3.24 AS runtime-base
RUN apk upgrade --no-cache

FROM runtime-base AS runtime

RUN apk add --no-cache ca-certificates libssl3 libgcc && adduser -D -u 10001 appuser

WORKDIR /app
COPY --from=builder /workspace/worker/worker/target/release/api /usr/local/bin/api

EXPOSE 3202
USER appuser

CMD ["api"]
