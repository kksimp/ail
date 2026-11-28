# Standard library: `std.validate`

Chapter key: `std`

> **Status:** Drafted. Written in AIL (`std/validate/validate.ail`, ADR 1297).

Checks for the text forms programs read from users: email addresses, domain names (IDNA and Punycode), phone
numbers in E.164 form, IP addresses and CIDR blocks, host names and card numbers. Every function is pure
(`eff.infer.1`) and needs no capability. Each rule below is tested against a reference implementation
(`tools/validate/reference.py`): Python's `punycode` codec and `idna` for names, `email-validator` in strict mode
without DNS for addresses, `ipaddress` for IP and CIDR, `python-stdnum` for Luhn and `phonenumbers` for E.164.
The cases where this module is deliberately narrower or looser are listed in `std.validate.20`.

## The module

**std.validate.1** — `std.validate` provides `email`, `is_email`, `to_ascii`, `to_unicode`, `is_domain`,
`punycode_encode`, `punycode_decode`, `ip`, `ipv4`, `ipv6`, `is_ip`, `cidr`, `hostname`, `is_hostname`, `luhn`,
`luhn_digit`, `card`, `phone` and `is_e164`; the methods `Net.contains` and `Net.last`; and the types `Email`,
`Addr`, `Net`, `Card`, `CardBrand`, `Phone`, `ValidateError` and `ValidateKind`. The module is in scope as
`validate` without a `use` line (`std.use.4`). *(ADR 1297)*

```ail rules=std.validate.1
fn main()
    print(validate.is_email("a@example.com"))    # true
```

## Punycode and IDNA

**std.validate.2** — `punycode_encode(s str) str` is the Punycode of RFC 3492 for `s`, without the `xn--` prefix:
`punycode_encode("bücher")` is `bcher-kva`. Text that is all ASCII gets a trailing `-`. `punycode_decode(p str)
str ! ValidateError` is the inverse. It fails with `BadIdna` on a character that is not a letter or digit, text
that ends inside a number, a number too large, or a surrogate code point. Both agree with Python's `punycode` codec
on every tested input. *(ADR 1297)*

```ail rules=std.validate.2
fn main() !
    print(validate.punycode_encode("bücher"))    # bcher-kva
    print(validate.punycode_decode("bcher-kva")?)   # bücher
```

**std.validate.3** — `to_ascii(domain str) str ! ValidateError` writes a domain name in its ASCII form: the name
in lower case, and each label with a non-ASCII character as `xn--` and its Punycode. A label is 1 to 63 characters
of ASCII letters, digits and `-`, or letters and digits of any script, and does not start or end with `-` or have
`--` in its third and fourth place (except `xn--`). A label that starts with `xn--` must be the Punycode of
something non-ASCII that `to_ascii` would accept, written back the same. The ASCII form is at most 253 characters,
and one trailing `.` is kept. Uppercase non-ASCII letters are lower-cased first. Digits, letters of every script
and combining marks are accepted without the IDNA 2008 tables: controls, spaces, punctuation, symbols, fullwidth
forms and emoji are rejected (`BadIdna`), and so are ASCII characters other than letters, digits and `-`
(`BadChar`). The text should be in Unicode normalisation form NFC, and no compatibility mapping (UTS 46 `ǆ` to
`dž`) is applied. `to_unicode(domain str) str ! ValidateError` checks the name as `to_ascii` does and decodes each
`xn--` label; `is_domain(domain str) bool` says whether `to_ascii` succeeds. *(ADR 1297)*

```ail rules=std.validate.3
fn main() !
    print(validate.to_ascii("Bücher.DE")?)               # xn--bcher-kva.de
    print(validate.to_unicode("xn--bcher-kva.de")?)      # bücher.de
    print(validate.is_domain("ex_ample.com"))            # false
```

## Email addresses

