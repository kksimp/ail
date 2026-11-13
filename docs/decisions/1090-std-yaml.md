# 1090. `std.yaml`: the common subset of YAML read into the `Json` value

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** std.yaml.1-10 ([17-std](../spec/17-std.md))

## Context

The AI-first review (item 17) found that agents reach for YAML first: CI files,
Kubernetes manifests, OpenAPI documents and tool configs are YAML. `std.toml`
(ADR 0901) covers one config format; a program that reads YAML today
hand-writes a parser.

## Decision

`std.yaml`, written in AIL (`std/yaml/yaml.ail`), imported with `use std.yaml`:
`parse(text) Json ! YamlError`. It reads the subset real config files use, into
the existing `Json` value, so `get`, `at`, `str`, `num` and `to_text` apply as for
`std.toml`. Read only; pure AIL, so no lowering or runtime change.

In the subset: block maps and lists by indentation, flow lists and maps (they may
span lines), plain, `'single'` and `"double"` quoted scalars, `|` and `>` block
strings with `-` and `+` chomping, comments, `---` and `...` around one document.
Scalars follow the YAML 1.2 core schema: `null`/`~`/empty, `true`/`false`,
decimal, `0x`, `0o` and float numbers; `yes`, `no`, `on` and `off` stay strings, so
`country: no` is the string "no". Keys are strings.

Refused with an error that says what to write instead (never read wrongly):
anchors, aliases, tags, merge keys (`<<`), complex keys (`?`), directives,
several documents, tab indentation, explicit block indentation indicators,
multi-line plain or quoted scalars, and a map started on its key's line
(`a: b: c`). `YamlError` has `why str` and `line int`.

## Rejected alternatives

- **Full YAML 1.2.** Anchors and tags need a second value model and a schema
  layer; the spec is large and parsers disagree on edge cases. Cost: a program using
  them gets an error. An AI reading the error rewrites the value out in full.
- **YAML 1.1 booleans (`yes`/`no`).** Reads `country: no` as false (the Norway
  problem); 1.2 core is the safer reading.
- **Silently accepting what is not understood** (folding anchors as text, treating
  tabs as spaces). Wrong data with no error costs more than a refusal.
- **A writer (`yaml.write`).** Deferred, as for TOML.
- **A Rust or native-runtime parser.** Against ADR 0873; pure AIL builds on every target.

## Measurements

None: tokbench has no parser twin for YAML; open for a later change.
