# 1297. `std.validate`: email, IDNA and Punycode, E.164 phone, IP and CIDR, host names, Luhn

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out (2026-11-26) and for modules that actually work (2026-11-27)
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.validate.1 to std.validate.21 in [std-validate.md](../spec/std-validate.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)

## Context

Agents asked for input checks write the same fragile code each time: a regular expression for email that accepts
`a..b@x.com` and rejects `jörg@müller.de`, an IPv4 split that accepts `01.2.3.4` or `256.1.1.1`, a phone number
check that drops the country code, and a Luhn loop with the doubling on the wrong side. `std.url` (ADR 1214) is
explicit that IDNA is out of its scope, `std.net.jar` has a private `is_ip`, and no std module parses an IP
address or a CIDR block. Punycode was not in std at all.

## Decision

One new module written in AIL, `std/validate/validate.ail`, in scope without `use` (std.use.4). No syntax change, no
runtime or checker work beyond the five registration lines every std module has.

- **Email:** `email(s) Email ! ValidateError` follows RFC 5321/5322/6531 in the plain `local@domain` form with
  IDNA domains. It rejects display names, quoted local parts and domain literals, which a form field never wants.
  The domain must be public (two labels, a letter-ending top-level domain, none of the special-use names of
  `email-validator`).
- **IDNA and Punycode:** `punycode_encode`, `punycode_decode` (RFC 3492, exact), `to_ascii`, `to_unicode`, `is_domain`.
  The label rules are IDNA 2008 as character classes without the Unicode tables (letters and digits of any script,
  no symbols, punctuation, controls, fullwidth forms or emoji); no UTS 46 mapping beyond lower case.
- **IP and CIDR:** `Addr` (4 or 16 octets), `ip`, `ipv4`, `ipv6`, `is_ip`, `Net`, `cidr(s, strict: true)`, `Net.contains`,
  `Net.last`. Strict like Python's `ipaddress`: no leading zeros, no zone ids, host bits must be zero.
- **Phone:** `phone(s, default_cc:)` to a `Phone` whose text is E.164, and `is_e164`. It knows the 215 assigned
  country calling codes and the length limits, not each country's numbering plan.
- **Host names, Luhn, cards:** `hostname` (RFC 1123), `luhn`, `luhn_digit`, `card` with a `CardBrand` by public prefix ranges.
- **Errors:** one `ValidateError` with a `ValidateKind`, `what`, `why` and `text`.

Tested by five programs in `tests/cases/std_validate_*.ail`. Their goldens come from the reference libraries
through `tools/validate/reference.py`, which stops when the library and a model of the module disagree.

## Rejected alternatives

- **Separate modules** (`std.email`, `std.ip`, `std.punycode`, `std.phone`): five registrations, five spec chapters
  and five names to guess for what a person calls "validation". One module, one search. Rejected.
- **The full IDNA 2008 tables and UTS 46 mapping:** about 40 KB of tables and a normalisation pass, for rare scripts
  and compatibility characters; `std.unicode` (PR 1523) may add normalisation, and `to_ascii` can use it then.
  Rejected for now; listed in std.validate.20.
- **A regular expression for email:** cannot express the dot rules and IDNA at once, and the checks need the reasons
  (`kind`, `why`). Rejected.
- **Accepting quoted local parts and display names:** allowed by the RFCs and by `email-validator` only under
  options; a web form that accepts `"a b"@x.com` is a bug. Rejected; `Ann <a@x.org>` is a message-header form
  for `std.net.smtp` or a mail parser.
- **Phone numbering plans:** they change monthly and need data tables (libphonenumber is 400 KB of metadata).
  Rejected; the module says "basic validation" and lists what it does not know.
- **A DNS or MX check:** would need the network and a capability; a pure check is what agents can test offline.
  Rejected (std.validate.19).
- **Naming the module `check` or `valid`:** `check` is the name of a command and a common local. `validate` hides
  behind a local function of that name (std.use.4); the spec says so (std.validate.21). Kept `validate`, as asked.

## Consequences

- A user function named `validate` hides the module in that file.
- `std.net.jar.is_ip` could call `validate.is_ip` (follow-up, not done here: it is another lane's file).
- IPv6 addresses with an IPv4 tail print in hexadecimal; Python 3.13 prints dotted.
