# Garlic: a web browser written in AIL

> **Non-normative design note**, 2026-10-20. The designer asked for a browser
> written entirely in AIL, named Garlic, that the browser itself or an AI agent
> driving it is "hyper-aware of what's on screen". This note covers the
> architecture, the first scope, the agent-facing page model and the
> dependencies. Code lives in [`tools/garlic/`](../../tools/garlic/).

## Name

A preliminary search (2026-10-20, web search only: the USPTO, WIPO, EUIPO
registers and domain checks were unreachable from the build container) found
no web browser named Garlic and no software trademark for the word. Other
software uses it: GarlicOS (a retro-handheld OS), an AI advertising platform,
restaurant-stock and recipe apps, and a molecular viewer packaged in Debian.
"Garlic routing" is I2P's anonymity technique. This is **not legal
clearance**. The designer chose (2026-10-20) to keep the name with no
further check; it stays an internal tool until they decide otherwise.

## Goals

1. **Written in AIL**, on the standard library, no third-party engine
   (ADR 0009, ADR 0010). Ported algorithms and test data must be
   Apache-2.0-compatible (ADR 0510); the HTML and CSS algorithms are written
   from the WHATWG and W3C specifications.
2. **Agent-first.** Every frame the browser draws is also a structured page
   model an agent can read and act on, with no screenshots and no OCR: the
   browser knows what it drew and where, because it drew it.
3. **Useful early.** Each milestone is a browser someone would use for
   something.

## Architecture

The pipeline is the classic one, with one addition: the layout output is kept
as data and exposed, not only painted.

```
url ─► fetch (std.net) ─► bytes ─► decode ─► HTML tokenizer ─► tree builder ─► DOM
                                                                      │
                     CSS (UA sheet, <style>, style=) ─► cascade ──────┤
                                                                      ▼
                                   layout (boxes, lines, positions) ─► paint ─► std.ui window
                                                  │
                                                  └─► page model (roles, names, rects,
                                                      actions) ─► agent: --json, panel
```

| Stage | Module | First milestone | Later |
|---|---|---|---|
| Fetch | `std.net` | `net.get` over in-house TLS 1.3 (ADR 0360), redirects followed by Garlic (status 301/302/303/307/308 + `location`) | methods, headers, cookies and POST forms (lane 124), `Accept-Encoding: gzip` (lane 128), bytes bodies for images (lane 123), HTTP/2 |
| Decode | `str` | UTF-8 only (std.net bodies are `str`) | charset sniffing once bodies are bytes |
| Tokenize | `html` | WHATWG tokenizer: data, tags, attributes (quoted, unquoted, bare), comments, doctype, raw text (`script`, `style`), escapable raw text (`title`, `textarea`), character references (numeric and a named subset) | the full named-reference table, every parse-error state |
| Tree | `html` | simplified tree construction: implied `html`/`head`/`body`, void elements, implied end tags (`p`, `li`, `dt`, `dd`, `option`, `tr`, `td`, `th`), end tags close up to the matching open element | adoption agency, foster parenting, templates, foreign content (SVG, MathML) |
| Style | `style` (later) | a fixed user-agent model: block vs inline elements, headings, `pre`, hidden elements (`head`, `script`, `style`, `[hidden]`) | a CSS tokenizer and parser, selectors, cascade, computed values; `display`, `color`, `font-weight`, `margin` first |
| Layout | `page` | text cells: block flow, inline runs, whitespace collapsing, word wrap to the window's width, lists, block quotes, `pre`, simple tables | CSS box model on the cell grid, then pixels (needs a pixel layer) |
| Paint | `std.ui` | the text-cell window (`put_styled`), colors per run, link highlighting | pixels, images, fonts (see Dependencies) |
| Script | none | no JavaScript (decision pending with the designer) | see below |

### Why text cells first

`std.ui` is a text-cell window today (ADR 0177) and no lane covers pixels,
images or font rendering. Text-mode browsers (Lynx, w3m) show that a
cell-grid browser is genuinely useful for documentation, Wikipedia, news,
search results and forms. Rendering into cells also keeps layout exact and
testable: a page renders to a list of text rows that a test compares, the
way the editor's screen does (ADR 0181).

## First milestone (built with this note)

`./ail run tools/garlic URL|FILE`:

- **Fetch and follow**: `http`, `https` and local files; up to 5 redirects;
  relative links resolved against the page URL (RFC 3986 merge).
- **Read**: headings, paragraphs, lists, block quotes, `pre`, horizontal
  rules, simple tables and inline emphasis, wrapped to the window width.
  Links are numbered `[3]`, and the title is in the window title.
- **Navigate**: Tab and Shift-Tab move between links, Enter follows,
  Backspace goes back, Up/Down/PageUp/PageDown/Home/End scroll, `g` opens
  a URL, `/` finds text, `q` quits.
- **Headless**: `--print` writes the rendered page as text; `--json` writes
  the page model (below); `--width N` sets the width. Tests run headless.

## The page model for agents

An agent driving Garlic gets what the browser knows, not pixels. The model is
the DOM reduced to what a user perceives, like an accessibility tree, plus
where each thing was drawn:

