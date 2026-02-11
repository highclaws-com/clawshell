#!/usr/bin/env bash
#
# Publish all ClawShell npm packages.
#
# Prerequisites:
#   1. Build the release binaries for all platforms and place them at:
#        npm/clawshell-<platform>/bin/clawshell
#   2. Run: npm login (or set NPM_TOKEN)
#
# Usage:
#   ./npm/publish.sh            # publish all packages
#   ./npm/publish.sh --dry-run  # preview without publishing
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DRY_RUN=""

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN="--dry-run"
    echo "==> Dry run mode — nothing will be published"
fi

PLATFORMS=(
    "clawshell-darwin-arm64"
    "clawshell-linux-arm64"
    "clawshell-linux-x64"
)

# Publish platform packages first
for pkg in "${PLATFORMS[@]}"; do
    pkg_dir="${SCRIPT_DIR}/${pkg}"
    if [[ ! -f "${pkg_dir}/bin/clawshell" ]] && [[ -z "$DRY_RUN" ]]; then
        echo "ERROR: Missing binary at ${pkg_dir}/bin/clawshell"
        echo "       Build the release binary for this platform first."
        exit 1
    fi
    echo "==> Publishing ${pkg}..."
    (cd "$pkg_dir" && npm publish --access public $DRY_RUN)
done

# Publish the main wrapper package last
echo "==> Publishing @runta-dev/clawshell..."
(cd "${SCRIPT_DIR}/clawshell" && npm publish --access public $DRY_RUN)

echo "==> Done!"
