# Architecture

One synchronous binary owns argument dispatch, stdin normalization, and the optional `op read` subprocess. `keyring-core` addresses credentials by service/account. `apple-native-keyring-store`, with only the `keychain` feature, supplies the native macOS Keychain implementation. The old `keyring` sample CLI dependency was removed without changing the store or its default configuration.

Secrets are held in process memory and are not guaranteed to be zeroized. `get` intentionally exposes its result on stdout; callers must not log it. Subprocess output is captured; failing `op` diagnostics are not forwarded. No automatic retries or fallback stores are used. Existing Keychain access permissions may require interactive approval for a new executable.

The repository contains no credentials or user-specific service names. Unit tests cover text/argument boundaries without accessing real Keychain entries. Release checks and the Homebrew test use non-secret CLI commands. Release archives contain the binary, README and license; Homebrew owns the installed executable independently of Cargo's target directory.
