#!/usr/bin/env bash
# std.net.smtp against a real server (aiosmtpd): plain, STARTTLS and implicit TLS, then the saved
# messages are parsed with Python's email package. Needs python3 and openssl; installs aiosmtpd with pip.
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"; root="$here/../.."
work="$(mktemp -d)"; trap 'kill $pid 2>/dev/null || true' EXIT
python3 -m pip install -q aiosmtpd >/dev/null 2>&1 || true
cd "$work"
openssl req -x509 -newkey rsa:2048 -nodes -keyout k.pem -out c.pem -days 2 -subj "/CN=localhost" \
  -addext "subjectAltName=DNS:localhost" -addext "basicConstraints=critical,CA:TRUE" >/dev/null 2>&1
cp "$here/srv.py" .; python3 srv.py >srv.log 2>&1 & pid=$!
sleep 3
SSL_CERT_FILE="$work/c.pem" "$root/ail" run "$here/cli.ail"
python3 - <<'PY'
import email, email.policy, glob
fs = glob.glob("inbox/*.eml"); assert len(fs) == 6, len(fs)
for f in fs:
    m = email.message_from_bytes(open(f, "rb").read(), policy=email.policy.default)
    assert m["From"] == "Zoë <zoe@example.com>" and m["Bcc"] is None and m["Subject"].startswith(("Grüße", "again"))
    kinds = [p.get_content_type() for p in m.walk() if not p.is_multipart()]
    assert kinds == ["text/plain", "text/html", "text/plain"], kinds
print("parsed", len(fs), "messages")
PY
