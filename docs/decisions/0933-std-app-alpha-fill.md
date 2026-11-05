# 0933. `std.app` translucent fills

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.19` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)
- **Requested by:** Husk (the Mac disk-space app, `kksimp/husk`), whose GUI needs it. Not implemented yet.

## Context

`Op.Fill` is opaque, so a sheet cannot dim the window behind it and a treemap tile cannot show a selection tint over its color. Apps fake it with precomputed mixes, which fails over images and gradients.

## Decision

New op `Shade(x, y, w, h, color, alpha float)`: a filled rectangle drawn with opacity `alpha` (0 clear to 1 opaque, clamped), over what is already drawn. `Fill` keeps its fields unchanged. The macOS host draws it with a `CGContext` fill at that alpha. Hosts lag by a version: a host that does not know `Shade` skips it, so the host spec says unknown ops are skipped; Windows and Linux hosts must add it (Direct2D and cairo both support alpha) before the next release.

## Rejected alternatives

- **Add `alpha` to `Fill`.** Changes a published variant's shape in every host and every program.
- **Colors with an alpha byte (`0xAARRGGBB`).** Today `0x000000` is black and `0xff000000` is a valid number; reading the high byte would change old programs.
- **Skip it, mix colors.** Cannot dim an image or a bitmap.

## Consequences

Apps that use `Shade` look flat (the dim is missing) on an old host, but stay usable.