**std.validate.4** — `email(s str) Email ! ValidateError` reads `s` as `local@domain` by RFC 5321, 5322 and 6531,
with no display name (`Ann <a@x.org>`), no comments, no quoted local part (`"a b"@x.org`) and no domain literal
(`a@[1.2.3.4]`). A space or control character anywhere is `BadChar`; no `@` is `BadShape`; an empty local part or
domain is `Empty`. The local part is split at the first `@`. It holds ASCII letters, digits and
``!#$%&'*+-/=?^_`{|}~``, with `.` only between other characters (`BadShape` for a leading, trailing or doubled `.`),
and non-ASCII characters that are not controls, spaces, format characters, surrogates, private-use or
noncharacters (an emoji is allowed, as in RFC 6531). The local part is case-sensitive and is not changed.
*(ADR 1297)*

```ail rules=std.validate.4
fn main()
    print(validate.is_email("a+tag@example.com"))      # true
    print(validate.is_email("a..b@example.com"))       # false
    print(validate.is_email("a b@example.com"))        # false
```

**std.validate.5** — The domain of an email address is a public domain name. It must be accepted by `to_ascii`
(`BadChar`, `BadLabel`, `BadIdna` or `TooLong` say why not), must not start or end with `.` or hold `..`
(`BadShape`), must have at least two labels, its last label must end with a letter (`.123` is not a top-level
domain), and it must not be `arpa`, `invalid`, `local`, `localhost`, `onion` or `test` or under one of them: all
`BadDomain`. `example.com`, `example.org` and `x.example` are accepted. *(ADR 1297)*

```ail rules=std.validate.5
fn main()
    print(validate.is_email("a@localhost"))            # false
    print(validate.is_email("a@host.test"))            # false
    print(validate.is_email("a@x.example"))            # true
```

**std.validate.6** — Lengths: the local part is at most 64 bytes of UTF-8 and the whole address, with the domain
in ASCII form, at most 254 bytes; otherwise `TooLong`. (`email-validator` checks the 64 bytes only in strict
mode, which is the mode the tests use.) *(ADR 1297)*

```ail rules=std.validate.6
fn main()
    print(validate.is_email("{"a".repeat(64)}@example.com"))    # true
    print(validate.is_email("{"a".repeat(65)}@example.com"))    # false
```

**std.validate.7** — `Email` has `local`, `domain` (the ASCII form in lower case), `unicode_domain` (the Unicode
form) and `smtputf8`, true when `local` has non-ASCII characters and the message needs the SMTPUTF8 extension.
`print(e)` and `e.to_str()` give `local@domain` with the ASCII domain. *(ADR 1297)*

```ail rules=std.validate.7
fn main() !
    e := validate.email("jörg@Müller.DE")?
    print(e)                                           # jörg@xn--mller-kva.de
    print("{e.unicode_domain} {e.smtputf8}")           # müller.de true
```

**std.validate.8** — `is_email(s str) bool` says whether `email(s)` succeeds. *(ADR 1297)*

```ail rules=std.validate.8
fn main()
    print(validate.is_email("\"a b\"@example.com"))    # false: quoted local parts are not accepted
```

## IP addresses and CIDR blocks

**std.validate.9** — An `Addr` has `v6` and `octets`, 4 numbers from 0 to 255 for IPv4 or 16 for IPv6. `print(a)`
and `a.to_str()` give the canonical text: dotted decimal, or lower-case hexadecimal groups without leading zeros
with the longest run of two or more zero groups written `::` (the first when two runs tie), by RFC 5952. An
IPv6 address that holds an IPv4 address (`::ffff:1.2.3.4`) is written in hexadecimal (`::ffff:102:304`). *(ADR 1297)*

```ail rules=std.validate.9
fn main() !
    print(validate.ip("2001:DB8:0:0:0:0:0:1")?)        # 2001:db8::1
    print(validate.ip("1:0:0:2:0:0:3:4")?)             # 1::2:0:0:3:4
```

**std.validate.10** — `ip(s str) Addr ! ValidateError` reads an IPv4 or an IPv6 address; `ipv4` and `ipv6` accept
one version only; `is_ip` is the bool form. Failures are `BadAddress`. IPv4 is four decimal numbers from 0 to 255
separated by `.`, with no leading zero (`01.2.3.4` is rejected, as in Python 3.9.5 and later) and no spaces.
IPv6 is eight groups of one to four hexadecimal digits separated by `:`; `::` stands for one or more groups of
zeros and may appear once; the last two groups may be written as an IPv4 address; a `%zone` is rejected. *(ADR 1297)*

