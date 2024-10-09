# Dockerfile for minimal Rust setup
FROM rust:1.81.0-alpine

RUN apk add --no-cache musl-dev libressl-dev gcc clang g++ make
ENV LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib
ENV LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib

WORKDIR /app

COPY ./worker/api/Cargo.toml /app/
COPY ./worker/api/src /app/src
COPY ./worker/common /app/common

# common のパスを "./common" に書き換える
RUN sed -i 's|\.\./common|./common|' /app/Cargo.toml

# 依存関係をフェッチ
RUN  cargo fetch

# 依存関係をビルド（キャッシュ有効化のため）
RUN cargo build --release --bin api

CMD ["cargo", "run", "--release", "--bin", "api"]