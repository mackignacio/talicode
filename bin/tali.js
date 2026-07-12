#!/usr/bin/env node
// SPDX-License-Identifier: MIT
//
// Implements #28. Launcher for the `tali` command shipped by the `talicode` package.
//
// TaliCode's core is a compiled Rust binary. The postinstall step
// (scripts/install.js) downloads the matching native `tali` for this host from
// the versioned GitHub Release and stores it as `bin/tali-native[.exe]`. This
// launcher resolves that binary and execs it, forwarding argv and the exit
// code. It owns no logic of its own.

"use strict";

const path = require("node:path");
const fs = require("node:fs");
const { spawnSync } = require("node:child_process");

/** The native binary's file name for a platform (`.exe` on Windows). */
function binaryName(platform) {
  return platform === "win32" ? "tali-native.exe" : "tali-native";
}

/** Absolute path to the downloaded native binary for this host. */
function binaryPath() {
  return path.join(__dirname, binaryName(process.platform));
}

function fail(message) {
  process.stderr.write(`${message}\n`);
  process.exit(1);
}

function main() {
  const bin = binaryPath();
  if (!fs.existsSync(bin)) {
    fail(
      `TaliCode: the native binary was not found at ${bin}.\n` +
        `The postinstall download may have been skipped or failed.\n` +
        `Try reinstalling: npm install -g talicode`
    );
  }
  const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });
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

module.exports = { binaryName, binaryPath };
