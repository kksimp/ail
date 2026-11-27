# Standard library: `std.feed`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR TBD). Reads RSS 2.0, RSS 1.0, Atom 1.0 and JSON Feed
> into one model, and writes RSS 2.0, Atom 1.0 and JSON Feed. Built on `std.xml` and `Json`. Fetching a feed
> (`net.get`), conditional requests, PubSubHubbub, namespaces beyond the prefixes below and OPML are out of scope
> (`std.feed.14`).

## Reading

**std.feed.1** — `use std.feed` provides `parse(text str) Feed ! FeedError`, which reads one feed in any of the four formats. If the first non-blank character of the text is `{`, it is JSON Feed (`std.feed.7`). Otherwise the text is XML (`std.xml.1`) and the root element decides: `rss` is RSS 2.0 (`std.feed.4`), `rdf:RDF` or `RDF` is RSS 1.0 (`std.feed.5`), `feed` is Atom (`std.feed.6`). Any other root fails with kind `Unknown`. The module is pure (`eff.infer.1`): it has no effects and needs no capability. *(ADR TBD)*

```ail rules=std.feed.1
fn main() !
    src := "<rss version=\"2.0\"><channel><title>T</title><link>https://e.org/</link><item><title>A</title><link>https://e.org/a</link></item></channel></rss>"
    f := feed.parse(src)?
    print("{f.format.name()} {f.title} {f.items.len()}")
```

**std.feed.2** — `enum FeedFormat` is `Rss2`, `Rss1`, `Atom` or `JsonFeed`. `Feed.format` is the format the feed was read from, and `feed.new_feed` starts as `Rss2`. The writers (`std.feed.9` to `std.feed.11`) choose their own format and ignore the field. *(ADR TBD)*

```ail rules=std.feed.2
fn main()
    f := feed.new_feed("T")
    print(f.format.name())
```

