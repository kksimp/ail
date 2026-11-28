# Crypto assurance

Non-normative. What the in-house cryptography and TLS 1.3 client are checked
against, what is fuzzed, what is constant time, and what is not checked at all.
This is the page [SECURITY.md](../../SECURITY.md) points reviewers to. The older,
longer account of the TLS client is [tls-security.md](tls-security.md). Update this page in the
same change as the code or the checks.

**There has been no third-party audit.** Everything below shows that the code
computes the right answers and survives mangled input in the cases tried. It does
not show the absence of side channels or of bugs in untried inputs.

## Code under test

- Hashes, HMAC, HKDF, PBKDF2: `compiler/rt/native/hash.ail`, `crypto.ail`
  (`std.hash`, `std.crypto`).
- Public authenticated encryption, `crypto.seal`/`open` (std.crypto.9-10):
  `crypto_aead.ail` over the TLS ChaCha20-Poly1305 and AES-256-GCM below.
- TLS primitives: AES-128/256-GCM (`tls_aes.ail`, hardware kernels in
  `tls_aes_hw.ail`), ChaCha20-Poly1305 (`tls_chacha.ail`), X25519 (`tls_x25519.ail`),
  HKDF and the key schedule (`tls_kdf.ail`), big numbers (`tls_big.ail`), ECDSA and
  RSA signatures (`tls_sig.ail`).
- X.509 and DER: `tls_der.ail`, `tls_x509.ail`, chain checks in `tls_verify.ail`.
- The TLS 1.3 client: `tls_client.ail`.

## What runs in the gate

`./ail selftest crypto` (in `tools/gate/mac-gate.sh`) runs, on whatever machine the
gate runs on:

