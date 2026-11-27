# TLS security note

Non-normative. See also [crypto-assurance.md](crypto-assurance.md), the one-page list of what is checked. What the in-house TLS 1.3 client (ADR 0360) has and hasn't
been verified to do. Update it in the same change as the code.

## Status

`https` works in `net.get` and `net.post` (`std.net.5`). Slices:

| Slice | ADR | State |
|---|---|---|
| 1. Primitives | [0366](../decisions/0366-tls-crypto-primitives.md) | Done, tested |
| 2. Record layer and handshake | [0367](../decisions/0367-tls13-client-handshake.md) | Done, tested |
| 3. Certificates | [0368](../decisions/0368-tls-certificate-verification.md) | Done, tested |
| 4. `net.get`/`net.post` over TLS | [0369](../decisions/0369-https-in-std-net.md) | Done, tested |

## Verified

- **Known answers:** SHA-256/384/512 (FIPS 180-4 examples, including the
  million-`a` message), HMAC (RFC 4231 cases 1, 2, 6), HKDF (RFC 5869
  A.1–A.3; RFC 8448 early secret), Poly1305 (RFC 8439 §2.5.2, A.3 incl. the
  carry edge cases), ChaCha20 block, cipher and AEAD (RFC 8439 §2.3.2,
  §2.4.2, §2.6.2, §2.8.2), AES-128/256 (FIPS 197 C.1, C.3, A.1 key
  expansion), AES-GCM (GCM spec cases 1–4, 13, 14), X25519 (RFC 7748 §5.2
  incl. 1,000 iterations, §6.1).
- **S-box:** the bitsliced S-box equals the S-box computed from its
  definition for all 256 inputs.
- **Cross-check:** 160 random vectors whose outputs came from OpenSSL 3.0,
  lengths 0–1000 around every block edge, for every primitive
  (`tests/data/net/crypto/crosscheck.txt`).
- **Tamper:** a flipped bit in ciphertext, tag, AAD or nonce fails `open`.
- **Handshake, recorded:** a real TLS_AES_256_GCM_SHA384 handshake (The
  Illustrated TLS 1.3 Connection) replays with every client byte equal to
  the recording: key schedule, both directions' record protection, both
  Finished messages, tickets and application data. Every 7th byte of the
  server's flight corrupted, and truncation at each record, fails cleanly.
- **Handshake, live:** `openssl s_server` (OpenSSL 3.0) for each of the
  three suites, a server that prefers P-256, a server-requested KeyUpdate
  with data both ways afterwards, and refusals from a TLS 1.2-only and a
  P-256-only server with messages naming the cause. Skipped with a note
  where `openssl` isn't OpenSSL 1.1.1 or 3 (macOS ships LibreSSL), so on
  macOS only the recorded and RFC tests run unless OpenSSL is installed.
- **RFC 8448:** handshake and application secrets, `finished` key, traffic
  key and IV, and the first application-data record (values as transcribed
  by Mbed TLS's test suite).
- **Signatures:** 108 OpenSSL signatures (ECDSA P-256/P-384 with
  SHA-256/384/512; RSA 2048/3072/4096 PKCS#1 v1.5 and PSS with each hash),
  each valid one accepted and each with a flipped bit or changed message
  refused; curve constants checked (G on the curve, n·G = O).
- **Certificates:** a 19-certificate test PKI (`x509/testdata`) exercising
  each rule: good chains (ECDSA leaf under an RSA intermediate under a P-384
  root, RSA leaf, PSS-signed, SHA-512, root included, shuffled with extras),
  wildcards, IPv4/IPv6 names, expired, not yet valid, clientAuth-only,
  unknown critical extension, self-signed, intermediate that isn't a CA,
  path length exceeded, name constraints, tampered signature, no roots.
- **Live with real checks:** `openssl s_server` serving the test chain for
  an ECDSA leaf and an RSA leaf (CertificateVerify by ECDSA and by
  RSA-PSS), verified against the test root; the wrong host name fails as a
  certificate error naming the cause.
- **Root stores:** all 122 roots in Ubuntu's bundle parse.
- **End to end:** an AIL program (`tests/net/https.ail`) fetches
  from a local `openssl s_server` over `https` with the test root as
  `SSL_CERT_FILE`, and gets kind `tls` with the expected message for a
  wrong host and for an untrusted root.
- **Real servers (once, 2026-10-20):** through the sandbox's TLS-terminating
  egress proxy, handshakes, certificate checks and downloads (up to 343 KB)
  succeeded for github.com, api.github.com, raw.githubusercontent.com,
  pypi.org, files.pythonhosted.org, crates.io and static.rust-lang.org.
  `AIL_TLS_PUBLIC=1 ./ail selftest public_sites -- --nocapture` repeats a
  direct check against 14 public sites where the network allows.
- **Mutation:** 3,000 mutated ServerHello and Certificate bodies and 300
  mutated server flights per selftest (100,000 and 10,000 run once with
  `AIL_FUZZ_N=100000`) without a panic; 5,000 mutated certificates per
  selftest (100,000 once) through parsing, signature and chain checks.

## Not verified

- **Timing:** constant time is by construction (no secret branches or
  indexes, reviewed by reading), not measured. The compiler could in
  principle reintroduce a branch; no tool checks the machine code yet.
- **Zeroising:** keys aren't wiped from memory after use.
- **Side channels beyond timing** (power, faults): out of scope.
- **HelloRetryRequest with a cookie:** implemented per RFC 8446 §4.1.4
  and unit-tested, but no available test server sends one.
- **Proxies:** `std.net` connects directly; `HTTPS_PROXY` is ignored.
- **Revocation and Certificate Transparency** aren't checked; nor are
  policies, directory-name and email name constraints, or the user's own
  trust settings on macOS and Windows (ADR 0368).
- **Windows root store:** written, not run in the gate; Windows loads
  roots lazily, so `SSL_CERT_FILE` is the fallback.
- **macOS keychain export:** runs `/usr/bin/security`; checked by the
  gate's `system_roots_load` test only when it runs on macOS.
- **Big-number arithmetic** is variable time; it only ever sees public
  values (keys, signatures, hashes of public messages).

## Limits

- AES-GCM uses AES-NI and PCLMULQDQ on x86-64 and the AES and PMULL
  instructions on ARM64 (macOS always, Linux when AT_HWCAP reports AES and PMULL; `compiler/rt/native/tls_aes_hw.ail`,
  `compiler/x64/kernels_aes.ail`); elsewhere it runs the bit-sliced code,
  about 10 MB/s (ChaCha20-Poly1305 about 290 MB/s; Linux x86-64, release
  build). `tools/native-tls/aes-bench.sh` measures it. Hardware AES-128-GCM
  seal plus open, in MB/s (other load was running on the Mac, so the Mac
  figures are lower bounds):

  | Host | 64 MiB | 16 MiB |
  |---|---|---|
  | Linux x86-64 | 305.0 | 294.3 |
  | Windows x86-64 | 330.6 | 329.0 |
  | macOS ARM64 | 556.9 | 578.5 |
- TLS 1.2, client certificates, 0-RTT and session resumption are not
  implemented (ADRs 0360, 0367). The client offers X25519 and secp256r1; it
  sends an X25519 share and a P-256 share only after a HelloRetryRequest asks
  for group 23 (`p256_ecdh.ail`, unaudited, constant time on the secret scalar;
  the peer point is checked to be on the curve, and the Wycheproof ECDH vectors
  run in `tests/cases/std_crypto_p256_ecdh.ail`).
