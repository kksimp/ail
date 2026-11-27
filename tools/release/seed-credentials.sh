#!/usr/bin/env bash
# Store the R2 upload credentials for seed publishing in the macOS Keychain (run once, on the Mac).
# Nothing is written to the repository. Prompts for each value; the secret isn't echoed.
#   tools/release/seed-credentials.sh
set -euo pipefail
[ "$(uname -s)" = Darwin ] || { echo "seed-credentials: Keychain storage is macOS only; elsewhere set R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY" >&2; exit 2; }
for pair in "account-id:R2 account id" "access-key-id:R2 access key id" "secret-access-key:R2 secret access key"; do
    k="${pair%%:*}"; read -r -s -p "${pair#*:}: " v; echo
    [ -n "$v" ] || { echo "seed-credentials: empty value" >&2; exit 1; }
    security add-generic-password -U -s ail-seed-r2 -a "$k" -w "$v"
done
echo "seed-credentials: stored under the Keychain service ail-seed-r2" >&2
