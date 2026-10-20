# 0169. One canonical layout, written by `ail fmt`, with a 100-column limit

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20)
- **Topic:** 2 — Core syntax (layout; roadmap refinement 4: formatter in Stage 2)
- **Spec rules:** none (formatting is non-normative); style rules `fmt.*` in [formatting.md](../design/formatting.md)
- **Evidence:** `tools/tokbench/run.sh` on `examples/*.ail` before and after formatting (below)

## Context

Roadmap refinement 4 asks for the formatter in Stage 2, so that every later
example is canonical. Layout that a person or an AI can choose costs tokens to
decide and makes diffs noisy. ADRs 0040 (4-space blocks), 0083 (attributes on
the declaration's line) and 0118 (trailing commas on multi-line lists) already
fix parts of the layout; the rest was open.

## Deciding criterion

**Token usage ties** (under ADR 0076's ~2% threshold): formatting the seven
examples costs **+0.41%** tokens (mean of five tokenizers, 14,988.8 to
15,050.0). Per file: `json.ail` +0.51%, `sheet.ail` +0.57%, `interp.ail`
+0.30%, the other four 0. The cost is almost all blank lines between
multi-line declarations, which `json.ail` and `sheet.ail` didn't have.
**AI understanding decides**: one layout per program means an AI never chooses
one, every file it reads looks the same, and edits produce minimal diffs.
**Readability** then decides the blank lines.

## Options considered

### Option A — fully canonical: the formatter decides every break (chosen)
- **Pros:** one form per token sequence; nothing for an AI to decide; short
  lists written vertically join, long ones split, the same way everywhere.
- **Cons:** a hand-made vertical layout of a short list is lost (a comment
  inside the statement keeps it, see fmt.comment).

### Option B — preserve the source's line breaks and blank lines (gofmt)
- **Pros:** zero token cost on the examples; respects deliberate layout.
- **Cons:** two spellings of most statements; an AI still spends tokens
  deciding layout, which is the thing the formatter exists to remove.

### Option C — "magic trailing comma" (Black): a trailing comma keeps a list split
- **Pros:** an opt-in to vertical lists without comments.
- **Cons:** a layout choice again, signalled by a token whose absence or
  presence is itself a choice.

## Decision

`ail fmt` writes one canonical layout, defined rule by rule in
[formatting.md](../design/formatting.md):
1. 4-space blocks; continuation lines indented by bracket depth (fmt.indent).
2. One space between tokens except around `.`, `..`, inside brackets, before
   `,` `:` `?`, calls and indexes, and after prefix `-` and `!` (fmt.space).
3. **Line limit 100 characters.** A statement that fits goes on one line;
   otherwise its longest top-level list splits one element per line, hugging
   a final inner list that holds most of the text (fmt.wrap).
4. Trailing comma exactly when a list is split (fmt.comma, ADR 0118).
5. One blank line between top-level declarations when either spans several
   lines; elsewhere blank lines from the source are kept, collapsed to one
   (fmt.blank).
6. Every comment stays on its line; comment text is never changed
   (fmt.comment). Trailing comments follow one space.
7. Attributes stay on the declaration's line (fmt.attr, ADR 0083).
8. Never align anything in columns.

## Rationale

The token cost is a tie and is paid only where a file lacked blank lines
between multi-line declarations; those blank lines are what makes a 400-line
file scannable by a person, and they are free under `cl100k` and `o200k`
(+3 tokens in `json.ail`). 100 columns was picked, not measured: it fits every
example's longest statement but three, and matches common editor and review
widths. Alignment is rejected because one renamed identifier re-aligns a
whole block, which costs tokens in every diff.

## Consequences

- Examples and test cases are canonical; `ail fmt --check examples
  bootstrap/tests/cases` passes and a test enforces it.
- Blank lines between statements in a block remain an author's choice
  (collapsed to one). A later batch could measure making them canonical.
- Long lines with no list to split (long strings, long binary expressions)
  stay long for now ([formatting.md](../design/formatting.md), "Not yet done").

## Rejected alternatives

- **Preserve the source's breaks (B):** leaves layout as a choice, the cost
  the formatter exists to remove.
- **Magic trailing comma (C):** a layout choice encoded in a token.
- **Zero blank lines everywhere:** cheapest by 0.4%, but a tie on tokens and
  worse for readability, the next criterion that differs.
- **Two spaces before a trailing comment (PEP 8):** no reader benefit; one
  space is the same in every other gap.
- **80 or 120 columns:** 80 splits many ordinary calls with interpolated
  strings; 120 lets lines grow past what side-by-side review shows.
