# 0658. `parse_json`, `Json.to_text` and `json_quote` run natively

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 13 — Standard library
- **Spec rules:** json.parse.1, json.parse.2, json.print.1, json.print.2, json.err.1 (no rule changes)
- **Builds on:** [ADR 0359](0359-bytecode-tier.md) (the tier these calls leave)

## Context

Agent programs read and write JSON all the time. The prelude implements it
in AIL, so every character of a document ran through the interpreter: a
1.25 MB document of 20,000 records took 1.50 s to parse and 0.48 s to print,
about 20 times CPython's `json` for the whole round trip (`tools/perfbench/agent/json_roundtrip.ail`).

## Decision

The three functions keep their AIL source, so the checker, `ail explain`
and the self-hosted compiler read the same declarations. After checking,
the interpreter marks them `extern` with the library `ail:json`
(`interp::install_native_json`), and `call_extern` sends them to
`interp/json.rs`, a Rust port with the same values, error messages,
character offsets and call-depth traps as the AIL text. No behaviour or
rule changes. The native build (ADR 0393) still compiles the AIL source.

Measured on Linux x86-64, `json_roundtrip` (build + `to_text` + parse):

| | before | after |
|---|---|---|
| whole program | 1.99 s | 0.27 s |
| CPython twin | 0.09 s | |

`./ail selftest` passes, including the `json_*` and `derive_json_*` cases.

## Rejected alternatives

- **Port `Json.get`, `at`, `str`, `num`.** They are one or two tier
  instructions per call; the cost was in the character loop.
- **A new `json` opcode.** Lane 190 owns the opcode loop, and an extern
  call needs none.
- **Replace the AIL source.** The checker and the native build need it.