**std.feed.3** — `struct Feed` has the fields `format FeedFormat`, `id`, `title`, `link` (the website), `url` (the feed's own address), `description`, `language` and `author`, all `str`; `updated DateTime?`; and `items List[Item]`. `struct Item` has `id`, `title`, `link`, `summary`, `content` and `author`, all `str`; `published` and `updated`, both `DateTime?`; `tags List[str]`; and `enclosures List[Enclosure]`. A field the feed does not give is `""`, `none` or an empty list; reading never fails because a field is missing. An item with no id of its own takes its `link` as its `id`. Text is trimmed; HTML in a field is kept as written, with its references resolved by XML. *(ADR TBD)*

```ail rules=std.feed.3
fn main()
    it := feed.new_item("Title", "https://e.org/1")
    print("{it.id} {it.summary.len()} {it.published == none} {it.tags.len()}")
```

**std.feed.4** — RSS 2.0 (and 0.9x) is read from `channel`: `title`, `link`, `description`, `language`, `managingEditor` (the author) and `lastBuildDate`, else `pubDate`, as `updated`. Each `item` gives `title`, `link`, `description` as `summary`, `content:encoded` as `content`, `author` else `dc:creator`, every `category` and `dc:subject` as a tag, `guid` as `id`, `pubDate` else `dc:date` as `published`, and each `enclosure` with a `url` as an `Enclosure(url, mime from type, length from length)`; an unreadable `length` is 0. A document with no `channel` fails with kind `Unknown`. *(ADR TBD)*

```ail rules=std.feed.4
fn main() !
    f := feed.parse("<rss><channel><title>T</title><item><title>A</title><enclosure url=\"u\" type=\"audio/mpeg\" length=\"9\"/></item></channel></rss>")?
    e := f.items[0].enclosures[0]
    print("{e.url} {e.mime} {e.length}")
```

**std.feed.5** — RSS 1.0 is read like RSS 2.0, except that the `item` elements are children of the root, not of `channel`, an item's `id` is its `rdf:about` attribute when it has no `guid`, and dates come from `dc:date`. Prefixes stay part of the element name (`std.xml.14`): a document that binds the RSS namespace to another prefix is not read. *(ADR TBD)*

```ail rules=std.feed.5
fn main() !
    f := feed.parse("<rdf:RDF><channel><title>T</title></channel><item rdf:about=\"u1\"><title>A</title></item></rdf:RDF>")?
    print("{f.format.name()} {f.items[0].id}")
```

**std.feed.6** — Atom 1.0 is read from the root `feed`: `title`, `subtitle` as `description`, `id`, `updated` else `modified`, `xml:lang` as `language`, `author/name` else `contributor/name` as `author`, the `link` whose `rel` is absent or `alternate` as `link`, and the `link` with `rel="self"` as `url`. Each `entry` gives `id` (else its link), `title`, `summary`, `content`, `author`, `published` else `issued`, `updated` else `modified`, each `category`'s `label` else `term` as a tag, and each `link rel="enclosure"` as an `Enclosure`. An entry with no author of its own has none: it does not take the feed's. *(ADR TBD)*

```ail rules=std.feed.6
fn main() !
    f := feed.parse("<feed><title>T</title><link href=\"https://e.org/\"/><entry><title>A</title><link href=\"https://e.org/a\"/><category term=\"x\"/></entry></feed>")?
    print("{f.link} {f.items[0].id} {f.items[0].tags}")
```

**std.feed.7** — JSON Feed 1.0 and 1.1 is read from the object's `title`, `home_page_url` as `link`, `feed_url` as `url`, `description`, `language` and `authors[0].name` else `author.name` as `author`. Each member of `items` gives `id`, `url` as `link`, `title`, `summary`, `content_html` else `content_text` as `content`, the author the same way, `date_published` and `date_modified`, `tags`, and each `attachments` member as an `Enclosure(url, mime_type, size_in_bytes)`. A JSON object whose `version` does not contain `jsonfeed.org` fails with kind `Unknown`; text that is not JSON fails with kind `Json`. *(ADR TBD)*

```ail rules=std.feed.7
fn main() !
    f := feed.parse("\{\"version\":\"https://jsonfeed.org/version/1.1\",\"title\":\"T\",\"items\":[\{\"id\":\"1\",\"title\":\"A\"}]}")?
    print("{f.format.name()} {f.items[0].id}")
```

**std.feed.8** — `parse_date(text str) DateTime ! FeedError` reads a feed date. RFC 822 and RFC 2822 text is `[Wdy,] DD Mon YYYY HH:MM[:SS] ZONE`: the weekday is optional and not checked, a 2-digit year below 50 is 20YY and otherwise 19YY, and the zone is `GMT`, `UT`, `UTC`, `Z`, a US name (`EST` to `PDT`), `+hhmm`, `+hh:mm` or (as RFC 822 asks for any other name) 0. ISO 8601 and RFC 3339 text with an offset goes to `date.parse_iso` (`std.date.14`). The result keeps the text's own offset. Anything else fails with kind `Date`. Inside a feed, a date that does not parse is `none`, not an error. *(ADR TBD)*

```ail rules=std.feed.8
fn main() !
    a := feed.parse_date("Tue, 10 Jun 2003 04:00:00 -0500")?
    b := feed.parse_date("2003-06-10T09:00:00Z")?
    print(a.since(b))
```

## Writing

**std.feed.9** — `write_rss(f Feed) str` is the feed as an RSS 2.0 document with an XML declaration, indented by two spaces. `channel` always has `title`, `link` and `description`, written empty when the feed has none; `language`, `managingEditor` and `lastBuildDate` are written when set. An item writes its non-empty fields as `title`, `link`, `description`, `content:encoded`, `author`, `category`, `enclosure` and `guid` (`isPermaLink` true when the id is the link), and `pubDate` as RFC 822 with a numeric zone. Text is escaped (`std.xml.10`), so `parse(write_rss(f))` returns the same fields. *(ADR TBD)*

```ail rules=std.feed.9
fn main()
    mut f := feed.new_feed("Blog")
    f.items.push(feed.new_item("Hello", "https://b.example/1"))
    print(feed.write_rss(f))
```

**std.feed.10** — `write_atom(f Feed) str` is the feed as an Atom 1.0 document with an XML declaration, indented by two spaces. Atom requires `id`, `title` and `updated`: the feed's `id` falls back to its `url`, then its `link`, and `updated` is written when set. Dates are RFC 3339 with a numeric offset. An entry writes `id`, `title`, `link`, an `enclosure` link for each enclosure, `updated` (else its `published`), `published`, `author`, `category`, `summary`, and `content` with `type="html"`. *(ADR TBD)*

```ail rules=std.feed.10
fn main()
    mut f := feed.new_feed("Blog")
    f.url = "https://b.example/feed.xml"
    f.items.push(feed.new_item("Hello", "https://b.example/1"))
    print(feed.write_atom(f))
```

**std.feed.11** — `write_json(f Feed) str` is the feed as compact JSON Feed 1.1 text (`json.print.1`): `version`, `title`, `home_page_url`, `feed_url`, `description`, `language`, `authors`, then `items`, each with `id`, `url`, `title`, `summary`, `content_html`, `date_published`, `date_modified`, `authors`, `tags` and `attachments`. Empty fields are left out; `items` is always written. *(ADR TBD)*

```ail rules=std.feed.11
fn main()
    mut f := feed.new_feed("Blog")
    f.items.push(feed.new_item("Hello", "https://b.example/1"))
    print(feed.write_json(f))
```

**std.feed.12** — `new_feed(title str) Feed` is a feed with that title, the format `Rss2` and every other field empty. `new_item(title str, link str) Item` is an item with that title and link, `id` set to the link, and every other field empty. Both are plain values: fill them in by assigning fields of a `mut` binding and `push` the items. *(ADR TBD)*

```ail rules=std.feed.12
fn main()
    mut f := feed.new_feed("Blog")
    mut it := feed.new_item("Hello", "https://b.example/1")
    it.tags = ["news"]
    f.items.push(it)
    print(f.items.len())
```

## Errors and limits

**std.feed.13** — `error struct FeedError` has `kind FeedKind` and `why str`, and displays as `why`. `FeedKind` is `Xml` (the text is not well-formed XML, or has a DOCTYPE; `why` is the `XmlError` text with its line and column), `Json` (not valid JSON, with the offset), `Unknown` (well-formed but not a feed) or `Date` (`parse_date` only). *(ADR TBD)*

```ail rules=std.feed.13
fn main()
    _ := feed.parse("<html/>") else err
        print("{err.kind.name()}: {err.why}")
        return
```

**std.feed.14** — Element names are matched as written, with their prefixes (`std.xml.14`): the module reads `dc:creator`, `dc:date`, `dc:subject`, `content:encoded` and `rdf:about`, and Atom with the default namespace. A feed that binds those namespaces to other prefixes, or Atom under a prefix, loses those fields or fails with kind `Unknown`. Extensions other than the ones named (iTunes, Media RSS, `atom:link` inside RSS) are ignored. *(ADR TBD)*

```ail rules=std.feed.14
fn main() !
    f := feed.parse("<rss><channel><title>T</title><item><title>A</title><itunes:duration>1:00</itunes:duration></item></channel></rss>")?
    print(f.items[0].title)
```

**std.feed.15** — A feed written by `write_rss`, `write_atom` or `write_json` and read back by `parse` has the same `title`, `link`, `description`, `language` and `author`, and every item has the same `id`, `title`, `link`, `summary`, `content`, `author`, `tags`, `published` instant and `enclosures`. The fields a format has no place for come back empty: RSS has no feed `url` or `id`, JSON Feed no feed `updated`, and an Atom entry always writes `updated`, so it reads back as its `published` when it had no `updated`. *(ADR TBD)*

```ail rules=std.feed.15
fn main() !
    mut f := feed.new_feed("Blog")
    f.items.push(feed.new_item("Hello", "https://b.example/1"))
    for text in [feed.write_rss(f), feed.write_atom(f), feed.write_json(f)]
        g := feed.parse(text)?
        print("{g.format.name()} {g.items[0].title}")
```

**std.feed.16** — The reader and writers work on whole text in memory, so the limits of `std.xml.6` and `std.xml.16` hold: a DOCTYPE is refused and nesting stops at 200 levels. No call reads the network or the clock. *(ADR TBD)*

```ail rules=std.feed.16
fn main()
    _ := feed.parse("<!DOCTYPE rss><rss/>") else err
        print(err.kind.name())
        return
```
