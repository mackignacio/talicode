#!/usr/bin/env node
// SPDX-License-Identifier: MIT
//
// Implements #28. Launcher for the `tali` command shipped by @talicode.ai/core.
//
// TaliCode's core is a compiled Rust binary (the esbuild / Rolldown model): the
// native `tali` executable is delivered inside one of several per-platform npm
// packages, listed as optionalDependencies so npm installs only the one matching
// the host's os + cpu. This thin launcher resolves that package's binary and
// execs it, forwarding argv and the exit code. It owns no logic of its own.

"use strict";

const { spawnSync } = require("node:child_process");

// Map `${process.platform} ${process.arch}` to the platform package that ships
// the matching native binary. Keep in lockstep with the wrapper's
// optionalDependencies and the npm/platform/* packages.
const PLATFORM_PACKAGES = {
  "darwin arm64": "talicode-darwin-arm64",
  "darwin x64": "talicode-darwin-x64",
  "linux x64": "talicode-linux-x64",
  "linux arm64": "talicode-linux-arm64",
  "win32 x64": "talicode-win32-x64",
};

/** The platform package name for a given platform/arch, or undefined. */
function resolvePackage(platform, arch) {
  return PLATFORM_PACKAGES[`${platform} ${arch}`];
}

/** The native binary's file name for a platform (`.exe` on Windows). */
function binaryName(platform) {
  return platform === "win32" ? "tali.exe" : "tali";
}

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}

/**
 * Resolve the absolute path to the native `tali` binary for the current host,
 * or exit with a clear message when the platform is unsupported or the
 * matching platform package was not installed.
 */
function binaryPath() {
  const pkg = resolvePackage(process.platform, process.arch);
  if (!pkg) {
    const supported = Object.keys(PLATFORM_PACKAGES).join(", ");
    fail(
      `TaliCode: unsupported platform "${process.platform} ${process.arch}". ` +
        `Supported: ${supported}.`
    );
  }
  try {
    return require.resolve(`${pkg}/${binaryName(process.platform)}`);
  } catch {
    fail(
      `TaliCode: the platform package "${pkg}" is not installed.\n` +
        `It should install automatically as an optional dependency of @talicode.ai/core.\n` +
        `Try reinstalling: npm install -g @talicode.ai/core`
    );
  }
}

function main() {
  const result = spawnSync(binaryPath(), process.argv.slice(2), {
    stdio: "inherit",
  });
  if (result.error) {
    fail(`TaliCode: failed to launch the native binary: ${result.error.message}`);
  }
  // Forward the child's exit code; a null status means it was signalled.
  process.exit(result.status === null ? 1 : result.status);
}

// Only run when invoked as the CLI; exports let the resolution logic be tested.
if (require.main === module) {
  main();
}

module.exports = { PLATFORM_PACKAGES, resolvePackage, binaryName };
