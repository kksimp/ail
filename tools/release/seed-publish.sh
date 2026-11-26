#!/usr/bin/env bash
# Publish built seeds to seeds.voltare.us (a Cloudflare R2 bucket) and write the manifest.
#   tools/release/seed-publish.sh TAG DIR
# DIR holds the ail-seed-* files and seeds.sha256 that tools/release/seed.sh built. Uploads
#   ail/TAG/<asset>, ail/TAG/seeds.sha256, ail/TAG/manifest.json   (immutable: refuses to overwrite a tag)
#   ail/latest.json                                                 (a pointer for people; ./ail never reads it)
# Trust comes from the hashes pinned in the repo (tools/release/seeds.sha256), not from this site.
# SKIP_LATEST=1 leaves latest.json alone (used when migrating older tags).
# Credentials: env R2_ACCOUNT_ID, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY, else the Keychain
# (tools/release/seed-credentials.sh). Bucket and public host can be overridden for tests.
set -euo pipefail
tag="${1:?usage: tools/release/seed-publish.sh TAG DIR}"; dir="${2:?usage: tools/release/seed-publish.sh TAG DIR}"
bucket="${R2_BUCKET:-voltare-seeds}"; host="${SEED_HOST:-https://seeds.voltare.us}"
case "$tag" in seed-*[!A-Za-z0-9._-]* | "" | */*) echo "seed-publish: bad tag '$tag'" >&2; exit 2 ;; esac
kc() { security find-generic-password -s ail-seed-r2 -a "$1" -w 2>/dev/null || true; }
acct="${R2_ACCOUNT_ID:-$(kc account-id)}"; key="${R2_ACCESS_KEY_ID:-$(kc access-key-id)}"; secret="${R2_SECRET_ACCESS_KEY:-$(kc secret-access-key)}"
[ -n "$acct" ] && [ -n "$key" ] && [ -n "$secret" ] || { echo "seed-publish: no R2 credentials (tools/release/seed-credentials.sh)" >&2; exit 2; }
ep="https://$acct.r2.cloudflarestorage.com/$bucket"
s3() { curl -fsS --aws-sigv4 "aws:amz:auto:s3" --user "$key:$secret" "$@"; }
if s3 -I -o /dev/null "$ep/ail/$tag/seeds.sha256" 2>/dev/null; then echo "seed-publish: $tag is already published; versions are immutable" >&2; exit 1; fi
commit="${SEED_COMMIT:-$(git -C "$(dirname "${BASH_SOURCE[0]}")" rev-parse HEAD)}"
man="$dir/manifest.json"
{
    printf '{"tag":"%s","commit":"%s","built":"%s","seeds":[' "$tag" "$commit" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    sep=""
    while read -r sum name; do
        [ -f "$dir/$name" ] || { echo "seed-publish: missing $dir/$name" >&2; exit 1; }
        size="$(wc -c < "$dir/$name" | tr -d ' ')"
        printf '%s{"name":"%s","sha256":"%s","size":%s}' "$sep" "$name" "$sum" "$size"; sep=","
    done < "$dir/seeds.sha256"
    printf ']}\n'
} > "$man"
while read -r sum name; do
    echo "seed-publish: uploading $name ..." >&2
    s3 -T "$dir/$name" -H "Content-Type: application/octet-stream" "$ep/ail/$tag/$name" >/dev/null
    got="$(curl -fsSL "$host/ail/$tag/$name" | { sha256sum 2>/dev/null || shasum -a 256; } | cut -d' ' -f1)"
    [ "$got" = "$sum" ] || { echo "seed-publish: $name served from $host doesn't match its hash; leaving latest unchanged" >&2; exit 1; }
done < "$dir/seeds.sha256"
s3 -T "$dir/seeds.sha256" -H "Content-Type: text/plain" "$ep/ail/$tag/seeds.sha256" >/dev/null
s3 -T "$man" -H "Content-Type: application/json" "$ep/ail/$tag/manifest.json" >/dev/null
if [ -n "${SKIP_LATEST:-}" ]; then echo "seed-publish: $tag is live at $host/ail/$tag/ (latest.json left alone)" >&2; exit 0; fi
s3 -T "$man" -H "Content-Type: application/json" -H "Cache-Control: max-age=60" "$ep/ail/latest.json" >/dev/null
echo "seed-publish: $tag is live at $host/ail/$tag/ ($host/ail/latest.json points at it)" >&2
