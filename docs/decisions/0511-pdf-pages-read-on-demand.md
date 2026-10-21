# 0511. The PDF reader reads pages on demand

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** Designer (direction: speed, and agents knowing what is on screen; card "Speed and lazy first, then OCR", 2026-10-20), details AI (delegated, ADR 0076)
- **Topic:** Tooling
- **Spec rules:** none (a tool, not language behaviour)
- **Files:** `tools/pdf/reader/model.ail`, `tools/pdf/pdf.ail`, `tools/pdf/view/view.ail`, [design note](../design/pdf-viewer.md#only-what-is-asked-for-is-read-adr-0511)

## Context

ADR 0520's reader parsed every page before doing anything: 2.9 s on a
53-page document before the viewer showed page one. The designer set the
direction on 2026-10-20: AIL's strength should be speed and agents
understanding the words on screen, since other readers process the whole
document first.

## Options considered

### Option A — lazy pages, streaming search (chosen)
`Doc.lazy_pages()` reads the page tree; `Pages.get(i)` reads and caches one
page. One-page commands read one page; the viewer reads what it shows and
searches forward page by page; `grep` prints per page and `--first` stops
early; the viewer's `--json` lists the lines visible on screen.
- **Pros:** opening and showing any page of the 53-page file takes 0.18 s
  instead of 3 s; an agent asking "what is on screen" or "is X in here" pays
  for the pages it touches only.
- **Cons:** the viewer's match count is per page ("match 1 of 3 on page
  7"), not a document total, unless every page is searched.

### Option B — read everything, faster
- **Cons:** still linear in the document for every question; parallel
  reading (next step) helps both options, so it is not an alternative to A.

### Option C — an index file written on first open
- **Cons:** writes beside the user's files or into a cache directory;
  stale-index handling; not needed while a page reads in ~55 ms.

## Decision

Option A. Criterion 4 (speed) and criterion 2 (an agent gets the visible
words directly) both favour it; it costs no tokens.

## Consequences

- `Pages` holds a `ref` cache, so page reading stays on one task; reading
  pages on several cores needs the object cache restructured (design note,
  next slices).
- OCR, when it comes, follows the same rule: only pages shown or searched
  are recognised.

## Rejected alternatives

- Reading everything, faster (option B): rejected because it stays linear in the document for every question. Parallel reading helps both options, so it is not an alternative to A.
- An index file written on first open (option C): rejected for now because it writes beside the user's files or into a cache directory, needs stale-index handling, and is not needed while a page reads in about 55 ms.