```json
{"api":1,"url":"https://example.com/","title":"Example Domain","width":80,"height":24,"scroll":0,"rows":31,
 "nodes":[
  {"id":4,"role":"heading","level":1,"name":"Example Domain","row":0,"col":0,"w":14,"h":1,"visible":true},
  {"id":9,"role":"link","name":"More information...","href":"https://www.iana.org/domains/example","row":5,"col":0,"w":24,"h":1,"visible":true,"actions":["follow"]}
 ]}
```

| Key | Meaning |
|---|---|
| `id` | the DOM node's index; stable for one load of a page |
| `role` | `heading`, `paragraph`, `link`, `list`, `listitem`, `blockquote`, `code`, `table`, `row`, `cell`, `image`, `button`, `textbox`, `separator` (ARIA role names where one exists; an explicit `role=` attribute wins) |
| `name` | the accessible name: text content, `aria-label`, `alt` or `title` |
| `row`, `col`, `w`, `h` | where it is in the rendered document, in cells; 0-based |
| `visible` | whether any of it is in the current viewport |
| `href`, `level` | links' resolved targets; headings' levels |
| `actions` | what an agent can do: `follow` (links), later `type`, `press`, `select` |

The same model drives the window (Tab goes to the next `link` node), so what
an agent sees is exactly what a person sees. The design follows `ail api`
(versioned `"api":1`, byte-stable, one schema) and the editor's agent panel
(ADR 0312): the program reports its own state instead of the agent scraping it.

Next steps for the agent angle, in order:

1. **Actions in, not only state out**: `--do follow:9`, `--do find:text`,
   `--do scroll:20`, so an agent drives Garlic headless like the editor's
   `--print --key=` (one process per step, deterministic).
2. **A session mode**: JSON lines on stdin and stdout (`{"do":"follow","id":9}`
   in, the new page model out), so an agent keeps one browser open.
3. **Diffs**: after an action, report which nodes appeared, changed or left
   the viewport, rather than the whole page; this saves tokens on long pages.
4. **Token budgets**: `--budget N` trims the model (viewport first, then
   headings and links, then text) like `ail api --card --budget`.
5. **Provenance**: each node keeps its source position in the HTML, so an
   agent can tell markup from rendering problems.

The same idea gives the designer's PDF viewer its "search without OCR": a
PDF's text operators already carry the text and its position, so a viewer
that keeps them as data (like this page model) can search and point at
text directly. Scanned image-only PDFs still need OCR; that limit is in the
file, not the viewer.

## JavaScript

JavaScript is a separate, large decision (the engine is bigger than the
rest of the browser). The designer chose option 2 on 2026-10-20: **start an
in-house engine in AIL now**, as a parallel lane under `tools/garlic/js/`
(brief: lane 160). The options were:

1. **No JS now; an in-house engine in AIL later** (recommended): text and
   reader mode work on server-rendered pages; a JS lane (lexer, parser,
   bytecode interpreter, then the DOM bindings) starts once DOM and layout
   are solid.
2. **Start an engine now**, in parallel.
3. **QuickJS through FFI** as a stopgap (MIT, so licence-compatible), once
   lane 126 lands; costs ADR 0009's custom-by-default rule and a C dependency.
4. **Never run page JavaScript**: a no-script browser by design.

Until the engine lands, Garlic doesn't run scripts and renders `<noscript>`
content. The engine's DOM bindings work on `html.Doc`, and script changes to
the page show up in the page model so an agent can see what a script did.

## Dependencies

| Need | Where it stands | Blocks |
|---|---|---|
| HTTP GET over TLS 1.3 | done (`std.net`, ADR 0360) | nothing |
| Request headers, methods, cookies | lane 124 | logins, forms with POST, sites that check `User-Agent` |
| Binary bodies | lane 123 | images, non-UTF-8 pages, fonts |
| gzip/deflate | lane 128 | `Accept-Encoding`, smaller transfers |
| Regex | lane 127 | nothing essential (CSS and HTML are hand-written state machines) |
| Pixel drawing, images, fonts | **no lane** | graphical rendering. Needs a `std.ui` pixel surface (blit an RGBA buffer), PNG/JPEG decoders (PNG needs deflate), and text shaping and rasterization (a bitmap font first, then TrueType outlines) |
| JS engine | parallel lane (lane 160) | dynamic sites |
| Native backend (lane 120) | in progress | speed: today Garlic runs on the bytecode tier |

## Milestones

1. **Text mode** (this note): fetch, parse, wrap, links, history, page model.
2. **Forms and agent actions**: inputs, buttons, GET forms; `--do` and the
   session mode; needs lane 124 for POST and cookies.
3. **CSS on cells**: a CSS parser, selectors, cascade; `display`, colors,
   bold, margins, `display:none`; real tables.
4. **Pixels**: a pixel surface in `std.ui`, a bitmap font, then images (PNG
   after lane 128) and a box-model layout in pixels.
5. **Scripts**: the JS engine's DOM bindings and event loop.

## Rejected alternatives

- **Embedding an existing engine** (WebKit, Servo, Ladybird's LibWeb): C++ or
  Rust, contradicts ADR 0009 and the goal, and hides the page from the agent
  behind a foreign API.
- **Screenshot plus OCR for agent awareness**: lossy and slow; the browser
  already has the exact text and positions.
- **Pixels first**: blocked on a pixel layer, images and fonts that no lane
  covers; text mode delivers a usable browser now and its layout and page
  model carry over.
- **Exposing the raw DOM to agents**: far more tokens than the perceivable
  page (scripts, styles, wrappers), and no positions.
