# Standard library: `std.jwt`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1255).
> **Unaudited:** the HMAC and signature code underneath is in-house and has had
> no outside review. Do not use it to guard anything whose breach is costly.

JSON Web Tokens in the compact JWS form (RFC 7519, RFC 7515): `HEADER.PAYLOAD.SIGNATURE`,
each part base64url without padding. HMAC (`HS256`, `HS384`, `HS512`) is built in; other
algorithms (`EdDSA`, `ES256`) plug in through `sign_with` and `verify_with` over the
`std.crypto` signing calls. Nothing here encrypts: a token's contents are readable by anyone.
The module is written in AIL and is in scope without `use` (`std.use.4`); `jwt.sign(...)` names
it. `sign`, `verify` and `verify_with` read the clock only when `Options.now` is 0.

## The module

**std.jwt.1** — `std.jwt` provides `sign`, `sign_with`, `verify`, `verify_with`,
`decode_unverified` and `options`, and the types `Alg`, `Token`, `Options`, `JwtError` and
`JwtKind`. Details: [below](#more-on-stdjwt1). *(ADR 1255)*

```ail tool=run,FILE expect=HS256 rules=std.jwt.1
fn main() !
    tok := jwt.sign(parse_json("\{\"sub\": \"a\"}")?, b"key", jwt.Alg.HS256)?
    got := jwt.verify(tok, b"key", [jwt.Alg.HS256], jwt.options())?
    print(got.alg)
```

**std.jwt.2** — `enum Alg` is `HS256`, `HS384` or `HS512`: HMAC with SHA-256, SHA-384 or
SHA-512 (RFC 7518 section 3.2). *(ADR 1255)*

**std.jwt.3** — `sign(claims Json, key bytes, alg Alg) str ! JwtError` returns the token. Details: [below](#more-on-stdjwt3). *(ADR 1255)*

```ail tool=run,FILE expect=3 rules=std.jwt.3
fn main() !
    tok := jwt.sign(parse_json("\{\"n\": 1}")?, b"key", jwt.Alg.HS512)?
    print(tok.split(".").len())
```

**std.jwt.4** — `alg` of `none` (any case) is never accepted by `verify`, `verify_with` or
signed by `sign_with`: it fails with `UnsupportedAlg`, whatever the caller's lists hold. Details: [below](#more-on-stdjwt4). *(ADR 1255)*

**std.jwt.5** — The caller, not the token, picks the algorithm. Details: [below](#more-on-stdjwt5). *(ADR 1255)*

```ail tool=run,FILE expect=AlgNotAllowed rules=std.jwt.5
fn main() !
    tok := jwt.sign(parse_json("\{\}")?, b"key", jwt.Alg.HS256)?
    _ := jwt.verify(tok, b"key", [jwt.Alg.HS512], jwt.options()) else err
        print(err.kind.name())
        return
```

**std.jwt.6** — Algorithms the module does not build in go through two hooks. Details: [below](#more-on-stdjwt6). *(ADR 1255)*

```ail tool=run,FILE expect=ok rules=std.jwt.6
fn signer(input bytes) bytes
    b"ok"

fn checker(input bytes, sig bytes) bool
    sig == b"ok"

fn main() !
    tok := jwt.sign_with(parse_json("\{\}")?, "EdDSA", signer)?
    t := jwt.verify_with(tok, "EdDSA", checker, jwt.options())?
    print("ok")
```

**std.jwt.7** — `decode_unverified(token) Token ! JwtError` reads the header and claims with
no check of the signature or the claims. Details: [below](#more-on-stdjwt7). *(ADR 1255)*

```ail tool=run,FILE expect=Malformed rules=std.jwt.7
fn main()
    _ := jwt.decode_unverified("abc.def") else err
        print(err.kind.name())
        return
```

**std.jwt.8** — `options() Options` is the defaults; `Options` has `leeway int` (seconds,
default 0), `issuer str?`, `audience str?` (both `none`), `now int` (Unix seconds; 0 reads the
clock) and `require_exp bool` (false). Details: [below](#more-on-stdjwt8). *(ADR 1255)*

**std.jwt.9** — Time claims are whole numbers of Unix seconds; a present claim that is not
one fails with `BadClaims`. Details: [below](#more-on-stdjwt9). *(ADR 1255)*

```ail tool=run,FILE expect=Expired,ok rules=std.jwt.9
fn outcome(tok str, o jwt.Options) str
    _ := jwt.verify(tok, b"k", [jwt.Alg.HS256], o) else err
        return err.kind.name()
    "ok"

fn main() !
    tok := jwt.sign(parse_json("\{\"exp\": 100}")?, b"k", jwt.Alg.HS256)?
    mut o := jwt.options()
    o.now = 200
    first := outcome(tok, o)
    o.leeway = 200
    print("{first},{outcome(tok, o)}")
```

**std.jwt.10** — With `issuer` set, the `iss` claim must be that text, else `BadIssuer`. Details: [below](#more-on-stdjwt10). *(ADR 1255)*

```ail tool=run,FILE expect=BadAudience rules=std.jwt.10
fn main() !
    tok := jwt.sign(parse_json("\{\"aud\": [\"a\"]}")?, b"k", jwt.Alg.HS256)?
    mut o := jwt.options()
    o.audience = "b"
    _ := jwt.verify(tok, b"k", [jwt.Alg.HS256], o) else err
        print(err.kind.name())
        return
```

**std.jwt.11** — Checks run in this order: shape (`Malformed`), header (`BadHeader`,
`UnsupportedAlg`, `AlgNotAllowed`), signature (`BadSignature`), then claims (`BadClaims`,
`Expired`, `NotYetValid`, `MissingClaim`, `BadIssuer`, `BadAudience`). No claim of a token with
a bad signature is looked at. *(ADR 1255)*

**std.jwt.12** — `error struct JwtError` has `kind JwtKind` and `why str` (its text);
`enum JwtKind` is `Malformed`, `BadHeader`, `BadClaims`, `UnsupportedAlg`, `AlgNotAllowed`,
`BadKey`, `BadSignature`, `Expired`, `NotYetValid`, `MissingClaim`, `BadIssuer` and
`BadAudience`. *(ADR 1255)*

## Rejected alternatives

- Taking the algorithm from the header: the source of the `none` and key-confusion attacks;
  the caller's list decides (`std.jwt.5`).
- One `verify` that takes an `HS` or a public key: a public key as `bytes` would be usable as an
  HMAC secret; `verify_with` therefore refuses the `HS` names (`std.jwt.6`).
- JWE (encryption), JWK sets and JSON serialisation: not in this module.

### `std.jwt` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.jwt.10

With `audience` set, the `aud` claim must be that text or an array holding it, else `BadAudience`. With neither set the claims are not checked.

#### More on std.jwt.3

The header is `{"alg":"HS256","typ":"JWT"}` (the algorithm's name), the payload is `claims`, which must be a JSON object (else `BadClaims`), and both are written as compact JSON. An empty `key` fails with `BadKey`.

#### More on std.jwt.5

`verify(token, key, allowed, options)` fails with `AlgNotAllowed` unless the header's `alg` names an `Alg` in `allowed` (an empty list allows none), and with `UnsupportedAlg` for a name that is not `HS256`, `HS384` or `HS512`. An empty `key` fails with `BadKey`. The signature is compared in constant time (`std.crypto.1`) before any claim is read; a mismatch fails with `BadSignature`.

#### More on std.jwt.6

`verify_with(token, alg str, check fn(bytes, bytes) bool, options)` requires the header's `alg` to equal `alg` (else `AlgNotAllowed`), then calls `check(signing_input, signature)`; false fails with `BadSignature`. `verify_with` refuses `none` and the names `verify` handles (`HS256`, `HS384`, `HS512`) with `UnsupportedAlg`, so a public key can never be taken as an HMAC secret. For `EdDSA` the signature is `crypto.ed25519_sign(seed, msg)` and the check `crypto.ed25519_verify(public_key, msg, sig)`; for `ES256` it is the raw 64-byte `r||s` of `crypto.ecdsa_p256_sign` and `crypto.ecdsa_p256_verify`, as RFC 7518 section 3.4 writes it.

#### More on std.jwt.7

Its result must not be trusted; use it only to read a `kid` before choosing a key. A token that is not three base64url parts, or whose header or payload is not a UTF-8 JSON object, fails with `Malformed`; a header without a text `alg` fails with `BadHeader`. Non-canonical base64url (padding, `+`, `/`, stray tail bits) is `Malformed`.

#### More on std.jwt.9

With `now` the current time: `exp` present and `now - leeway >= exp` fails with `Expired`; `nbf` present and `now + leeway < nbf`, or `iat` present and `now + leeway < iat`, fails with `NotYetValid`. A missing `exp` is accepted unless `require_exp` is set, then `MissingClaim`.

### `std.jwt` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.jwt.1

A `Token` has the fields `header` and `claims` (both `Json` objects) and `alg` (the header's `alg` text).

#### More on std.jwt.4

A header with a `crit` member fails with `BadHeader`, since no critical extension is understood.

#### More on std.jwt.6

`sign_with(claims Json, alg str, signer fn(bytes) bytes) str ! JwtError` writes `alg` in the header and the bytes `signer` returns for the signing input (`HEADER.PAYLOAD` as text bytes) as the signature. Details: [below](#more-on-stdjwt6).

#### More on std.jwt.8

Set a field on a `mut` copy.
