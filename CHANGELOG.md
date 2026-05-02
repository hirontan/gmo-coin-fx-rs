# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
