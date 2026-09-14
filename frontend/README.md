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

## 入力と通信の扱い

- UIとSDKは同じ `normalizeZip` / `isValidZip` を使います。全角数字を半角へ変換し、空白・ハイフン（`-`、`－`、`−`）を取り除いてから7桁の数字か検証します。
- `１００－０００１` は検索できます。`10000012` や英字を含む入力は切り詰めたり英字を削除したりせず、UIで入力エラーを表示します。SDKの `lookupZip` は不正入力に対してAPIを呼ばず `[]` を返します。
- 検索中に郵便番号・住所キーワード・補完先の住所を編集すると、そのフォームの検索をキャンセルし、古い応答を反映しません。入力変更時と検索開始時に候補とメッセージを消します。既に入力済みの住所は保持します。
- 通信はレスポンス本文の読み取りを含め、既定10秒でタイムアウトします。失敗時はメッセージを表示し、再検索できます。自動リトライはしません。

SDKの待ち時間は `timeoutMs`（1〜2,147,483,647の整数、ミリ秒）で変更できます。各検索・一覧取得メソッドは `signal` を受け付けます。タイムアウトは `TimeoutError`、標準のキャンセルは `AbortError` としてrejectされます。独自の `fetcher` を渡す場合も、標準fetchと同様に `signal` による中断に対応してください。

```ts
const sdk = createPostalSdk({ timeoutMs: 10_000 });
const controller = new AbortController();
const result = sdk.lookupZip("１００－０００１", { signal: controller.signal });
// 入力変更・画面破棄時などに controller.abort() で中断できます。
```

互換性の変更: 以前は8桁以上の入力や英字を含む入力から先頭7桁を検索できましたが、誤った住所への補完を防ぐため、現在は不正入力として扱います。従来上限のなかった通信待ちにも既定10秒の上限があります。

## E2E

Playwright E2E は API レスポンスをモックして、主要3フォーム、全角・桁不足・桁超過、検索中の入力変更、通信失敗・タイムアウト後の再検索、デスクトップ・モバイル幅の横はみ出しを検証します。同じコマンドでSDK単体の入力検証・中断・タイムアウト・後始末も検証します。実API・DBへの接続はこのテストに含みません。

初回または Playwright 更新後は、E2E 実行前に Chromium を取得してください。

```bash
nix develop --command bash -c "cd frontend && yarn playwright install chromium && yarn test:e2e"
```
