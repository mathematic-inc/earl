# Cross-Compilation Notes

Earl builds release binaries for multiple targets using GitHub Actions. This
document records the platform-specific constraints and workarounds discovered
during cross-compilation.

## Supported Release Targets

| Target                      | Build Tool                                             | Notes                                       |
| --------------------------- | ------------------------------------------------------ | ------------------------------------------- |
| `aarch64-apple-darwin`      | cargo (native)                                         | macOS ARM, built on `macos-15` runner       |
| `x86_64-unknown-linux-gnu`  | cargo (native)                                         | Standard Linux, built on `ubuntu-latest`    |
| `aarch64-unknown-linux-gnu` | [cross](https://github.com/cross-rs/cross)             | ARM64 Linux, Docker-based cross-compilation |
| `x86_64-unknown-linux-musl` | [cross](https://github.com/cross-rs/cross)             | Static musl binary                          |
| `x86_64-pc-windows-msvc`    | [cargo-xwin](https://github.com/rust-cross/cargo-xwin) | Windows x86_64, cross-compiled from Linux   |

## Dropped Targets

### `x86_64-apple-darwin`

Dropped because `ort-sys` (ONNX Runtime, via `fastembed`) does not provide
prebuilt binaries for Intel macOS. GitHub also deprecated `macos-13` runners
(the last Intel macOS runners), and `macos-15` (ARM) cannot cross-compile ONNX
Runtime to x86_64.

### `aarch64-pc-windows-msvc`

Dropped due to a `ring` + `cargo-xwin` incompatibility. When `cc-rs` compiles
`ring`'s ARM64 assembly files, it invokes `clang` in GCC mode but passes
MSVC-style `/imsvc` include flags (which only work with `clang-cl`). This
causes `clang` to interpret `/imsvc` as a file path rather than a flag. This is
an upstream issue in how `cc-rs` selects the compiler for assembly files under
`cargo-xwin`.

## Feature Restrictions by Target

Not all features compile on all targets. The build script
(`scripts/release/build-artifact.sh`) adjusts features per target:

- **Windows**: `bash` feature disabled. The bash protocol uses Unix-only APIs
  (`setsid`, `killpg`, `SIGKILL`, `Command::pre_exec`) that have no Windows
  equivalent.
- **musl, aarch64-linux**: `local-search` feature disabled. `ort-sys` (ONNX
  Runtime) only ships prebuilt binaries for `x86_64-unknown-linux-gnu`,
  `aarch64-apple-darwin`, and `x86_64-pc-windows-msvc`. Targets without
  prebuilt binaries fall back to lexical search.

## C Library Dependencies

Several transitive dependencies require C libraries, which complicates
cross-compilation.

### OpenSSL (`openssl-sys`)

`native-tls` (pulled in by `fastembed` -> `hf-hub` -> `reqwest` with default
features) depends on `openssl-sys`. For cross targets, we vendor OpenSSL via:

```toml
# Cargo.toml
[target.'cfg(target_os = "linux")'.dependencies]
openssl = { version = "0.10", features = ["vendored"] }
```

This compiles OpenSSL from source using the correct cross-compiler toolchain,
avoiding the need for system `libssl-dev` packages in cross containers.

**Important**: When cross-compiling, `openssl-sys` may be compiled for both the
HOST (for proc macros / build scripts) and the TARGET. The vendored feature
handles the TARGET compilation, but the HOST still needs system OpenSSL headers.
The `Cross.toml` pre-build hooks install `libssl-dev` to satisfy the HOST build.

### D-Bus (`libdbus-sys`)

The `keyring` crate's `sync-secret-service` feature depends on
`dbus-secret-service` -> `dbus` -> `libdbus-sys`, which requires the C
`libdbus-1` library. This is problematic for cross-compilation because:

- musl containers don't have musl-compatible D-Bus libraries
- Cross-compiled `libdbus-1-dev:arm64` packages can have include path issues

**Solution**: Use `async-secret-service` instead of `sync-secret-service`. This
switches from the C `dbus` crate to `zbus`, a pure Rust D-Bus implementation.
The keyring API remains synchronous — only the internal transport changes.

```toml
# Cargo.toml
keyring = { version = "3.6.3", default-features = false, features = [
    "apple-native",
    "windows-native",
    "linux-native",
    "async-secret-service",  # pure Rust, no C deps
    "tokio",
    "crypto-rust",
    "vendored",
] }
```

### LLVM tools (`llvm-lib`)

`cargo-xwin` Windows cross-compilation requires `llvm-lib` (the LLVM static
library archiver) for linking. Install via `sudo apt-get install -y llvm` on
the CI runner.

## Cross.toml

The `Cross.toml` file configures [cross](https://github.com/cross-rs/cross)
Docker containers. The `pre-build` hooks install system packages inside the
container before `cargo build` runs:

- `libssl-dev` + `pkg-config`: Required for the HOST compilation of
  `openssl-sys` (the TARGET uses vendored OpenSSL)
- `perl` + `make`: Required by the vendored OpenSSL build script

## Cargo.toml Version Parsing

The release workflow validates that the git tag version matches `Cargo.toml`.
In a workspace with both `[workspace.package]` and `[package]` sections, the
awk command must match the `[package]` section specifically:

```bash
# Correct: matches version under [package]
awk -F'"' '/^\[package\]/{p=1} p && /^version = /{print $2; exit}' Cargo.toml

# Wrong: matches first version line (could be [workspace.package])
awk -F'"' '/^version = / { print $2; exit }' Cargo.toml
```

## Reqwest Version Split

The project depends on `reqwest 0.13`, but `oauth2 5.0` depends on
`reqwest 0.12`. When `fastembed` is enabled, `hf-hub` also pulls in
`reqwest 0.12` with the `json` feature. When `fastembed` is disabled (via
`local-search` feature), `reqwest 0.12` loses the `json` feature, breaking
`oauth2::reqwest::Response::json()`.

**Solution**: Avoid using `response.json()` on oauth2's response type. Use
`response.text()` + `serde_json::from_str()` instead, which has no feature
gate.
