#!/usr/bin/env bash
# --- scripts/check_heap.sh ---
# Peak heap of a cold start, measured under massif.
#
# Two measurements: the CLI itself (`hob --version`) and the runtime a flow
# pulls in (`hob run` on a one-line script). Massif reports the same number on
# every machine, unlike RSS snapshots, so this is a gate and not a weather
# report. Same ratchet rules as binary size.

set -euo pipefail

cli_budget_kib=${HOB_CLI_HEAP_BUDGET_KIB:-16}
flow_budget_kib=${HOB_FLOW_HEAP_BUDGET_KIB:-128}

if ! command -v valgrind >/dev/null 2>&1; then
  echo "heap: valgrind is required (see 'just env')" >&2
  exit 1
fi

cargo build --release --locked --quiet

flow=$(mktemp)
trap 'rm -f "$flow"' EXIT
printf 'local answer = 42\n' > "$flow"

measure() {
  local label=$1 budget=$2
  shift 2
  local out="target/massif-$label.out"
  valgrind --tool=massif --massif-out-file="$out" --quiet "$@" >/dev/null
  local peak
  peak=$(grep '^mem_heap_B=' "$out" | cut -d= -f2 | sort -n | tail -1)
  local peak_kib=$(( (peak + 1023) / 1024 ))
  printf 'heap: %-4s %s byte(s) peak (budget %s KiB)\n' "$label" "$peak" "$budget"
  if [ "$peak_kib" -gt "$budget" ]; then
    echo "heap: $label over budget" >&2
    return 1
  fi
}

measure cli "$cli_budget_kib" target/release/hob --version
measure flow "$flow_budget_kib" target/release/hob run "$flow"
