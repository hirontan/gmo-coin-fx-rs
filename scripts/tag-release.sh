#!/usr/bin/env bash
# このスクリプトは PR マージ後に main から実行してください
# Usage: ./scripts/tag-release.sh 0.1.0

set -euo pipefail

VERSION="${1:?Usage: $0 <version> (e.g. 0.1.0)}"
TAG="v${VERSION}"

echo "Tagging release ${TAG}..."

git tag -a "${TAG}" -m "Release ${TAG}"
git push origin "${TAG}"

echo "✅ Tag ${TAG} pushed. GitHub Actions が自動で Release を作成します。"
echo ""
echo "利用者は以下を Cargo.toml に追加するだけで使えます："
echo ""
echo "  gmo-coin-fx-client = { git = \"https://github.com/hirontan/gmo-coin-fx-rs.git\", tag = \"${TAG}\" }"
