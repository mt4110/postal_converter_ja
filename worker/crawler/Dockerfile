# Dockerfile for minimal Rust setup
FROM rust:1.81.0-alpine

RUN apk add --no-cache curl musl-dev libressl-dev gcc clang g++ make pkgconfig openssl-dev \
    && curl -L https://github.com/dotenv-linter/dotenv-linter/releases/latest/download/dotenvx-alpine -o /usr/local/bin/dotenvx \
    && chmod +x /usr/local/bin/dotenvx

ENV LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib
ENV LD_LIBRARY_PATH=/usr/local/lib:/usr/lib:/lib
# 作業ディレクトリを作成
WORKDIR /app

COPY ./worker/crawler/.env /app/
COPY ./worker/crawler/Cargo.toml /app/
COPY ./worker/crawler/src /app/src
COPY ./worker/common /app/common

# common のパスを "./common" に書き換える
RUN sed -i 's|\.\./common|./common|' /app/Cargo.toml

# 依存関係をフェッチ
RUN dotenvx run -- cargo fetch

# 依存関係をビルド（キャッシュ有効化のため）
RUN dotenvx cargo build --release --bin crawler

# 実行運用: コメントを外すと実行されます
CMD ["dotenvx", "cargo", "run", "--release", "--bin", "crawler"]

# 運用方法のコメント:
# ビルド運用（SQLxキャッシュとビルドのみ実行）:
# docker build -t crawler-image .
#
# 実行運用（ビルド済みバイナリを実行）:
# docker run -e MYSQL_DATABASE_URL=<mysql_url> -e POSTGRES_DATABASE_URL=<postgres_url> crawler-image