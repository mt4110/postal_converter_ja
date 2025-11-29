# Dockerfile for minimal Rust setup
FROM rust:1.83-alpine

RUN apk add --no-cache musl-dev libressl-dev gcc clang g++ make curl \
    && curl -sfS https://dotenvx.sh/install.sh | sh
ENV LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib
ENV LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib
ENV PATH="/root/.dotenvx/bin:$PATH"

WORKDIR /app

COPY ./worker/api/Cargo.toml /app/
COPY worker/api/src ./src
COPY worker/api/.env .env
COPY ./worker/common /app/common

# common のパスを "./common" に書き換える
RUN sed -i 's|\.\./common|./common|' /app/Cargo.toml

# 依存関係をフェッチ
RUN  cargo fetch

# 依存関係をビルド（キャッシュ有効化のため）
RUN cargo build --release --bin api

CMD ["cargo", "run", "--release", "--bin", "api"]