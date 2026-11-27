# Standard library: `std.net.smtp`

Chapter key: `std`

> **Status:** Drafted. Implemented in AIL (`std/net/smtp.ail`, ADR 1262) over
> `net.Conn`, plus `Conn.start_tls` (`std.net.50`) in the native runtime.
> Written without a build; the first gate run is its first compile.

An SMTP client (RFC 5321) and a MIME message builder (RFC 5322, RFC 2045 to
2047). `use std.net.smtp` is always written; the module is used as `smtp`.
Every function that touches a connection has the effects `net` and `io`;
`mailbox`, `message`, `Message.build` (but for the clock and random bytes it
reads when `sent_at`, `message_id` or `boundary` is empty) and `dot_stuff` do
no network I/O.

## The client

**smtp.client.1** — `connect(host str, port int, security Security, hello str)
Client ! SmtpError` opens a session and says EHLO as `hello` (a name with no
space or line break). Details: [below](#more-on-smtpclient1). *(ADR 1262)*

**smtp.client.2** — `c.supports(ext str) bool` is true when EHLO announced the
extension (`SIZE`, `AUTH`, `STARTTLS`, `PIPELINING`, `8BITMIME`), in any case.
`c.limit() int` is the size `SIZE` announced, 0 when there is none. *(ADR 1262)*

**smtp.client.3** — With `Security.StartTls`, `connect` sends `STARTTLS`
after the first EHLO, expects `220`, runs the TLS handshake for `host` with
`c.start_tls(host)` (`std.net.50`) and says EHLO again, so the extensions it
keeps are the encrypted session's. Details: [below](#more-on-smtpclient3). *(ADR 1262)*

**smtp.client.4** — `c.auth(user str, password str) ! SmtpError` logs in with
`PLAIN` (RFC 4616) when the server's `AUTH` extension lists it, else `LOGIN`,
else fails with kind `Auth`. Details: [below](#more-on-smtpclient4). *(ADR 1262)*

**smtp.client.5** — `c.send_raw(sender str, recipients List[str], data bytes) !
SmtpError` sends `data`, a complete message with CRLF line ends, with the
envelope `sender` and `recipients` (`MAIL FROM`, one `RCPT TO` each, `DATA`). Details: [below](#more-on-smtpclient5). *(ADR 1262)*

**smtp.client.6** — `c.send(m Message) ! SmtpError` builds `m` (`smtp.msg.3`)
and sends it with `send_raw`; the envelope sender is the address of
`m.sender`, and the recipients are `to`, `cc` and `bcc` in that order. *(ADR 1262)*

**smtp.client.7** — `c.quit()` sends `QUIT` and closes the connection;
`c.close()` closes it without `QUIT`. Neither fails and either may be called
again. *(ADR 1262)*

**smtp.client.8** — `dot_stuff(data bytes) bytes` doubles each `.` at the start
of a line, the first byte of `data` included (RFC 5321 section 4.5.2). *(ADR 1262)*

**smtp.client.9** — A failure is `error struct SmtpError { kind SmtpKind, code
int, message str }`. `code` is the server's reply code, 0 when the failure was
not a reply. Details: [below](#more-on-smtpclient9). *(ADR 1262)*

```ail rules=smtp.client.8,smtp.client.9
use std.net.smtp

fn main() !
    print(smtp.dot_stuff(b".a\r\nb\r\n.\r\n").text().replace("\r\n", "|"))   # ..a|b|..|
    c := smtp.connect("127.0.0.1", 1, smtp.Security.Plain, "has space") else err
        print(err.kind.name())   # BadMessage
        return
    print(c.is_secure())
```

## The message

**smtp.msg.1** — `mailbox(s str) Mailbox ! SmtpError` reads `ann@example.com`
or `Name <ann@example.com>` (the name may be in quotes) into `Mailbox { name,
addr }`. Details: [below](#more-on-smtpmsg1). *(ADR 1262)*

**smtp.msg.2** — A header value that is not printable ASCII is written as RFC
2047 encoded words, `=?UTF-8?B?...?=`, each of at most 36 bytes and never
splitting a character, folded onto continuation lines. Details: [below](#more-on-smtpmsg2). *(ADR 1262)*

**smtp.msg.3** — `message(sender str, to List[str], subject str) Message` makes
a message with empty `cc`, `bcc`, `reply_to`, `text`, `html`, `attachments`,
`headers`, `sent_at`, `message_id` and `boundary`; fields are set by
assignment, and `m.attach(name, content_type, data)` adds an attachment. Details: [below](#more-on-smtpmsg3). *(ADR 1262)*

**smtp.msg.4** — A text or html body has its line breaks changed to CRLF and
is written as `7bit` when it is ASCII with lines of at most 900 bytes, and as
base64 in lines of 76 characters otherwise, `charset=utf-8` either way. Details: [below](#more-on-smtpmsg4). *(ADR 1262)*

**smtp.msg.5** — With only `text` or only `html` the message is that one part
(`text/plain`, `text/html`; no `text` and no `html` is an empty `text/plain`). Details: [below](#more-on-smtpmsg5). *(ADR 1262)*

**smtp.msg.6** — A boundary is the base followed by `a` (alternative) or `m`
(mixed). Details: [below](#more-on-smtpmsg6). *(ADR 1262)*

**smtp.msg.7** — `build` fails with kind `BadMessage` for a subject, header
value, `sent_at`, `message_id`, `reply_to`, file name or type with a line break
or NUL, for an extra header whose name is not a header name or is one `build`
writes (`From`, `To`, `Cc`, `Bcc`, `Subject`, `Date`, `Message-ID`, `Reply-To`,
`MIME-Version`, `Content-*`), and when no `to`, `cc` or `bcc` address is given;
and with kind `BadAddress` for an address that `mailbox` refuses. Details: [below](#more-on-smtpmsg7). *(ADR 1262)*

```ail rules=smtp.msg.1,smtp.msg.2,smtp.msg.3,smtp.msg.4,smtp.msg.5,smtp.msg.6,smtp.msg.7
use std.net.smtp

fn main() !
    mut m := smtp.message("Zoë <zoe@example.com>", ["bob@example.com"], "Grüße")
    m.text = "Hello"
    m.html = "<p>Hello</p>"
    m.attach("a.txt", "text/plain", b"hi there")
    m.boundary = "XB1"
    m.sent_at = "Mon, 01 Jan 2026 00:00:00 +0000"
    m.message_id = "<id@example.com>"
    raw := m.build()?
    print(raw.text().replace("\r\n", "\n"))   # From: =?UTF-8?B?Wm/Dqw==?= <zoe@example.com>, multipart/mixed, XB1m
    jo := smtp.mailbox("\"Doe, Jo\" <jo@example.com>")?
    print(jo.name)   # Doe, Jo
    m.subject = "a\r\nBcc: x@example.com"
    again := m.build() else err
        print(err.kind.name())   # BadMessage
        return
    print(again.len())
```

## Sending

A session against a server on port 587:

```ail skip=context
use std.net.smtp

fn main() !
    mut c := smtp.connect("smtp.example.com", 587, smtp.Security.StartTls, "client.example.com")?
    c.auth("ann", "secret")?
    mut m := smtp.message("Ann <ann@example.com>", ["bob@example.com"], "Hello")
    m.text = "Hi Bob."
    c.send(m)?
    c.quit()
```

The TLS code is in-house and unaudited, as `std.net.13` and
[tls-security.md](../design/tls-security.md) say; an SMTP password is as safe
as the TLS client is. A server certificate is checked against the platform's
roots for `host`, as for `https`.

## Rejected alternatives

- **A `Transport` trait** (SMTP, sendmail, an API) behind `send`. One transport
  today; the trait can wrap `Client` when a second exists.
- **Header folding of every long line.** Only encoded words and address lists
  fold; other lines are far below the 998-byte limit and a plain header is
  clearer unfolded.
- **Quoted-printable bodies.** Base64 is one encoder and decodes everywhere;
  quoted-printable keeps text readable in raw form, which matters to no
  caller of `build`.
- **Authenticating in `connect`.** A separate `auth` keeps `connect` free of
  credentials and lets a program send without logging in (a local relay).
- **Sending a password over a plain connection by default.** Refused with
  kind `Insecure` unless the program opts in; the loopback test server does.

### `smtp.msg` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on smtp.msg.1

An address is ASCII, has exactly one `@` with text on both sides, and has no space, control character or any of `< > " , ; : \ ( ) [ ]`; otherwise the call fails with kind `BadAddress`. SMTPUTF8, quoted local parts and address literals are not supported.

#### More on smtp.msg.5

With both it is `multipart/alternative`, the text first. With attachments the body is the first part of a `multipart/mixed`, followed by the attachments in order.

#### More on smtp.msg.6

The base is `boundary` when set, else random, and `build` fails with kind `BadMessage` when a part contains the base, when `boundary` is longer than 50 characters or has a character that is not a letter, a digit or one of `'()+_,-./:=?`.

#### More on smtp.msg.2

This covers the subject, the display name of a mailbox, and the values of extra headers. A display name of ASCII text with a special character (`( ) < > [ ] : ; @ \ , . "`) is written as a quoted string. A file name is written in quotes, as encoded words when it is not ASCII.

#### More on smtp.msg.3

`m.build() bytes ! SmtpError` writes `Date`, `From`, `To`, `Cc`, `Reply-To`, `Subject`, `Message-ID`, `MIME-Version` and the extra headers, then the body, with CRLF line ends and a final CRLF. `Bcc` is never written. With no `to` and no `cc` the `To` header is `undisclosed-recipients:;`. `Date` is `sent_at`, or the time of the call as `Mon, 01 Jan 2026 00:00:00 +0000` (UTC) when empty; `Message-ID` is `message_id`, or random text at the sender's domain.

#### More on smtp.msg.4

An attachment is base64, with `Content-Type` (`application/octet-stream` when the type is empty), `Content-Disposition: attachment` and the file name as `name` and `filename`.

#### More on smtp.msg.7

A line break can therefore never end a header and start another.

### `smtp.client` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on smtp.client.1

`Security.Implicit` runs TLS 1.3 from the first byte (`net.connect_tls`, `std.net.13`; port 465), `Security.StartTls` connects in the clear and upgrades (port 587), and `Security.Plain` stays unencrypted. The greeting must be `220`; a server that refuses EHLO is sent HELO. A failure after the connection opens closes it. `c.set_timeout(ms)` sets the timeout of `std.net.12`.

#### More on smtp.client.4

A refused login fails with kind `Auth` and the reply's code. On a connection that is not encrypted, all three fail with kind `Insecure` before sending anything, unless `c.allow_insecure_auth` was set to true. `c.auth_plain` and `c.auth_login` use one mechanism.

#### More on smtp.client.9

`SmtpKind` is `BadAddress`, `BadMessage`, `TooBig`, `NoStartTls`, `Insecure`, `Auth`, `Rejected`, `Transient`, `Protocol` (the reply is not SMTP: a code outside 200 to 599, a changed code inside a reply), `Closed`, `TimedOut` or `Net`.

#### More on smtp.client.3

When the server does not announce `STARTTLS` the call fails with kind `NoStartTls` and nothing is sent in the clear. `c.is_secure()` is true after TLS from either route.

#### More on smtp.client.5

It adds `SIZE=` when the server announced `SIZE`, and fails with kind `TooBig`, before sending, when `data` is longer than the limit. A reply the call does not expect fails with the reply's code and kind `Rejected` (5xx) or `Transient` (4xx) and sends `RSET`, so a refused recipient leaves the session reusable and nothing is delivered. The data is dot-stuffed (`smtp.client.8`) and ends with the `.` line.
