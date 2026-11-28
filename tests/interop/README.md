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

Database clients run against real servers in docker, on ail-linux only (`db-job.sh SHA [module ...]`, or
`DOCKER="sudo docker" tests/interop/run.sh redis postgres mysql mariadb`). They are never in the default list.
`servers.py` starts redis:7, postgres:16, mysql:8.4 and mariadb:11 on 127.0.0.1 ports 16379, 15432, 13306 and
13307, runs `<module>/cli.ail URL CASE` per case, compares its lines with the expected ones, and reads the data
back with redis-cli, psql or the mysql client. Each case is one PASS/FAIL line; results go to `results-servers.tsv`.
`LOCAL_SERVERS=1` runs redis, postgres and mariadb from packages installed on the host (redis-server, postgresql-16, mariadb-server; no docker), as the cloud container does; MySQL 8.4 needs docker.
Until the module PRs (#1563, #1546, #1559) are on main, `db-job.sh` merges their heads into the tested tree.

| module | AIL module | cases against the real server |
|---|---|---|
| redis | std.db.redis | AUTH (password, ACL user, wrong, none), strings, hashes, lists, sets, sorted sets, SCAN, a 200-command pipeline, MULTI/EXEC and EXECABORT, RESP3, pub/sub both ways, pool |
| postgres | std.db.postgres | trust, cleartext, md5, SCRAM-SHA-256 and wrong passwords, typed rows, `$n` parameters, SQLSTATEs, 200 KB parameter, 5000 rows, transactions, pool |
| mysql, mariadb | std.db.mysql | mysql_native_password, caching_sha2 cold (clean Auth error) and warm, text and binary protocol types, `?` parameters, errnos, prepared statements, 17 MB row, transactions, pool |

Network modules run against real implementations by `net_servers.py` (`tests/interop/run.sh smtp sse s3 oauth2`, or
`net-job.sh SHA [module ...]` on ail-linux). They are never in the default list. smtp and sse need only Python packages and curl;
s3 and oauth2 start docker containers (`chrislusf/seaweedfs`, `oryd/hydra:v2.3.0`). Ports and names: `ail-interop-s3` on 18333,
`ail-interop-hydra` on 14444 and 14445. Results go to `results-net.tsv`.

| module | AIL module | reference | cases |
|---|---|---|---|
| smtp | std.net.smtp | aiosmtpd (server), Python `email` (parser) | plain, STARTTLS and implicit TLS with a private CA; AUTH PLAIN and LOGIN, wrong password; envelope with Cc and Bcc; encoded words in subject and names; text, html, attachments; dot-stuffing; 1.2 MB body; `send_raw` bytes; 550 and 450 answers; SIZE limit; no STARTTLS; untrusted certificate |
| sse | std.net.sse | curl, requests and sseclient-py (read the AIL server); a scripted Python server and the WHATWG parsing algorithm (read by the AIL client) | wire bytes, ids, retry, comments, multi-line data, `Last-Event-ID`, `open_with`, `refuse`, `pump` and keep-alives, 200 KB events, a client that leaves; CR, CRLF and LF line ends, BOM, NUL ids, chunked framing, 1-byte pieces, unterminated last event, reconnect and retry time, statuses and content types, 1 MB event |
| s3 | std.cloud.s3 | SeaweedFS (S3 gateway), boto3, curl, botocore's SigV4 | put, get, head, range, delete, awkward keys, errors, paged listing (1250 keys), multipart (3 parts, manual, abort), presigned GET and PUT used by curl, expired and altered URLs, presigned signature equal to botocore's for a frozen clock |
| oauth2 | std.oauth2 | Ory Hydra v2.3.0 (client credentials, authorization code with PKCE, refresh, revoke; login and consent answered through its admin API), Authlib's RFC 8628 server (device flow) | Basic and body secrets with special characters, errors (`invalid_client`, `invalid_scope`, `unauthorized_client`, `invalid_grant`, `access_denied`), tokens checked by introspection, id_token verified with the server's key, refresh rotation, device approve, deny, slow_down, expiry |

Not yet here: HTTP/2 (curl --http2, nghttp), WebSocket from a reference client, and the official suites for unicode (UCD test files),
CommonMark, toml-test, html5lib-tests, YAML test suite, plus cron, date formatting and decimal against Python.
