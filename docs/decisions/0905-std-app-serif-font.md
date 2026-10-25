# 0905. `std.app` text family `serif`, and Garlic's font-family mapping

- **Status:** Accepted
- **Date:** 2026-10-25
- **Decided by:** Designer (2026-10-25 14:48 UTC: "Yes, do it now" on the serif-fonts decision card in the Garlic thread)
- **Topic:** std.app
- **Spec rules:** std.app.3, std.app.13 ([std-app](../spec/std-app.md)); amends [ADR 0838](0838-std-app-native-windows.md) and [ADR 0839](0839-std-app-text-measure.md)

## Context

`Str` and `Measure` carried `font` as `mono` or `sans`, and every host drew
anything else as monospace. Pages that name Georgia, Times or `serif` therefore
rendered in the sans face.

## Decision

- `font` is `mono`, `sans` or `serif` (std.app.3, std.app.13). The field was
  already a string, so the AIL side needs no change.
- Hosts: macOS uses the serif design of the system font, Linux uses Pango
  `Serif`, Windows uses Georgia.
- Garlic's layout carries the family name in its measuring probes instead of a
  `mono` flag (`gfx.font_of`). A `font-family` list that names `mono`, `courier`
  or `consolas` anywhere is mono; otherwise the first known face decides
  (Georgia, Times, Garamond, Palatino, Cambria, Book Antiqua, Baskerville or
  `serif` is serif; Arial, Helvetica, Verdana, Tahoma, Roboto, Inter, Segoe UI,
  `system-ui`, `-apple-system` or `sans-serif` is sans). Unknown or missing
  stays sans, so pages without a family look as before.

## Rejected alternatives

- **A fixed serif face per host named by the app.** Hosts have different
  installed fonts; the generic family keeps apps portable.
- **Serif by default, as browsers do.** Garlic's default look is sans; only
  pages that ask for serif get it.
- **Guessing from the face name in the host.** Garlic knows the CSS list; the
  host only maps three families.
