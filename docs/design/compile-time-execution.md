# Compile-time execution

> **Non-normative design note,** 2026-10-20 (lane 37 of the
> [AI-first plan](ai-first-plan.md)). Topic 18. Decided by ADRs
> [0225](../decisions/0225-constants-call-compile-time-safe-functions.md) and
> [0226](../decisions/0226-compile-time-limits-determinism-and-caching.md);
> normative rules `ct.const.*` and `ct.eval.*` in
> [spec chapter 15](../spec/15-compile-time.md). Builds on ADR 0032 (the
> interpreter is the oracle), ADR 0132 (constants), ADR 0163 (effect sets)
> and [ir-architecture.md](ir-architecture.md).

## 1. The problem

ADR 0132 lets a module constant hold only literals, operators and other
constants. A lookup table that code computes (`CRC_TABLE`, `SQUARES`, a
keyword set split from a string) has to be built in `main` and passed down,
or rebuilt in every function that needs it. Both cost tokens and hide a
fixed value behind a call. Every mature systems language grew an answer:
C++ `constexpr`/`consteval`, Rust `const fn`, Zig `comptime`, D CTFE. Each
had to answer four questions; AIL's answers follow.

## 2. What may run: compile-time-safe functions

A constant's value may be **any expression**, including calls, method calls,
`if`, `match`, ranges and indexing, as long as everything it calls is
**compile-time-safe**: its inferred effect set (ADR 0163) has none of `io`,
`unsafe`, `spawn` or any capability (`fs`, `env`, `proc`, `clock`, `random`,
`net`). That is exactly the set `@pure` forbids (ADR 0164), so "`@pure`"
and "callable at compile time" mean the same thing, with or without the
attribute.

```ail skip=bootstrap
SQUARES := squares(16)
KEYWORDS := "fn if else for while match".split(" ")
LIMIT := parse_limit("4096")?

fn squares(n int) List[int]
    mut out := List[int]()
    for i in 0..n
        out.push(i * i)
    out
```

