#!/usr/bin/env bash
# The native TLS server (ADR 1013, std.net.32) against `openssl s_client`,
# `curl` and AIL's own client, with a throwaway CA: each connects, writes one
# line, and must read it back in an `echo: ` reply. Also a client that offers
# only TLS 1.2, and one that trusts no CA. Run from anywhere:
#   tools/native-tls/server-check.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
tmp=$(mktemp -d)
srv=""
trap 'kill $srv 2>/dev/null || true; rm -rf "$tmp"' EXIT
cd "$tmp"
openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout ca.key -out ca.pem \
  -subj "/CN=AIL Local CA" -days 30 -addext "basicConstraints=critical,CA:TRUE" \
  -addext "keyUsage=critical,keyCertSign,cRLSign" 2>/dev/null
openssl req -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 -nodes -keyout leaf.pk8 -out leaf.csr -subj "/CN=localhost" 2>/dev/null
printf "subjectAltName=DNS:localhost,IP:127.0.0.1\nextendedKeyUsage=serverAuth\nkeyUsage=digitalSignature\nbasicConstraints=CA:FALSE\n" > ext.cnf
openssl x509 -req -in leaf.csr -CA ca.pem -CAkey ca.key -CAcreateserial -out leaf.crt -days 30 -extfile ext.cnf 2>/dev/null
# The chain is the leaf then the CA; the key is also tried in SEC 1 form.
cat leaf.crt ca.pem > chain.pem
openssl pkey -in leaf.pk8 -traditional -out leaf.sec1 2>/dev/null
cd - > /dev/null
port=$((20000 + RANDOM % 20000))
cat > "$tmp/serve.ail" <<'P'
use std.env
use std.fs
use std.net

fn main() !
    args := env.args()
    l := net.listen_tls("127.0.0.1:{args[0]}", fs.read_text(args[1])?, fs.read_text(args[2])?)?
    while true
        c := l.accept()?
        line := c.read_line() else err
            print("server: {err.kind}: {err.message}")
            c.close()
            continue
        c.write("echo: {line else ""}\n") else err
            print("server: {err.kind}")
        c.close()
P
cat > "$tmp/client.ail" <<'P'
use std.env
use std.net

fn main()
    c := net.connect_tls("localhost", env.args()[0].to_int() else 0) else err
        print("client: {err.kind}")
        return
    c.write("hello\n") else err
        print("write: {err.kind}")
        return
    line := c.read_line() else err
        print("read: {err.kind}")
        return
    print(line else "(none)")
P
fail=0
for keyfile in leaf.pk8 leaf.sec1; do
  ./ail run "$tmp/serve.ail" "$port" "$tmp/chain.pem" "$tmp/$keyfile" > "$tmp/server.log" 2>&1 &
  srv=$!
  sleep 3
  got=$(printf 'hello\n' | openssl s_client -connect "localhost:$port" -tls1_3 -CAfile "$tmp/ca.pem" -verify_return_error -quiet 2>"$tmp/s_client.err" | head -1 || true)
  [ "$got" = "echo: hello" ] || { echo "FAIL s_client ($keyfile): '$got'"; cat "$tmp/s_client.err"; fail=1; }
  got=$(curl -s --cacert "$tmp/ca.pem" --tls-max 1.3 --http0.9 "https://localhost:$port/hello" 2>&1 | head -1 || true)
  case "$got" in "echo: GET /hello"*) ;; *) echo "FAIL curl ($keyfile): '$got'"; fail=1;; esac
  got=$(SSL_CERT_FILE="$tmp/ca.pem" ./ail run "$tmp/client.ail" "$port" 2>&1)
  [ "$got" = "echo: hello" ] || { echo "FAIL ail client ($keyfile): '$got'"; fail=1; }
  # TLS 1.2 only: the server must refuse, not hang or crash.
  got=$(printf 'x\n' | openssl s_client -connect "localhost:$port" -tls1_2 -quiet 2>&1 | head -3 || true)
  echo "tls1.2: ${got:-(closed)}"
  # A client that trusts no CA rejects the chain.
  got=$(SSL_CERT_FILE="$tmp/leaf.crt.none" ./ail run "$tmp/client.ail" "$port" 2>&1 || true)
  echo "no trust: $got"
  # The server is still serving after the failed handshakes.
  got=$(printf 'again\n' | openssl s_client -connect "localhost:$port" -tls1_3 -CAfile "$tmp/ca.pem" -quiet 2>/dev/null | head -1 || true)
  [ "$got" = "echo: again" ] || { echo "FAIL after failures ($keyfile): '$got'"; fail=1; }
  kill $srv 2>/dev/null || true
  wait $srv 2>/dev/null || true
  echo "--- server log ($keyfile)"; cat "$tmp/server.log"
done
[ "$fail" = 0 ] && echo "server-check: OK" || { echo "server-check: FAILED"; exit 1; }
