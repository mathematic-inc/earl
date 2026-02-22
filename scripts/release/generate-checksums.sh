#!/usr/bin/env bash
set -euo pipefail

ARTIFACT_DIR="${1:-}"
if [[ -z "$ARTIFACT_DIR" ]]; then
  echo "usage: generate-checksums.sh <artifact-dir>" >&2
  exit 1
fi

if [[ ! -d "$ARTIFACT_DIR" ]]; then
  echo "artifact directory does not exist: $ARTIFACT_DIR" >&2
  exit 1
fi

cd "$ARTIFACT_DIR"

mapfile -t FILES < <(find . -maxdepth 1 -type f | sed 's|^\./||' | grep -v '^SHA256SUMS' | sort)

if [[ ${#FILES[@]} -eq 0 ]]; then
  echo "no files to checksum in $ARTIFACT_DIR" >&2
  exit 1
fi

: > SHA256SUMS
for FILE in "${FILES[@]}"; do
  if command -v sha256sum >/dev/null 2>&1; then
    HASH="$(sha256sum "$FILE" | awk '{ print $1 }')"
  else
    HASH="$(shasum -a 256 "$FILE" | awk '{ print $1 }')"
  fi
  printf '%s  %s\n' "$HASH" "$FILE" >> SHA256SUMS
done

cp SHA256SUMS SHA256SUMS.txt
