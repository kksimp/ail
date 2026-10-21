# 0711. Agent programs that drive AIL's own tools, and how their twins are measured

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; lane 224)
- **Topic:** Measurement (agent corpus round 5)
- **Spec rules:** none; this is measurement convention, not language behaviour.

## Context

Rounds 1 to 4 measured programs that call libraries. Agents also use AIL's own tools
(`check`, `explain`, `api`, `diff`, `edit`, `fix`, `fmt`, `test`, `effects`, `run --record`
and `--replay`, Garlic, the PDF reader). A measurement of those needs a convention
for what the Python twin does and for what counts as the same output.

## Decision

1. **The twin calls the same tool.** A Python twin cannot print the bytes of `ail diff`
   or `ail check` without reimplementing them, so it runs the same `ail` binary through
   `subprocess` and does the processing itself. The program measures the processing
   around the call, not the tool.
2. **Output must match byte for byte.** Each pair is checked with `cmp` against the
   program's `.out`. Output contains no absolute paths, and the PDF and Garlic commands
   run with fixed settings (`--print`, `--width`).
3. **Line splitting is newline-only.** AIL's `lines()` splits on `\n`. Python's
   `str.splitlines()` also splits on form feeds, so the twins use a `lines_of()` helper
   that splits on `\n` only.
4. **Booleans print as `true` and `false`,** as AIL prints them.
5. **Fixtures are committed** under `tools/tokbench/corpus/agent-r5/data/`, outside the
   `examples/` tree, so that the `fmt --check` gate does not reject deliberately broken
   or unformatted files.
6. **Each twin carries only the helpers and imports it uses.** The shared header
   added about 187 tokens to every twin; the reported numbers use the trimmed twins.

## Token figures

tokbench means, 20 programs. Programs alone: AIL 3,490, Python 5,340 (−34.7%).
Both pay (card 7,044 and reference 6,813 each counted once): AIL 10,533, Python 12,153
(−13.3%). Full table in [the round 5 measurements](../design/program-measurements-agent-r5.md).

## Rejected alternatives

- **Reimplement `ail check`, `diff` or `fmt` in Python** to get a twin with its own
  output format. Rejected: the twin would then measure a second implementation, and
  the output could not be byte-identical with confidence.
- **Use `ail diff --rev`.** Rejected: its output depends on the git history, so the
  expected output would change with each commit.
- **Keep the full shared header on every twin.** Rejected: it counts boilerplate
  against Python that a short twin would not carry.

## Consequences

- Programs that print a count of spec rules (`explain_chapters`) change expected output
  when a rule is added. The change must regenerate the `.out` file in the same commit.
- The twins are not the shortest possible Python; the measurement is of idiomatic code.
