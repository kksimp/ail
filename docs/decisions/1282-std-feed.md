# 1282. `std.feed`: read RSS and Atom feeds, write RSS, Atom and JSON Feed

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 17 — Standard library
- **Spec rules:** [std.feed.1 to std.feed.16](../spec/std-feed.md)

## Context

Feeds are the commonest XML an agent meets after config files: blogs, podcasts, release notes and news. `std.xml`
(ADR 1212) reads the text but leaves every format's field names, date styles and quirks to the caller. The std-gap
plan lists a feed parser as a follow-up on top of `std.xml`.

## Decision

- A new module `std.feed`, written in AIL (`std/feed/feed.ail`), pure, built on `std.xml`, `Json` and `std.time.date`.
  Fresh code.
- One model for four formats: `Feed` and `Item` structs with text fields that are `""` when absent, optional
  `DateTime`s, `tags` and `Enclosure`s. `parse(text) Feed ! FeedError` detects the format from the first character
  (`{` is JSON Feed) or the XML root (`rss`, `rdf:RDF`, `feed`). `Feed.format` records the result.
- Lenient content, strict shape: a missing field or an unreadable date inside a feed is empty or `none`; a text that
  is not a feed fails with `FeedError` kind `Xml`, `Json`, `Unknown` or `Date`.
- `parse_date` reads RFC 822/2822 (optional weekday, 2-digit years, zone names, `+hhmm`) and ISO 8601 with an offset
  (through `date.parse_iso`), keeping the text's offset.
- Writers: `write_rss` (RSS 2.0, with `content:encoded`), `write_atom` (Atom 1.0) and `write_json` (JSON Feed 1.1),
  each reading back to the same fields. The XML writers use `xml.write_doc`, so escaping is the module's.
- `new_feed(title)` and `new_item(title, link)` start a value to fill in; fields are plain `pub` fields.

## Rejected alternatives

- **One enum of per-format feed types.** Callers would match on four shapes to print a title; the shared fields are
  what programs use.
- **Failing on a bad date or a missing title.** Real feeds are sloppy, and a reader that rejects a feed over one
  `pubDate` is useless; the fields are `none`, and `parse_date` is available when a caller wants the error.
- **Namespace resolution.** `std.xml` keeps prefixes (`std.xml.14`); matching the common prefixes (`dc:`, `content:`,
  `rdf:`) covers the feeds seen in practice. A feed under other prefixes loses those fields (`std.feed.14`).
- **Reading iTunes and Media RSS extensions.** Large surface, rarely needed by agents; enclosures cover audio and
  video links. A follow-up can add fields.
- **Fetching in the module.** `net.get` plus `feed.parse` is two lines, and keeping the module pure keeps it free of
  capabilities.
- **Entries inheriting the feed's author.** The rule differs between formats and hides where a name came from;
  callers can fall back with `it.author` and `f.author`.

## Consequences

One new file under `std/`, one spec chapter, registration lines in the compiler's module lists, tests
`tests/cases/std_feed*.ail`. No syntax change and no runtime or checker change. The golden outputs were written by
hand without a build; if the gate shows a difference, the module is the thing to fix toward the spec.
