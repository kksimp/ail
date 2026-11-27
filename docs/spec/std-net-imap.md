# Standard library: `std.net.imap`

Chapter key: `imap`

> **Status:** Drafted. Implemented in AIL (`std/net/imap.ail`, ADR 1291) over
> `net.Conn`. Written without a build; the first gate run is its first compile.

An IMAP4rev1 client (RFC 3501) and a MIME message parser (RFC 2045 to 2047,
RFC 2231). `use std.net.imap` is always written; the module is used as `imap`.
Every function that touches a connection has the effects `net` and `io`; the
MIME functions, `quote` and the mailbox-name functions do no I/O.

Not here: SASL mechanisms other than PLAIN, `ENABLE UTF8=ACCEPT` (mailbox names are
modified UTF-7), CONDSTORE, QRESYNC, BINARY, NAMESPACE, ACLs, a server, and
charsets other than UTF-8, US-ASCII, ISO-8859-1 and Windows-1252.

## The client

**imap.client.1** — `connect(host str, port int, security Security) Client !
ImapError` opens a session, reads the greeting (`OK`, or `PREAUTH`) and learns
the server's capabilities from the greeting or with `CAPABILITY`.
`Security.Implicit` runs TLS 1.3 from the first byte (`net.connect_tls`,
`std.net.13`; port 993), `Security.StartTls` connects in the clear and
upgrades (imap.client.15) and `Security.Plain` stays unencrypted. A `BYE`
greeting fails with kind `Bye`; a failure after the connection opens closes
it. `c.is_secure() bool` says whether the connection is encrypted and
`c.set_timeout(ms)` sets the timeout of `std.net.12`. A `Client` is used from
one task at a time. *(ADR 1291)*

**imap.client.2** — `c.has(cap str) bool` is true when the server announced
the capability (`IDLE`, `AUTH=PLAIN`, `MOVE`), in any case. `c.capability()
List[str]` asks again and returns the list in upper case; a login also asks
again, because capabilities change after it. *(ADR 1291)*

**imap.client.3** — `c.auth(user str, password str) ! ImapError` logs in with
`authenticate_plain` when the server lists `AUTH=PLAIN`, else with `login`
unless it lists `LOGINDISABLED`, else fails with kind `Unsupported`.
`c.authenticate_plain` sends the initial response with the command when the
server lists `SASL-IR` and after a continuation otherwise; the user name and
password may be any UTF-8 text without NUL. `c.login` sends LOGIN and takes
ASCII text (`quote`). A refused login fails with kind `Auth`. On a connection
that is not encrypted all three fail with kind `Insecure` before sending
anything, unless `c.allow_insecure_auth` was set to true. *(ADR 1291)*

**imap.client.4** — `c.list(reference str, pattern str) List[MailboxInfo] !
ImapError` sends LIST. `MailboxInfo` has `name` (modified UTF-7 decoded,
`imap.client.11`), `delimiter` (empty when none), `flags` (as the server wrote
them) and `selectable()`, false for `\Noselect` and `\NonExistent`.
*(ADR 1291)*

**imap.client.5** — `c.select(name str)` and `c.examine(name str)` open a
mailbox and return `Selected { name, exists, recent, uidvalidity, uidnext,
unseen, flags, permanent_flags, read_only }`, read from the untagged replies and
the response codes; a value the server did not send is 0 or empty. `examine`
is read-only, and so is a `select` that the server answers with `[READ-ONLY]`.
A refused call fails with kind `No` and leaves no mailbox open.
`c.current() str` is the open mailbox, `""` when none. `c.close_mailbox()`
sends CLOSE. `c.status(name str, items List[str]) Map[str, int]` returns the
counts of STATUS by item name in upper case (`MESSAGES`, `UNSEEN`, `UIDNEXT`).
*(ADR 1291)*

**imap.client.6** — `c.search(criteria str) List[int]` and
`c.uid_search(criteria str) List[int]` send SEARCH and UID SEARCH and return
the numbers. `criteria` is the text of RFC 3501 section 6.4.4 (`ALL`,
`UNSEEN`, `FROM "ann@example.com" SINCE 1-Nov-2026`); it must be printable
ASCII without braces, so a quoted string comes from `quote`. *(ADR 1291)*

**imap.client.7** — `c.fetch(set str, items str) List[Fetched]` and
`c.uid_fetch` send FETCH and UID FETCH; `set` is `1`, `2:4`, `7:*` or a
comma list, `items` is the RFC's list written as text (`(UID FLAGS ENVELOPE)`,
`BODY.PEEK[HEADER]`). A `Fetched` has `seq`, `uid`, `flags`, `size`
(`RFC822.SIZE`), `internal_date`, `has_envelope` with `envelope`, `structure`
(the `BODYSTRUCTURE` as text) and `sections`; `f.body(section) bytes` gives a
`BODY[section]` item (`""` is the whole message, also `HEADER`, `TEXT`, `1`),
`f.has_flag(flag)` ignores case and `f.mail()` parses `BODY[]`. An `Envelope`
has `date`, `subject` (decoded, `imap.mime.2`), the address lists `from`,
`sender`, `reply_to`, `to`, `cc`, `bcc` (`Address { name, addr }`),
`in_reply_to` and `message_id`. `c.fetch_mail(uid int) Mail` fetches one
message whole without setting `\Seen` and parses it. Literals of the reply are
read as bytes; one longer than `c.max_literal` (default 32 MiB) fails with kind
`TooBig` and closes the connection. *(ADR 1291)*

