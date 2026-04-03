# Keyseed

<p align="center">
  <strong>Tiny deterministic secret encryption CLI.</strong>
</p>

<p align="center">
  <img src="https://img.shields.io/npm/v/%40efekurucay%2Fkeyseed" alt="npm version">
  <img src="https://img.shields.io/github/actions/workflow/status/efekurucay/keyseed/ci.yml?branch=main&label=ci" alt="CI">
  <img src="https://img.shields.io/github/actions/workflow/status/efekurucay/keyseed/release.yml?label=release" alt="Release">
  <img src="https://img.shields.io/github/license/efekurucay/keyseed" alt="License">
</p>

Keyseed is a small local-first CLI for generating deterministic encrypted payloads from a text input and a master key.

If the input and master key stay the same, the output stays the same.

## Install

### npx

```bash
npx @efekurucay/keyseed encrypt "efe.facebook" -k "master-key"
```

### global install

```bash
npm install -g @efekurucay/keyseed
keyseed encrypt "efe.facebook" -k "master-key"
```

### local development

```bash
npm install
npx . encrypt "efe.facebook" -k "master-key"
```

## Usage

### Encrypt

```bash
keyseed encrypt "efe.facebook" -k "master-key"
```

### Decrypt

```bash
keyseed decrypt "hashit:v2:..." -k "master-key"
```

### Use environment variable

```bash
export HASHIT_MASTER_KEY="master-key"
keyseed encrypt "efe.facebook"
```

## Behavior

- same input + same master key => same output
- output format is `hashit:v2:...`
- old `hashit:v1:...` payloads still decrypt
- works as an npm/npx command backed by a Rust binary

## Development

```bash
cargo test
npm run cli -- encrypt "efe.facebook" -k "master-key"
```

## Release notes

The npm wrapper tries to download a matching prebuilt binary from GitHub Releases.
If no prebuilt binary exists for the platform, it falls back to building with Cargo.

## License

MIT — see [LICENSE](LICENSE).
