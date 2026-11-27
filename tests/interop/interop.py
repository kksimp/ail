#!/usr/bin/env python3
"""AIL std interop checks: each module is compared with a real reference implementation.

Run through run.sh. Environment: BIN (dir of built cli_<module> programs), WORK (scratch dir),
OUT (results dir). Writes OUT/results.tsv: module, case, PASS|FAIL|SKIP, detail.
"""
import base64, hashlib, hmac, io, json, os, random, shutil, subprocess, sys, tarfile, zlib, gzip, struct, time

ROOT = os.environ["ROOT"]; BIN = os.environ["BIN"]; WORK = os.environ["WORK"]; OUT = os.environ["OUT"]
ONLY = set(sys.argv[1:])
rows = []

def rec(mod, case, ok, detail=""):
    status = "PASS" if ok is True else "SKIP" if ok is None else "FAIL"
    rows.append((mod, case, status, detail.replace("\n", " ")[:300]))

def cli(mod, *args, timeout=300):
    exe = os.path.join(BIN, "cli_" + mod)
    if not os.path.exists(exe):
        return None
    p = subprocess.run([exe, *map(str, args)], capture_output=True, timeout=timeout)
    return p.stdout.decode(errors="replace").strip() + (("\nSTDERR " + p.stderr.decode(errors="replace")[:200]) if p.returncode else "")

def have(tool): return shutil.which(tool) is not None

def need(mod):
    log = os.path.join(OUT, "build_%s.log" % mod)
    if not os.path.exists(os.path.join(BIN, "cli_" + mod)) and os.path.exists(log):
        txt = open(log, errors="replace").read()
        if "error[std.use.1]" in txt or ("error[std.use.2]" in txt and "has no function" in txt):
            rec(mod, "build", None, "std module(s) not in this tree"); return False
    if not os.path.exists(os.path.join(BIN, "cli_" + mod)):
        rec(mod, "build", False, "cli_%s did not build (see build log)" % mod); return False
    return True

def wr(path, data):
    with open(path, "wb") as f: f.write(data)

def rd(path):
    with open(path, "rb") as f: return f.read()

def corpus():
    r = random.Random(20261127)
    text = b" ".join(r.choice([b"the", b"quick", b"brown", b"fox", b"jumps", b"over", b"lazy", b"dog", b"alice", b"rabbit", b"\xe2\x82\xac", b"\n"]) for _ in range(40000))
    return {
        "empty": b"", "one": b"x", "text": text, "text_small": text[:1000],
        "random_1k": bytes(r.randrange(256) for _ in range(1024)),
        "random_100k": bytes(r.randrange(256) for _ in range(100000)),
        "zeros_1m": bytes(1 << 20), "runs": b"".join(bytes([r.randrange(256)]) * r.randrange(1, 300) for _ in range(2000)),
        "binary_struct": b"".join(struct.pack("<IHf", r.randrange(1 << 32), r.randrange(65536), r.random()) for _ in range(20000)),
        "alt_70k": bytes(i % 2 for i in range(70000)),
    }

