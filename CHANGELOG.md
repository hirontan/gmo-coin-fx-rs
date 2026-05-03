# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-05-04

### Added
- `.github/dependabot.yml` — Cargo・GitHub Actions の依存を週次で自動 PR
- `.github/ISSUE_TEMPLATE/bug_report.md` — バグ報告テンプレート
- `.github/ISSUE_TEMPLATE/feature_request.md` — 機能追加リクエストテンプレート
- `.github/pull_request_template.md` — PR テンプレート（チェックリスト付き）
- `SECURITY.md` — 脆弱性の報告ポリシー
- CI に `msrv-check` ジョブ追加（Rust 1.86 での `cargo check --locked` を保証）
- CI に `doc-check` ジョブ追加（`RUSTDOCFLAGS=-D warnings` で doc 品質を担保）
- 全 public API に `///` ドキュメントコメントを追加
- `.github/workflows/auto-tag.yml` — main へのマージで Cargo.toml のバージョンを読み自動タグ付け
- `scripts/bump-version.sh` — PR でバージョンを上げるための専用スクリプト

### Changed
- MSRV を 1.75 → 1.86 に更新（依存クレートの edition2024 対応）
- `msrv-check` CI を `--locked` で実行するよう変更（依存バージョンのピン留め）

## [0.2.0] - 2026-05-03

### Added
- `rust-version = "1.75"` を workspace に追加（MSRV 保証）
- `release.yml` に batonel verify / audit / guard チェックを追加
- `release.yml` に Clippy・rustfmt チェックを追加（CI と統一）
- `tag-release.sh` にリリース前のローカルテスト自動実行を追加
- `CHANGELOG.md` を新規追加

### Fixed
- `repository` URL の誤記（`Arcflect/` → `hirontan/`）を修正

## [0.1.0] - 2026-05-03

### Added
- Public REST API クライアント（ステータス・ティッカー・銘柄・ローソク足）
- Private REST API クライアント（資産・注文・建玉・約定・キャンセル・決済・スピード注文）
- Public WebSocket クライアント（自動再接続・購読復元）
- Private WebSocket クライアント（自動再接続・トークン自動更新・購読復元）
- WebSocket イベントの型安全デシリアライズ（`PublicWsMessage` / `PrivateWsMessage` Enum）
- GitHub Actions CI ワークフロー（cargo check / clippy / fmt / test / batonel）
- GitHub Actions Release ワークフロー（タグ push で自動リリース）
- `scripts/tag-release.sh` によるバージョンバンプ + タグ + プッシュの一括自動化

[Unreleased]: https://github.com/hirontan/gmo-coin-fx-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/hirontan/gmo-coin-fx-rs/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/hirontan/gmo-coin-fx-rs/releases/tag/v0.1.0
