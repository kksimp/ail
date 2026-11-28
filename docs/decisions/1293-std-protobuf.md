# 1293. `std.protobuf`: the proto3 wire format, a dynamic `Message`, `.proto` schemas and typed messages through `Json`

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.protobuf.1-16](../spec/std-protobuf.md)

The number is a placeholder: the integrator assigns it when chaining and renames this file, the index row and the
`ADR 1293` marks in `std/protobuf/protobuf.ail` and `docs/spec/std-protobuf.md`.

## Context

Protocol Buffers is the wire format of gRPC and of most service-to-service data. AIL has `bytes.uvarint`,
`push_uvarint` and the fixed-width reads (ADR 1215) but nothing above them: a program that talks to a protobuf
service hand-writes tags, zigzag, packed lists and length prefixes. A schema-less reader is the first need (tools
that inspect or forward messages); a typed reader needs the `.proto` file.

## Decision

- `std.protobuf` is an AIL module (`std/protobuf/protobuf.ail`, pure, no runtime, checker or lowering work).
- **Wire level.** `decode(bytes) Message ! ProtoError` and `encode(Message) bytes`. A `Message` is `fields
  List[Field]` in wire order; `Value` is `Varint | Fixed64 | Bytes | Fixed32`. Every field is kept, so unknown
  fields survive a decode and an encode in order.
- **Dynamic access by number.** `add_*` builders (scalars, zigzag, nested, packed) and `get_*` reads, plus
  `ints`, `sints`, `doubles`, `floats`, `strs` and `messages` for repeated fields (packed and unpacked both read).
  A singular read takes the last value, as the format requires.
- **Varints use all 64 bits.** `bytes.uvarint` stops at 63 bits, so this module reads and writes the 10-byte form
  itself; a negative `int32` is the sign-extended 10-byte varint, as the reference implementations write it.
- **Schemas.** `parse_schema` reads the proto3 subset in the spec (nested messages and enums, `repeated`,
  `optional`, `map`, `oneof`, `packed`), skips `import`, `option`, `reserved`, `service` and `extend`, and resolves
  type names from the innermost scope outward.
- **Typed encode and decode go through `Json`**, as the canonical protobuf JSON mapping does: `to_json(schema,
  message, bytes)` and `from_json(schema, message, json)`. 64-bit integers are decimal strings, because a `Json`
  number is a `float` and loses digits above 2^53; `bytes` is base64; enums are names. Members are read by the field
  name or its lowerCamelCase form.
- Proto3 presence: a singular scalar at its default is not written unless `optional` or in a `oneof`.
- Errors are a `ProtoError` with an enum `kind` (type.enum.7), a message, and the byte offset or `.proto` line.
- The module name `protobuf` clashes with no local name in the corpus or examples (std.use.4).
- No syntax changes.

## Rejected alternatives

- **Generated code from `.proto` (a `protoc` plugin).** Needs a build step and a code generator per target, and
  the generated types would be the user's to keep in step with the file. Runtime schemas read the file the service
  publishes.
- **A struct-mirroring derive (`ToProto` on user types).** Needs trait and checker work and a field-number
  annotation syntax; ADR 1077 puts no-syntax-changes first. A dynamic message plus `Json` serves the same programs
  today and can sit under a derive later.
- **`bytes.uvarint` for all varints.** Its 63-bit limit cannot hold a negative `int32` or a `uint64` above 2^63,
  both common on the wire.
- **64-bit integers as `Json` numbers.** Loses digits above 2^53 silently; the string form is the canonical one.
- **Dropping unknown fields on decode.** A proxy or a read-modify-write program would lose data the schema's
  newer version added. Keeping every field costs nothing for the dynamic API.
- **Groups (wire types 3 and 4).** Deprecated since proto2; an error is clearer than a half reading.
- **proto2 defaults, extensions and `required` checks.** The module reads proto2 syntax as far as the fields go
  and treats `required` as `optional`; the rest is outside proto3 and outside what services publish today.
- **Streaming and length-prefixed message framing.** A single `Message` is the unit; framing (gRPC, delimited
  streams) is a few lines over `bytes.uvarint`.
