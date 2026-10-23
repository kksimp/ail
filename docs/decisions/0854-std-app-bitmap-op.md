# 0854. `std.app` `Bitmap`: pixels in the display list, kept by the host under an id

- **Status:** Accepted
- **Date:** 2026-10-22
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.3` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)

## Context

`Op.Pic` names an image file or URL and leaves loading and decoding to the host. Garlic fetches and decodes pictures itself (it needs their sizes for layout, and the decoders are AIL), and the host can't reach pages' pictures the way the browser does (cookies, referrers, `data:` URLs).

## Decision

1. `Op.Bitmap(x, y, w, h, id, pw, ph, data)` draws a `pw` by `ph` image scaled to the rectangle. `data` is its `R, G, B, A` bytes (non-premultiplied) in standard base64; `app.bitmap_data(rgba)` makes it.
2. The host keeps each image under `id` (as a premultiplied `CGImage` in the macOS host). An empty `data` draws the image already sent under that `id`; an unknown `id` draws a placeholder.
3. A program sends an image's pixels once and the `id` alone afterwards, because a view line is sent whenever it changes and scrolling changes it on every tick. Garlic does this with `Gui.sent` and `Gui.fresh`.
4. `Pic` stays for images the host can load itself, and is Garlic's fallback when a picture can't be decoded.

## Rejected alternatives

- **Send the pixels in every view.** A scroll tick would resend megabytes.
- **A separate binary channel or file for pixels.** A second transport for one op; the JSON line already carries it, and base64 is native in both modes.
- **Raw hex in place of base64.** Twice the bytes for the same code size.
- **Let the host fetch and decode (`Pic` only).** Duplicates the decoders in Swift and loses the page's session.

## Consequences

Large pictures cost 4/3 of their RGBA size once per session in the pipe. Garlic caps a decoded picture at 4 million pixels. The host never evicts; a long session with many pictures grows. The Swift side is checked on the Mac gate only.
