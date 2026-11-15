#!/usr/bin/env python3
"""Turn Project Wycheproof test vectors into AIL's vector files (ADR 1118). Measurement and test data only.

    python3 -I tools/native-tls/wycheproof.py JSON_DIR OUT_DIR

JSON_DIR holds the Wycheproof files named in FILES (tools/native-tls/wycheproof.sh downloads them). OUT_DIR gets
`wycheproof.txt` (the crosscheck.txt format: an operation, hex fields, the expected output) and
`wycheproof-sigs.txt` (the signatures.txt format). The vectors are Apache-2.0 (C2SP/wycheproof); NOTICE says so.
Cases whose result is `acceptable` are left out: the file allows either answer. A case this script cannot express
(a 192-bit AES key, a 64-bit tag, a nonce that is not 96 bits) is counted and left out, and the counts go in the header.
Fields are hex; `-` is empty. Everything read from the JSON is data: it is checked as hex and never executed.
"""
import hashlib, json, os, re, sys

FILES = [
    "aes_gcm_test", "chacha20_poly1305_test", "x25519_test", "hmac_sha256_test", "hkdf_sha256_test",
    "ecdsa_secp256r1_sha256_test", "ecdsa_secp256r1_sha512_test", "ecdsa_secp384r1_sha384_test",
    "rsa_signature_2048_sha256_test", "rsa_pss_2048_sha256_mgf1_32_test",
]
HEX = re.compile(r"^[0-9a-f]*$")


def h(s):
    if not isinstance(s, str) or not HEX.match(s) or len(s) % 2:
        raise ValueError("not hex: %r" % (s,))
    return s or "-"


def load(d, name):
    raw = open(os.path.join(d, name + ".json"), "rb").read()
    return json.loads(raw), hashlib.sha256(raw).hexdigest()


def strip_zero(n):
    while n.startswith("00") and len(n) > 2:
        n = n[2:]
    return n


def main():
    src, out = sys.argv[1:3]
    lines, sigs, sums, skipped = [], [], [], {}

    def skip(why):
        skipped[why] = skipped.get(why, 0) + 1

    for name in FILES:
        doc, digest = load(src, name)
        sums.append("# %s.json sha256 %s (%d tests)" % (name, digest, doc["numberOfTests"]))
        for g in doc["testGroups"]:
            for t in g["tests"]:
                r = t["result"]
                if r == "acceptable":
                    skip("acceptable")
                    continue
                if name in ("aes_gcm_test", "chacha20_poly1305_test"):
                    if g["ivSize"] != 96 or g["tagSize"] != 128 or (name[0] == "a" and g["keySize"] not in (128, 256)):
                        skip("aead size")
                        continue
                    aes = name[0] == "a"
                    key, iv, aad, msg, ct, tag = (h(t[k]) for k in ("key", "iv", "aad", "msg", "ct", "tag"))
                    if r == "valid":
                        # Seal: the output is the ciphertext and the tag.
                        lines.append("%s %s %s %s %s %s" % ("aesgcm" if aes else "chacha20poly1305", key, iv, aad, msg,
                                                            h(t["ct"] + t["tag"])))
                    lines.append("%s %s %s %s %s %s %s" % ("aesgcm_open" if aes else "chacha_open", key, iv, aad, ct, tag,
                                                           msg if r == "valid" else "reject"))
                elif name == "x25519_test":
                    lines.append("x25519 %s %s %s" % (h(t["private"]), h(t["public"]), h(t["shared"])))
                elif name == "hmac_sha256_test":
                    if g["tagSize"] != 256 or r != "valid":
                        skip("hmac truncated or invalid")
                        continue
                    lines.append("hmac256 %s %s %s" % (h(t["key"]), h(t["msg"]), h(t["tag"])))
                elif name == "hkdf_sha256_test":
                    if r != "valid" or t["size"] > 255 * 32:
                        skip("hkdf")
                        continue
                    lines.append("hkdf256 %s %s %s %04x %s" % (h(t["salt"]), h(t["ikm"]), h(t["info"]), t["size"], h(t["okm"])))
                elif name.startswith("ecdsa_"):
                    curve = "p256" if "secp256" in name else "p384"
                    hsh = name.split("_")[2]
                    pub = g["publicKey"]["uncompressed"]
                    sigs.append("ecdsa_%s_%s %s %s %s %d" % (curve, hsh, h(pub), h(t["msg"]), h(t["sig"]), r == "valid"))
                elif name.startswith("rsa_signature"):
                    pk = g["publicKey"]
                    sigs.append("rsa_pkcs1_sha256 %s:%s %s %s %d" % (h(strip_zero(pk["modulus"])), h(strip_zero(pk["publicExponent"])),
                                                                     h(t["msg"]), h(t["sig"]), r == "valid"))
                else:
                    pk = g["publicKey"]
                    sigs.append("rsa_pss_sha256 %s:%s %s %s %d" % (h(strip_zero(pk["modulus"])), h(strip_zero(pk["publicExponent"])),
                                                                   h(t["msg"]), h(t["sig"]), r == "valid"))
    head = ["# Project Wycheproof vectors (C2SP/wycheproof, Apache-2.0), converted by tools/native-tls/wycheproof.py (ADR 1118).",
            "# Do not edit by hand: rerun tools/native-tls/wycheproof.sh. Left out: " + (", ".join(
                "%s %d" % (k, v) for k, v in sorted(skipped.items())) or "nothing") + "."] + sums
    open(os.path.join(out, "wycheproof.txt"), "w").write("\n".join(head + [
        "# aesgcm, chacha20poly1305: KEY NONCE AAD MSG CT||TAG; aesgcm_open, chacha_open: KEY NONCE AAD CT TAG PLAINTEXT|reject;",
        "# x25519: SCALAR POINT SHARED; hmac256: KEY MSG TAG; hkdf256: SALT IKM INFO LEN OKM"] + lines) + "\n")
    open(os.path.join(out, "wycheproof-sigs.txt"), "w").write("\n".join(head + [
        "# name pubkey msg sig valid(1|0), as signatures.txt; the DER of an ECDSA signature is parsed as the TLS client parses it"] + sigs) + "\n")
    print("wrote %d vectors and %d signature cases; left out %s" % (len(lines), len(sigs), skipped))


main()
