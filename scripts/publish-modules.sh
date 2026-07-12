#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
#
# Implements #29. Publish the @talicode.ai/* reservation-stub packages.
#
# These are minimal placeholder packages under the org-owned @talicode.ai scope
# (the TaliCode CLI itself ships as the unscoped `talicode` package). Publishing
# is a one-time / occasional operation, kept out of the tag-triggered release.
#
# Usage:
#   scripts/publish-modules.sh            # publish for real
#   DRY_RUN=1 scripts/publish-modules.sh  # show what would publish, touch nothing
#
# Requires: logged in to npm with publish rights to the @talicode.ai scope.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

NPM_ARGS=()
[ -n "${DRY_RUN:-}" ] && NPM_ARGS+=(--dry-run)

echo "npm whoami -> $(npm whoami 2>/dev/null || echo '(not logged in)')"

for dir in "$REPO_ROOT"/npm/modules/*/; do
  name="$(node -p "require('${dir}package.json').name")"
  echo "publishing $name ..."
  ( cd "$dir" && npm publish "${NPM_ARGS[@]}" )
done

echo "modules published ✅"
