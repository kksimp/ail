# Standard library: `std.cbor`

Chapter key: `std`

> **Status:** Drafted. Implemented in AIL (`std/cbor/cbor.ail`, ADR-TBD).

CBOR (RFC 8949) encoded from and decoded into the `Json` value ([std-json](std-json.md)), so one set
of accessors serves every format. Every function is pure (`eff.infer.1`). `Json` has one number type,
a `float`, and no byte strings, tags or non-text keys; the rules below say how each CBOR item meets it.

## The module

**std.cbor.1** — `use std.cbor` provides `encode(j Json) bytes ! CborError`,
`decode(data bytes) Json ! CborError` and the types `CborError` and `CborKind`. *(ADR-TBD)*

**std.cbor.2** — `encode` writes a `Num` that is a whole number in the `int` range as an integer
(major type 0 or 1) in the shortest form, and any other `Num` as a 64-bit float (`0xfb`): a fraction,
a whole number past the `int` range, `-0.0`, an infinity or NaN. `Null`, `Bool`, `Str`, `Arr` and
`Obj` are `null`, `false`/`true`, a text string, an array and a map, each with a definite length in
the shortest form. *(ADR-TBD)*

```ail rules=std.cbor.1,std.cbor.2
use std.cbor

fn main() !
    doc := parse_json("\{\"a\": 1, \"b\": [2.5, \"x\", null]}")?
    data := cbor.encode(doc)?
    print(data.hex())        # a2616101616283fb40040000000000006178f6
    back := cbor.decode(data)?
    print(back.to_text())    # {"a":1,"b":[2.5,"x",null]}
```

**std.cbor.3** — `decode` reads the first value of `data` and returns it. Bytes after it fail with
kind `Trailing`, so `data` holds exactly one value. *(ADR-TBD)*

```ail rules=std.cbor.3
use std.cbor

fn main()
    _ := cbor.decode(b"\x01\x00") else err
        print(err.kind.name())   # Trailing
        return
```

**std.cbor.4** — Integers (major types 0 and 1, up to 64 bits) and floats (16, 32 and 64 bit) decode
to `Num`. A whole number past 2^53 rounds to the nearest `float`. A half-float infinity or NaN
decodes to that `Num`; `to_text` prints it as `null` (json.print.2). *(ADR-TBD)*

```ail rules=std.cbor.4
use std.cbor

fn main() !
    print(cbor.decode(bytes.from_hex("1a000f4240") else b"")?.to_text())   # 1000000
    print(cbor.decode(bytes.from_hex("3903e7") else b"")?.to_text())       # -1000
    print(cbor.decode(bytes.from_hex("f93e00") else b"")?.to_text())       # 1.5
```

**std.cbor.5** — A text string decodes to `Str`; one that is not UTF-8 fails with kind `Malformed`. A
byte string decodes to a `Str` of its standard base64 text, because `Json` has no bytes. A tag is
dropped and its content decoded. `undefined` decodes to `Null`. *(ADR-TBD)*

```ail rules=std.cbor.5
use std.cbor

fn main() !
    print(cbor.decode(bytes.from_hex("4401020304") else b"")?.to_text())   # "AQIDBA=="
    print(cbor.decode(bytes.from_hex("c11a514b67b0") else b"")?.to_text()) # 1363896240
    print(cbor.decode(bytes.from_hex("f7") else b"")?.to_text())           # null
```

**std.cbor.6** — A map key that is not a text string fails with kind `Unsupported`, and so does a
simple value other than `false`, `true`, `null` and `undefined`. *(ADR-TBD)*

```ail rules=std.cbor.6
use std.cbor

fn main()
    _ := cbor.decode(b"\xa1\x01\x02") else err
        print(err.kind.name())   # Unsupported
        return
```

**std.cbor.7** — `decode` reads indefinite-length text strings, byte strings, arrays and maps (the
`0xff` break), with the same result as the definite form. Chunks of a string must be strings of the
same type with definite lengths. *(ADR-TBD)*

```ail rules=std.cbor.7
use std.cbor

fn main() !
    print(cbor.decode(bytes.from_hex("9f018202039f0405ffff") else b"")?.to_text())   # [1,[2,3],[4,5]]
```

**std.cbor.8** — Members of an `Obj` are written and read in order, and duplicate keys are kept.
*(ADR-TBD)*

**std.cbor.9** — Arrays and maps nested deeper than 512 fail with kind `TooDeep`, in `encode` and in
`decode`, so a hostile input cannot overflow the stack. *(ADR-TBD)*

**std.cbor.10** — `CborError` has `kind CborKind`, `at int` (the 0-based byte offset where the reader
stopped) and `why str`; its text is `why (byte at)`. `CborKind` is `Truncated` (the data ends inside a
value), `Malformed` (not well-formed CBOR, or text that is not UTF-8), `Unsupported`, `Trailing` and
`TooDeep`. *(ADR-TBD)*

```ail rules=std.cbor.8,std.cbor.9,std.cbor.10
use std.cbor

fn main()
    _ := cbor.decode(b"\x1a\x00\x00") else err
        print("{err.kind.name()}: {err}")   # Truncated: the data ends inside a value (byte 1)
        return
```

## Rejected alternatives

- Failing on byte strings: COSE, WebAuthn and most CBOR in the wild carry them, so a reader would
  refuse real data. A `Str` of base64 keeps the bytes; the program decodes it with
  `bytes.from_base64`. A `Json` with a bytes variant would change every `match` over `Json`.
- Integer keys as decimal text: hides that the document was not text-keyed, and two keys (`1` and
  `"1"`) would collide. COSE-style maps are refused with a kind that says why.
- Keeping tags (a `{tag, value}` object): changes the shape of the value for the common case, the
  date tags. A program that needs a tag reads the bytes itself.
- Writing the shortest float (16 or 32 bit) when exact: saves bytes, but costs a conversion pass and
  makes the output depend on the value's bits. A 64-bit float is always correct.
- Writing `Num` as a float always: `3` would read back as `3.0` in other languages, which the same
  rule for `to_text` (json.print.1) already avoids.
