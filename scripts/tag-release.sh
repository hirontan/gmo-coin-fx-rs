#!/usr/bin/env bash
# 新バージョンのリリース手順を一括で行うスクリプト
#
# 実行前の前提:
#   - main ブランチ上にいること
#   - 変更がすべてコミット済みで、working tree がクリーンなこと
#
# Usage: ./scripts/tag-release.sh <version>
# 例:    ./scripts/tag-release.sh 0.2.0

set -euo pipefail

# ────────────────────────────────────────────
# 引数チェック
# ────────────────────────────────────────────
VERSION="${1:?Usage: $0 <version>  (例: 0.2.0)}"
TAG="v${VERSION}"

# ────────────────────────────────────────────
# ガード: main ブランチ以外では実行しない
# ────────────────────────────────────────────
CURRENT_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "${CURRENT_BRANCH}" != "main" ]]; then
  echo "❌ main ブランチで実行してください（現在: ${CURRENT_BRANCH}）"
  exit 1
fi

# ────────────────────────────────────────────
# ガード: working tree がクリーンであること
# ────────────────────────────────────────────
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "❌ コミットされていない変更があります。先にコミットしてください。"
  git status --short
  exit 1
fi

# ────────────────────────────────────────────
# ガード: タグが既に存在しないこと
# ────────────────────────────────────────────
if git tag --list | grep -qx "${TAG}"; then
  echo "❌ タグ ${TAG} は既に存在します。"
  exit 1
fi

echo "🚀 Release ${TAG} を開始します..."
echo ""

# ────────────────────────────────────────────
# 各クレートの version を一括書き換え
# ────────────────────────────────────────────
CARGO_TOMLS=(
  "crates/gmo-coin-fx-core/Cargo.toml"
  "crates/gmo-coin-fx-client/Cargo.toml"
  "crates/gmo-coin-fx-ws/Cargo.toml"
)

for toml in "${CARGO_TOMLS[@]}"; do
  # [package] セクションの version 行だけを置換（dependencies の version は変えない）
  sed -i "0,/^version = \"[^\"]*\"/{s/^version = \"[^\"]*\"/version = \"${VERSION}\"/}" "${toml}"
  echo "  ✏️  ${toml} → version = \"${VERSION}\""
done

# ────────────────────────────────────────────
# Cargo.lock を更新
# ────────────────────────────────────────────
echo ""
echo "📦 Cargo.lock を更新中..."
cargo generate-lockfile --quiet

# ────────────────────────────────────────────
# バージョンバンプをコミット
# ────────────────────────────────────────────
git add "${CARGO_TOMLS[@]}" Cargo.lock
git commit -m "chore: bump version to ${TAG}"
echo "  ✅ バージョンバンプコミット完了"

# ────────────────────────────────────────────
# アノテーション付きタグを作成 & プッシュ
# ────────────────────────────────────────────
git tag -a "${TAG}" -m "Release ${TAG}"
git push origin main "${TAG}"

echo ""
echo "════════════════════════════════════════"
echo "✅ ${TAG} のリリースが完了しました！"
echo ""
echo "GitHub Actions が自動で CI → Release を実行します。"
echo "https://github.com/hirontan/gmo-coin-fx-rs/releases"
echo ""
echo "利用者の Cargo.toml に追加する記述："
echo ""
echo "  # REST API クライアント"
echo "  gmo-coin-fx-client = { git = \"https://github.com/hirontan/gmo-coin-fx-rs.git\", tag = \"${TAG}\" }"
echo ""
echo "  # WebSocket クライアント"
echo "  gmo-coin-fx-ws = { git = \"https://github.com/hirontan/gmo-coin-fx-rs.git\", tag = \"${TAG}\" }"
echo "════════════════════════════════════════"
