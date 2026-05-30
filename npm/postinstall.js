#!/usr/bin/env node
"use strict";

const fs = require("fs");
const os = require("os");
const path = require("path");
const https = require("https");
const http = require("http");
const { execSync } = require("child_process");

// ── Constants ──────────────────────────────────────────────────────
const REPO = "Agions/syncode";
const BINARY = "syncode";
const GITEE_RELEASES = `https://gitee.com/${REPO}/releases`;

// ── Helpers ────────────────────────────────────────────────────────

function getPlatform() {
  switch (os.platform()) {
    case "linux": return "linux";
    case "darwin": return "macos";
    default:
      console.error(`[syncode] Unsupported platform: ${os.platform()}`);
      process.exit(1);
  }
}

function getArch() {
  switch (os.arch()) {
    case "x64": return "x86_64";
    case "arm64": return "aarch64";
    default:
      console.error(`[syncode] Unsupported architecture: ${os.arch()}`);
      process.exit(1);
  }
}

function fetch(url) {
  return new Promise((resolve, reject) => {
    const client = url.startsWith("https") ? https : http;
    const req = client.get(url, { headers: { "User-Agent": "syncode-npm" } }, (res) => {
      // Follow redirects (301/302/303/307/308)
      if ([301, 302, 303, 307, 308].includes(res.statusCode) && res.headers.location) {
        return fetch(res.headers.location).then(resolve, reject);
      }
      if (res.statusCode !== 200) {
        return reject(new Error(`HTTP ${res.statusCode}: ${url}`));
      }
      const chunks = [];
      res.on("data", (chunk) => chunks.push(chunk));
      res.on("end", () => resolve(Buffer.concat(chunks)));
      res.on("error", reject);
    });
    req.on("error", reject);
    req.end();
  });
}

async function getLatestTag() {
  try {
    const url = `https://gitee.com/api/v5/repos/${REPO}/releases?page=1&per_page=1`;
    const data = await fetch(url);
    const json = JSON.parse(data.toString());
    if (json[0] && json[0].tag_name) return json[0].tag_name;
  } catch (_) { /* fall through */ }
  return "v1.2.1";
}

function extractTarGz(archive, dest) {
  execSync(`tar xzf "${archive}" -C "${dest}"`, { stdio: "pipe" });
}

// ── Main ───────────────────────────────────────────────────────────

async function main() {
  const platform = getPlatform();
  const arch = getArch();
  const tag = await getLatestTag();
  const archiveName = `${BINARY}-${tag}-${platform}-${arch}.tar.gz`;
  const downloadUrl = `${GITEE_RELEASES}/download/${tag}/${archiveName}`;

  const binDir = path.join(__dirname, "..", "bin");
  const binPath = path.join(binDir, BINARY);

  // Skip if already installed (e.g. dev mode)
  if (fs.existsSync(binPath)) {
    console.log(`[syncode] Binary already exists at ${binPath}, skipping download.`);
    return;
  }

  console.log(`[syncode] Downloading ${BINARY} ${tag} for ${platform}/${arch}...`);
  console.log(`[syncode] ${downloadUrl}`);

  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "syncode-"));
  const archivePath = path.join(tmpDir, archiveName);

  try {
    const data = await fetch(downloadUrl);
    fs.writeFileSync(archivePath, data);
    extractTarGz(archivePath, tmpDir);

    // Find the binary in extracted contents
    let found = false;
    const candidates = [
      path.join(tmpDir, BINARY),
      path.join(tmpDir, `${BINARY}-${tag}-${platform}-${arch}`, BINARY),
    ];
    // Also scan tmpDir for any file named syncode
    try {
      for (const f of fs.readdirSync(tmpDir)) {
        const fp = path.join(tmpDir, f);
        if (fs.statSync(fp).isFile() && f === BINARY) {
          candidates.unshift(fp);
        }
        // Check one level down
        if (fs.statSync(fp).isDirectory()) {
          try {
            for (const ff of fs.readdirSync(fp)) {
              if (ff === BINARY) candidates.push(path.join(fp, ff));
            }
          } catch (_) {}
        }
      }
    } catch (_) {}

    for (const candidate of candidates) {
      if (fs.existsSync(candidate)) {
        fs.copyFileSync(candidate, binPath);
        fs.chmodSync(binPath, 0o755);
        found = true;
        break;
      }
    }

    if (!found) {
      console.error(`[syncode] Binary not found in archive. Contents:`);
      try {
        const files = fs.readdirSync(tmpDir);
        files.forEach((f) => console.error(`  ${f}`));
      } catch (_) {}
      process.exit(1);
    }

    console.log(`[syncode] Installed ${BINARY} ${tag} successfully.`);
  } catch (err) {
    console.error(`[syncode] Download failed: ${err.message}`);
    console.error(`[syncode] Pre-built binary not available for ${platform}/${arch}.`);
    console.error(`[syncode] Please install from source: cargo install --git https://gitee.com/${REPO}.git`);
    process.exit(1);
  } finally {
    // Cleanup
    try { fs.rmSync(tmpDir, { recursive: true, force: true }); } catch (_) {}
  }
}

main().catch((err) => {
  console.error(`[syncode] Installation failed: ${err.message}`);
  process.exit(1);
});