**imap.client.8** — `c.store(set, mode, flags) List[Fetched]` (and
`uid_store`) sends STORE with `StoreMode.Add`, `Remove` or `Replace` and
returns the flags the server reports. `c.expunge() List[int]` returns the
numbers of the removed messages, in the order reported (each number is valid
after the removals before it). `c.copy(set, dest)` and `c.uid_copy` copy to
the mailbox `dest`. A flag is `\Seen`, `$Label1`: letters, digits, `\`, `$`,
`-`, `_`, `.`. *(ADR 1291)*

**imap.client.9** — `c.append(name str, flags List[str], message bytes)` adds
a message (CRLF line ends) to a mailbox as a literal, with the flags (may be
empty). `c.create(name)`, `c.delete(name)` and `c.rename(from, to)` manage
mailboxes. *(ADR 1291)*

**imap.client.10** — `c.noop() List[Event]` asks for news. `c.idle(wait_ms
int) List[Event]` sends IDLE (RFC 2177), waits up to `wait_ms` for the server
to report a change, collects what follows for 50 ms, sends DONE and returns the
reports; an empty list means nothing happened. A call is one round of the
wait: call it in a loop and use the events (or `fetch` them) between rounds.
It fails with kind `Unsupported` without the `IDLE` capability. An `Event` has
`kind` (`Exists`, `Expunge`, `Recent`, `Flags` or `Bye`), `number` (a count for
`Exists` and `Recent`, a message number otherwise) and `flags`. *(ADR 1291)*

**imap.client.11** — `decode_mailbox(s str) str` and `encode_mailbox(s str)
str` convert between a mailbox name as text and as the server sends it
(modified UTF-7 of RFC 3501 section 5.1.3: `Entw&APw-rfe` is `Entwürfe`, `&-`
is `&`, characters outside the BMP are surrogate pairs). Every call that takes
a mailbox name encodes it. *(ADR 1291)*

**imap.client.12** — `quote(s str) str ! ImapError` is `s` as a quoted string.
Text that is not ASCII, or has a control character, fails with kind `BadArg`.
Every other argument is checked before anything is sent: a message set must
be digits, `:`, `,` and `*`; fetch items, search criteria and status items are
printable ASCII without braces; a line break anywhere fails with `BadArg`, so
no argument can add a command. *(ADR 1291)*

**imap.client.13** — `c.logout()` says LOGOUT and closes the connection;
`c.close()` closes without it. Neither fails and either may be called again.
*(ADR 1291)*

**imap.client.14** — A failure is `error struct ImapError { kind ImapKind,
message str }`. `ImapKind` is `BadArg`, `Insecure`, `Auth`, `No` (the server
answered `NO`), `Bad` (`BAD`), `Bye` (the server is closing), `Unsupported`,
`TooBig`, `Protocol` (the reply is not IMAP), `Closed`, `TimedOut` or `Net`.
The message of `No` and `Bad` has the command and the server's text. A
protocol error, a `BYE` and `TooBig` close the connection; a `No` or `Bad`
leaves the session usable. *(ADR 1291)*

**imap.client.15** — `Security.StartTls` (RFC 3501 section 6.2.1, port 143):
after the greeting `connect` requires the `STARTTLS` capability (kind
`Unsupported` when it is missing), sends `STARTTLS`, and on the tagged `OK`
upgrades the connection with `Conn.start_tls(host)` (`std.net.50`; it refuses
when unread bytes are buffered, so a server cannot inject plaintext). The
capabilities learned before TLS are discarded and asked for again, and
`c.is_secure()` is then true, so `auth` is allowed. A `NO` or `BAD` answer to
`STARTTLS` is kind `No` or `Bad`; a failed handshake is kind `Net`. In every
failure `connect` closes the connection. *(ADR 1291)*

```ail rules=imap.client.1,imap.client.13,imap.client.14
use std.net.imap

fn main() !
    mut c := imap.connect("127.0.0.1", 1, imap.Security.Plain) else err
        print(err.kind.name())   # Net
        return
    c.close()
```

```ail rules=imap.client.11,imap.client.12
use std.net.imap

fn main() !
    print(imap.decode_mailbox("Entw&APw-rfe"))   # Entwürfe
    print(imap.encode_mailbox("Tom & Jerry"))    # Tom &- Jerry
    print(imap.quote("a \"b\"")?)                # "a \"b\""
    q := imap.quote("é") else err
        print(err.kind.name())                   # BadArg
        return
    print(q)
