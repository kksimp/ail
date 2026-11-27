# TBD. `std.barcode`: one-dimensional barcodes

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26 and 2026-11-27
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.barcode.1 to std.barcode.14 in [std-barcode.md](../spec/std-barcode.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)

The number is a placeholder: the integrator allocates it.

## Context

Labels, tickets and inventory programs print or read Code 128, EAN-13/UPC-A and 2 of 5 barcodes, and no std module makes
them. Agents write the pattern tables, the parity table of EAN-13 and the Code 128 subset switching from memory, and
the common mistakes (a wrong check digit, subset C used for an odd digit run, a missing stop element) give a barcode
that looks right and does not scan. It is pure computation, so it is AIL, in the API style of `std.qr` (ADR TBD, PR 1553).

## Decision

A new module written in AIL, `std/barcode/barcode.ail`, in scope without `use` (std.use.4). No syntax change, no
runtime or checker work.

- Encoders `code128`, `code39`, `ean13`, `upca`, `itf` return a `Barcode` (`kind`, `text`, `widths`): the alternating bar
  and space widths in modules, which is the form every renderer and every decoder works on. `pattern()` and
  `modules()` are views of it. EAN-13, UPC-A and ITF add or verify the GTIN check digit; Code 39 adds its mod-43 one
  on request.
- Code 128 picks subsets A, B and C itself (digit runs of four or more, or an all-digit text, in C). The result equals
  python-barcode's for every golden.
- `decode(widths)` reads Code 128, EAN-13/UPC-A, 2 of 5 and Code 39 from measured widths in any unit, either direction.
  Each symbol is rescaled to its own width, so no global unit estimate is needed; narrow and wide elements of 2 of 5
  and Code 39 are split per group by rank.
- Outputs: `to_text` (full blocks, height, quiet zone, label) and `to_svg` (one path of bars, escaped label).
- Failures are `BarcodeError` with an enum `kind` (`NoData`, `BadChar`, `BadLength`, `BadCheck`, `NotFound`)
  (type.enum.7).
- Verified before submission in Python: EAN-13, UPC-A, Code 39 (with and without check) and Code 128 bar patterns equal
  python-barcode's; all goldens, including 2 of 5, are read by a ZXing-based reader (zxing-cpp) as the same text; the
  decoder was run on 200 randomly rescaled, jittered (±10%) copies of each symbol. The AIL build runs at the Mac gate.

## Rejected alternatives

- **Rendering to a pixel image.** `std` has no image module; `widths` is what a PNG writer would draw.
- **Full-ASCII Code 39 and FNC1 to FNC4 in Code 128.** Rare, and each adds a shift table; left for when something needs
  GS1-128.
- **Accepting lower case in `code39`.** It would silently encode upper case and make `decode(encode(x))` differ from `x`;
  the error names the character.
- **One global unit estimate in `decode`.** Fails when the print scale drifts across the symbol; per-symbol rescaling
  is simpler and reads the same inputs.
- **A single `encode(kind, text)`.** One function with five different text rules hides which digit counts are valid;
  five names let the checker show each signature.
- **A module named `code`.** A common local name, which a local would hide (std.use.4).
