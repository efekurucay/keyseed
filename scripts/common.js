#!/usr/bin/env node
const fs = require('node:fs');
const os = require('node:os');
const path = require('node:path');
const https = require('node:https');
const { spawnSync } = require('node:child_process');

const root = path.resolve(__dirname, '..');
const pkg = JSON.parse(fs.readFileSync(path.join(root, 'package.json'), 'utf8'));
const binaryName = process.platform === 'win32' ? 'keyseed.exe' : 'keyseed';
const localReleaseBinary = path.join(root, 'target', 'release', binaryName);
const installedBinary = path.join(root, 'bin', 'native', binaryName);

function cargoCommand() {
  const homeCargo = process.platform === 'win32'
    ? path.join(os.homedir(), '.cargo', 'bin', 'cargo.exe')
    : path.join(os.homedir(), '.cargo', 'bin', 'cargo');

  if (fs.existsSync(homeCargo)) {
    return homeCargo;
  }

  return process.platform === 'win32' ? 'cargo.exe' : 'cargo';
}

function run(command, args, options = {}) {
  return spawnSync(command, args, {
    cwd: root,
    shell: process.platform === 'win32',
    stdio: options.stdio ?? 'inherit',
    ...options,
  });
}

function resolveAsset() {
  if (process.platform === 'linux' && process.arch === 'x64') {
    return { archive: 'tar.gz', suffix: 'linux-x64' };
  }

  if (process.platform === 'win32' && process.arch === 'x64') {
    return { archive: 'zip', suffix: 'windows-x64' };
  }

  if (process.platform === 'darwin' && process.arch === 'arm64') {
    return { archive: 'tar.gz', suffix: 'macos-apple-silicon' };
  }

  if (process.platform === 'darwin' && process.arch === 'x64') {
    return { archive: 'tar.gz', suffix: 'macos-intel' };
  }

  return null;
}

function assetFileName() {
  const asset = resolveAsset();
  if (!asset) {
    return null;
  }

  return `keyseed-v${pkg.version}-${asset.suffix}.${asset.archive}`;
}

function releaseUrl() {
  const fileName = assetFileName();
  if (!fileName) {
    return null;
  }

  return `https://github.com/efekurucay/keyseed/releases/download/v${pkg.version}/${fileName}`;
}

function ensureNativeDir() {
  fs.mkdirSync(path.dirname(installedBinary), { recursive: true });
}

function followRedirect(url, callback) {
  https
    .get(url, (response) => {
      if (response.statusCode && response.statusCode >= 300 && response.statusCode < 400 && response.headers.location) {
        response.resume();
        followRedirect(response.headers.location, callback);
        return;
      }

      callback(response);
    })
    .on('error', (error) => callback(null, error));
}

function download(url, destination) {
  return new Promise((resolve, reject) => {
    followRedirect(url, (response, error) => {
      if (error) {
        reject(error);
        return;
      }

      if (!response) {
        reject(new Error('No response received while downloading binary.'));
        return;
      }

      if (response.statusCode !== 200) {
        response.resume();
        reject(new Error(`Unexpected HTTP status ${response.statusCode} for ${url}`));
        return;
      }

      const file = fs.createWriteStream(destination);
      response.pipe(file);

      file.on('finish', () => file.close(resolve));
      file.on('error', (streamError) => reject(streamError));
    });
  });
}

function extractArchive(archivePath, destination) {
  const asset = resolveAsset();
  if (!asset) {
    throw new Error(`Unsupported platform: ${process.platform} ${process.arch}`);
  }

  if (asset.archive === 'tar.gz') {
    const result = run('tar', ['-xzf', archivePath, '-C', destination], { stdio: 'inherit' });
    if (result.error || result.status !== 0) {
      throw result.error || new Error('Failed to extract tar.gz archive.');
    }
    return;
  }

  const psCommand = [
    '-NoProfile',
    '-Command',
    `Expand-Archive -Path '${archivePath.replace(/'/g, "''")}' -DestinationPath '${destination.replace(/'/g, "''")}' -Force`,
  ];
  const result = run('powershell', psCommand, { stdio: 'inherit' });
  if (result.error || result.status !== 0) {
    throw result.error || new Error('Failed to extract zip archive.');
  }
}

function findBinary(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const fullPath = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      const nested = findBinary(fullPath);
      if (nested) {
        return nested;
      }
      continue;
    }

    if (entry.isFile() && entry.name === binaryName) {
      return fullPath;
    }
  }

  return null;
}

async function tryDownloadBinary(log = console.log) {
  const url = releaseUrl();
  if (!url) {
    return false;
  }

  const tempDir = fs.mkdtempSync(path.join(os.tmpdir(), 'keyseed-'));
  const archivePath = path.join(tempDir, assetFileName());

  try {
    log(`[keyseed] Downloading prebuilt binary from ${url}`);
    await download(url, archivePath);
    extractArchive(archivePath, tempDir);

    const binary = findBinary(tempDir);
    if (!binary) {
      throw new Error('Binary not found inside downloaded archive.');
    }

    ensureNativeDir();
    fs.copyFileSync(binary, installedBinary);
    if (process.platform !== 'win32') {
      fs.chmodSync(installedBinary, 0o755);
    }

    return true;
  } finally {
    fs.rmSync(tempDir, { recursive: true, force: true });
  }
}

function cargoAvailable() {
  const result = run(cargoCommand(), ['--version'], { stdio: 'ignore' });
  return !result.error && result.status === 0;
}

function buildWithCargo(log = console.log) {
  log('[keyseed] Building Rust binary with cargo --release');
  const build = run(cargoCommand(), ['build', '--release']);

  if (build.error) {
    throw build.error;
  }

  if (build.status !== 0) {
    throw new Error(`cargo build failed with exit code ${build.status}`);
  }

  if (!fs.existsSync(localReleaseBinary)) {
    throw new Error('Cargo build completed but the binary was not found.');
  }

  ensureNativeDir();
  fs.copyFileSync(localReleaseBinary, installedBinary);
  if (process.platform !== 'win32') {
    fs.chmodSync(installedBinary, 0o755);
  }
}

async function ensureBinary(log = console.log) {
  if (fs.existsSync(installedBinary)) {
    return installedBinary;
  }

  if (fs.existsSync(localReleaseBinary)) {
    ensureNativeDir();
    fs.copyFileSync(localReleaseBinary, installedBinary);
    if (process.platform !== 'win32') {
      fs.chmodSync(installedBinary, 0o755);
    }
    return installedBinary;
  }

  try {
    const downloaded = await tryDownloadBinary(log);
    if (downloaded && fs.existsSync(installedBinary)) {
      return installedBinary;
    }
  } catch (error) {
    log(`[keyseed] Prebuilt download failed: ${error.message}`);
  }

  if (!cargoAvailable()) {
    throw new Error('Cargo was not found and no prebuilt binary could be downloaded.');
  }

  buildWithCargo(log);
  return installedBinary;
}

module.exports = {
  assetFileName,
  buildWithCargo,
  cargoAvailable,
  ensureBinary,
  installedBinary,
  localReleaseBinary,
  releaseUrl,
  resolveAsset,
  root,
};
