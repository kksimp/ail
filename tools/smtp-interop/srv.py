import asyncio, ssl, json, os, sys
from aiosmtpd.controller import Controller
from aiosmtpd.smtp import AuthResult, LoginPassword
os.makedirs("inbox", exist_ok=True)
n = 0
class H:
    async def handle_DATA(self, server, session, envelope):
        global n; n += 1
        open(f"inbox/{n}.eml","wb").write(envelope.original_content)
        json.dump({"from":envelope.mail_from,"to":envelope.rcpt_tos,"tls":session.ssl is not None,"user":str(getattr(session,'login_data',None))}, open(f"inbox/{n}.json","w"))
        return "250 OK"
    async def handle_RCPT(self, server, session, envelope, address, rcpt_options):
        if address.startswith("bad@"): return "550 5.1.1 no such user"
        envelope.rcpt_tos.append(address); return "250 OK"
def auth(server, session, envelope, mechanism, data):
    if isinstance(data, LoginPassword) and data.login == b"ann" and data.password == b"pw":
        return AuthResult(success=True, handled=True) if False else AuthResult(success=True)
    return AuthResult(success=False, handled=False)
sc = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER); sc.load_cert_chain("c.pem","k.pem")
cs = [Controller(H(), hostname="127.0.0.1", port=8025, authenticator=auth, auth_require_tls=False),
      Controller(H(), hostname="127.0.0.1", port=8587, authenticator=auth, tls_context=sc, require_starttls=True),
      Controller(H(), hostname="127.0.0.1", port=8465, authenticator=auth, ssl_context=sc)]
for c in cs: c.start()
print("up", flush=True)
import time; time.sleep(3000)
