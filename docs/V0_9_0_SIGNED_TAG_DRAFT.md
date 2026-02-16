# v0.9.0 署名タグドラフト / Signed Tag Draft

## 日本語（先頭）

### タグ名
`v0.9.0`

### 目的
v0.9.0 の運用準備（Runbook、SLO/アラート、監査・導入テンプレート、Human Test）と、
ランチャーの運用UX改善、およびフロントエンド郵便番号検索の不具合修正を反映した署名リリースタグ。

### 署名タグ作成コマンド（ドラフト）
```bash
git checkout feature/v0.9.0
git pull --ff-only
git tag -s v0.9.0 -m "v0.9.0

運用準備とUX強化を中心としたリリース。

- v0.9.0 Runbook/運用ドキュメント群を追加
- 月次監査・NDA・導入見積テンプレートを追加
- QA-01 人間テストシナリオを追加
- launcher の起動信頼性とローディング視認性を改善
- frontend の Postal SDK で fetch Illegal invocation を修正
"
```

### タグpushコマンド（ドラフト）
```bash
git push origin v0.9.0
```

### 参照PR
- [#94](https://github.com/mt4110/postal_converter_ja/pull/94)

## English

### Tag Name
`v0.9.0`

### Purpose
Signed release tag for v0.9.0, including operations-readiness assets (runbooks, SLO/alerts,
audit/onboarding templates, human test scenario), launcher UX/reliability hardening, and
the frontend postal lookup bug fix.

### Signed Tag Command (Draft)
```bash
git checkout feature/v0.9.0
git pull --ff-only
git tag -s v0.9.0 -m "v0.9.0

Release focused on operations readiness and operator UX.

- Added v0.9.0 runbooks and operational documentation
- Added monthly audit / NDA / onboarding-estimation templates
- Added QA-01 human test scenario
- Improved launcher startup reliability and loading visibility
- Fixed frontend Postal SDK fetch Illegal invocation
"
```

### Push Tag Command (Draft)
```bash
git push origin v0.9.0
```

### Related PR
- [#94](https://github.com/mt4110/postal_converter_ja/pull/94)
