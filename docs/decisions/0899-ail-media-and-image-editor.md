# 0899. A media player or image editor, as a showcase program

- **Status:** Deferred past 0.1.0
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, chose the program on a decision card); the details by AI (delegated, ADR 0076), for integrator review
- **Builds on:** [ADR 0838](0838-std-app-native-windows.md), [ADR 0878](0878-screen-model-for-std-app.md), [ADR 0854](0854-std-app-bitmap-op.md), [ADR 0893](0893-ail-try.md)
- **Topic:** 24 — Showcase programs

## Context

The speed claims for SIMD, multi-core and tensors are measured on small race programs. A visual program makes them visible: a filter that stays at frame rate, with an agent driving the same program through the screen model.

## Decision

- Build `tools/photo`: an image editor on `std.app` (`Bitmap` ops, ADR 0854). It opens PNG and JPEG (decoders from `tools/pdf/image`), and offers crop, levels, blur, sharpen, resize and a few filters, each an AIL function over tensors or packed pixels that the compiler vectorizes and spreads across cores with no thread code in the source.
- Edits are a non-destructive list of steps, so undo is replay and an agent reads the list through the screen model.
- A benchmark row compares each filter with a C version at -O2 and -O3 and with NumPy where it applies.
- It shows: automatic SIMD and multi-core; the screen model and the inbox (an agent edits while a person watches); capabilities (the program may read only the folder it was given); and `ail try` for agent edits of the program itself.
- A media player (decode and play) is a later extension, after the editor.

## Consequences

The decoders already exist in `tools/pdf/image`; they move to a shared module (planned as `std.ui.image`) when this lands. Video and audio are out of scope for the first version.

## Rejected alternatives

- **A video player first:** needs codecs and audio output; the image editor shows the same speed claims sooner.
- **A game:** `std.gfx` is only a sketch; the editor builds on what has shipped.
- **A command-line image tool only:** nothing to see, and the screen model goes unused.
