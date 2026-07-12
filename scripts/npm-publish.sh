#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
#
# Implements #29. Publish the npm distribution: the five per-platform binary
# packages first, then the `talicode` wrapper last (so its optionalDependencies
# already resolve on the registry).
#
# Each platform package must have its native binary staged in before this runs
# (the CI release matrix does this from cross-compiled artifacts; locally you
# can only stage the host target). Platform packages without a staged binary are
# skipped with a warning — never published empty.
#
# Usage:
#   scripts/npm-publish.sh              # publish for real
#   DRY_RUN=1 scripts/npm-publish.sh    # show what would publish, touch nothing
#
# Requires: logged in to npm (`npm whoami`) with publish rights.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

DRY_RUN="${DRY_RUN:-}"
NPM_ARGS=()
[ -n "$DRY_RUN" ] && NPM_ARGS+=(--dry-run)

# platform package -> the binary file name it ships
declare -a PLATFORM_PKGS=(
  "talicode-darwin-arm64:tali"
  "talicode-darwin-x64:tali"
  "talicode-linux-x64:tali"
  "talicode-linux-arm64:tali"
  "talicode-win32-x64:tali.exe"
)

echo "npm-publish: whoami -> $(npm whoami 2>/dev/null || echo '(not logged in)')"
[ -n "$DRY_RUN" ] && echo "npm-publish: DRY RUN — nothing will be published"

published_any=0
for entry in "${PLATFORM_PKGS[@]}"; do
  pkg="${entry%%:*}"
  bin="${entry##*:}"
  dir="$REPO_ROOT/npm/platform/$pkg"
  if [ ! -f "$dir/$bin" ]; then
    echo "npm-publish: SKIP $pkg — binary '$bin' not staged in $dir" >&2
    continue
  fi
  echo "npm-publish: publishing $pkg ..."
  ( cd "$dir" && npm publish "${NPM_ARGS[@]}" )
  published_any=1
done

if [ "$published_any" -eq 0 ]; then
  echo "npm-publish: no platform packages had a staged binary — refusing to" >&2
  echo "            publish the wrapper alone (installs would find no binary)." >&2
  exit 1
fi

echo "npm-publish: publishing the talicode wrapper ..."
npm publish "${NPM_ARGS[@]}"

echo "npm-publish: done ✅"