```ail rules=std.validate.10
fn main()
    print(validate.is_ip("::1"))                       # true
    print(validate.is_ip("01.2.3.4"))                  # false
    print(validate.is_ip("1::2::3"))                   # false
```

**std.validate.11** — `cidr(s str, strict bool = true) Net ! ValidateError` reads `ADDRESS/PREFIX`: the prefix is
digits from 0 to 32 (IPv4) or 128 (IPv6); without `/PREFIX` the block is a single host. With `strict`, host bits
after the prefix must be zero (`10.0.0.1/8` is `BadPrefix`); with `strict: false` they are cleared. A bad prefix is
`BadPrefix`, a bad address `BadAddress`. `n.contains(a Addr) bool` says whether `a` is in the block (never for an
address of the other version), and `n.last() Addr` is the highest address, with every host bit set. `print(n)`
gives `NETWORK/PREFIX`. Netmask and hostmask notations (`10.0.0.0/255.0.0.0`) are not accepted. *(ADR 1297)*

```ail rules=std.validate.11
fn main() !
    n := validate.cidr("10.0.0.0/8")?
    print("{n} {n.last()} {n.contains(validate.ip("10.1.2.3")?)}")   # 10.0.0.0/8 10.255.255.255 true
    loose := validate.cidr("172.16.5.4/12", strict: false)?
    print(loose)                                       # 172.16.0.0/12
```

## Host names, Luhn and card numbers

**std.validate.12** — `hostname(s str) str ! ValidateError` checks a host name by RFC 952 and 1123: labels of 1 to
63 ASCII letters, digits and `-` that do not start or end with `-` (`BadLabel`, or `BadChar` for another
character), joined by `.`, at most 253 characters (`TooLong`), one trailing `.` allowed, and a last label that is
not all digits (`1.2.3.4` and `a.123` are not host names). One label is a host name (`localhost`), and `_` is not
allowed. The value is `s` in lower case without the trailing `.`. Names with other characters go through
`to_ascii` first. `is_hostname(s str) bool` is the bool form. *(ADR 1297)*

```ail rules=std.validate.12
fn main() !
    print(validate.hostname("Example.COM.")?)          # example.com
    print(validate.is_hostname("my_host.com"))         # false
    print(validate.is_hostname("1.2.3.4"))             # false
```

**std.validate.13** — `luhn(s str) bool` is true when `s` is at least two ASCII digits whose Luhn sum (ISO/IEC
7812: from the right, every second digit doubled, digits of a product summed) is a multiple of 10. Any other
character, spaces included, gives false. `luhn_digit(payload str) int ! ValidateError` is the check digit that makes
`payload` followed by it pass `luhn`; a payload that is empty or not all digits is `BadNumber`. *(ADR 1297)*

```ail rules=std.validate.13
fn main() !
    print(validate.luhn("79927398713"))                # true
    print(validate.luhn_digit("7992739871")?)          # 3
```

**std.validate.14** — `card(s str) Card ! ValidateError` reads a payment card number: digits with spaces and `-`
ignored (another character is `BadChar`), 12 to 19 digits and `luhn` true (`BadNumber`). `Card` has `number` (the
digits), `last4` and `brand`, a `CardBrand`: `Visa` (4, 13, 16 or 19 digits), `Mastercard` (51-55 or 2221-2720,
16), `Amex` (34 or 37, 15), `Discover` (6011, 65 or 644-649, 16-19), `Diners` (300-305, 309, 36, 38 or 39, 14-19),
`Jcb` (3528-3589, 16-19), `UnionPay` (62, 16-19), else `Unknown`. The brand is a guess from public prefix ranges;
it says nothing about whether the account exists. *(ADR 1297)*

```ail rules=std.validate.14
fn main() !
    c := validate.card("4111 1111 1111 1111")?
    print("{c.brand.name()} {c.last4}")                # Visa 1111
    bad := validate.card("4111 1111 1111 1112") else err
        print(err.kind.name())                         # BadNumber
        return
    print(bad.number)
```

## Phone numbers

