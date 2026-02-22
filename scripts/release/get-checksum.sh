#!/usr/bin/env bash
set -euo pipefail

CHECKSUM_FILE="${1:-}"
FILENAME="${2:-}"

if [[ -z "$CHECKSUM_FILE" || -z "$FILENAME" ]]; then
  echo "usage: get-checksum.sh <checksum-file> <filename>" >&2
  exit 1
fi

if [[ ! -f "$CHECKSUM_FILE" ]]; then
  echo "checksum file not found: $CHECKSUM_FILE" >&2
  exit 1
fi

HASH="$(awk -v name="$FILENAME" '$2 == name { print $1 }' "$CHECKSUM_FILE")"
if [[ -z "$HASH" ]]; then
  echo "checksum for $FILENAME not found in $CHECKSUM_FILE" >&2
  exit 1
fi

echo "$HASH"
