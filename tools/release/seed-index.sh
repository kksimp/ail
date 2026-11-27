#!/usr/bin/env bash
# Rebuild https://seeds.voltare.us/ail/index.json: every published seed tag with its manifest, newest first.
#   tools/release/seed-index.sh
# seed-publish.sh runs this after each publish, so the download page on voltare.us lists a new seed with no site
# redeploy. Run it by hand once to create the first index. It reads ail/TAG/manifest.json for every tag in the bucket;
# a tag without a manifest is skipped with a warning. Trust comes from the hashes pinned in the repo
# (tools/release/seeds.sha256), not from this file (ADR 0976, ADR 1210).
# Credentials: env R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, else the Keychain (seed-credentials.sh).
set -euo pipefail
bucket="${R2_BUCKET:-voltare-seeds}"; host="${SEED_HOST:-https://seeds.voltare.us}"
kc() { security find-generic-password -s ail-seed-r2 -a "$1" -w 2>/dev/null || true; }
acct="${R2_ACCOUNT_ID:-$(kc account-id)}"; key="${R2_ACCESS_KEY_ID:-$(kc access-key-id)}"; secret="${R2_SECRET_ACCESS_KEY:-$(kc secret-access-key)}"
[ -n "$acct" ] && [ -n "$key" ] && [ -n "$secret" ] || { echo "seed-index: no R2 credentials (tools/release/seed-credentials.sh)" >&2; exit 2; }
ep="https://$acct.r2.cloudflarestorage.com/$bucket"
s3() { curl -fsS --aws-sigv4 "aws:amz:auto:s3" --user "$key:$secret" "$@"; }
tmp="$(mktemp -d)"; trap 'rm -rf "$tmp"' EXIT
# Tags, newest first. Tag names start with the date, so a reverse sort puts 2026.10.25b before 2026.10.25.
s3 "$ep?list-type=2&prefix=ail/seed-&delimiter=/" > "$tmp/list.xml"
tags="$(grep -o '<Prefix>ail/seed-[^<]*</Prefix>' "$tmp/list.xml" | sed 's#<Prefix>ail/##;s#/</Prefix>##' | LC_ALL=C sort -r)"
[ -n "$tags" ] || { echo "seed-index: no seed tags found in $bucket" >&2; exit 1; }
{
    printf '{"updated":"%s","seeds":[' "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    sep=""
    for t in $tags; do
        if m="$(curl -fsSL "$host/ail/$t/manifest.json" 2>/dev/null)" && [ -n "$m" ]; then
            printf '%s%s' "$sep" "$(printf '%s' "$m" | tr -d '\n')"; sep=","
        else
            echo "seed-index: $t has no manifest.json, skipped" >&2
        fi
    done
    printf ']}\n'
} > "$tmp/index.json"
s3 -T "$tmp/index.json" -H "Content-Type: application/json" -H "Cache-Control: max-age=60" "$ep/ail/index.json" >/dev/null
echo "seed-index: $(echo "$tags" | wc -l | tr -d ' ') tags written to $host/ail/index.json" >&2
