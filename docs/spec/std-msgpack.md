# Standard library: `std.msgpack`

Chapter key: `std`

> **Status:** Drafted. Implemented in AIL (`std/msgpack/msgpack.ail`, ADR 1248).

MessagePack encoded from and decoded into the `Json` value ([std-json](std-json.md)), so one set of
accessors serves every format. Every function is pure (`eff.infer.1`). `Json` has one number type, a
`float`, and no binary or extension values; the rules below say how each MessagePack value meets it.

## The module

**std.msgpack.1** — `use std.msgpack` provides `encode(j Json) bytes ! MsgpackError`,
`decode(data bytes) Json ! MsgpackError` and the types `MsgpackError` and `MsgpackKind`. *(ADR 1248)*

**std.msgpack.2** — `encode` writes a `Num` that is a whole number in the `int` range as an integer
in its smallest form (positive fixint, negative fixint, `uint 8-64` or `int 8-64`; a non-negative
number uses the `uint` family), and any other `Num` as a 64-bit float (`0xcb`): a fraction, a whole
number past the `int` range, `-0.0`, an infinity or NaN. *(ADR 1248)*

```ail rules=std.msgpack.1,std.msgpack.2
use std.msgpack

fn main() !
    for n in [0.0, 127.0, 128.0, -32.0, -33.0, 70000.0, 1.5]
        print(msgpack.encode(Json.Num(n))?.hex())   # 00, 7f, cc80, e0, d0df, ce00011170, cb3ff8000000000000
```

**std.msgpack.3** — `Null`, `Bool`, `Str`, `Arr` and `Obj` are `nil`, `false`/`true`, `str`, `array`
and `map`, in the smallest form: fixstr, fixarray and fixmap for 31 bytes or 15 items, then the 8-,
16- and 32-bit forms. A string, array or object of 2^32 or more items fails with kind
`Unsupported`. `encode` never writes `bin` or `ext`. *(ADR 1248)*

```ail rules=std.msgpack.3
use std.msgpack

fn main() !
    doc := parse_json("\{\"a\": 1, \"b\": [2, 3]}")?
    print(msgpack.encode(doc)?.hex())   # 82a16101a162920203
```

**std.msgpack.4** — `decode` reads the first value of `data` and returns it. Bytes after it fail
with kind `Trailing`, so `data` holds exactly one value. *(ADR 1248)*

```ail rules=std.msgpack.4
use std.msgpack

fn main()
    _ := msgpack.decode(b"\xc0\x00") else err
        print(err.kind.name())   # Trailing
        return
```

**std.msgpack.5** — Integers (`uint` and `int`, up to 64 bits) and floats (32 and 64 bit) decode to
`Num`; a whole number past 2^53 rounds to the nearest `float`. A `str` decodes to `Str`; one that is
not UTF-8 fails with kind `Malformed`. A `bin` decodes to a `Str` of its standard base64 text, because
`Json` has no bytes. *(ADR 1248)*

```ail rules=std.msgpack.5
use std.msgpack

fn main() !
    print(msgpack.decode(bytes.from_hex("cd0100") else b"")?.to_text())     # 256
    print(msgpack.decode(bytes.from_hex("c4020102") else b"")?.to_text())   # "AQI="
```

**std.msgpack.6** — The byte `0xc1`, which MessagePack never uses, fails with kind `Malformed`. An `ext`
or `fixext` value fails with kind `Unsupported` (so the timestamp extension is refused, not guessed), and so
does a map key that is not a `str`. *(ADR 1248)*

```ail rules=std.msgpack.6
use std.msgpack

fn main()
    _ := msgpack.decode(b"\xd4\x00\x01") else err
        print(err.kind.name())   # Unsupported
        return
```

**std.msgpack.7** — Arrays and maps nested deeper than 512 fail with kind `TooDeep`, in `encode` and
in `decode`. A count that is larger than the bytes left fails with kind `Truncated` before any
allocation. *(ADR 1248)*

**std.msgpack.8** — Members of an `Obj` are written and read in order, and duplicate keys are kept.
*(ADR 1248)*

**std.msgpack.9** — `MsgpackError` has `kind MsgpackKind`, `at int` (the 0-based byte offset where the
reader stopped) and `why str`; its text is `why (byte at)`. `MsgpackKind` is `Truncated` (the data ends
inside a value), `Malformed`, `Unsupported`, `Trailing` and `TooDeep`. *(ADR 1248)*

```ail rules=std.msgpack.7,std.msgpack.8,std.msgpack.9
use std.msgpack

fn main()
    _ := msgpack.decode(b"\xcd\x01") else err
        print("{err.kind.name()}: {err}")   # Truncated: the data ends inside a value (byte 1)
        return
```

## Rejected alternatives

- Failing on `bin`: it is common MessagePack, so a reader would refuse real data. A `Str` of base64
  keeps the bytes; the program decodes it with `bytes.from_base64`. A bytes variant on `Json` would
  change every `match` over `Json`.
- Reading the timestamp extension (`-1`) as a number or text: the three layouts differ in unit and
  range, and a guess would be silently wrong for some. The program that needs it decodes the bytes.
- Writing `float 32` when exact: saves bytes but makes the output depend on the value's bits.
- Writing `Num` as a float always: `3` would read back as `3.0` elsewhere, which `to_text`
  (json.print.1) already avoids.
- Writing `bin` for a `Str` that is not text: `Str` is always valid text.
