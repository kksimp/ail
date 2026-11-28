#!/usr/bin/env python3
"""Writes Brotli test streams for tools/native-brotli/check.ail (ADR 1241).

    python3 vectors.py OUT_DIR BROTLI_FILE

BROTLI_FILE (tests/data/brotli/alice29.txt.br) gives the text corpus. Uses the Brotli project's reference encoder and decoder (the `brotli` Python
module). Part one compresses varied inputs (text, binary, UTF-8 in several
scripts, dictionary words, long runs) at qualities 0-11, windows 10-24 and
generic and text modes; part two damages small streams (truncation, bit flips,
byte overwrites, a trailing byte) and records whether the reference decoder
accepts each one. Deterministic: fixed seeds.
"""
import hashlib
import os
import random
import sys

import brotli


def want(data):
    return f"{len(data)} {hashlib.sha256(data).hexdigest()}"


def main():
    out = sys.argv[1]
    os.makedirs(out, exist_ok=True)
    corpus = brotli.decompress(open(sys.argv[2], "rb").read())
    r = random.Random(7)
    samples = {
        "corpus": corpus,
        "corpus_small": corpus[:3000],
        "random": bytes(r.getrandbits(8) for _ in range(50000)),
        "zeros": bytes(100000),
        "abc": b"abc" * 20000,
        "utf8": ("Привет мир, 你好世界, नमस्ते दुनिया, مرحبا بالعالم. The Quick Brown Fox! " * 300).encode(),
        "html": b'<html><head><title>T</title></head><body><div class="a">Hello, World.</div><p>the THE for and</p></body></html>\n' * 400,
        "json": b'{"name": "value", "list": [1, 2, 3], "nested": {"a": true, "b": null}}\n' * 500,
        "mixed": bytes(r.choice(b"aaaaabbbcd \n\x00\xff") for _ in range(80000)),
        "one": b"Q",
        "big": (corpus * 8)[:900000],
    }
    small = []
    for name, data in samples.items():
        for q in (0, 1, 2, 4, 5, 9, 11):
            for lgwin in (10, 16, 22, 24):
                for mode in (brotli.MODE_GENERIC, brotli.MODE_TEXT):
                    if name == "big" and q > 5:
                        continue
                    c = brotli.compress(data, quality=q, lgwin=lgwin, mode=mode)
                    stem = f"{out}/{name}_q{q}_w{lgwin}_m{mode}"
                    open(stem + ".br", "wb").write(c)
                    open(stem + ".want", "w").write(want(data))
                    if len(c) < 20000:
                        small.append(c)
    r = random.Random(11)
    for i in range(4000):
        s = bytearray(r.choice(small))
        k = r.randint(0, 3)
        if k == 0 and len(s) > 1:
            s = s[: r.randint(0, len(s) - 1)]
        elif k == 1:
            for _ in range(r.randint(1, 3)):
                p = r.randrange(len(s))
                s[p] ^= 1 << r.randrange(8)
        elif k == 2:
            s[r.randrange(len(s))] = r.getrandbits(8)
        else:
            s += bytes([r.getrandbits(8)])
        try:
            w = want(brotli.decompress(bytes(s)))
        except brotli.error:
            w = "ERR"
        open(f"{out}/damaged{i:04d}.br", "wb").write(s)
        open(f"{out}/damaged{i:04d}.want", "w").write(w)


main()
