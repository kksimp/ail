# 1255. `std.jwt`: JSON Web Tokens, HMAC signing and claim checks (unaudited)

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer approved shipping the crypto lanes labelled unaudited (2026-11-27)
- **Topic:** 17 Standard library
- **Spec rules:** [std.jwt.1-12](../spec/std-jwt.md)

## Decision

- `sign` and `verify` with HS256, HS384 and HS512, written in AIL over `std.hash`. The caller names the allowed
  algorithms; `alg: none` and `crit` are refused, the signature is compared in constant time and is checked before
  any claim.
- Claims: `exp`, `nbf`, `iat`, `iss`, `aud` (text or array), a leeway, `require_exp`, and an injectable `now` for tests.
- `sign_with` and `verify_with` take a signing function for EdDSA and ES256; `verify_with` refuses HS names, so a public
  key can never be used as an HMAC secret. `decode_unverified` reads the header (`kid`) only.
- The module and its spec say it is unaudited.

## Rejected alternatives

- Accepting the token's own `alg`: the classic algorithm-confusion attack.
- Claims checked before the signature: it reveals timing about unauthenticated input.
- Built-in EdDSA and ES256 now: the signing primitives land with lane 14 (ADR 1236); the hooks avoid waiting for it.