| Check | Script | What it does |
|---|---|---|
| Vectors | `tools/native-tls/prim-check.sh` | Builds the runtime with a test hook (`tools/native-tls/tlsprim.ail`, never shipped) and runs every vector file below. Fails unless the summary says `0 failed`. Also runs ARM64 under qemu where it is installed. |
| Wycheproof | `tools/native-tls/prim-check.sh` | The same script also runs about 3,000 [Project Wycheproof](https://github.com/C2SP/wycheproof) cases (see below). |
| Wycheproof, public API | `tools/native-tls/api-check.sh` | `tools/native-tls/wycheproof_api.ail`, an ordinary program, calls `std.crypto` on about 2,400 Wycheproof-derived cases and on checks of its own (below). |
| Server signatures | `tools/native-tls/server-sign-check.sh` | Signs with the TLS server's key loader and signer (RSASSA-PSS SHA-256/384/512, ECDSA P-256; PKCS#1, PKCS#8 and SEC 1 keys) and makes OpenSSL verify. Skips with a note in a tree without the server. |
| Parser fuzz | `tools/native-tls/fuzz-check.sh 1500 1` | Mutates the test certificates and the recorded ServerHello (see below). |
| Wire fuzz | `tools/native-tls/wire-fuzz.sh 60 1` | The real `https` client against a server that sends mutated copies of a recorded server flight. |

Longer runs: `tools/native-tls/fuzz-check.sh 100000 SEED` and
`tools/native-tls/wire-fuzz.sh 2000 SEED`. A seed reproduces a failure.
The std-level vectors (`std.hash`, `std.crypto`) run in the gate's `cases` suite
(`tests/cases/std_hash*.ail`, `std_crypto*.ail`; `std_crypto_aead.ail` opens the RFC 8439
and GCM-specification vectors through the public API and checks that every tampered byte gives `none`).

## Vector-checked

| Subject | Vectors | Source |
|---|---|---|
| SHA-256/384/512 | FIPS 180-4 examples, 16 random inputs each | `tests/cases/std_hash.ail`, `tests/data/net/crypto/crosscheck.txt` |
| SHA-1, MD5, SHA3-256/512 | Padding edges | `tests/cases/std_hash_legacy.ail` (FIPS 180-4, RFC 1321, FIPS 202) |
| HMAC, HKDF | RFC 4231 and RFC 5869 cases, 16 random inputs each for SHA-256, SHA-384 vectors generated with Python hashlib | `crosscheck.txt`, `tools/native-tls/vectors384.txt`, `tests/cases/std_crypto.ail` |
| PBKDF2-HMAC-SHA-256 | Published vectors | `tests/cases/std_crypto.ail` |
| AES-GCM | 32 random inputs with a flipped-bit tamper check on each | `crosscheck.txt` |
| ChaCha20-Poly1305, Poly1305 | 16 random inputs each, tamper check | `crosscheck.txt` |
| X25519 | 16 random inputs; RFC 7748 | `crosscheck.txt` |
| ECDSA P-256/P-384, RSA PKCS#1 v1.5 and PSS | 108 OpenSSL signatures (valid, and with a flipped bit or message) | `tests/data/net/crypto/signatures.txt` |
| Key schedule | RFC 8448 secrets | `tlsprim.ail` `rfc8448` |
| X.509 path validation | 26 chains on a test PKI: expired, not yet valid, wrong name, wildcards, IP names, EKU, unknown critical extension, not a CA, path length, name constraints, no root | `tests/data/net/x509/chain.txt`, `prim.ail` |
| Handshake | A recorded TLS 1.3 handshake replays byte for byte; live `openssl s_server` for each suite | `tests/data/net/tls/illustrated-tls13.txt`, `tools/native-tls/https-check.sh` |

Random vectors came from OpenSSL 3.0; the other vectors are the published ones named.

**Project Wycheproof** (ADR 1118; Apache-2.0, `tests/data/net/crypto/wycheproof.txt` and `wycheproof-sigs.txt`, converted by
`tools/native-tls/wycheproof.py`, which records the SHA-256 of each source file): AES-128/256-GCM and ChaCha20-Poly1305
(seal for valid cases; open for valid and invalid ones, so wrong tags, truncated tags and edge-case nonces must be
rejected), X25519 (non-canonical and low-order inputs), HMAC-SHA-256, HKDF-SHA-256, ECDSA P-256 (SHA-256 and SHA-512) and
P-384 (SHA-384) with the TLS client's strict DER parser, RSA 2048 PKCS#1 v1.5 (SHA-256) and RSA 2048 PSS (SHA-256).
Left out: cases Wycheproof marks `acceptable`, AES-192, 64-bit and other short tags, nonces that are not 96 bits,
truncated HMAC tags, RSA 3072. The counts are in the header of each file.

**Wycheproof on the public API** (`tests/data/net/crypto/wycheproof-api.txt`, converted by `tools/native-tls/wycheproof_api.py`; ADR 1118):
what `prim-check.sh` checks on the primitives with fixed nonces, this checks on the calls a program makes.
Ed25519 `ed25519_verify` (210 cases: malleable S, non-canonical keys and signatures, truncation; `acceptable` small-order and
mixed-order keys may go either way but must not trap), X25519 `x25519` (518, low-order peers give `none`), ECDSA P-256
`ecdsa_p256_verify` (262 raw `r || s` cases: zero, out-of-range and wrong-size values), AES-256-GCM and ChaCha20-Poly1305 `open`
and `open_aes_gcm` (1,141 cases, with the nonce, ciphertext and tag laid out as `seal` writes them, so a Wycheproof case with
a short tag or a non-96-bit nonce must give `none`; eight truncated or tampered copies of 40 valid cases per cipher), `hkdf_sha256`
(83) and `hash.hmac_sha256` (the cases whose key and message are UTF-8 text; prefix matching for truncated tags). The same
program checks, with no outside vector, that `seal` output is 28 bytes longer and opens, that `seal` never repeats a nonce, that
every cut, changed byte, changed `aad` and the other cipher's `open` give `none`, that Ed25519 and ECDSA signatures are
deterministic and verify, that each changed byte is refused, and that S + L is refused (Ed25519 malleability).
Not covered: Wycheproof's AES-128/192 and non-96-bit nonce cases (the API has neither), HKDF sizes above 8160 (the call traps),
ECDSA with other hashes or curves (no public call), DER-encoded ECDSA (the API takes `r || s`; the DER parser is covered by
`wycheproof.txt`), and timing.

## Fuzzed

- **X.509 and DER** (`tools/native-tls/fuzz.ail`): one to two random edits (digit,
  byte, insert, delete, truncate, duplicate, overwrite) of a leaf or intermediate
  certificate in four test chains, then the full parse, signature and chain
  check. Properties: nothing traps or hangs, and **a changed certificate is never
  accepted**.
- **ServerHello** (same program): random edits of the recorded ServerHello body
  through `tc_server_hello`. Property: nothing traps or hangs.
- **Record layer and handshake framing over a socket** (`wire-fuzz.py`): the
  client talks to a local server that sends a mutated recorded flight (record
  headers, lengths, ServerHello, then bytes that fail authentication). Property:
  each connection ends in an error; none is accepted, none crashes or hangs.
- **Std modules** (`ail selftest --fuzz N`): the module fuzzers of `tests/fuzz/modules`.

## Not fuzzed

- The Certificate and CertificateVerify message framing inside an encrypted
  handshake: reaching it needs a valid session key, so the wire fuzzer stops at the
  first record that fails authentication. The X.509 fuzzer covers the certificates
  in it; the framing loop is covered by the recorded handshake and the live servers.
- The key-update and post-handshake message paths.
- The big-number, AES, ChaCha20 and X25519 code is checked by vectors only.
- Coverage-guided fuzzing: the fuzzers are plain random mutation.

## Constant time

- `crypto.ct_eq` reads every byte and has no data-dependent branch (std.crypto.1).
- AES uses hardware instructions where present (AES-NI/PCLMULQDQ, ARM AES/PMULL);
  otherwise a bit-sliced software AES with no secret-indexed table.
- ChaCha20, Poly1305 and X25519 have no secret-dependent branch or index, by
  construction and by review.
- **Timing measured, machine code not checked** (ADR 1118). `tools/native-tls/timing-check.sh` times `ct_eq`, X25519,
  ChaCha20-Poly1305, AES-GCM and Poly1305 on a fixed secret against random secrets, in random order, and reports Welch's t
  (a dudect-style test; |t| above 10 is a leak). A comparison that stops at the first difference is the control and
  must be flagged. It is not in the gate, because a shared or busy machine gives false alarms: run it on an idle host.
  It shows that no leak was seen at that sample size on that machine; it does not prove there is none, and no tool checks
  the generated machine code, so the compiler could still introduce a secret-dependent branch the test misses.
- Big-number arithmetic (RSA and ECDSA verification) is variable time. It only sees
  public values: public keys, signatures and message hashes.

## Not checked

- Side channels other than timing (cache, power, fault injection).
- Zeroising: keys are not wiped from memory after use.
- Third-party review, or formal verification of any primitive.
- Revocation (CRL, OCSP) and Certificate Transparency.
- TLS 1.2, client certificates, 0-RTT, session resumption.
- Behaviour against hostile servers on the public internet beyond the listed tests
  (a one-off check of 14 public sites is a manual run, `AIL_TLS_PUBLIC=1`).
- Windows root-store loading is not run in the gate.

## How to add a check

A new vector goes in the file for its primitive under `tests/data/net/crypto/` and,
for an operation `prim.ail` does not know, one arm in `tlsprim_call`. A new fuzz
target is an operation in `tlsprim.ail` and a loop arm in `fuzz.ail`. Run
`./ail selftest crypto`.
