# 1305. Standard modules first released after 0.2.0 are experimental in 0.3

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** the designer, on the recommendation of the 2026-11-27 state review
- **Topic:** 17 — Standard library
- **Spec rules:** [std.use.1](../spec/17-std.md) (the index of the standard modules)

## Context

About 34,000 lines of standard library arrived in the two days before 0.3: 57 import paths that were not in 0.2.0.
Each has tests that ran against reference data or real servers, but none has had outside users, and several face
the network on in-house, unaudited cryptography. A release that promised all 88 modules equally would make every
early signature a compatibility promise.

## Decision

- The spec's index of the standard modules names the modules first released after 0.2.0 as *experimental* in 0.3:
  their names, signatures and behaviour may change in a minor release. They are `std.barcode`, `std.base32`, `std.base58`, `std.bitset`, `std.bloom`, `std.cbor`, `std.cloud.s3`, `std.cron`, `std.db`, `std.db.mysql`, `std.db.postgres`, `std.db.redis`, `std.decimal`, `std.feed`, `std.graphql`, `std.html`, `std.humanize`, `std.i18n`, `std.image`, `std.image.gif`, `std.image.jpeg`, `std.image.ops`, `std.image.png`, `std.image.webp`, `std.ini`, `std.jobs`, `std.jwt`, `std.lru`, `std.markdown`, `std.msgpack`, `std.net.fileserve`, `std.net.grpc`, `std.net.http2`, `std.net.imap`, `std.net.smtp`, `std.net.sse`, `std.net.web`, `std.net.websocket`, `std.oauth2`, `std.ordcoll`, `std.protobuf`, `std.qr`, `std.resilience`, `std.semver`, `std.sqlite`, `std.stats`, `std.tar`, `std.template`, `std.textdiff`, `std.textwrap`, `std.throttle`, `std.unicode`, `std.url`, `std.uuid`, `std.validate`, `std.wasm`, `std.xml`.
- The signing, key exchange and authenticated encryption of `std.crypto` (`std.crypto.5` to `std.crypto.11`) and
  the TLS server of `std.net` (`std.net.46` to `std.net.49`) are experimental too, and stay labelled unaudited.
- The language card names them on one line as experimental ([ADR 1306](1306-card-std-index-line.md)).
- A module leaves the list by a later decision, once it has outside use and its interop checks.

## Rejected alternatives

- **A per-module `stable`/`experimental` column in the index table.** The table groups several modules per row, and
  one list is easier to check against the compiler's module list.
- **An attribute or compiler warning on use.** It would warn in every program that uses the module, for a promise
  about future releases rather than a fault in the program.
- **Mark nothing.** The review's risk 1 (breadth outruns proof) stays unstated to users.
