// SPDX-License-Identifier: MIT
//
// Implements #28. Unit tests for the launcher's binary-resolution logic.
// Run with `node --test` (no external test framework).

"use strict";

const test = require("node:test");
const assert = require("node:assert");
const path = require("node:path");

const { binaryName, binaryPath } = require("../bin/tali.js");

test("binaryName is tali-native.exe only on Windows", () => {
  assert.strictEqual(binaryName("win32"), "tali-native.exe");
  assert.strictEqual(binaryName("darwin"), "tali-native");
  assert.strictEqual(binaryName("linux"), "tali-native");
});

test("binaryPath resolves next to the launcher for this host", () => {
  const p = binaryPath();
  assert.strictEqual(path.dirname(p), path.join(__dirname, "..", "bin"));
  assert.strictEqual(path.basename(p), binaryName(process.platform));
});

test("binaryName always yields a tali-native* file", () => {
  for (const platform of ["darwin", "linux", "win32"]) {
    assert.ok(binaryName(platform).startsWith("tali-native"));
  }
});