**std.validate.15** — `phone(s str, default_cc str = "") Phone ! ValidateError` reads an international number. It
starts with `+` or `00`, then an assigned ITU country calling code (1 to 3 digits, the longest-prefix match in the
list of 215 codes), then the national number. Spaces, `-`, `.` and round brackets between the digits are ignored;
any other character, letters included, is `BadPhone`. The result has `country_code` and `national`, and `print(p)`
and `p.to_str()` give the E.164 form `+CCNATIONAL`. Together the digits are at most 15, the national number has at
least 4, and for code `1` exactly 10 (`BadPhone` otherwise). This is basic validation: it does not know each
country's numbering plan, so a number can pass here and not be assigned. *(ADR 1297)*

```ail rules=std.validate.15
fn main() !
    p := validate.phone("+44 20 7946 0958")?
    print("{p} {p.country_code} {p.national}")         # +442079460958 44 2079460958
    print(validate.phone("+1 (415) 555-2671")?)        # +14155552671
```

**std.validate.16** — `is_e164(s str) bool` is true when `s` is `+` and digits only and `phone(s)` accepts it:
the strict E.164 form with no separators. *(ADR 1297)*

```ail rules=std.validate.16
fn main()
    print(validate.is_e164("+14155552671"))            # true
    print(validate.is_e164("+1 415 555 2671"))         # false
```

**std.validate.17** — A number with neither `+` nor `00` is national. `default_cc` names its country calling code
(`"44"`) and the number is read as that country's: one leading trunk `0` is dropped (`020 7946 0958` is
`+442079460958`), and for code `1` a leading `1` before 10 digits is dropped. Without `default_cc` a national
number is `BadPhone`. *(ADR 1297)*

```ail rules=std.validate.17
fn main() !
    print(validate.phone("020 7946 0958", default_cc: "44")?)    # +442079460958
    print(validate.phone("1-415-555-2671", default_cc: "1")?)    # +14155552671
```

## Errors, differences and names

**std.validate.18** — `ValidateError` has `kind` (a `ValidateKind`: `Empty`, `BadChar`, `BadShape`, `TooLong`,
`BadLabel`, `BadIdna`, `BadDomain`, `BadAddress`, `BadPrefix`, `BadNumber`, `BadPhone`), `what` (the kind of text
read, as in `email address`), `why` (a sentence) and `text` (the input). Its text is `not a valid WHAT: WHY`. When
text has several faults the kind of the first check that fails is reported, in the order the rules above list them.
*(ADR 1297)*

```ail rules=std.validate.18
fn main()
    e := validate.email("a@b") else err
        print("{err.kind.name()}: {err}")             # BadDomain: not a valid email address: the domain needs a `.`
        return
    print(e)
```

**std.validate.19** — Every function is pure and needs no capability, no clock and no network. No function does a
DNS lookup: `email` does not check that the domain exists or takes mail, and `hostname` and `to_ascii` do not
resolve anything. *(ADR 1297)*

```ail rules=std.validate.19
fn main()
    print(validate.is_hostname("a.b"))                 # true
```

**std.validate.20** — Where the module differs from the reference libraries (each is outside the tested cases):
(1) IDNA uses the character classes of `std.validate.3`, not the full IDNA 2008 tables or UTS 46 mapping, so
a rare script-specific rule (bidi, joiners, `·` context) is not checked and compatibility characters such as `ǆ`
are not mapped; (2) quoted local parts, display names, comments and domain literals, which `email-validator`
accepts under options, are always rejected; (3) the local part is checked for 64 bytes as `email-validator`
does in strict mode; (4) `phone` has no region data: `phonenumbers` also needs the national number to be a
possible length for the country, and reads vanity letters (`1-800-FLOWERS`) and a `00` prefix only with a
region; (5) IPv6 addresses with an IPv4 tail are written in hexadecimal, where Python 3.13 writes
`::ffff:1.2.3.4`. *(ADR 1297)*

```ail rules=std.validate.20
fn main()
    print(validate.is_email("Ann <a@example.com>"))    # false: no display names
```

**std.validate.21** — Written in AIL, native only. A local named `validate` (a function that validates something is
a common name for one) hides the module (`std.use.4`); name it `check_input` or call the module through a longer
name in your own code. *(ADR 1297)*

```ail rules=std.validate.21
fn check_input(s str) bool
    validate.is_email(s)

fn main()
    print(check_input("a@example.com"))               # true
```
