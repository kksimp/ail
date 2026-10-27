#!/bin/sh
# Regenerates tools/tzdata/zones from tzdata.zi (IANA tz database, public domain) as slim TZif files.
# Windows has no system zone files, so tz.offset reads these (std.tz.3, ADR 0448).
# To update: replace tzdata.zi with the new release's file, run this, commit both.
set -e
here=$(cd "$(dirname "$0")" && pwd)
rm -rf "$here/zones"
zic -b slim -d "$here/zones" "$here/tzdata.zi"
