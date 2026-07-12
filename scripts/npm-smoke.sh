#!/usr/bin/env bash
# SPDX-License-Identifier: MIT
#
# Implements #29. Single-platform smoke test for the npm distribution layer.
#
# Proves the launcher/resolution path end-to-end on the HOST platform only
# (full multi-platform publish is out of scope for the MVP):
#   1. build the release `tali` binary,
#   2. stage it into this host's npm/platform/* package,
#   3. `npm pack` the wrapper + platform package into tarballs,
#   4. install both tarballs into a scratch project (offline; optional deps
#      omitted so the four other-platform packages aren't fetched),
#   5. confirm `tali --version` execs the native binary and that a bad
#      subcommand forwards a non-zero exit code.
#
# Usage: scripts/npm-smoke.sh   (run from the repo root)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

# --- resolve the host platform package (same mapping as bin/tali.js) ----------
PLATFORM="$(node -p process.platform)"
ARCH="$(node -p process.arch)"
case "$PLATFORM $ARCH" in
  "darwin arm64") PKG="talicode-darwin-arm64" ;;
  "darwin x64")   PKG="talicode-darwin-x64" ;;
  "linux x64")    PKG="talicode-linux-x64" ;;
  "linux arm64")  PKG="talicode-linux-arm64" ;;
  "win32 x64")    PKG="talicode-win32-x64" ;;
  *) echo "npm-smoke: unsupported host platform '$PLATFORM $ARCH'" >&2; exit 1 ;;
esac
BIN_NAME="tali"; [ "$PLATFORM" = "win32" ] && BIN_NAME="tali.exe"
echo "npm-smoke: host is $PLATFORM/$ARCH -> $PKG"

# --- 1. build the release binary ---------------------------------------------
echo "npm-smoke: building release binary..."
cargo build --release -p talicode-cli
BUILT="target/release/$BIN_NAME"
[ -f "$BUILT" ] || { echo "npm-smoke: build did not produce $BUILT" >&2; exit 1; }

# --- 2. stage it into the host's platform package ----------------------------
STAGED_BIN="$REPO_ROOT/npm/platform/$PKG/$BIN_NAME"
cp "$BUILT" "$STAGED_BIN"
chmod +x "$STAGED_BIN"

# --- scratch workspace (cleaned up on exit) ----------------------------------
# Absolute paths in cleanup: the script cd's into $SCRATCH below, so a relative
# path would resolve against the wrong directory at trap time.
SCRATCH="$(mktemp -d)"
cleanup() { rm -rf "$SCRATCH"; rm -f "$STAGED_BIN"; }
trap cleanup EXIT

# --- 3. pack the wrapper + platform package ----------------------------------
echo "npm-smoke: packing tarballs..."
WRAPPER_TGZ="$(cd "$SCRATCH" && npm pack "$REPO_ROOT" --silent)"
PLATFORM_TGZ="$(cd "$SCRATCH" && npm pack "$REPO_ROOT/npm/platform/$PKG" --silent)"

# --- 4. install both tarballs into a scratch project (offline) ----------------
echo "npm-smoke: installing into scratch project..."
cd "$SCRATCH"
npm init -y >/dev/null
# --omit=optional skips the wrapper's four other-platform optionalDependencies
# (no registry needed); the host platform package is installed explicitly.
npm install --omit=optional --no-audit --no-fund --loglevel=error \
  "$SCRATCH/$WRAPPER_TGZ" "$SCRATCH/$PLATFORM_TGZ"

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

echo "npm-smoke: PASS ✅  ($PKG launcher path verified)"
