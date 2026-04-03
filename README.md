# Hashit

A small local-first CLI for deterministic secret encryption with one master key.

Published shape: an npm/npx command backed by a Rust binary.

## What it does

Given the same:
- input text, for example `efe.facebook`
- master key

Hashit produces the same encrypted output every time.

You can later decrypt that payload with the same master key.

## Why this shape?

This repo is now intentionally a **clean CLI project**:
- core crypto logic stays in Rust
- npm/npx is used as the convenient entrypoint
- the `hashit` command is easy to run globally or through `npx`

## Install and run

### Option 1: run from the repo

```bash
npm install
npm run cli -- encrypt "efe.facebook" -k "my-master-key"
```

### Option 2: global install

```bash
npm install -g @efekurucay/hashit
hashit encrypt "efe.facebook" -k "my-master-key"
```

### Option 3: npx

```bash
npx @efekurucay/hashit encrypt "efe.facebook" -k "my-master-key"
```

### Option 4: local package during development

```bash
npm install -g .
hashit encrypt "efe.facebook" -k "my-master-key"
```

## Commands

### Encrypt

```bash
hashit encrypt "efe.facebook" --master-key "my-master-key"
```

Short form:

```bash
hashit encrypt "efe.facebook" -k "my-master-key"
```

### Decrypt

```bash
hashit decrypt "hashit:v2:..." --master-key "my-master-key"
```

### Environment variable

You can also provide the master key with `HASHIT_MASTER_KEY`:

```bash
export HASHIT_MASTER_KEY="my-master-key"
hashit encrypt "efe.facebook"
```

## Crypto

- Key derivation: `PBKDF2-HMAC-SHA256`
- Iterations: `210,000`
- Default format: `hashit:v2:<base64>`
- Default encryption: `AES-256-GCM-SIV`
- Legacy decryption support: `AES-256-GCM` for `hashit:v1`

## Development

```bash
cargo test
cargo build --release
```

## Packaging

```bash
chmod +x build.sh
./build.sh
```

## Notes

- The npm wrapper first tries to download a matching prebuilt binary from GitHub Releases.
- If no prebuilt binary is available, it falls back to building with Cargo.
- Supported prebuilt targets in this repo: Linux x64, Windows x64, macOS Apple Silicon, macOS Intel.

## License

MIT — see [LICENSE](LICENSE).
