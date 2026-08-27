#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_DIR="$(mktemp -d)"
trap 'rm -rf "$TEST_DIR"' EXIT

mkdir -p "$TEST_DIR/bin"

printf '%s\n' \
	'#!/usr/bin/env bash' \
	'exit 0' >"$TEST_DIR/bin/rustup"

# The generated stub expands these variables when the test invokes it.
# shellcheck disable=SC2016
printf '%s\n' \
	'#!/usr/bin/env bash' \
	'set -euo pipefail' \
	'printf "%s\n" "$*" >"$EARL_TEST_CARGO_LOG"' \
	'mkdir -p target/aarch64-apple-darwin/release' \
	': >target/aarch64-apple-darwin/release/earl' >"$TEST_DIR/bin/cargo"

printf '%s\n' \
	'#!/usr/bin/env bash' \
	'printf "%s\n" "host: test-host"' >"$TEST_DIR/bin/rustc"

chmod +x "$TEST_DIR/bin/cargo" "$TEST_DIR/bin/rustc" "$TEST_DIR/bin/rustup"

export EARL_TEST_CARGO_LOG="$TEST_DIR/cargo.log"
(
	cd "$TEST_DIR"
	BASH_ENV=/dev/null PATH="$TEST_DIR/bin:$PATH" /bin/bash "$SCRIPT_DIR/build-artifact.sh" \
		--target aarch64-apple-darwin \
		--version 0.0.0 \
		--output-dir dist
)

EXPECTED="build --locked --release --target aarch64-apple-darwin"
ACTUAL="$(cat "$EARL_TEST_CARGO_LOG")"

if [[ "$ACTUAL" != "$EXPECTED" ]]; then
	echo "unexpected cargo invocation: $ACTUAL" >&2
	exit 1
fi

ARCHIVE="$TEST_DIR/dist/earl-0.0.0-aarch64-apple-darwin.tar.gz"
if [[ ! -f "$ARCHIVE" ]]; then
	echo "expected archive not found: $ARCHIVE" >&2
	exit 1
fi
