#!/usr/bin/env bash
# Mail interop (std.net.smtp #1530, std.net.imap #1566 + STARTTLS) against real
# servers on ail-linux: aiosmtpd (STARTTLS required, AUTH) delivers into a
# Maildir that dovecot serves (STARTTLS required on its imap port, TLS on its
# imaps port). Run ON ail-linux from the repo root of the commit to test:
#   tools/interop/mail/run.sh
# Installs dovecot-imapd, python3-aiosmtpd and openssl with apt if missing (sudo,
# Kaleb's OK 2026-11-27). Uses ports 14025/14143/14993 and a temp dir; the
# system dovecot is left alone. Last line: DONE GREEN or DONE RED.
set -u
ROOT=$(cd "$(dirname "$0")/../../.." && pwd)
cd "$ROOT" || exit 2
HERE=tools/interop/mail
export SMTP_PORT=14025 IMAP_PORT=14143 IMAPS_PORT=14993
W=$(mktemp -d); chmod 755 "$W"; export TMPDIR=$W
ME=$(id -un); UID_=$(id -u); GID_=$(id -g)
# dovecot refuses uid 0 for mail: run its mail as nobody when this is root (a cloud container)
[ "$UID_" = 0 ] && { UID_=65534; GID_=65534; }
pids=""
cleanup() {
  [ -f "$W/run/master.pid" ] && sudo kill "$(cat "$W/run/master.pid")" 2>/dev/null
  for p in $pids; do kill "$p" 2>/dev/null; done
  sleep 1; rm -rf "$W" 2>/dev/null || sudo rm -rf "$W"
}
trap cleanup EXIT
echo "# mail interop: $(git rev-parse --short HEAD), $(uname -sm), $(date -u +%FT%TZ)"

need=""
command -v openssl >/dev/null || need="$need openssl"
dpkg -s dovecot-imapd >/dev/null 2>&1 || need="$need dovecot-imapd"
python3 -c 'import aiosmtpd' 2>/dev/null || need="$need python3-aiosmtpd"
if [ -n "$need" ]; then echo "# installing:$need"; sudo apt-get install -y -qq $need >/dev/null || { echo "DONE RED (apt install$need)"; exit 1; }; fi
# the package starts its own dovecot on 143/993: not ours, leave it

echo "=== A. certificates (CA + leaf for localhost)"
cd "$W" || exit 2
openssl ecparam -name prime256v1 -genkey -noout -out ca.key
openssl req -x509 -new -key ca.key -sha256 -days 2 -subj "/CN=probe-ca" -addext "basicConstraints=critical,CA:TRUE" -out ca.pem 2>/dev/null
openssl ecparam -name prime256v1 -genkey -noout -out leaf.key
openssl req -new -key leaf.key -subj "/CN=localhost" -out leaf.csr
printf 'subjectAltName=DNS:localhost\nbasicConstraints=CA:FALSE\nextendedKeyUsage=serverAuth\n' > ext.cnf
openssl x509 -req -in leaf.csr -CA ca.pem -CAkey ca.key -CAcreateserial -days 2 -sha256 -extfile ext.cnf -out leaf.pem 2>/dev/null
mkdir -p mail/bob/cur mail/bob/new mail/bob/tmp run
[ "$(id -u)" = 0 ] && chown -R "$UID_:$GID_" mail
echo "bob:{PLAIN}pw" > users

echo "=== B. servers"
cat > dovecot.conf <<CONF
base_dir = $W/run
listen = 127.0.0.1
log_path = $W/dovecot.log
protocols = imap
ssl = required
ssl_cert = <$W/leaf.pem
ssl_key = <$W/leaf.key
disable_plaintext_auth = yes
auth_mechanisms = plain login
mail_location = maildir:$W/mail/%u
first_valid_uid = 1
passdb {
  driver = passwd-file
  args = scheme=PLAIN username_format=%u $W/users
}
userdb {
  driver = static
  args = uid=$UID_ gid=$GID_ home=$W/mail/%u
}
service imap-login {
  inet_listener imap {
    port = $IMAP_PORT
  }
  inet_listener imaps {
    port = $IMAPS_PORT
    ssl = yes
  }
}
service anvil {
  chroot =
}
CONF
sudo "$(command -v dovecot)" -c "$W/dovecot.conf" || { cat "$W/dovecot.log"; echo "DONE RED (dovecot did not start)"; exit 1; }
python3 "$ROOT/$HERE/mail_server.py" $SMTP_PORT leaf.pem leaf.key "$W/mail/bob" bob pw >"$W/smtp.log" 2>&1 &
pids="$pids $!"
for _ in $(seq 1 30); do grep -q "smtp ready" "$W/smtp.log" 2>/dev/null && break; sleep 0.5; done
grep -q "smtp ready" "$W/smtp.log" || { cat "$W/smtp.log"; echo "DONE RED (smtp server did not start)"; exit 1; }
echo "servers up (dovecot $(dovecot --version | cut -d' ' -f1))"
echo "--- sanity with openssl s_client"
echo | openssl s_client -starttls imap -connect localhost:$IMAP_PORT -CAfile ca.pem -verify_return_error 2>&1 | grep -E "Verify return code" || true
echo | openssl s_client -starttls smtp -connect localhost:$SMTP_PORT -CAfile ca.pem -verify_return_error 2>&1 | grep -E "Verify return code" || true

echo "=== C. probe"
cd "$ROOT" || exit 2
export SSL_CERT_FILE=$W/ca.pem
AIL_FIX=0 ./ail build $HERE/mail_interop.ail -o "$W/probe" >"$W/build.txt" 2>&1 || { grep -E 'error\[' -A6 "$W/build.txt" | head -80; echo "DONE RED (probe did not build)"; exit 1; }
timeout 120 "$W/probe"; rc=$?
if [ $rc -ne 0 ]; then
  echo "--- smtp.log"; tail -20 "$W/smtp.log"; echo "--- dovecot.log"; tail -20 "$W/dovecot.log"
  echo "DONE RED (probe exit $rc)"; exit 1
fi
echo "DONE GREEN"
