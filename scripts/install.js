#!/usr/bin/env node
// SPDX-License-Identifier: MIT
//
// Implements #28. Postinstall downloader for the `talicode` package.
//
// TaliCode's core is a compiled Rust binary. Rather than ship a separate npm
// package per platform, the single `talicode` package downloads the matching
// native `tali` binary from the versioned GitHub Release at install time and
// stores it next to the launcher. This keeps the published package tiny and
// avoids publishing many similarly-named platform packages.
//
// Set TALICODE_SKIP_DOWNLOAD=1 to skip (e.g. in the source checkout or when
// vendoring the binary another way).

"use strict";

const fs = require("node:fs");
const path = require("node:path");
const https = require("node:https");

const pkg = require("../package.json");
const REPO = "mackignacio/talicode";

// Map `${process.platform} ${process.arch}` to the release asset that ships the
// matching native binary. Keep in lockstep with the release workflow's matrix.
const ASSETS = {
  "darwin arm64": "tali-darwin-arm64",
  "darwin x64": "tali-darwin-x64",
  "linux x64": "tali-linux-x64",
  "linux arm64": "tali-linux-arm64",
  "win32 x64": "tali-win32-x64.exe",
};

/** Local path the launcher execs (`bin/tali-native[.exe]`). */
function binaryTarget() {
  const ext = process.platform === "win32" ? ".exe" : "";
  return path.join(__dirname, "..", "bin", `tali-native${ext}`);
}

/** GET with redirect following (GitHub release assets 302 to a CDN). */
function download(url, dest, redirectsLeft = 5) {
  return new Promise((resolve, reject) => {
    const req = https.get(
      url,
      { headers: { "User-Agent": "talicode-installer", Accept: "application/octet-stream" } },
      (res) => {
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          res.resume();
          if (redirectsLeft === 0) return reject(new Error("too many redirects"));
          return resolve(download(res.headers.location, dest, redirectsLeft - 1));
        }
        if (res.statusCode !== 200) {
          res.resume();
          return reject(new Error(`HTTP ${res.statusCode} for ${url}`));
        }
        const tmp = `${dest}.download`;
        const file = fs.createWriteStream(tmp);
        res.pipe(file);
        file.on("finish", () => file.close(() => {
          fs.renameSync(tmp, dest);
          resolve();
        }));
        file.on("error", (err) => { fs.rmSync(tmp, { force: true }); reject(err); });
      }
    );
    req.on("error", reject);
  });
}

async function main() {
  if (process.env.TALICODE_SKIP_DOWNLOAD) {
    console.log("talicode: TALICODE_SKIP_DOWNLOAD set — skipping binary download.");
    return;
  }
  const key = `${process.platform} ${process.arch}`;
  const asset = ASSETS[key];
  if (!asset) {
    console.error(
      `talicode: unsupported platform "${key}". ` +
        `Supported: ${Object.keys(ASSETS).join(", ")}. ` +
        `The \`tali\` command will not be available.`
    );
    process.exit(1);
  }
  const url = `https://github.com/${REPO}/releases/download/v${pkg.version}/${asset}`;
  const dest = binaryTarget();
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  console.log(`talicode: downloading ${asset} (v${pkg.version})...`);
  try {
    await download(url, dest);
    if (process.platform !== "win32") fs.chmodSync(dest, 0o755);
    console.log("talicode: native binary installed.");
  } catch (err) {
    console.error(
      `talicode: failed to download the native binary from ${url}\n` +
        `  ${err.message}\n` +
        `  Check your network, or install manually from ` +
        `https://github.com/${REPO}/releases/tag/v${pkg.version}`
    );
    process.exit(1);
  }
}

main();
