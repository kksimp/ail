# 0984. `std.app` `Font`: web fonts in the display list, kept by the host under an id

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.3` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md), [ADR 0905](0905-std-app-serif-font.md)

## Context

`Op.Str` names one of three families: `mono`, `sans` and `serif`. Garlic reads `@font-face` rules but can only draw the page's text in one of the three, so a page that sets its headings in a web font (html5test's condensed headings, icon fonts) looks different from Safari. Garlic fetches the font files itself, as it fetches pictures; the host can't reach the page's session.

## Decision

1. `Op.Font(id, data)` carries a TrueType or OpenType file in standard base64. The host keeps the font under `id`. An empty `data` keeps what `id` has. The op draws nothing.
2. A `Str` whose `font` is `web:` followed by an `id` is drawn, and measured (ADR 0839), in that font. A host that has no font under that `id`, or could not read the bytes, draws `sans`.
3. Like `Bitmap` (ADR 0854), a program sends the bytes once and the `id` alone afterwards.
4. The hosts register the font in memory for the process: `CTFontManagerCreateFontDescriptorFromData` on macOS, a temporary file loaded with fontconfig on Linux, a `PrivateFontCollection` on Windows. A style the file lacks (bold, italic) is not synthesised on macOS and Linux beyond what the platform does, and falls back to the regular face on Windows.
5. WOFF and WOFF2 are decoded by the program, not the host; the host only sees TrueType or OpenType.

## Rejected alternatives

- **Load by URL in the host (`web:https://...`).** Loses the page's session and cookies, as `Pic` does, and every host would need an HTTP client path for fonts.
- **A font name in `Str.font` with the host searching the system.** Web fonts aren't installed.
- **Send WOFF2 to the host.** Three hosts would each need a Brotli decoder.

## Consequences

A font file costs 4/3 of its size once per session in the pipe. The host never evicts. The Swift, C and C# sides are checked on the gates of their own machines only.

Tests use a real font, `tests/fonts/inconsolata-ascii.ttf` (Inconsolata, SIL OFL 1.1, subset to ASCII), so no test needs the network. Font-family lists are ADR 1005's; this ADR adds no second fallback path.
