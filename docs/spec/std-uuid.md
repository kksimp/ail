# Standard library: `std.uuid`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1242).

UUIDs (RFC 9562): `v4()` is random, `v7()` is time-ordered for database keys.
`v4` and `v7` read the random source (effect `random`), and `v7` the clock
(`time`), so they work in `ail test` and follow `test.clock` (`test.stub.4`).
The rest is pure. The module is written in AIL and is in scope without `use`
(`std.use.4`); `uuid.v4()` names it.

**std.uuid.1** — A `Uuid` is 16 bytes. `==` compares them. `Uuid` implements
`Ord` by the bytes, so `a < b`, `ids.sorted()` and `ids.max()` work, and ids of
version 7 sort by their time. It implements `ToStr`: `print(id)` and `"{id}"`
show the canonical text (`std.uuid.8`). *(ADR 1242)*

```ail tool=run,FILE expect=true rules=std.uuid.1
fn main() !
    a := uuid.v7_at(1000)?
    b := uuid.v7_at(2000)?
    print(a < b)
```

**std.uuid.2** — `zero()` (the RFC 9562 nil UUID; `nil` is a name the checker rejects) is the UUID of 16 zero bytes, and `u.is_zero()` says
whether `u` is it. *(ADR 1242)*

```ail tool=run,FILE expect=00000000-0000-0000-0000-000000000000 rules=std.uuid.2
fn main()
    print(uuid.zero())
```

**std.uuid.3** — `v4() Uuid` is a random UUID: 122 random bits from
`crypto.random_bytes`, with the version nibble 4 and the variant bits `10`.
*(ADR 1242)*

```ail tool=run,FILE expect=4,36 rules=std.uuid.3
fn main()
    id := uuid.v4()
    print("{id.version()},{id.to_str().len()}")
```

**std.uuid.4** — `v7() Uuid` is `v7_at(time.now_ms())`: the first 48 bits are
the time in milliseconds since 1970 (big-endian), then the version nibble 7, 74
random bits and the variant bits `10`. Ids made in different milliseconds sort
in creation order; two made in the same millisecond are not ordered.
`v7_at(unix_ms int) Uuid ! UuidError` takes the time as an argument and fails
with kind `BadTime` when it is negative or does not fit in 48 bits. *(ADR 1242)*

```ail tool=run,FILE expect=7,1700000000000 rules=std.uuid.4
fn main() !
    id := uuid.v7_at(1700000000000)?
    print("{id.version()},{id.unix_ms() else -1}")
```

**std.uuid.5** — `from_bytes(b bytes) Uuid ! UuidError` makes a UUID of exactly
16 bytes (kind `BadLength` otherwise). `u.to_bytes()` gives them back.
*(ADR 1242)*

```ail tool=run,FILE expect=16 rules=std.uuid.5
fn main() !
    id := uuid.from_bytes(bytes.zeros(16))?
    print(id.to_bytes().len())
```

**std.uuid.6** — `parse(text str) Uuid ! UuidError` reads `8-4-4-4-12`
hexadecimal digits with hyphens (36 characters) or 32 digits without. Upper and
lower case are the same. Any version and variant is accepted. A braced, URN or
space-padded form is an error. `is_valid(text) bool` says whether `parse`
succeeds. *(ADR 1242)*

```ail tool=run,FILE expect=123e4567-e89b-42d3-a456-426614174000 rules=std.uuid.6
fn main() !
    print(uuid.parse("123E4567E89B42D3A456426614174000")?)
```

**std.uuid.7** — `u.version() int` is the high nibble of byte 6 (4 for `v4`, 7
for `v7`). `u.to_bytes() bytes` is the 16 bytes. *(ADR 1242)*

```ail tool=run,FILE expect=4 rules=std.uuid.7
fn main() !
    print(uuid.parse("123e4567-e89b-42d3-a456-426614174000")?.version())
```

**std.uuid.8** — The text of a `Uuid` is 36 lower-case characters in the groups
8-4-4-4-12. `u.simple() str` is the 32 digits with no hyphens. *(ADR 1242)*

```ail tool=run,FILE expect=123e4567e89b42d3a456426614174000 rules=std.uuid.8
fn main() !
    print(uuid.parse("123e4567-e89b-42d3-a456-426614174000")?.simple())
```

**std.uuid.9** — `u.unix_ms() int?` is the time in milliseconds since 1970 of
a version 7 UUID, and `none` for any other version. *(ADR 1242)*

```ail tool=run,FILE expect=-1 rules=std.uuid.9
fn main()
    print(uuid.v4().unix_ms() else -1)
```

**std.uuid.10** — `UuidError` is an error with `kind UuidKind` and `why str`;
its text is `why`. The kinds are `BadLength` (not 36 or 32 characters, or not
16 bytes), `BadFormat` (a hyphen missing or out of place), `BadChar` (a
character that is not a hexadecimal digit) and `BadTime` (`v7_at`). Match on
`err.kind`, never on `why`. *(ADR 1242)*

```ail tool=run,FILE expect=BadChar rules=std.uuid.10
fn main()
    u := uuid.parse("123e4567-e89b-42d3-a456-42661417400g") else err
        print(err.kind.name())
        return
    print(u)
```

## Rejected alternatives

- **UUID as a `str`.** Nothing stops a mistyped id; a `Uuid` is parsed once and compares as 16 bytes.
- **A monotonic counter in `v7`.** RFC 9562 allows one, but it needs state shared between tasks; random low bits keep
  the function pure of program state. Millisecond order is enough for database keys.
- **Versions 1, 3, 5, 6 and 8.** v1 and v6 expose a MAC address or a made-up node; v3 and v5 need a bytes digest, and
  `std.hash` takes text (follow-up for v5).
