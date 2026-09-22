#!/usr/bin/env bash
# --- scripts/check_binary_size.sh ---
# Gate on the size of the release binary.
#
# The cap is a ratchet: raising it is a deliberate act that has to be argued in
# a diff, which is the point. Re-derive it when a feature legitimately inflates
# the artifact; never raise it just to make a red gate green.

set -euo pipefail

budget_kib=${HOB_BINARY_BUDGET_KIB:-1024}

cargo build --release --locked --quiet
binary=target/release/hob
size_kib=$(( ($(wc -c < "$binary") + 1023) / 1024 ))

printf 'binary-size: %s KiB (budget %s KiB)\n' "$size_kib" "$budget_kib"
if [ "$size_kib" -gt "$budget_kib" ]; then
  printf 'binary-size: over budget by %s KiB\n' "$((size_kib - budget_kib))" >&2
  exit 1
fi
