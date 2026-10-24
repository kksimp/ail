# 0878. The screen model: what every std.app program shows, for agents

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** Designer (2026-10-24, "Approve; attach on in ail run")
- **Topic:** 20 — AI semantic API; std.app
- **Builds on:** [ADR 0868](0868-ai-first-tooling-after-rust-removal.md)

## Context

Agents need to know what a GUI program shows and what state it holds without a vision
model or scraping. Garlic's AI-first plan (designer, 2026-10-23) proposed both. Neither
may cost anything when no agent is attached.

## Decision

What an agent gets: "button 'Submit', red, at 412,300 100x32, enabled, links to /pay", without a vision model.

1. Where it lives: in std.app and its host, not only in Garlic. Every std.app program (Garlic, the editor, the PDF viewer, user apps) gets the same model, because the host already receives the view tree and the drawing ops.
2. The display list comes first. Each drawn box already knows its node, role, text, colour and size, so the model joins them for a few bytes per node.
3. Pixels only where the tree is blind: canvas, images and video frames. A native segmenter finds flat-colour shapes, contrast and rounded corners, and takes a label from the nearest text. Each guess carries a confidence and its evidence ("probably a button, 0.8: rounded red rectangle with text 'Pay'").
4. Answers on demand: `what_is_at(x, y)`, `find(role | colour | text)`, `describe(region)` and `changes_since(t)`. Nothing streams.
5. Token budget: every call takes `budget: N` and returns the most important items first (focused, interactive, visible, large), plus a count of the rest ("+37 more"). Nothing is cut silently.
6. Stable ids across frames. A repeat call on an unchanged frame is answered from a cache.
7. Colours by name, from a fixed palette ("red", "dark red"), so themes, dark mode and anti-aliasing don't change the answer.
8. Watches are diffs of the model ("tell me when this box turns green"), not pixel polling.
9. Slices: Garlic first; then the pixel classifier on the editor and the PDF viewer, which are programs we own and can test; then std.app for all programs.

## Rejected alternatives

- A vision model over screenshots: slow and token-heavy, and it guesses what the program already knows.
- Streaming every frame: costs tokens whether or not anyone asks.
