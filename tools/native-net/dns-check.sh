#!/usr/bin/env bash
# The native DNS client (net_dns.ail, Linux) against a scripted UDP server: a
# name that is not in /etc/hosts resolves through the server named in a
# temporary resolv.conf, and an unknown name fails. The checker is built in a
# temporary copy of compiler/ with the resolv.conf path and the DNS port
# replaced, since the real ones need root. Linux x86-64, and ARM64 under qemu
# when it is installed. Run from anywhere: tools/native-net/dns-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
trap 'kill $dns $http 2>/dev/null || true; rm -rf "$tmp"' EXIT
python3 tools/native-net/server.py > "$tmp/port" &
http=$!
while [ ! -s "$tmp/port" ]; do sleep 0.1; done
hport=$(head -1 "$tmp/port")
cat > "$tmp/dns.py" <<'P'
import socket, struct, sys
s = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
s.bind(("127.0.0.1", 0))
print(s.getsockname()[1], flush=True)
while True:
    data, addr = s.recvfrom(512)
    i = 12
    labels = []
    while data[i]:
        labels.append(data[i + 1:i + 1 + data[i]].decode())
        i += data[i] + 1
    name = ".".join(labels)
    head = data[:2]
    question = data[12:i + 5]
    if name == "fakehost.test":
        # A CNAME first, then the address, as real resolvers answer.
        cname = b"\xc0\x0c\x00\x05\x00\x01\x00\x00\x00\x3c" + struct.pack(">H", 2) + b"\xc0\x0c"
        a = b"\xc0\x0c\x00\x01\x00\x01\x00\x00\x00\x3c\x00\x04\x7f\x00\x00\x01"
        s.sendto(head + b"\x81\x80\x00\x01\x00\x02\x00\x00\x00\x00" + question + cname + a, addr)
    else:
        s.sendto(head + b"\x81\x83\x00\x01\x00\x00\x00\x00\x00\x00" + question, addr)
P
python3 "$tmp/dns.py" > "$tmp/dnsport" &
dns=$!
while [ ! -s "$tmp/dnsport" ]; do sleep 0.1; done
dport=$(head -1 "$tmp/dnsport")
printf 'nameserver 127.0.0.1\n' > "$tmp/resolv.conf"
cp -r compiler "$tmp/compiler"
cp -r std "$tmp/std"
sed -i "s|\"/etc/resolv.conf\"|\"$tmp/resolv.conf\"|" "$tmp/compiler/rt/native/net_dns.ail"
sed -i "s|net_sockaddr(ns, 53)|net_sockaddr(ns, $dport)|" "$tmp/compiler/rt/native/net_dns.ail"
export AIL_COMPILER="$tmp/compiler"
qemu=$(command -v qemu-aarch64-static || command -v qemu-aarch64 || true)
want=$(./ail run tools/native-net/dns_get.ail 127.0.0.1 "$hport" 2>&1)
./ail build tools/native-net/dns_get.ail -o "$tmp/x64" > /dev/null
got=$("$tmp/x64" fakehost.test "$hport" 2>&1)
[ "$want" = "$got" ] || { echo "FAIL x86-64: want $want, got $got"; exit 1; }
echo "x86-64: $got"
miss=$("$tmp/x64" nosuch.test "$hport" 2>&1)
case "$miss" in *not_found*) echo "x86-64 unknown name: not_found";; *) echo "FAIL unknown name: $miss"; exit 1;; esac
if [ -n "$qemu" ]; then
  ./ail build tools/native-net/dns_get.ail --target aarch64-linux -o "$tmp/a64" > /dev/null
  got=$("$qemu" "$tmp/a64" fakehost.test "$hport" 2>&1)
  [ "$want" = "$got" ] || { echo "FAIL arm64: want $want, got $got"; exit 1; }
  echo "arm64: $got"
fi
echo "dns-check: all passed"
