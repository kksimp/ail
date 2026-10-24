# Not run by selftest: regenerates this directory (python3 gencerts.py .).
# Generates tests/data/net/x509: test PKI for the certificate
# verifier (ADR 0368). Run once with Python `cryptography`; output committed.
import datetime as dt, sys, os
from cryptography import x509
from cryptography.x509.oid import NameOID, ExtendedKeyUsageOID, ObjectIdentifier
from cryptography.hazmat.primitives import hashes, serialization as s
from cryptography.hazmat.primitives.asymmetric import ec, rsa, padding
import ipaddress

out = sys.argv[1]
os.makedirs(out, exist_ok=True)
T0 = dt.datetime(2026, 1, 1, tzinfo=dt.timezone.utc)
T1 = dt.datetime(2036, 1, 1, tzinfo=dt.timezone.utc)

def name(cn):
    return x509.Name([x509.NameAttribute(NameOID.ORGANIZATION_NAME, "AIL test"), x509.NameAttribute(NameOID.COMMON_NAME, cn)])

def make(subject, key, issuer, issuer_key, *, ca=False, pathlen=None, sans=None, eku=None, nb=T0, na=T1,
         hash_=hashes.SHA256(), extra=(), ku=True, pss=False):
    b = (x509.CertificateBuilder().subject_name(name(subject)).issuer_name(name(issuer))
         .public_key(key.public_key()).serial_number(x509.random_serial_number())
         .not_valid_before(nb).not_valid_after(na))
    b = b.add_extension(x509.BasicConstraints(ca=ca, path_length=pathlen if ca else None), critical=True)
    if ku:
        if ca:
            b = b.add_extension(x509.KeyUsage(False, False, False, False, False, True, True, False, False), critical=True)
        else:
            b = b.add_extension(x509.KeyUsage(True, False, False, False, False, False, False, False, False), critical=True)
    if sans:
        def one(v):
            try:
                return x509.IPAddress(ipaddress.ip_address(v))
            except ValueError:
                return x509.DNSName(v)
        gn = [one(v) for v in sans]
        b = b.add_extension(x509.SubjectAlternativeName(gn), critical=False)
    if eku:
        b = b.add_extension(x509.ExtendedKeyUsage(eku), critical=False)
    for ext, crit in extra:
        b = b.add_extension(ext, critical=crit)
    if isinstance(issuer_key, rsa.RSAPrivateKey) and pss:
        return b.sign(issuer_key, hash_, rsa_padding=padding.PSS(padding.MGF1(hash_), padding.PSS.DIGEST_LENGTH))
    return b.sign(issuer_key, hash_)

root_key = ec.generate_private_key(ec.SECP384R1())
root = make("AIL Test Root", root_key, "AIL Test Root", root_key, ca=True, hash_=hashes.SHA384())
inter_key = rsa.generate_private_key(65537, 2048)
inter = make("AIL Test Intermediate", inter_key, "AIL Test Root", root_key, ca=True, pathlen=0, hash_=hashes.SHA384(),
             eku=[ExtendedKeyUsageOID.SERVER_AUTH])
leaf_key = ec.generate_private_key(ec.SECP256R1())
SANS = ["example.test", "*.wild.test", "127.0.0.1", "::1"]
leaf = make("example.test", leaf_key, "AIL Test Intermediate", inter_key, sans=SANS, eku=[ExtendedKeyUsageOID.SERVER_AUTH])
rsa_leaf_key = rsa.generate_private_key(65537, 2048)
rsa_leaf = make("rsa.test", rsa_leaf_key, "AIL Test Intermediate", inter_key, sans=["rsa.test", "localhost"])
ec_leaf2_key = ec.generate_private_key(ec.SECP256R1())
ec_leaf_localhost = make("localhost", ec_leaf2_key, "AIL Test Intermediate", inter_key, sans=["localhost"])

certs = {
    "root": root, "inter": inter, "leaf": leaf, "rsa_leaf": rsa_leaf, "localhost_leaf": ec_leaf_localhost,
    "expired": make("example.test", leaf_key, "AIL Test Intermediate", inter_key, sans=SANS,
                    na=dt.datetime(2026, 6, 1, tzinfo=dt.timezone.utc)),
    "future": make("example.test", leaf_key, "AIL Test Intermediate", inter_key, sans=SANS,
                   nb=dt.datetime(2030, 1, 1, tzinfo=dt.timezone.utc)),
    "client_eku": make("example.test", leaf_key, "AIL Test Intermediate", inter_key, sans=SANS,
                       eku=[ExtendedKeyUsageOID.CLIENT_AUTH]),
    "unknown_critical": make("example.test", leaf_key, "AIL Test Intermediate", inter_key, sans=SANS,
                             extra=[(x509.UnrecognizedExtension(ObjectIdentifier("1.3.6.1.4.1.99999.1"), b"\x05\x00"), True)]),
    "self_signed": make("example.test", leaf_key, "example.test", leaf_key, sans=SANS),
    # An intermediate that isn't a CA, and a leaf under it.
    "not_ca": make("Not A CA", inter_key, "AIL Test Root", root_key, ca=False, hash_=hashes.SHA384()),
    # pathlen 0 intermediate issuing another CA: the leaf under it is too deep.
    "sub_ca": make("AIL Sub CA", rsa_leaf_key, "AIL Test Intermediate", inter_key, ca=True),
    # Name-constrained intermediate (permitted .allowed.test).
    "constrained": make("AIL Constrained CA", inter_key, "AIL Test Root", root_key, ca=True, hash_=hashes.SHA384(),
                        extra=[(x509.NameConstraints([x509.DNSName("allowed.test")], None), True)]),
    "pss_signed": make("pss.test", leaf_key, "AIL Test Intermediate", inter_key, sans=["pss.test"], pss=True),
    "sha512_leaf": make("example.test", leaf_key, "AIL Test Intermediate", inter_key, sans=SANS, hash_=hashes.SHA512()),
}
certs["under_not_ca"] = make("example.test", leaf_key, "Not A CA", inter_key, sans=SANS)
certs["under_sub_ca"] = make("example.test", leaf_key, "AIL Sub CA", rsa_leaf_key, sans=SANS)
certs["constrained_ok"] = make("x.allowed.test", leaf_key, "AIL Constrained CA", inter_key, sans=["x.allowed.test"])
certs["constrained_bad"] = make("x.other.test", leaf_key, "AIL Constrained CA", inter_key, sans=["x.other.test"])

with open(os.path.join(out, "chain.txt"), "w") as f:
    f.write("# Test certificates (DER, hex), made once with Python `cryptography` by lane\n"
            "# 112 (ADR 0368). Valid 2026-01-01 to 2036-01-01 unless the name says\n"
            "# otherwise. See gencerts.py in the ADR's notes for what each one tests.\n")
    for k, c in certs.items():
        f.write(f"{k} {c.public_bytes(s.Encoding.DER).hex()}\n")
for k, key in [("leaf", leaf_key), ("rsa_leaf", rsa_leaf_key), ("localhost_leaf", ec_leaf2_key)]:
    open(os.path.join(out, f"{k}.key.pem"), "wb").write(
        key.private_bytes(s.Encoding.PEM, s.PrivateFormat.PKCS8, s.NoEncryption()))
for k in ["leaf", "rsa_leaf", "localhost_leaf", "inter", "root"]:
    open(os.path.join(out, f"{k}.pem"), "wb").write(certs[k].public_bytes(s.Encoding.PEM))
print("ok", len(certs))
