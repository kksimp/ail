# 0691. Fuzzers for `regex.group_all` and native `bytes`

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 16 — Testing
- **Spec rules:** std.regex.24, type.bytes.1-4, native.build.1, fuzz.mod.1-7
- **Builds on:** [ADR 0461](0461-module-fuzzers.md), [ADR 0667](0667-regex-group-all.md), [ADR 0702](0702-native-bytes.md)

## Decision

1. `regex_group_all.ail` (input kind `regex`): group 0 equals the match texts of `find_all`; every group has one entry per match; each entry is part of the haystack; a group above the last or below 0 traps with a rule ID.
2. `bytes_native.ail` (input kind `bytes`, `fuzz-native: yes`): the two hex inputs become `b"\xHH..."` literals through two new placeholders, `{{BLIT}}` and `{{BLIT2}}`, so the native build gets bytes it can't read from text. The program prints length, equality, hex, lossy text length, slices and bytes read at an arbitrary index. `ail build` output must match `ail run`, traps included (fuzz.mod.6).

## Results

`group_all`: 1,500 inputs, 0 failures. Native bytes, on a merge with the native bytes PR: 300 programs built and compared, 0 differences.

## Rejected

- Fuzzing native bytes through `bytes.from_hex`: it isn't in the native subset, so no program would build.
