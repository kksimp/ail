# 1248. `std.cbor` and `std.msgpack`: binary formats into and out of `Json`, written in AIL

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.cbor.1-10](../spec/std-cbor.md), [std.msgpack.1-9](../spec/std-msgpack.md)

The number is a placeholder; the integrator assigns it and renames this file.

## Context

Programs exchange CBOR (WebAuthn, COSE, IoT, many protocols) and MessagePack (RPC, caches, game state). The std had
JSON, TOML, YAML, XML, INI and CSV but no binary format. The binary reads on `bytes` and pushes on `List[u8]` (type.bytes.15-18, PR #1485) make both formats pure AIL.

## Decision

- Two modules, `std.cbor` and `std.msgpack`, each `encode(j Json) bytes ! XError` and `decode(data bytes) Json !
  XError`, in `std/NAME/NAME.ail`. They share no code: the formats share only the idea of a tagged value, and a
  shared helper module would be a third name for agents to learn (ADR 1077, AI understanding).
- The value is `Json`, so every accessor already in the language serves them.
- A whole `Num` in the `int` range is an integer in the shortest form, any other `Num` a 64-bit float. Reading back
  gives the same `Json`. An integer past 2^53 rounds, because `Json.Num` is a `float`.
- Byte strings (`bin`) read as a `Str` of standard base64; CBOR tags are dropped; `undefined` is `Null`.
- Refused with a kind: non-text map keys, `ext` values, unknown simple values, byte `0xc1`.
- Indefinite-length strings, arrays and maps are read (CBOR); they are never written.
- Depth is capped at 512 both ways; counts larger than the bytes left fail before any allocation.
- Errors are values with a kind enum (type.enum.7) and the byte offset.
- Native only, no runtime or checker work, no syntax change.

## Rejected alternatives

- **One module with a `format` argument.** Two unrelated wire formats under one name; the call site would say less.
- **A `Value` type with bytes, tags and integer keys.** Holds everything, but then no accessor of `Json` works and
  every program converts. `Json` covers the documents programs actually exchange.
- **Failing on byte strings or tags.** Refuses real data (COSE, WebAuthn). Base64 keeps the bytes.
- **Integer map keys as text.** Silent collisions (`1` and `"1"`); refusing with a kind says why.
- **Shortest float (16/32-bit) on write.** Saves bytes at a conversion cost; a 64-bit float is always exact.
- **Streaming decode.** Needs an I/O abstraction the std does not have; `decode` takes the whole `bytes`.
- **Canonical (deterministic) CBOR key sorting.** Object order is the program's data; sorting changes it. A program
  that signs wants RFC 8949 section 4.2 and can sort the members first.
