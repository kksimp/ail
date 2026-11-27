#!/usr/bin/env python3
"""Turn Project Wycheproof test vectors into the vector file of the public std.crypto pass (ADR 1118). Test data only.

    python3 -I tools/native-tls/wycheproof_api.py JSON_DIR OUT_FILE

JSON_DIR holds the files named in FILES (tools/native-tls/wycheproof-api.sh downloads them). OUT_FILE is
`tests/data/net/crypto/wycheproof-api.txt`: lines for tools/native-tls/wycheproof_api.ail, which calls `std.crypto`
itself, so what is checked is the API a program sees (sizes, nonce layout, `none` and `false` answers), not only the
primitives that tools/native-tls/prim-check.sh and wycheproof.py cover. The vectors are Apache-2.0 (C2SP/wycheproof).
Fields are hex; `-` is empty. Everything read from the JSON is data: it is checked as hex and never executed.
Expected answers: `1`/`0` for a verdict, `*` when Wycheproof calls the case `acceptable` (either verdict passes, a
trap does not), `=HEX` for a value (`=` for an empty one) and `none`.
"""
import hashlib, json, os, re, sys

FILES = ["ed25519_test", "x25519_test", "ecdsa_secp256r1_sha256_p1363_test", "chacha20_poly1305_test",
         "aes_gcm_test", "hkdf_sha256_test", "hmac_sha256_test"]
HEX = re.compile(r"^[0-9a-f]*$")
DERIVED = 40  # valid AEAD cases that also get the truncation and tamper variants


def h(s):
    if not isinstance(s, str) or not HEX.match(s) or len(s) % 2:
        raise ValueError("not hex: %r" % (s,))
    return s or "-"


def flip(hexs):
    """The hex with its last byte changed."""
    last = int(hexs[-2:], 16) ^ 0x80
    return hexs[:-2] + "%02x" % last


def main():
    src, out = sys.argv[1:3]
    lines, sums, counts = [], [], {}

    def add(op, *fields):
        counts[op] = counts.get(op, 0) + 1
        lines.append(" ".join((op,) + fields))

    for name in FILES:
        raw = open(os.path.join(src, name + ".json"), "rb").read()
        doc = json.loads(raw)
        sums.append("# %s.json sha256 %s (%d tests)" % (name, hashlib.sha256(raw).hexdigest(), doc["numberOfTests"]))
        derived = 0
        for g in doc["testGroups"]:
            for t in g["tests"]:
                r = t["result"]
                verdict = "*" if r == "acceptable" else ("1" if r == "valid" else "0")
                if name == "ed25519_test":
                    add("ed25519_verify", h(g["publicKey"]["pk"]), h(t["msg"]), h(t["sig"]), verdict)
                elif name == "x25519_test":
                    shared = t["shared"]
                    # std.crypto.6: a result of all zero bytes (a low-order point) is `none`.
                    add("x25519", h(t["private"]), h(t["public"]), "none" if set(shared) <= {"0"} else "=" + shared)
                elif name.startswith("ecdsa_"):
                    add("p256_verify", h(g["publicKey"]["uncompressed"]), h(t["msg"]), h(t["sig"]), verdict)
                elif name in ("chacha20_poly1305_test", "aes_gcm_test"):
                    aes = name[0] == "a"
                    if aes and g["keySize"] != 256:
                        continue  # seal_aes_gcm is AES-256 only (std.crypto.9); other keys trap
                    op = "aesgcm_open" if aes else "chacha_open"
                    key, iv, aad, msg, ct, tag = (t[k] for k in ("key", "iv", "aad", "msg", "ct", "tag"))
                    sealed = iv + ct + tag
                    if g["ivSize"] != 96 or g["tagSize"] != 128:
                        # The API has a 12-byte nonce and a 16-byte tag: this layout is never a sealed message of it.
                        add(op, h(key), h(aad), h(sealed), "none")
                        continue
                    add(op, h(key), h(aad), h(sealed), ("=" + msg) if r == "valid" else "none")
                    if r == "valid" and derived < DERIVED:
                        derived += 1
                        for label, s, a in (
                            ("last byte dropped", sealed[:-2], aad),
                            ("tag cut to 8 bytes", sealed[:-16], aad),
                            ("tag cut to 12 bytes", sealed[:-8], aad),
                            ("nonce only", iv, aad),
                            ("one byte appended", sealed + "00", aad),
                            ("tag flipped", flip(sealed), aad),
                            ("nonce flipped", flip(iv) + ct + tag, aad),
                            ("aad changed", sealed, aad + "00"),
                        ):
                            add(op, h(key), h(a), h(s), "none")
                        add(op, h(key), h(aad), "-", "none")
                elif name == "hkdf_sha256_test":
                    if t["size"] > 255 * 32:
                        continue  # std.crypto.3 traps above 8160 bytes
                    if r == "valid":
                        add("hkdf256", h(t["ikm"]), h(t["salt"]), h(t["info"]), str(t["size"]), h(t["okm"]))
                elif name == "hmac_sha256_test":
                    if t["tag"]:
                        add("hmac256", h(t["key"]), h(t["msg"]), h(t["tag"]), "1" if r == "valid" else "0")
    head = [
        "# Project Wycheproof vectors (C2SP/wycheproof, Apache-2.0) for the public std.crypto API, converted by",
        "# tools/native-tls/wycheproof_api.py (ADR 1118). Do not edit by hand: rerun tools/native-tls/wycheproof-api.sh.",
    ] + sums + [
        "# ed25519_verify PK MSG SIG VERDICT; p256_verify PUB65 MSG SIG64 VERDICT; x25519 SECRET PEER none|=SHARED;",
        "# chacha_open, aesgcm_open KEY AAD NONCE||CT||TAG none|=PLAIN; hkdf256 IKM SALT INFO LEN OKM;",
        "# hmac256 KEY MSG TAG-PREFIX VERDICT (the case counts, as the hex of the HMAC starts with the tag, or not)",
    ]
    open(out, "w").write("\n".join(head + lines) + "\n")
    print("wrote %d cases: %s" % (len(lines), counts))


main()
