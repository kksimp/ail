# tests/interop

Conformance of the std modules against real reference implementations. The lane tests under `tests/cases` use goldens
written by hand or from a model of the module; a bug the author shares with the golden passes them. These checks use
the reference's own output as input to AIL, and AIL's output as input to the reference. They are not gate suites
(they need Python packages and host tools) and run on the designer's machines only (ADR 0842).

Run from the repo root of a tree that has the modules (a chain branch or main): `tests/interop/run.sh [module ...]`.
Each `<module>/cli.ail` is a thin command line over the std module; `interop.py` makes the inputs with the reference,
runs the CLI and compares. A case is PASS, FAIL (AIL disagrees with the reference) or SKIP (reference not installed).
`run.sh` exits 1 on any FAIL and writes `results.tsv` (module, case, status, detail) and `summary.txt`.

| module | AIL module | reference |
|---|---|---|
| compress | gzip, deflate, zstd, brotli, both ways | python zlib/gzip, zstd and brotli CLIs, truncated input |
| image | std.image.png decode, encode | Pillow (all colour types, tRNS, filters), pypng (16 bit, Adam7) |
| crypto | std.hash, hmac, crc32, hkdf, pbkdf2 | hashlib, hmac, zlib, cryptography |
| aead | std.crypto seal and open (ChaCha20-Poly1305, AES-GCM) | cryptography; a separate module so the hashes run when #1509 is not in the tree |
| encoding | std.base32, base58, uuid | python base64, base58, uuid |
| jwt | std.jwt HS256/384/512, alg none, wrong key, expiry | PyJWT |
| binfmt | std.cbor, std.msgpack | cbor2, msgpack |
| tar | std.tar read, write, gzip | python tarfile (gnu, pax, ustar), the tar CLI |

Scripts the lanes ship are run as they are and counted per `ok` line: `tls` (tools/native-tls/server-check.sh, openssl
s_client and curl against the TLS server), `static` (tools/http/static-curl.sh, curl against the static file server),
`wsdeflate` (tools/native-net/ws-deflate-check.sh, Python websockets). A script missing from the tree is SKIP.
Docker-backed Redis, Postgres and MySQL checks are another lane's, on top of this tree.

Not yet here: HTTP/2 (curl --http2, nghttp), SSE and WebSocket from curl, SMTP (aiosmtpd), s3 (MinIO), and the
official suites for unicode (UCD test files), CommonMark, toml-test, html5lib-tests, YAML test suite, plus cron, date
formatting and decimal against Python.
