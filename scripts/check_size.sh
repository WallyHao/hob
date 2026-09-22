#!/usr/bin/env bash
# --- scripts/check_size.sh ---
# Enforce the 120-line source limit from the project conventions.
#
# Measured the way the convention defines it: comments and blank lines are
# excluded, so a well-commented file is not punished for its explanations. Rust
# line comments (`//`) and Lua line comments (`--`) are both recognised.
#
# Tracked and untracked files are both listed. Using `git ls-files` alone would
# pass vacuously in a fresh checkout where nothing has been added yet, which is
# exactly the case where a gate is most likely to be believed.

set -euo pipefail

limit=120
failed=0
checked=0

while IFS= read -r file; do
  [ -f "$file" ] || continue
  lines=$(grep -cvE '^[[:space:]]*(--|//|$)' "$file" || true)
  checked=$((checked + 1))
  if [ "$lines" -gt "$limit" ]; then
    printf '%s: %s lines of code (limit %s)\n' "$file" "$lines" "$limit"
    failed=1
  fi
done < <(git ls-files --cached --others --exclude-standard '*.rs' '*.lua')

if [ "$checked" -eq 0 ]; then
  echo "size: no source files found; is this the repository root?" >&2
  exit 1
fi

if [ "$failed" -ne 0 ]; then
  exit 1
fi

printf 'size: %s source files, all within %s lines of code\n' "$checked" "$limit"
