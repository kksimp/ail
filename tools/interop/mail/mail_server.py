"""Test SMTP server for run.sh: aiosmtpd with STARTTLS required, AUTH PLAIN/LOGIN,
and delivery of every message into the Maildir of dovecot's user.
usage: mail_server.py PORT CERT KEY MAILDIR USER PASSWORD"""
import os, ssl, sys, time, itertools
from aiosmtpd.controller import Controller
from aiosmtpd.smtp import AuthResult, LoginPassword

port, cert, key, maildir, user, password = sys.argv[1:7]
ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
ctx.load_cert_chain(cert, key)
counter = itertools.count()

def authenticator(server, session, envelope, mechanism, auth_data):
    if isinstance(auth_data, LoginPassword):
        ok = auth_data.login.decode() == user and auth_data.password.decode() == password
        return AuthResult(success=ok, handled=True)
    return AuthResult(success=False, handled=False)

class Deliver:
    async def handle_DATA(self, server, session, envelope):
        name = f"{int(time.time())}.{os.getpid()}.{next(counter)}.probe"
        tmp = os.path.join(maildir, "tmp", name)
        with open(tmp, "wb") as f:
            f.write(envelope.original_content)
        os.rename(tmp, os.path.join(maildir, "new", name))
        return "250 OK delivered"

ctl = Controller(Deliver(), hostname="127.0.0.1", port=int(port), tls_context=ctx,
                 require_starttls=True, authenticator=authenticator,
                 auth_required=True, auth_require_tls=True)
ctl.start()
print("smtp ready", flush=True)
try:
    while True:
        time.sleep(3600)
finally:
    ctl.stop()
