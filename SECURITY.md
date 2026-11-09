# Security policy

## Reporting a vulnerability

Report security problems **privately** through GitHub: open the repository's
**Security** tab, choose **Advisories**, then **Report a vulnerability**. Only
the maintainers see the report.

Please do not open a public issue, pull request or discussion for a
vulnerability until it is fixed or the disclosure window below has passed.

A useful report has:

- what is affected: the compiler, the runtime it embeds into programs, the
  standard library (`std.net`, `std.crypto`, TLS, X.509), or the tools;
- the commit you tested (`git rev-parse HEAD`) and your host (OS and CPU);
- the smallest AIL program or input that shows it, and what you expected;
- the impact as you see it (memory safety, wrong crypto result, certificate
  accepted that should be rejected, and so on).

## What happens next

- We acknowledge the report within **7 days**.
- We confirm or reject it and share a plan within **30 days**.
- We aim to release a fix within **90 days** of the report. After 90 days, or
  once a fix is published, whichever comes first, you may disclose the issue.
  If a fix needs longer, we will say why and agree a new date with you.
- With your permission we credit you in the fix's commit and advisory.

## Scope

In scope: everything in this repository, including code the toolchain embeds
into compiled programs (`compiler/rt/native/`) and the standard library.

Of special interest: AIL ships its **own** cryptography and TLS, written in
AIL with no outside library: TLS 1.3 client (`compiler/rt/native/tls_*.ail`),
X.509 path validation, AES-GCM, ChaCha20-Poly1305, X25519, SHA-2, SHA-3,
HMAC and HKDF (`compiler/rt/native/crypto.ail`, `hash.ail`). **This code has had no
third-party security audit.** It is checked against published test vectors
(`tools/native-tls/`), which shows it computes the right answers, not that it
resists side channels or hostile input. What is checked, fuzzed and
constant time, and what is not, is listed in
[docs/design/crypto-assurance.md](docs/design/crypto-assurance.md). Review and
reports are welcome.

Out of scope: the apps built with AIL (Garlic, Clove, Garlic Press, the editor)
live in their own repositories; programs that use `unsafe` blocks incorrectly
(ADR 0102); denial of service by a program that is given unbounded input on
purpose.

## Supported versions

AIL is pre-1.0. Only the current `main` and the newest published seed
(`tools/release/SEED_TAG`) get fixes. How seeds are built and how to check
one is in [ADR 0976](docs/decisions/0976-public-release-and-seed-trust.md).
