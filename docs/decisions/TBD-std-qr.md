# TBD. `std.qr`: QR Code Model 2 encoder

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26 and 2026-11-27
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.qr.1 to std.qr.12 in [std-qr.md](../spec/std-qr.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)

The number is a placeholder: the integrator allocates it.

## Context

Programs that print a login or pairing link, a Wi-Fi share code or a ticket need a QR code, and no std module makes one.
Agents write the Reed-Solomon, the format bits and the mask scoring from memory, and the common mistakes (wrong block
interleave, wrong BCH check, a mask chosen by taste) give a symbol that looks right and does not scan. The std plan lists
it in the refill backlog after the in-house crypto and compression lanes; it is pure computation, so it is AIL.

## Decision

A new module written in AIL, `std/qr/qr.ail`, in scope without `use` (std.use.4). No syntax change, no runtime or
checker work.

- `qr.encode(text, level, min_version) Code ! QrError` picks numeric, alphanumeric or UTF-8 byte mode by the text, the
  smallest version from `min_version` that holds it, and the best of the eight masks (Nayuki's scoring, ties to the
  lower mask). `encode_bytes` is byte mode over raw bytes. `level` is an enum `Level`, default `Medium`.
- `Code` holds `version`, `size`, `level`, `mask` and the matrix, read with `dark(x, y)` (false outside, so a quiet zone
  is plain arithmetic).
- Outputs: `to_text` (half blocks, two rows to a line, quiet zone, `invert`) and `to_svg` (one path of row runs).
- Failures are `QrError` with an enum `kind` (`NoData`, `TooLong`, `BadVersion`) (type.enum.7).
- Segments are not mixed: the whole text is one mode. The matrix was checked against two independent reference encoders
  in Python (all levels, three modes, versions 1 to 36, auto and fixed masks); the AIL build and the goldens run at the
  Mac gate.

No PNG: `std.image` is not on main. When it lands, `to_png` is a small follow-up on `dark`.

## Rejected alternatives

- **Mixed-mode segments (optimal split of digits, letters and bytes).** Saves a few percent of modules on mixed text; the
  planner is 100 lines and the one-mode result is always valid. Left for when a measurement asks for it.
- **A built-in runtime function.** The work is a few thousand integer operations per code; AIL is fast enough and avoids
  checker, effects and lowering rows per function.
- **Returning the matrix as `List[List[bool]]`.** A caller can mutate it into an invalid symbol; the opaque `Code` with
  `dark` keeps the invariant. A `rows` accessor can be added when asked.
- **An ECI header for non-ASCII text.** Many scanners then show the text wrongly; plain byte mode with UTF-8 is what
  phone cameras expect.
- **A module named `code`.** A common local name; `qr` clashes with two local names in the tests, and a local may hide a
  module (std.use.4).