```

## The MIME parser

**imap.mime.1** — `parse_mail(raw bytes) Mail` parses a message: its headers
(`Header { name, value }`, in order, folded lines unfolded) and the leaves of
its MIME structure in order as `Part`s. Nested `multipart/*` are flattened;
`Part.path` is the section number (`1` for a message with one part, `1.1`,
`1.2`, `2` in a nested one). A `message/rfc822` part is a leaf whose `data`
`parse_mail` can read. Line ends may be CRLF or LF. It never fails: text
before the first boundary and after the closing one is dropped, a part with an
unknown encoding keeps its bytes, and a missing boundary parameter makes the
multipart a single leaf. *(ADR 1291)*

**imap.mime.2** — `decode_words(s str) str` replaces RFC 2047 encoded words
(`=?UTF-8?B?...?=`, `=?ISO-8859-1?Q?...?=`) with their text and drops the
blank between two of them; a `?=` that does not close a well-formed word is
left as it is. *(ADR 1291)*

**imap.mime.3** — `parse_content_type(v str) ContentType` reads
`type/subtype; name=value` (also a `Content-Disposition` value, whose `kind`
is `attachment` or `inline`). `kind`, `subtype` and the parameter names are in
lower case, values are unquoted, and `media()` is `kind/subtype`. RFC 2231
parameters are joined and decoded: `filename*=utf-8''%C3%A9.txt` and
`filename*0="a"; filename*1="b"` give `filename`. An empty value is
`text/plain`. *(ADR 1291)*

**imap.mime.4** — `charset_text(charset str, data bytes) str` converts UTF-8,
US-ASCII, ISO-8859-1 and Windows-1252 text; any other charset is read as UTF-8
with each invalid sequence as U+FFFD. `Part.text()` uses the part's charset
and `Part.charset` says what it was. *(ADR 1291)*

**imap.mime.5** — `parse_addresses(s str) List[Address]` reads an address
header: `Ann <ann@example.com>, "Doe, Jo" <jo@example.com>, bob@example.com`,
also `bob@example.com (Bob)`. Names are decoded; a group and text that is not
an address (no `@`) are skipped. *(ADR 1291)*

**imap.mime.6** — `m.header(name str) str` is the first value of a header with
its encoded words decoded, `""` when absent, the name in any case;
`m.header_all(name)` lists every value, `m.subject()` is `header("subject")` and
`m.addresses(name) List[Address]` parses a header. *(ADR 1291)*

**imap.mime.7** — `m.text() str` is the first `text/plain` part that is not an
attachment, `m.html() str` the first `text/html` one, `""` when there is none,
both decoded with the part's charset; `m.attachments() List[Part]` lists the
parts for which `is_attachment()` is true. *(ADR 1291)*

**imap.mime.8** — A `Part` has `path`, `content_type` (lower case, `text/plain`),
`charset`, `filename` (from `Content-Disposition`, else the `name` of the type;
decoded), `content_id` (without angle brackets), `disposition` (`attachment`,
`inline` or empty), `headers` and `data`. `p.is_attachment()` is true for
`Content-Disposition: attachment`, false for `inline`, and otherwise true for a
part that is not `text/*` or has a file name. *(ADR 1291)*

**imap.mime.9** — The transfer encoding of a part is removed in `data`:
`base64` (whitespace ignored, missing padding accepted), `quoted-printable`
(soft line breaks and `=XX`), and 7bit, 8bit and binary as they are. A malformed
base64 part gives empty data. *(ADR 1291)*

```ail rules=imap.mime.1,imap.mime.2,imap.mime.3,imap.mime.4,imap.mime.5,imap.mime.6,imap.mime.7,imap.mime.8,imap.mime.9
use std.net.imap

fn main() !
    raw := "From: =?UTF-8?B?R3LDvMOfZQ==?= <g@example.com>\r\nSubject: Hi\r\nContent-Type: multipart/mixed; boundary=\"B\"\r\n\r\n--B\r\nContent-Type: text/plain; charset=iso-8859-1\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\ncaf=E9\r\n--B\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename*=utf-8''%C3%A9.bin\r\nContent-Transfer-Encoding: base64\r\n\r\nAAEC/w==\r\n--B--\r\n"
    m := imap.parse_mail(raw.to_bytes())
    print(m.header("from"))                  # Grüße <g@example.com>
    print(m.addresses("from")[0].addr)       # g@example.com
    print(m.text())                          # café
    for p in m.attachments()
        print("{p.path} {p.filename} {p.data.hex()}")   # 2 é.bin 000102ff
    print(imap.decode_words("=?UTF-8?Q?a?= =?UTF-8?Q?b?="))   # ab
    print(imap.parse_content_type("text/html; charset=\"x\"").media())   # text/html
    print(imap.charset_text("windows-1252", b"\x80"))   # €
```

## Rejected alternatives

See [ADR 1291](../decisions/1291-std-net-imap.md).
