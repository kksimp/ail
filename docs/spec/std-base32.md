# Standard library: `std.base32`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR-TBD).

Base32 text for `bytes`: the RFC 4648 alphabet, the RFC 4648 section 7
"extended hex" alphabet and Crockford's. Every function is pure (`eff.infer.1`).
The module is written in AIL and is in scope without `use` (`std.use.4`);
`base32.encode(...)` names it. Standard Base64 is `b.base64()` (`type.bytes.12`).

**std.base32.1** — `encode(data bytes) str` is the RFC 4648 text of `data`: the
letters `A` to `Z` and the digits `2` to `7`, upper case, padded with `=` to a
multiple of 8 symbols. Empty bytes give `""`. *(ADR-TBD)*

```ail tool=run,FILE expect=MZXW6YTBOI====== rules=std.base32.1
fn main()
    print(base32.encode(b"foobar"))
```

**std.base32.2** — `decode(text str) bytes ! Base32Error` reads RFC 4648 text.
Upper and lower case are the same. The `=` padding may be left off, but when
present it must be complete: the text, padding included, is a multiple of 8
symbols and has at most 6 `=`. The symbols before the padding must be a length
some byte string encodes to (2, 4, 5 or 7 symbols in the last group of 8), and
the spare bits of the last symbol must be zero, so every byte string has one
accepted text per form. The empty text is no bytes. *(ADR-TBD)*

```ail tool=run,FILE expect=foobar rules=std.base32.2
fn main() !
    print(base32.decode("mzxw6ytboi")?.text())
```

**std.base32.3** — `encode_hex(data bytes) str` and `decode_hex(text str)
bytes ! Base32Error` use the "extended hex" alphabet (`0` to `9`, then `A` to
`V`), padded like `encode` and read like `decode`. The text sorts in the same
order as the bytes. *(ADR-TBD)*

```ail tool=run,FILE expect=CPNMUOJ1E8====== rules=std.base32.3
fn main()
    print(base32.encode_hex(b"foobar"))
```

**std.base32.4** — `encode_crockford(data bytes) str` is Crockford's Base32:
the digits and the upper-case letters without `I`, `L`, `O` and `U`, and no
padding. *(ADR-TBD)*

```ail tool=run,FILE expect=CSQPYRK1E8 rules=std.base32.4
fn main()
    print(base32.encode_crockford(b"foobar"))
```

**std.base32.5** — `decode_crockford(text str) bytes ! Base32Error` ignores
case and `-`, reads `O` as `0` and `I` and `L` as `1`, and refuses `=`. The
symbol count must be 2, 4, 5 or 7 modulo 8 (or 0), and the spare bits zero.
*(ADR-TBD)*

```ail tool=run,FILE expect=foobar rules=std.base32.5
fn main() !
    print(base32.decode_crockford("CSQ-PYR-KIE8")?.text())
```

**std.base32.6** — `Base32Error` is an error with `kind Base32Kind` and `why
str`; its text is `why`. The kinds are `BadChar` (a character outside the
alphabet), `BadLength` (a symbol count no byte string encodes to) and
`BadPadding` (`=` that does not complete a group of 8, more than 6 `=`, or
nonzero spare bits). Match on `err.kind`, never on `why`. *(ADR-TBD)*

```ail tool=run,FILE expect=BadChar rules=std.base32.6
fn main()
    b := base32.decode("MZXW1===") else err
        print(err.kind.name())
        return
    print(b.hex())
```

## Rejected alternatives

- **Methods on `bytes` (`b.base32()`).** Costs a checker, lowering and card change per name in shared files; the
  module is pure AIL and collides with nothing.
- **One `encode(data, alphabet)` with an enum.** Three alphabets with different padding rules read better as named
  functions: the name says which text comes out.
