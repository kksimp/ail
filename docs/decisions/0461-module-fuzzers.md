# 0461. Module fuzzers are templates plus seeded input generators

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md); the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 28 — Testing strategy (tool)
- **Spec rules:** none (a tool); harness rule IDs `fuzz.mod.0–7`, `fuzz.run.2`

## Context

[ADR 0269](0269-fuzz-generator-design.md)'s generator makes whole programs. The batch adds
`bytes`, regex, dates, compression, hashing and an HTTP server, each with a
parser for hostile input. They need fuzzers the day they land, written by
the lane that owns them, without Rust.

## Options considered

### Option A — template programs with input placeholders (chosen)
- **Pros:** a module's fuzzer is one `.ail` file of about 10 lines; the Rust side
  owns the hard part (adversarial inputs, minimising, tiers, timeouts); a template
  can carry its own oracle (`FAIL ...` lines), so the same file is a property test.
- **Cons:** inputs are text, so binary data goes in as a hex string literal.

### Option B — one Rust generator per module
- **Pros:** typed, free-form programs.
- **Cons:** every lane edits Rust test code; merge conflicts; the program shape is the part that matters least.

### Option C — extend the typed program generator with module calls
- **Pros:** reuses the generator.
- **Cons:** the generator can't know an API that doesn't exist yet; random typed programs rarely reach a parser's corners.

## Decision

`bootstrap/tests/fuzz_modules.rs`, templates in `bootstrap/tests/fuzz/modules/`,
input generators in `fuzz/inputs.rs`. A template's header names `# fuzz-input:`
(`text`, `json`, `bytes`, `regex`, `date`, `http`, `compress`) and `# fuzz-needs:`
(a module, blank if built in; absent modules are skipped and reported).
`{{TEXT}}`, `{{TEXT2}}` and `{{INT}}` receive the input.

Properties: no crash (`fuzz.mod.1`); no hang within 10 s, so a regex engine must be
linear (`.2`); no output line starting `FAIL` (`.3`); a trap names a spec rule and
never `bootstrap.*` (`.4`); the bytecode tier equals the walker (`.5`); native
equals the walker once `ail build` exists (`.6`, ADR 0462); a run repeats (`.7`);
and the template must check (`.0`). A failing input is shrunk, saved with its
program, and `run.log` records seeds. `ail selftest --fuzz N [SEED]` runs this and
the program fuzzer (`fuzz.run.2`); the default smoke is 40 inputs per template.
