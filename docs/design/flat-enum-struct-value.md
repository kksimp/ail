# Flat enum and struct values (proposal)

Status: proposal, not decided. An ADR number is assigned by the integrator when
this is picked up. Non-normative; no language behaviour changes.

## Problem

`Value::Struct(usize, Rc<Vec<Value>>)` and `Value::Enum(usize, usize, Rc<Vec<Value>>)`
cost two heap blocks per value: the `Rc` box (strong, weak, `Vec` header) and the
`Vec` buffer. Building one is two allocations, dropping it is two frees plus a
pointer hop on every field read.

## Proposed layout

Replace the `Rc<Vec<Value>>` payload with a single-block `Rc<[Value]>`-style
object: `Struct(u32, Fields)` and `Enum(u32, u32, Fields)` where `Fields` is a
thin pointer to one allocation holding the count and the fields inline. The
variant tag fits in the same word as the type index (`u32` each), so `Value`
stays 16 bytes. Copy-on-write stays as `Rc::make_mut` does today:
`Fields::make_mut` clones the block when the count is above one (own.copy.1
semantics unchanged).

A zero-field value (`no_fields`, unit-like variants) needs no allocation at all:
`Fields` holds a null pointer.

## What changes

Roughly 140 match sites across 16 files under `bootstrap/src/interp/` and
`interp.rs`: `interp.rs` 41, `bytecode.rs` 22 (lane 190's file), `stdlib.rs` 20,
`prop.rs` 8, `ui.rs` 8, `netsock.rs` 7, `json.rs` 6, `parallel.rs` 6, `regex.rs` 5,
`replay.rs` 4, the rest 1-2 each. Most are mechanical (`&f[i]` still works through
`Deref<Target = [Value]>`). The real edits are the builders (`op_build`,
`json::Parser`, `with_field`, struct update) and the `make_mut` sites.
Estimate: 400-600 changed lines, about 120 new lines for `Fields`
(one `unsafe` block pair for the header layout, or a safe version on `Rc<[Value]>`
if the ADR 0737 no-`unsafe` rule is kept, see below).

`Rc<[Value]>` is the safe route: one allocation, a fat pointer. It makes `Value`
24 bytes (data pointer plus length), so the safe version trades a larger `Value`
against one fewer allocation. It is measurable but not free: every `Value` move
and every list element grows 50%. The thin-pointer version keeps 16 bytes but
needs `unsafe`, which is a designer decision (lane 190 card on `unsafe` in the
interpreter hot loop is still pending).

## Measured upper bound

Callgrind instruction counts, release build at the batch head plus #338, one run.

`tools/perfbench/agent/json_roundtrip.ail`, 653.3 M instructions in total:

| Share | Item |
|---|---|
| 10.7% | `FrontEnd::alloc` (all allocations) |
| 7.3% | `FrontEnd::dealloc` |
| 3.6% | `Rc<Vec<Value>>::drop_slow` self cost |
| 5.6% | `op_build` (struct and enum construction) |
| 1.5% | `drop_glue::<Value>` self cost |

Not every allocation is an `Rc<Vec>`: strings, list growth, `format_inner` and
map tables also allocate. If half of `alloc`/`dealloc` and the `drop_slow` self
cost were removed, the ceiling is about 9% + 2% = **11%**; a realistic figure is
half of that, **5-6%**. That does not close the 2x gap to CPython on this program.
The rest is interpreter dispatch (`exec` 19%) and `format_inner` (6.4% inclusive).

`tools/perfbench/programs/binarytrees.ail` at depth 14, 8,704 M instructions:
the program stores its tree in two `List[int]` arenas and uses no struct or enum,
so a flat enum/struct value changes **nothing** here. 75% of the time is in
`exec`, 12% in `run_code`, 4.5% in `run_call`; `drop_glue::<Value>` is 3.2% and
`Rc<Vec>::drop_slow` 0.5%. The binarytrees gap is the call path and the
instruction loop, which is lane 190's `bytecode.rs`, not value layout.

## Recommendation

Do not do it for speed alone: the ceiling is about 11% on one benchmark and
zero on binarytrees. Reconsider if the bytecode tier makes `exec` cheap enough
that allocation becomes the largest share. Cheaper first steps, each under
100 lines: reuse of `Rc<Vec>` buffers on drop (a per-thread free list for
field vectors of up to 4 elements), and building enum payloads in place instead
of through a temporary `Vec`.

## Rejected alternatives

- **NaN-boxing or 8-byte `Value`.** Touches every arm of the interpreter and the
  tensor and atomic cells; a far larger change than the measured gain justifies.
- **Inline small payloads in `Value` (up to 8 bytes of fields).** Makes `Value`
  larger or needs tagging tricks; fields are `Value`s themselves, so the nesting
  does not fit.
- **Arena-allocated fields per statement.** Values outlive statements; needs
  lifetimes the interpreter does not have.
