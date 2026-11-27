# Standard library: `std.base58`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1242).

Base58 text for `bytes`, in the Bitcoin alphabet: the digits and letters
without `0`, `O`, `I` and `l`. Every function is pure (`eff.infer.1`). The
module is written in AIL and is in scope without `use` (`std.use.4`);
`base58.encode(...)` names it. The cost is quadratic in the length, so use it
for keys, ids and addresses, not for large files.

**std.base58.1** — `encode(data bytes) str` reads `data` as one big-endian
number and writes it in base 58, `1` being the digit 0. Each leading zero byte
is written as one `1`. Empty bytes give `""`. *(ADR 1242)*

```ail tool=run,FILE expect=Cn8eVZg rules=std.base58.1
fn main()
    print(base58.encode(b"hello"))
```

**std.base58.2** — `decode(text str) bytes ! Base58Error` is the inverse: each
leading `1` is a zero byte, and the rest is the number in base 58. The empty
text is no bytes. Case matters. *(ADR 1242)*

```ail tool=run,FILE expect=000001 rules=std.base58.2
fn main() !
    print(base58.decode("112")?.hex())
```

**std.base58.3** — `decode(encode(b)) == b` for every `b`, leading zero bytes
included. *(ADR 1242)*

```ail tool=run,FILE expect=true rules=std.base58.3
fn main() !
    b := b"\x00\x00\xffabc"
    print(base58.decode(base58.encode(b))? == b)
```

**std.base58.4** — `Base58Error` is an error with `kind Base58Kind` and `why
str`; its text is `why`. The only kind is `BadChar`, a character outside the
alphabet (`0`, `O`, `I`, `l`, a space, any other symbol). *(ADR 1242)*

```ail tool=run,FILE expect=BadChar rules=std.base58.4
fn main()
    b := base58.decode("0OIl") else err
        print(err.kind.name())
        return
    print(b.hex())
```

## Rejected alternatives

- **Base58Check (a double SHA-256 checksum).** `std.hash` takes text, not bytes, so the checksum needs a bytes digest
  first; left for a follow-up.
- **The Flickr and Ripple alphabets.** One alphabet keeps the text unambiguous; the Bitcoin one is the common one.
