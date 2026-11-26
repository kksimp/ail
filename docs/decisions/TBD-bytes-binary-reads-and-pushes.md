# TBD. Binary reads on `bytes` and binary pushes onto a `List[u8]`

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std to be filled out on 2026-11-26 (std-gap plan, lane 5)
- **Topic:** 4 — Types
- **Spec rules:** [type.bytes.15](../spec/04-types.md), [type.bytes.16](../spec/04-types.md), [type.bytes.17](../spec/04-types.md), [type.bytes.18](../spec/04-types.md)
- **Extends:** [ADR 0412](0412-bytes-type.md), [ADR 0702](0702-native-bytes.md)

## Context

Parsing or writing a binary format (a file header, a network frame, a varint stream) took a byte-by-byte loop with
shifts: `b[i].to_int() + b[i + 1].to_int() * 256 ...` for every field, and a `List[u8]` built one `push` at a time.
The corpus survey of the std-gap plan found that agents write these loops for `varint_bytes`, header parsing and
checksums, and get the order, the sign or the width wrong. Every language the agents know has the operation
(`struct.unpack_from`, `binary.LittleEndian.Uint32`, `u32::from_le_bytes`).

## Decision

- Reads are methods of `bytes`: `b.u8(at)`, `b.i8(at)`, and `_le` and `_be` forms of `u16 i16 u32 i32 u64 i64 f32
  f64`, all taking the offset `at`. They give an `int`, except `u64_*` (a `u64`, the only type that holds it) and
  the floats (a `float`; an `f32` is widened exactly). A read past the end traps like the slice `b[at..at + w]`.
- `b.uvarint(at)` is `(int, int)?`: the unsigned LEB128 number and its length; `none` when it is cut off or needs
  more than 63 bits. `at == b.len()` is allowed and gives `none`, so a loop can stop on it.
- The builder is the `List[u8]` that `bytes.from_list` already takes: `xs.push_` plus a read's name
  (`push_u32_le(x)`), `push_bytes(b)` and `push_uvarint(x)`. A push takes what the read gives; an `int` outside
  the kind's range traps as `x.to_u16()` does.
- Native: a read is one bounds test and one load (`load8`, `pload16`, `pload32`, `load64`, `loadf32`); a big-endian
  read adds one byte swap, a new primitive `bswap16/32/64` (`bswap` then `shr` on x86-64, `rev` then `lsr` on
  arm64). A push tests the room inline and calls the runtime only when the list is full, then stores the value and
  the new length. `uvarint`, `push_bytes` and `push_uvarint` are runtime calls (`rt/native/bytes_conv.ail`).

## Reasons

- AI understanding: the name says the wire format (`u32_le`), the result is the type the program computes with.
  An `int` result means `off += b.u32_be(4)` and `n < b.u16_le(0)` need no conversion, the most common use, and an
  agent's habit from Python. Sized results would add a `.to_int()` to nearly every call.
- One builder, not two: `List[u8]` already has `push`, `extend`, `len`, indexing and `bytes.from_list`, and its
  elements are one byte each (ADR 0907), so it is the growable buffer. A `bytes.Builder` type would be a second
  vocabulary for the same thing.
- Speed: a read compiles to the load a C `memcpy` read does, plus one compare pair; a push to one store and a
  length update on the fast path.

## Rejected alternatives

- **Sized results (`u32_le` gives a `u32`).** Matches Rust and Go, but every use in an `int` expression then needs
  `.to_int()`, and mixed-type arithmetic is a checker error (type.num.13); more iterations for the common case.
- **A `bytes.Builder` type.** A second growable byte type beside `List[u8]`; also needs its own `to_bytes`, `len`
  and indexing. Measured cost of `bytes.from_list` is one copy of the data, which a builder's `to_bytes` would pay too.
- **`get_*` forms giving `T?` instead of a trap.** A read past the end is a bug in the parser, like an index past
  the end; `b.len()` tests the length first when the data may be short. Can be added later without a break.
- **One generic `b.read(at, kind, order)`.** A string or enum argument hides the width from the reader and the
  checker; one name per format is what agents search for (`ail ask std "read u32 little endian"`).
- **Reads as runtime calls.** A call per field is several times the cost of the load it wraps.

## Follow-ups

- `bytes.from_list(xs)` copies the list's bytes; giving the list's block to the `bytes` when the list is not used
  again needs a last-use analysis and is left to the optimiser.
- The byte swap primitive is not on the load-forwarding pure list (`mir/forward.ail`), so a big-endian read forgets
  earlier bounds facts; harmless, a small cost in long chains of big-endian reads.
- Windows: the methods are pure computation and the runtime pieces use no OS call, so they work there unchanged.
