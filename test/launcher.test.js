// SPDX-License-Identifier: MIT
//
// Implements #28. Unit tests for the launcher's platform-resolution logic.
// Run with `node --test` (no external test framework).

"use strict";

const test = require("node:test");
const assert = require("node:assert");

const { PLATFORM_PACKAGES, resolvePackage, binaryName } = require("../bin/tali.js");

test("resolvePackage maps every supported platform/arch to its package", () => {
  assert.strictEqual(resolvePackage("darwin", "arm64"), "talicode-darwin-arm64");
  assert.strictEqual(resolvePackage("darwin", "x64"), "talicode-darwin-x64");
  assert.strictEqual(resolvePackage("linux", "x64"), "talicode-linux-x64");
  assert.strictEqual(resolvePackage("linux", "arm64"), "talicode-linux-arm64");
  assert.strictEqual(resolvePackage("win32", "x64"), "talicode-win32-x64");
});

test("resolvePackage returns undefined for an unsupported platform", () => {
  assert.strictEqual(resolvePackage("sunos", "sparc"), undefined);
  assert.strictEqual(resolvePackage("darwin", "ia32"), undefined);
});

test("binaryName is tali.exe only on Windows", () => {
  assert.strictEqual(binaryName("win32"), "tali.exe");
  assert.strictEqual(binaryName("darwin"), "tali");
  assert.strictEqual(binaryName("linux"), "tali");
});

test("the platform map and the wrapper optionalDependencies agree", () => {
  const pkg = require("../package.json");
  const declared = Object.keys(pkg.optionalDependencies).sort();
  const mapped = Object.values(PLATFORM_PACKAGES).sort();
  assert.deepStrictEqual(mapped, declared);
});
