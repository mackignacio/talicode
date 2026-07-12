#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
#
# Implements #29. Single-platform smoke test for the npm distribution layer.
#
# Proves the launcher path end-to-end on the HOST platform, OFFLINE (no registry
# and no GitHub download):
#   1. build the release `tali` binary,
#   2. `npm pack` the `talicode` wrapper into a tarball,
#   3. install it into a scratch project with the postinstall download SKIPPED
#      (TALICODE_SKIP_DOWNLOAD=1),
#   4. drop the freshly-built binary in where the downloader would have put it
#      (bin/tali-native), then
#   5. confirm `tali --version` execs the native binary and that a bad
#      subcommand forwards a non-zero exit code.
#
# The real postinstall download from the GitHub Release is exercised by an
# actual `npm install -g talicode` after a release; this offline test covers the
# launcher/resolution logic without network.
#
# Usage: scripts/npm-smoke.sh   (run from the repo root)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

PLATFORM="$(node -p process.platform)"
BIN_NAME="tali"; [ "$PLATFORM" = "win32" ] && BIN_NAME="tali.exe"
NATIVE_NAME="tali-native"; [ "$PLATFORM" = "win32" ] && NATIVE_NAME="tali-native.exe"

# --- 1. build the release binary ---------------------------------------------
echo "npm-smoke: building release binary..."
cargo build --release -p talicode-cli
BUILT="target/release/$BIN_NAME"
[ -f "$BUILT" ] || { echo "npm-smoke: build did not produce $BUILT" >&2; exit 1; }

# --- scratch workspace (cleaned up on exit) ----------------------------------
SCRATCH="$(mktemp -d)"
cleanup() { rm -rf "$SCRATCH"; }
trap cleanup EXIT

# --- 2. pack the wrapper -----------------------------------------------------
echo "npm-smoke: packing tarball..."
WRAPPER_TGZ="$(cd "$SCRATCH" && npm pack "$REPO_ROOT" --silent)"

# --- 3. install offline, skipping the postinstall download -------------------
echo "npm-smoke: installing into scratch project (download skipped)..."
cd "$SCRATCH"
npm init -y >/dev/null
TALICODE_SKIP_DOWNLOAD=1 npm install --no-audit --no-fund --loglevel=error "$SCRATCH/$WRAPPER_TGZ"

# --- 4. drop the built binary where the downloader would have put it ---------
NATIVE_DEST="$SCRATCH/node_modules/talicode/bin/$NATIVE_NAME"
cp "$REPO_ROOT/$BUILT" "$NATIVE_DEST"
chmod +x "$NATIVE_DEST"

# --- 5. assertions -----------------------------------------------------------
echo "npm-smoke: checking 'tali --version'..."
EXPECTED_VERSION="$(node -p "require('$REPO_ROOT/package.json').version")"
VERSION_OUT="$(npx --no-install tali --version)"
echo "  -> $VERSION_OUT"
case "$VERSION_OUT" in
  *"$EXPECTED_VERSION"*) : ;;
  *) echo "npm-smoke: FAIL — version output did not contain $EXPECTED_VERSION" >&2; exit 1 ;;
esac

echo "npm-smoke: checking exit-code forwarding on a bad subcommand..."
set +e
npx --no-install tali definitely-not-a-command >/dev/null 2>&1
CODE=$?
set -e
if [ "$CODE" -eq 0 ]; then
  echo "npm-smoke: FAIL — a bad subcommand should exit non-zero, got 0" >&2
  exit 1
fi
echo "  -> non-zero exit ($CODE) forwarded correctly"

echo "npm-smoke: PASS ✅  ($PLATFORM launcher path verified)"
