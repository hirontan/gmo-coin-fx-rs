#!/usr/bin/env bash
# バージョン番号を Cargo.toml に書き込むだけのスクリプト（コミット・タグは行わない）
#
# PR ブランチでバージョンを上げてからコミットし、main にマージすると
# GitHub Actions (auto-tag.yml) が自動でタグを打ちリリースを作成します。
#
# Usage: ./scripts/bump-version.sh <version>
# 例:    ./scripts/bump-version.sh 0.4.0

set -euo pipefail

VERSION="${1:?Usage: $0 <version>  (例: 0.4.0)}"
TAG="v${VERSION}"

# ────────────────────────────────────────────
# ガード: タグが既に存在しないこと
# ────────────────────────────────────────────
if git tag --list | grep -qx "${TAG}"; then
  echo "❌ タグ ${TAG} は既に存在します。別のバージョンを指定してください。"
  exit 1
fi

# ────────────────────────────────────────────
# 各クレートの version を一括書き換え
# ────────────────────────────────────────────
CARGO_TOMLS=(
  "crates/gmo-coin-fx-core/Cargo.toml"
  "crates/gmo-coin-fx-client/Cargo.toml"
  "crates/gmo-coin-fx-ws/Cargo.toml"
)

echo "✏️  バージョンを ${VERSION} に更新します..."

for toml in "${CARGO_TOMLS[@]}"; do
  sed -i "0,/^version = \"[^\"]*\"/{s/^version = \"[^\"]*\"/version = \"${VERSION}\"/}" "${toml}"
  echo "   ${toml}"
done

# ────────────────────────────────────────────
# Cargo.lock を更新
# ────────────────────────────────────────────
cargo generate-lockfile --quiet

echo ""
echo "════════════════════════════════════════"
echo "✅ バージョンを ${VERSION} に更新しました"
echo ""
echo "次のステップ:"
echo "  1. 変更をコミットして PR を作成・マージする"
echo "  2. main へのマージ後、GitHub Actions が自動で ${TAG} タグを作成します"
echo "  3. タグ作成をトリガーに release.yml が CI → GitHub Release を実行します"
echo "════════════════════════════════════════"
