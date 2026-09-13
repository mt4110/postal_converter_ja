# Postal Converter JA フロントエンド

EC配送・会員登録・コールセンターの3種類の住所入力サンプルです。
Next.js + Reactから、`src/lib/postal-sdk.ts` を通じて郵便番号APIを呼び出します。

## 起動

NixとDockerを用意し、[ルートREADME](../README.md)の手順で住所データを格納したDBとAPIを起動してください。
フロントエンドだけでも画面は開きますが、住所検索にはAPIが必要です。
Dockerに接続できない場合は、利用しているDocker環境を先に起動してください（Colimaの場合は `colima start`）。

リポジトリのルートから、別のターミナルで実行します。

```bash
nix develop --command bash -c "cd frontend && yarn install --frozen-lockfile && yarn dev"
```

[http://localhost:3203](http://localhost:3203) を開きます。
APIの既定URLは `http://localhost:3202` です。
`http://localhost:3202/ready` で接続状態を確認できます。
APIの接続先を変更する場合は、`frontend/.env.local` に以下を設定し、フロントエンドを再起動してください。

```dotenv
NEXT_PUBLIC_API_URL=http://localhost:3202
```

## 動作確認

- EC配送: `100-0001` を入力して「住所を補完」→ 東京都・千代田区・千代田。
- 会員登録: 住所キーワードに `西新宿` を入力して「キーワード検索」→ 候補を選択して住所と郵便番号を反映。
- コールセンター: `5300001` を入力して「郵便番号検索」→ 大阪府・大阪市北区・梅田。

検索結果は接続先DBの収録データに依存します。住所キーワードには、まず町域名などの連続した文字列を使ってください。
ページの実装は `src/app/page.tsx` と `src/components/postalShowcase.tsx` にあります。

## E2E

Playwright E2E は API レスポンスをモックして、主要3フォーム、桁不足エラー、デスクトップ・モバイル幅の横はみ出しを検証します。

```bash
nix develop --command bash -c "cd frontend && yarn test:e2e"
```
