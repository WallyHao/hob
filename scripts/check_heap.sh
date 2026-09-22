#!/usr/bin/env bash
# --- scripts/check_heap.sh ---
# Peak heap of `hob --version`, measured under massif.
#
# Massif reports the same number on every machine, unlike RSS snapshots, so
# this is a gate and not a weather report. Same ratchet rules as binary size.

set -euo pipefail

budget_kib=${HOB_HEAP_BUDGET_KIB:-16}

if ! command -v valgrind >/dev/null 2>&1; then
  echo "heap: valgrind is required (see 'just env')" >&2
  exit 1
fi

cargo build --release --locked --quiet
out=target/massif.out
valgrind --tool=massif --massif-out-file="$out" --quiet target/release/hob --version >/dev/null

peak_bytes=$(grep '^mem_heap_B=' "$out" | cut -d= -f2 | sort -n | tail -1)
peak_kib=$(( (peak_bytes + 1023) / 1024 ))

printf 'heap: %s byte(s) peak (budget %s KiB)\n' "$peak_bytes" "$budget_kib"
if [ "$peak_kib" -gt "$budget_kib" ]; then
  echo "heap: over budget" >&2
  exit 1
fi
