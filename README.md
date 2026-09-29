# keychain-secret

A small stdin-friendly CLI for macOS Keychain. Store secrets without putting their values in command-line arguments; retrieve them for local scripts.

## Install

macOS on Apple Silicon:

```sh
brew install xiaotianxt/tap/keychain-secret
```

Homebrew installs a prebuilt release binary. No Rust toolchain is required. Keychain may ask you to approve access, especially after replacing the executable.

## Commands

```sh
keychain-secret set example-service example-account < /path/to/private-secret
keychain-secret check example-service example-account
keychain-secret get example-service example-account  # secret on stdout
keychain-secret delete example-service example-account
keychain-secret import-op example-service example-account op://vault/item/field
keychain-secret --version
```

- `set` reads UTF-8 stdin, strips one trailing LF or CRLF, and rejects an empty value. It creates or replaces the service/account entry.
- `get` writes the secret without a trailing newline. Redirect or capture stdout; do not log it.
- `check` prints `present` (exit 0) or `missing` (exit 1), never the value.
- `delete` removes the exact entry. There is no confirmation prompt.
- `import-op` requires a separately installed, authenticated `op` CLI. It reads one `op://` reference into Keychain, without echoing its value or subprocess diagnostics.
- Errors exit 1; missing/unknown commands exit 2. Help and version do not access Keychain.

This uses the legacy macOS Keychain store with service/account addressing. There is no secret file, daemon, network service, telemetry, or alternative credential-store fallback. An explicitly requested `import-op` invokes 1Password. Service/account labels appear in write confirmations on stderr; choose non-sensitive labels. The system Keychain and its access controls govern access, not this CLI.

## Develop and release

Requires macOS and Rust 1.88+:

```sh
cargo fmt --all -- --check
cargo clippy --locked --all-targets -- -D warnings
cargo test --locked
cargo build --locked --release
```

CI checks formatting, Clippy and tests. A `vVERSION` tag runs the same checks, builds an Apple Silicon archive, and publishes a GitHub release. `scripts/release.sh` publishes the current committed Cargo version, waits for Actions, then updates `xiaotianxt/tap` using the downloaded asset's SHA-256. It requires clean source/tap checkouts, `gh`, and normal Git signing approval. For a new version, edit Cargo.toml, regenerate Cargo.lock, and commit first. See [architecture](docs/architecture.md).

After installing a release with Homebrew, `cargo clean` removes local build outputs without affecting the installed executable or Keychain entries.