No marker is written on the function. The effect system already knows the
answer, and a marker would split functions into two colours that agents
must track (§6). The cost of inference is that an edit far away (adding a
`print` to a callee) can make a constant invalid; the diagnostic then names
the chain, as `@pure` does: ``LIMIT` is computed at compile time but
`parse_limit` has effect `io` via parse_limit -> log -> print``. An author
who wants the promise stated at the function writes `@pure`.

`fail` and `mutates` are allowed: a function may mutate its own locals and
`mut` parameters, and may fail. A constant can't *hold* a failure, so a
fallible value must be handled: `?` turns a failure into a compile error at
the constant (`ct.const.3`), and `else` supplies a fallback. A trap
(overflow, index out of range, `unreachable`, a `todo` hole) while computing
a constant is a compile error too, reported with the compile-time call
stack.

A constant's type may not contain `ref` or a `unique` type: those have
identity, and a constant is an immutable value copied into the program
(`ct.const.4`). `any Trait` values are fine; they are values.

## 3. Same semantics as run time

Compile-time evaluation runs the **interpreter's semantics** (ADR 0032):
the bootstrap literally runs the constant with the interpreter, and the AIL
compiler will evaluate the constant's HIR with an evaluator held to the same
conformance tests. There is no separate "constant semantics" to learn, and no
Miri-style second definition to keep in step.

AIL removes the usual host-versus-target trap: every target is 64-bit and
little-endian (ADR 0007), `int` is 64 bits everywhere, overflow traps
(ADR 0035), and float operations are IEEE 754 binary64 with no contraction.
A value computed on a Linux x86-64 build machine is bit-identical to the
value the program would compute on macOS ARM64. If a standard function ever
has an ISA-dependent result (a fast approximate `sin`), it must either be
correctly rounded or carry an effect that makes it non-compile-time-safe.

## 4. Limits

A runaway constant must not hang `ail check`, which is the agent's main
feedback loop (wishlist 2.5).

- **Steps:** each constant may take at most **100 million evaluation steps**
  (one per expression evaluated, counted the same way on every machine, so
  the limit is deterministic, unlike a time limit).
- **Memory:** at most **1 GiB** of live values per constant.
- **Depth:** the same call-depth limit as run time.

Exceeding one is a compile error (`ct.eval.3`) naming the constant, the
limit, and the call stack at the point where it stopped. A package manifest
(lane 38, Topic 13) may raise the step limit for one package; nothing lowers
it silently.

## 5. Determinism and caching

Compile-time code reaches no capability, so it sees no clock, environment,
files or randomness. Map and set iteration is in insertion order
(`type.map.1`, `type.map.5`). Evaluation is therefore a pure function of the
program text, which is what the query cache (ADR 0015, ADR 0224) requires.

- **Query:** `const_value(id)`, keyed by the constant's stable id.
- **Fingerprint:** the span-free hash of the constant's HIR plus the HIR of
  every function the evaluation actually entered (recorded as it runs), plus
  the evaluator's version. Editing a function the constant never called
  doesn't re-run it.
- **Result:** the value, serialised canonically, stored in the shared cache
  and emitted as read-only data. The program reads it; it is never computed
  at run time. Value semantics (ADR 0060) mean a function that receives it
  and mutates it works on its own copy.
- **Order:** constants still have no order (`mod.const.3`). A constant that
  reaches itself through a function is the same cycle error, naming the
  chain.

## 6. What tools show

- `ail api` shows a constant's **computed value** (proposed optional key
  `value`, the text form, for values up to 200 characters; additive, so still
  `api 1`). An agent reading a dependency sees `SQUARES = [0, 1, 4, …]`
  without running anything. This is a follow-up for the `ail api` owner.
- `ail effects` needs no change: a constant contributes no effect to the
  program, since it runs at compile time (`eff.infer.3`).

## 7. The bootstrap

The bootstrap implements `ct.const.1–4` (this lane's prototype): the checker
accepts the full expression language in a constant's value, then rejects it
if anything it may call isn't compile-time-safe, using the effect sets of
`check/effects.rs`. The interpreter already computes constants before
`main`; that is the bootstrap's "compile time", and a failure there is
reported as a trap at the constant before `main` starts (as ADR 0132 already
specified). Step and memory limits (`ct.eval.3`) and evaluation inside
`ail check` wait for the query compiler.

## 8. Rejected alternatives

| Alternative | Why rejected |
|---|---|
| **`const fn` marker** (Rust) | +1 token per function, and a second colour of function: every helper a constant uses must be re-marked, and libraries forget. The effect set already holds the answer (criterion 1, then 2). Measured with `tools/tokbench`: `ct_b_const_fn` costs 39.2 tokens vs 17.4 for the plain form on a two-constant sample, though most of that is the extra declarations. |
| **`comptime` at the use** (Zig) | +2 tokens per constant (21.4 vs 17.4 mean) to say what a top-level constant already means. |
| **Compile-time parameters and types as values** (Zig `comptime T: type`) | Generics (ADR 0080) already cover type parameters. Checking would depend on evaluation, so a signature no longer tells an agent what a function accepts (criterion 2), and the cache could no longer type-check a body without running code. |
| **`constexpr` / `consteval` split** (C++) | Two keywords for one idea, and "may run at compile time" versus "must" is a distinction agents get wrong. |
| **A separate constant evaluator with its own rules** (Rust's const-eval before Miri) | Two semantics drift apart. ADR 0032 already gives one oracle. |
| **Build scripts** (Rust `build.rs`, arbitrary code at build time) | Runs with every capability; a dependency can read files or the network during a build, which is the supply-chain hole wishlist 5.1 closes. |
| **Keep computing constants at startup** (status quo) | Startup cost, failures appear only when the program runs, and the value isn't visible to `ail api`. |
| **A wall-clock time limit** | Not deterministic: the same program could build on one machine and fail on a slower one. |