# ---------------------------------------------------------------- compress
def t_compress():
    mod = "compress"
    if not need(mod): return
    d = os.path.join(WORK, mod); os.makedirs(d, exist_ok=True)
    data = corpus()
    kinds = set(os.environ.get("COMPRESS_KINDS", "gzip deflate zstd brotli zstd-dec brotli-dec").split())   # a tree with only some of the codecs
    def run_dec(kind, name, comp, raw, label):
        if kind not in kinds and kind + "-dec" not in kinds: return
        i, o = os.path.join(d, "i.bin"), os.path.join(d, "o.bin")
        wr(i, comp)
        if os.path.exists(o): os.remove(o)
        res = cli(mod, "dec", kind, i, o)
        ok = res == "ok" and os.path.exists(o) and rd(o) == raw
        rec(mod, "%s dec %s %s" % (kind, name, label), ok, "" if ok else "got %r" % (res,))
    def run_enc(kind, name, raw, decompress):
        if kind not in kinds: return
        i, o = os.path.join(d, "i.bin"), os.path.join(d, "o.bin")
        wr(i, raw)
        if os.path.exists(o): os.remove(o)
        res = cli(mod, "enc", kind, i, o)
        try:
            ok = res == "ok" and decompress(rd(o)) == raw
            detail = "" if ok else "got %r" % (res,)
        except Exception as e:
            ok, detail = False, "reference could not read the output: %s" % e
        rec(mod, "%s enc %s" % (kind, name), ok, detail)
    # gzip and deflate: Python zlib/gzip (zlib is the reference implementation)
    for name, raw in data.items():
        for lvl in (0, 1, 6, 9):
            run_dec("gzip", name, gzip.compress(raw, lvl, mtime=0), raw, "level %d" % lvl)
            c = zlib.compressobj(lvl, zlib.DEFLATED, -15); run_dec("deflate", name, c.compress(raw) + c.flush(), raw, "level %d" % lvl)
        run_enc("gzip", name, raw, gzip.decompress)
        run_enc("deflate", name, raw, lambda b: zlib.decompress(b, -15))
    # zstd and brotli: the CLIs, else the Python modules
    zstd_cli = have("zstd")
    try:
        import zstandard
    except ImportError:
        zstandard = None
    def zcomp(raw, level, extra=()):
        if zstd_cli:
            return subprocess.run(["zstd", "-q", "-c", "-%d" % level if level > 0 else "-1", *extra], input=raw, capture_output=True).stdout
        return zstandard.ZstdCompressor(level=level).compress(raw)
    def zdecomp(b):
        if zstd_cli:
            p = subprocess.run(["zstd", "-q", "-d", "-c"], input=b, capture_output=True)
            if p.returncode: raise RuntimeError(p.stderr.decode()[:100])
            return p.stdout
        return zstandard.ZstdDecompressor().decompress(b, max_output_size=1 << 28)
    if zstd_cli or zstandard:
        for name, raw in data.items():
            for lvl in (1, 3, 9, 19):
                run_dec("zstd", name, zcomp(raw, lvl), raw, "level %d" % lvl)
            if zstd_cli:
                run_dec("zstd", name, zcomp(raw, 3, ["--no-check"]), raw, "no checksum")
                run_dec("zstd", name, zcomp(raw, 3, ["--long=27"]), raw, "long window")
                run_dec("zstd", name, zcomp(raw, 19, ["--ultra", "-22"] if False else ["--zstd=strategy=7"]), raw, "btultra2")
            run_enc("zstd", name, raw, zdecomp)
        if zstd_cli:   # concatenated frames and a skippable frame
            run_dec("zstd", "frames", zcomp(b"first ") + zcomp(b"second"), b"first second", "two frames")
    else:
        rec("compress", "zstd", None, "no zstd CLI or python module")
    brotli_cli = have("brotli")
    try:
        import brotli as brotli_py
    except ImportError:
        brotli_py = None
    def bcomp(raw, q, lgwin=22):
        if brotli_cli:
            return subprocess.run(["brotli", "-c", "-q", str(q), "--lgwin=%d" % lgwin], input=raw, capture_output=True).stdout
        return brotli_py.compress(raw, quality=q, lgwin=lgwin)
    def bdecomp(b):
        if brotli_cli:
            p = subprocess.run(["brotli", "-d", "-c"], input=b, capture_output=True)
            if p.returncode: raise RuntimeError(p.stderr.decode()[:100])
            return p.stdout
        return brotli_py.decompress(b)
    if brotli_cli or brotli_py:
        for name, raw in data.items():
            for q in (0, 1, 5, 9, 11):
                run_dec("brotli", name, bcomp(raw, q), raw, "quality %d" % q)
            for w in (10, 16, 24):
                run_dec("brotli", name, bcomp(raw, 5, w), raw, "lgwin %d" % w)
            run_enc("brotli", name, raw, bdecomp)
    else:
        rec("compress", "brotli", None, "no brotli CLI or python module")
    # damaged streams must fail cleanly, never hang or crash
    for kind, comp in (("gzip", gzip.compress(data["text_small"])), ("deflate", zlib.compress(data["text_small"])[2:-4])):
        for cut in (1, len(comp) // 2, len(comp) - 1):
            i, o = os.path.join(d, "i.bin"), os.path.join(d, "o.bin")
            wr(i, comp[:cut]); res = cli(mod, "dec", kind, i, o)
            rec(mod, "%s truncated at %d" % (kind, cut), bool(res) and res.startswith("err"), res or "")

# ---------------------------------------------------------------- image
def t_image():
    mod = "image"
    if not need(mod): return
    try:
        from PIL import Image
    except ImportError:
        rec(mod, "pillow", None, "PIL not installed"); return
    d = os.path.join(WORK, mod); os.makedirs(d, exist_ok=True)
    r = random.Random(7)
    def rnd(mode, w, h):
        n = {"1": 0, "L": 1, "LA": 2, "RGB": 3, "RGBA": 4, "P": 1}[mode]
        im = Image.new(mode, (w, h))
        if mode == "1": im.putdata([r.randrange(2) * 255 for _ in range(w * h)])
        elif mode == "P":
            im.putpalette([r.randrange(256) for _ in range(768)]); im.putdata([r.randrange(256) for _ in range(w * h)])
        else: im.putdata([tuple(r.randrange(256) for _ in range(n)) if n > 1 else r.randrange(256) for _ in range(w * h)])
        return im
    cases = []
    for mode in ("1", "L", "LA", "RGB", "RGBA", "P"):
        for (w, h) in ((1, 1), (7, 5), (64, 48), (257, 3), (300, 200)):
            cases.append((mode, w, h, {}))
    cases.append(("RGBA", 50, 50, {"optimize": True})); cases.append(("RGB", 50, 50, {"compress_level": 0}))
    cases.append(("RGB", 50, 50, {"compress_level": 9})); cases.append(("P", 40, 40, {"transparency": 3}))
    cases.append(("RGB", 40, 40, {"transparency": (1, 2, 3)})); cases.append(("L", 40, 40, {"transparency": 9}))
    for mode, w, h, kw in cases:
        im = rnd(mode, w, h); buf = io.BytesIO(); im.save(buf, "PNG", **kw)
        i, o = os.path.join(d, "i.png"), os.path.join(d, "o.rgba")
        wr(i, buf.getvalue())
        if os.path.exists(o): os.remove(o)
        res = cli(mod, "dec", i, o)
        want = Image.open(io.BytesIO(buf.getvalue())).convert("RGBA").tobytes()
        ok = bool(res) and res.startswith("ok %d %d" % (w, h)) and os.path.exists(o) and rd(o) == want
        rec(mod, "png dec %s %dx%d %s" % (mode, w, h, sorted(kw)), ok, "" if ok else "got %r" % (res,))
    try:   # 16-bit and Adam7 interlace through pypng
        import png as pypng
        for planes, alpha, greyscale in ((1, False, True), (3, False, False), (4, True, False)):
            for interlace in (False, True):
                w, h = 13, 11
                rows16 = [[r.randrange(65536) for _ in range(w * planes)] for _ in range(h)]
                buf = io.BytesIO(); pypng.Writer(w, h, bitdepth=16, greyscale=greyscale, alpha=alpha, interlace=interlace).write(buf, rows16)
                i, o = os.path.join(d, "i.png"), os.path.join(d, "o.rgba"); wr(i, buf.getvalue())
                res = cli(mod, "dec", i, o)
                want = bytearray()
                for row in rows16:
                    for x in range(w):
                        px = [v >> 8 for v in row[x * planes:(x + 1) * planes]]
                        if planes == 1: px = [px[0]] * 3 + [255]
                        elif planes == 3: px = px + [255]
                        want += bytes(px)
                ok = bool(res) and res.startswith("ok") and rd(o) == bytes(want)
                rec(mod, "png dec 16-bit planes=%d interlace=%s (high byte)" % (planes, interlace), ok, "" if ok else "got %r" % (res,))
    except ImportError:
        rec(mod, "png dec 16-bit and interlaced", None, "pypng not installed")
    for (w, h) in ((1, 1), (17, 9), (200, 150)):   # encode: PIL reads what AIL wrote
        raw = bytes(r.randrange(256) for _ in range(w * h * 4))
        i, o = os.path.join(d, "i.rgba"), os.path.join(d, "o.png"); wr(i, raw)
        res = cli(mod, "enc", w, h, i, o)
        try:
            got = Image.open(o); got.load()
            ok = res == "ok" and got.size == (w, h) and got.convert("RGBA").tobytes() == raw
            detail = "" if ok else "got %r mode %s" % (res, got.mode)
        except Exception as e:
            ok, detail = False, "PIL could not read the output: %s" % e
        rec(mod, "png enc %dx%d" % (w, h), ok, detail)
    for name, blob in (("bad crc", None), ("truncated", None)):
        im = rnd("RGB", 20, 20); buf = io.BytesIO(); im.save(buf, "PNG"); b = bytearray(buf.getvalue())
        if name == "bad crc": b[-20] ^= 1
        else: b = b[:len(b) // 2]
        i, o = os.path.join(d, "i.png"), os.path.join(d, "o.rgba"); wr(i, bytes(b))
        res = cli(mod, "dec", i, o); rec(mod, "png " + name, bool(res) and res.startswith("err"), res or "")

# ---------------------------------------------------------------- crypto and hash
def t_crypto():
    mod = "crypto"
    if not need(mod): return
    r = random.Random(11)
    msgs = [b"", b"abc", b"a" * 55, b"a" * 56, b"a" * 64, b"a" * 111, b"a" * 112, b"a" * 128, bytes(r.randrange(256) for _ in range(1000)), bytes(r.randrange(256) for _ in range(50000))]
    algos = {"sha256": "sha256", "sha384": "sha384", "sha512": "sha512", "sha3_256": "sha3_256", "sha3_512": "sha3_512", "sha1": "sha1", "md5": "md5"}
    for n, hn in algos.items():
        ok = all(cli(mod, "hash", n, m.hex() or "") == hashlib.new(hn, m).hexdigest() for m in msgs)
        rec(mod, "hash " + n, ok)
    for n in ("sha256", "sha384", "sha512"):
        bad = []
        for key in (b"", b"k", b"k" * 64, b"k" * 128, b"k" * 129, bytes(r.randrange(256) for _ in range(200))):
            for m in (b"", b"msg", bytes(r.randrange(256) for _ in range(500))):
                if cli(mod, "hmac", n, key.hex(), m.hex()) != hmac.new(key, m, n).hexdigest(): bad.append((len(key), len(m)))
        rec(mod, "hmac " + n, not bad, str(bad[:3]))
    rec(mod, "crc32", all(cli(mod, "crc32", m.hex()) == str(zlib.crc32(m)) for m in msgs))
    bad = []
    for _ in range(25):
        ikm, salt, info = (bytes(r.randrange(256) for _ in range(r.randrange(0, 80))) for _ in range(3)); n = r.randrange(0, 200)
        try:
            from cryptography.hazmat.primitives.kdf.hkdf import HKDF
            from cryptography.hazmat.primitives import hashes
            want = HKDF(hashes.SHA256(), n, salt or None, info).derive(ikm).hex() if n else ""
        except ImportError:
            rec(mod, "hkdf", None, "cryptography not installed"); break
        if cli(mod, "hkdf", ikm.hex() or "", salt.hex() or "", info.hex() or "", n) != want: bad.append(n)
    else: rec(mod, "hkdf-sha256", not bad, str(bad[:3]))
    bad = []
    for it in (1, 2, 1000, 4096):
        for pw, salt, n in ((b"password", b"salt", 32), (b"", b"s", 20), (b"p" * 100, b"s" * 40, 64)):
            if cli(mod, "pbkdf2", pw.hex(), salt.hex(), it, n) != hashlib.pbkdf2_hmac("sha256", pw, salt, it, n).hex(): bad.append((it, len(pw), n))
    rec(mod, "pbkdf2-sha256", not bad, str(bad[:3]))
    try:
        from cryptography.hazmat.primitives.ciphers.aead import ChaCha20Poly1305, AESGCM
    except ImportError:
        rec(mod, "aead", None, "cryptography not installed"); return
    for alg, C in (("chacha", ChaCha20Poly1305), ("gcm", AESGCM)):
        bad_open, bad_seal, bad_tamper = [], [], []
        for n in (0, 1, 15, 16, 17, 63, 64, 65, 1000, 60000):
            key = os.urandom(32); aad = os.urandom(r.randrange(0, 40)); pt = os.urandom(n)
            nonce = os.urandom(12); blob = nonce + C(key).encrypt(nonce, pt, aad or None)
            if cli(mod, "open", alg, key.hex(), aad.hex() or "", blob.hex()) != pt.hex(): bad_open.append(n)
            s = bytes.fromhex(cli(mod, "seal", alg, key.hex(), aad.hex() or "", pt.hex() or "") or "")
            try: ok = len(s) == n + 28 and C(key).decrypt(s[:12], s[12:], aad or None) == pt
            except Exception: ok = False
            if not ok: bad_seal.append(n)
            t = bytearray(blob); t[len(t) // 2] ^= 1
            if cli(mod, "open", alg, key.hex(), aad.hex() or "", bytes(t).hex()) != "none": bad_tamper.append(n)
        rec(mod, alg + " open reference output", not bad_open, str(bad_open))
        rec(mod, alg + " seal read by reference", not bad_seal, str(bad_seal))
        rec(mod, alg + " tampered refused", not bad_tamper, str(bad_tamper))

# ---------------------------------------------------------------- encodings
def t_encoding():
    mod = "encoding"
    if not need(mod): return
    r = random.Random(3)
    blobs = [b"", b"f", b"fo", b"foo", b"foob", b"fooba", b"foobar", b"\x00", b"\x00\x00ab", bytes(range(256))] + [bytes(r.randrange(256) for _ in range(r.randrange(1, 60))) for _ in range(40)]
    rec(mod, "base32 rfc4648", all(cli(mod, "b32", "enc", b.hex() or "x") == base64.b32encode(b).decode() or not b for b in blobs if b))
    rec(mod, "base32 decode", all(cli(mod, "b32", "dec", base64.b32encode(b).decode()) == b.hex() for b in blobs if b))
    rec(mod, "base32hex", all(cli(mod, "b32", "hexenc", b.hex()) == base64.b32hexencode(b).decode() for b in blobs if b))
    try:
        import base58 as b58
        rec(mod, "base58 encode", all(cli(mod, "b58", "enc", b.hex()) == b58.b58encode(b).decode() for b in blobs if b))
        rec(mod, "base58 decode", all(cli(mod, "b58", "dec", b58.b58encode(b).decode()) == b.hex() for b in blobs if b))
    except ImportError:
        rec(mod, "base58", None, "python base58 not installed")
    import uuid as pyuuid
    bad = [str(u) for u in (pyuuid.uuid4() for _ in range(30)) if cli(mod, "uuid", "parse", str(u)) != "%s 4" % u.hex]
    rec(mod, "uuid parse v4", not bad, str(bad[:2]))
    bad = [str(u) for u in (pyuuid.uuid1() for _ in range(5)) if cli(mod, "uuid", "parse", str(u)) != "%s 1" % u.hex]
    rec(mod, "uuid parse v1", not bad, str(bad[:2]))
    out = cli(mod, "uuid", "v7at", 1700000000000) or ""
    p = out.split()
    ok = len(p) == 3 and pyuuid.UUID(p[0]).version == 7 and int(p[0][:12], 16) == 1700000000000 and p[2] == "1700000000000"
    rec(mod, "uuid v7 layout (RFC 9562)", ok, out)

# ---------------------------------------------------------------- jwt
def t_jwt():
    mod = "jwt"
    if not need(mod): return
    try:
        import jwt as pyjwt
    except ImportError:
        rec(mod, "pyjwt", None, "PyJWT not installed"); return
    key = b"your-256-bit-secret"; claims = {"sub": "1234567890", "name": "John Doe", "iat": 1516239022}
    for alg in ("HS256", "HS384", "HS512"):
        tok = cli(mod, "sign", alg, key.hex(), json.dumps(claims)) or ""
        try:
            got = pyjwt.decode(tok, key, algorithms=[alg], options={"verify_iat": False}); ok = got == claims
        except Exception as e: ok = False; got = str(e)
        rec(mod, alg + " sign read by PyJWT", ok, "" if ok else str(got))
        ref = pyjwt.encode(claims, key, algorithm=alg)
        res = cli(mod, "verify", alg, key.hex(), ref, 1600000000) or ""
        try: ok = json.loads(res) == claims
        except Exception: ok = False
        rec(mod, alg + " PyJWT token verified", ok, "" if ok else res)
        res = cli(mod, "verify", alg, b"wrong".hex(), ref, 1600000000) or ""
        rec(mod, alg + " wrong key refused", res.startswith("err"), res)
        rec(mod, alg + " alg confusion refused", (cli(mod, "verify", "HS256" if alg != "HS256" else "HS512", key.hex(), ref, 1600000000) or "").startswith("err"))
    exp = pyjwt.encode({"exp": 1000}, key, algorithm="HS256")
    rec(mod, "expired refused", (cli(mod, "verify", "HS256", key.hex(), exp, 2000) or "").startswith("err"))
    none = base64.urlsafe_b64encode(b'{"alg":"none"}').rstrip(b"=").decode() + "." + base64.urlsafe_b64encode(b'{"sub":"x"}').rstrip(b"=").decode() + "."
    rec(mod, "alg none refused", (cli(mod, "verify", "HS256", key.hex(), none, 1) or "").startswith("err"))

# ---------------------------------------------------------------- cbor and msgpack
def t_binfmt():
    mod = "binfmt"
    if not need(mod): return
    values = [None, True, False, 0, 1, -1, 23, 24, 255, 256, 65535, 65536, 4294967295, 4294967296, -24, -25, -256, -257, -65537, 2**53, -2**53,
              0.0, 1.5, -2.25, 1e300, 3.14159, "", "a", "héllo €", "x" * 300, "y" * 50000, [], [1, [2, [3]]], {}, {"a": 1, "b": [True, None]},
              list(range(40)), {str(i): i for i in range(40)}, {"nested": {"k": {"k": [1.5, "z"]}}}]
    for name, enc, dec in (("cbor", "cbor2", None), ("msgpack", "msgpack", None)):
        try:
            lib = __import__(enc)
        except ImportError:
            rec(name, "library", None, enc + " not installed"); continue
        e = (lambda v: lib.dumps(v)) if name == "cbor" else (lambda v: lib.packb(v, use_bin_type=True))
        d = (lambda b: lib.loads(b)) if name == "cbor" else (lambda b: lib.unpackb(b, raw=False, strict_map_key=False))
        bad_dec, bad_enc = [], []
        for v in values:
            got = cli(mod, name, "dec", e(v).hex()) or ""
            try: ok = json.loads(got) == v
            except Exception: ok = False
            if not ok: bad_dec.append((v if len(str(v)) < 40 else str(v)[:40], got[:60]))
            hx = cli(mod, name, "enc", json.dumps(v)) or ""
            try: ok = d(bytes.fromhex(hx)) == v
            except Exception: ok = False
            if not ok: bad_enc.append((v if len(str(v)) < 40 else str(v)[:40], hx[:60]))
        rec(name, "decode reference output", not bad_dec, str(bad_dec[:3]))
        rec(name, "encode read by reference", not bad_enc, str(bad_enc[:3]))
        for hx, label in (("", "empty"), ("ff", "break"), ("9f", "open list")) if name == "cbor" else (("", "empty"), ("c1", "reserved"), ("dd", "truncated array")):
            got = cli(mod, name, "dec", hx) or ""
            rec(name, "malformed refused: " + label, got.startswith("err"), got)

# ---------------------------------------------------------------- tar
def t_tar():
    mod = "tar"
    if not need(mod): return
    d = os.path.join(WORK, mod); shutil.rmtree(d, ignore_errors=True); os.makedirs(d)
    src = os.path.join(d, "src"); os.makedirs(os.path.join(src, "sub/deep")); os.makedirs(os.path.join(src, "empty"))
    r = random.Random(5)
    wr(os.path.join(src, "a.txt"), b"alpha\n"); wr(os.path.join(src, "sub/b.txt"), b"beta\n")
    wr(os.path.join(src, "sub/deep/c.bin"), bytes(r.randrange(256) for _ in range(5000)))
    wr(os.path.join(src, "long_" + "n" * 120 + ".txt"), b"long name\n")   # needs a pax or ustar prefix
    wr(os.path.join(src, "sub/" + "d" * 90 + ".txt"), b"prefix\n")
    wr(os.path.join(src, "uni_é€.txt"), b"unicode\n"); wr(os.path.join(src, "big.bin"), bytes(r.randrange(256) for _ in range(300000)))
    os.chmod(os.path.join(src, "a.txt"), 0o600)
    def tree(root):
        out = {}
        for dp, dn, fn in os.walk(root):
            for n in fn: out[os.path.relpath(os.path.join(dp, n), root)] = hashlib.sha256(rd(os.path.join(dp, n))).hexdigest()
            for n in dn: out[os.path.relpath(os.path.join(dp, n), root) + "/"] = "dir"
        return out
    want = tree(src)
    formats = {"gnu": tarfile.GNU_FORMAT, "pax": tarfile.PAX_FORMAT, "ustar": tarfile.USTAR_FORMAT}
    for fmt, f in formats.items():
        for gz in (False, True):
            a = os.path.join(d, "ref.tar" + (".gz" if gz else ""))
            try:
                with tarfile.open(a, "w:gz" if gz else "w", format=f) as t:
                    for n in sorted(os.listdir(src)):
                        if fmt == "ustar" and len(n.encode()) > 100: continue
                        t.add(os.path.join(src, n), arcname=n)
            except Exception as e:
                rec(mod, "python tarfile %s" % fmt, None, str(e)); continue
            dst = os.path.join(d, "x_" + fmt + str(gz)); shutil.rmtree(dst, ignore_errors=True)
            res = cli(mod, "extract", a, dst)
            got = tree(dst) if os.path.isdir(dst) else {}
            exp = {k: v for k, v in want.items() if not (fmt == "ustar" and len(os.path.basename(k.rstrip("/")).encode()) > 100)}
            if fmt == "ustar": exp = {k: v for k, v in exp.items() if not k.startswith("sub/" + "d" * 90)} if False else exp
            ok = res == "ok" and got == exp
            rec(mod, "extract %s%s read from python tarfile" % (fmt, " gz" if gz else ""), ok, "" if ok else "got %r diff %s" % (res, sorted(set(exp) ^ set(got))[:4]))
    if have("tar"):
        for gz in (False, True):
            a = os.path.join(d, "gnu.tar" + (".gz" if gz else ""))
            subprocess.run(["tar", "-C", src, "-c" + ("z" if gz else "") + "f", a, "."], check=True)
            dst = os.path.join(d, "gx" + str(gz)); shutil.rmtree(dst, ignore_errors=True)
            res = cli(mod, "extract", a, dst); got = tree(dst) if os.path.isdir(dst) else {}
            rec(mod, "extract tar CLI archive%s" % (" gz" if gz else ""), res == "ok" and got == want, "" if got == want else "got %r diff %s" % (res, sorted(set(want) ^ set(got))[:4]))
    for ext in (".tar", ".tar.gz"):
        a = os.path.join(d, "ail" + ext)
        if os.path.exists(a): os.remove(a)
        res = cli(mod, "create", src, a)
        try:
            with tarfile.open(a) as t:
                names = sorted(m.name.strip("./").rstrip("/") for m in t.getmembers() if m.name not in (".", "./"))
                dst = os.path.join(d, "py" + ext.replace(".", "")); shutil.rmtree(dst, ignore_errors=True); t.extractall(dst)
            ok = res == "ok" and tree(dst) == want
            detail = "" if ok else "got %r diff %s" % (res, sorted(set(want) ^ set(tree(dst)))[:4])
        except Exception as e:
            ok, detail = False, "python tarfile could not read the archive: %s" % e
        rec(mod, "create %s read by python tarfile" % ext, ok, detail)
        if have("tar"):
            p = subprocess.run(["tar", "-tf", a], capture_output=True); rec(mod, "create %s listed by tar CLI" % ext, res == "ok" and p.returncode == 0, p.stderr.decode()[:100])
            dst = os.path.join(d, "cx" + ext.replace(".", "")); os.makedirs(dst, exist_ok=True)
            p = subprocess.run(["tar", "-C", dst, "-xf", a]); rec(mod, "create %s extracted by tar CLI" % ext, p.returncode == 0 and tree(dst) == want)

# ---------------------------------------------------------------- scripts shipped by the lanes
SCRIPTS = {
    "tls": ("tools/native-tls/server-check.sh", "TLS 1.3 server (#1539): openssl s_client suites, HelloRetryRequest, TLS 1.2 refused, RSA/ECDSA, curl ALPN"),
    "static": ("tools/http/static-curl.sh", "static file server (#1562): etag, 304, ranges, precompressed, traversal, with curl"),
    "wsdeflate": ("tools/native-net/ws-deflate-check.sh", "WebSocket permessage-deflate (#1567): Python websockets client and zlib"),
}
def script(mod):
    def run():
        path, what = SCRIPTS[mod]; full = os.path.join(ROOT, path)
        if not os.path.exists(full):
            rec(mod, path, None, "not in this tree"); return
        try:
            p = subprocess.run(["bash", full], capture_output=True, timeout=900, cwd=ROOT, env=dict(os.environ, PATH=os.path.join(WORK, "venv/bin") + ":" + os.environ["PATH"]))
            out = p.stdout.decode(errors="replace") + p.stderr.decode(errors="replace")
        except subprocess.TimeoutExpired:
            rec(mod, path, False, "timed out after 900s"); return
        lines = out.splitlines()
        for l in lines:
            if l.startswith("ok"): rec(mod, l[2:].strip() or "ok", True)
            elif l.startswith("FAIL") or l.startswith("RED"): rec(mod, l[:80], False, l)
        rec(mod, "script exit (%s)" % what[:40], p.returncode == 0, "" if p.returncode == 0 else " | ".join(lines[-6:]))
    return run

MODULES = {"compress": t_compress, "image": t_image, "crypto": t_crypto, "encoding": t_encoding, "jwt": t_jwt, "binfmt": t_binfmt, "tar": t_tar, **{k: script(k) for k in SCRIPTS}}
for name, fn in MODULES.items():
    if ONLY and name not in ONLY: continue
    t0 = time.time()
    try: fn()
    except Exception as e: rec(name, "harness", False, "exception %r" % (e,))
    print("%-10s %.0fs" % (name, time.time() - t0), flush=True)
os.makedirs(OUT, exist_ok=True)
with open(os.path.join(OUT, "results.tsv"), "w") as f:
    for row in rows: f.write("\t".join(row) + "\n")
mods = sorted({r[0] for r in rows})
for m in mods:
    c = {s: sum(1 for r in rows if r[0] == m and r[2] == s) for s in ("PASS", "FAIL", "SKIP")}
    print("%-10s PASS %-4d FAIL %-4d SKIP %d" % (m, c["PASS"], c["FAIL"], c["SKIP"]))
    for r in rows:
        if r[0] == m and r[2] == "FAIL": print("   FAIL %s: %s" % (r[1], r[3]))
sys.exit(1 if any(r[2] == "FAIL" for r in rows) else 0)
