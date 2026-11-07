#!/bin/sh
# Regenerates tools/tzdata/windows-zones.txt from CLDR's windowsZones.xml (Unicode License v3):
# one `Windows zone ID<TAB>IANA name` line per ID, territory 001 (the default zone of the ID).
# Used by tz.system() on Windows (std.tz.2, ADR 0972). Usage: windows-zones.sh windowsZones.xml
set -e
here=$(cd "$(dirname "$0")" && pwd)
awk -F'"' '/<mapZone / && /territory="001"/ { split($6, a, " "); print $2 "\t" a[1] }' "$1" | LC_ALL=C sort > "$here/windows-zones.txt"
