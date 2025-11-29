### ポート競合エラー（Address already in use）

API を起動時に以下のエラーが表示される場合があります：

```
called `Result::unwrap()` on an `Err` value: Os { code: 48, kind: AddrInUse, message: "Address already in use" }
```

これは既に別のプロセスがポート 3202 を使用している場合に発生します。以下の手順で解決してください：

```bash
# ポート3202を使用しているプロセスを特定
lsof -i:3202

# 出力例:
# COMMAND   PID           USER   FD   TYPE             DEVICE SIZE/OFF NODE NAME
# api     36659 masakitakemura   10u  IPv4 0x1e7f9d9246fd07f9      0t0  TCP *:intraintra (LISTEN)

# プロセスを終了（PIDは実際の値に置き換えてください）
kill -9 36659

# APIを再起動
cargo run --release
```

### データベース接続エラー

Crawler や api が「Failed to connect to database」のようなエラーを出す場合：

1. Docker コンテナが起動しているか確認：

   ```bash
   docker ps
   ```

2. コンテナが起動していない場合は起動：

   ```bash
   docker-compose up -d
   ```

3. `.env` ファイルのデータベース URL が正しいか確認
