#!/usr/bin/env bash
# --- scripts/check_deps.sh ---
# Dependency policy: a light tool stays light because its dependency list is
# short. Normal dependencies only; dev-dependencies do not ship and are allowed
# to be heavier.
#
# The denylist is short and explicit on purpose. Anything needing a broader
# policy (licences, advisories, duplicate versions) belongs in cargo-deny, not
# in this script.

set -euo pipefail

budget=${HOB_DEPS_BUDGET:-150}
# rustls everywhere: no OpenSSL means no build-time system dependency.
denylist='^(openssl-sys|native-tls)$'

packages=$(cargo tree --edges normal --prefix none | sed 's/ v.*//' | sort -u)
count=$(printf '%s\n' "$packages" | grep -c .)

printf 'deps: %s normal crates (budget %s)\n' "$count" "$budget"
if [ "$count" -gt "$budget" ]; then
  echo "deps: over budget; a new dependency has to be argued, not just added" >&2
  exit 1
fi

if printf '%s\n' "$packages" | grep -qE "$denylist"; then
  printf '%s\n' "$packages" | grep -E "$denylist" | sed 's/^/deps: banned: /' >&2
  exit 1
fi
