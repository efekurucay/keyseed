# Hashit

<p align="center">
  <strong>A clean, cross-platform desktop app for encrypting secrets locally with one master key.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/github/actions/workflow/status/efekurucay/hashit/ci.yml?branch=main&label=ci" alt="CI">
  <img src="https://img.shields.io/github/actions/workflow/status/efekurucay/hashit/release.yml?label=release" alt="Release">
  <img src="https://img.shields.io/badge/Rust-1.74%2B-000000?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/UI-egui-2563eb" alt="egui">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Linux%20%7C%20Windows-0f766e" alt="Platforms">
  <img src="https://img.shields.io/badge/crypto-AES--256--GCM%20%2F%20AES--256--GCM--SIV%20%2B%20PBKDF2-7c3aed" alt="Crypto">
  <img src="https://img.shields.io/badge/license-MIT-16a34a" alt="License">
</p>

Hashit is a small open-source desktop app for keeping secrets encrypted on your own machine.
You remember **one master key**. Hashit uses it to encrypt or decrypt passwords, notes, and other sensitive text without sending anything to a server.

## Why Hashit?

- **Local-first** — no cloud sync, no accounts, no telemetry
- **Simple UI** — paste text, enter your master key, encrypt or decrypt
- **Cross-platform** — built in Rust and designed for macOS, Linux, and Windows
- **Portable output** — encrypted text is stored as a plain string you can save anywhere
- **Backward compatible** — decrypts existing `hashit:v1:<base64>` payloads and writes deterministic `hashit:v2:<base64>` payloads by default

## Features

- Single-window desktop app
- Responsive layout for narrow and wide window sizes
- Copy plain text and encrypted output to clipboard
- Master key is not persisted to disk
- Clean output format for easy storage and transfer

## Security model

Hashit is intentionally small and focused.

- Key derivation: **PBKDF2-HMAC-SHA256**
- Iterations: **210,000**
- Default encryption: **AES-256-GCM-SIV** with deterministic output
- Legacy decryption support: **AES-256-GCM** for existing v1 payloads
- Legacy v1 random salt: **16 bytes**
- Legacy v1 random nonce: **12 bytes**
- Payload formats: **`hashit:v1:<base64>`** and **`hashit:v2:<base64>`**

Payload layout:

- **v2 (default)**
  - `1 byte` version
  - `ciphertext + 16 bytes authentication tag`
  - deterministic: the same input + the same master key produce the same output
- **v1 (legacy)**
  - `1 byte` version
  - `16 bytes` salt
  - `12 bytes` nonce
  - `ciphertext + 16 bytes authentication tag`

## Running locally

### Development

```bash
cargo run
```

### Release build

```bash
cargo build --release
```

### Packaged build

```bash
chmod +x build.sh
./build.sh
```

## Platform notes

### macOS

`build.sh` produces:

- `dist/Hashit.app`
- `dist/hashit-darwin-<arch>.tar.gz`

### Linux

The app is intended to run on Ubuntu and other desktop Linux distributions.
Depending on your system, you may need GUI-related development packages before building from source:

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libx11-dev libxi-dev \
  libgl1-mesa-dev libxcursor-dev libxrandr-dev libxinerama-dev libwayland-dev libxkbcommon-dev
```

Then run:

```bash
cargo run
```

### Windows

Windows packaging is planned as part of the upcoming release workflow.
The application code is being prepared to ship the same GUI on Windows as well.

## Project status

Hashit is under active development.

Planned next steps:

- polished signed release builds for macOS, Linux, and Windows
- GitHub Actions release workflow
- GitHub Releases distribution
- optional app icon and platform-specific packaging improvements

## Contributing

Issues, UX suggestions, and pull requests are welcome.

If you open an issue, useful details include:

- operating system and version
- what action you performed
- expected result
- actual result
- whether the issue is reproducible

## License

MIT — see [LICENSE](LICENSE).
