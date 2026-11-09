# 0998. std.net.stun and std.net.ice: STUN and ICE

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** `std.stun.1` to `std.stun.4`, `std.ice.1` to `std.ice.6`
- **Builds on:** [ADR 0997](0997-std-net-udp.md), [ADR 0959](0959-std-crypto.md)

## Context

WebRTC peers must find a path to each other through NATs. That is STUN
(RFC 8489: a binding request tells a host its public address) and ICE (RFC
8445: both sides gather candidate addresses, pair them, and probe each pair
with STUN until one works). Nothing else in `std` speaks either protocol.

## Decision

1. `std.net.stun` (`std.stun.1-4`):
   - `binding_request() Message` builds a Binding request with a fresh 96-bit
     transaction id from system random; `encode(m) bytes` and `decode(b bytes)
     Message ! StunError` do the wire format (20-byte header, magic cookie
     0x2112A442, attributes padded to 4 bytes).
   - `Message { kind int, txid bytes, attrs List[Attr] }`; `Attr { kind int,
     value bytes }`. Helpers `xor_mapped(m) str?` (text `ip:port`, XOR undone,
     v4 and v6), `add_username`, `add_priority`, `add_use_candidate`,
     `add_ice_controlling`.
   - `add_integrity(m, key bytes)` and `add_fingerprint(m)` append
     MESSAGE-INTEGRITY (HMAC-SHA1) and FINGERPRINT (CRC-32 XOR 0x5354554E);
     `check_integrity` verifies in constant time. HMAC-SHA1 and SHA-1 are added
     to `std.crypto` if absent (needed anyway by SRTP, ADR 1001).
   - `query(u Udp, server str) str ! NetError` sends a Binding request and
     returns the reflexive address, retrying 3 times (RTO 500 ms, doubled).
2. `std.net.ice` (`std.ice.1-6`), full ICE agent, no lite mode:
   - `Agent` with `new(controlling bool, stun_servers List[str]) Agent`;
     `gather(mut a) List[Candidate]` makes host candidates for the local
     interfaces and server-reflexive ones through STUN; `Candidate` has the
     RFC 8839 `candidate:` line text form (`c.line()`, `parse(line)`).
   - `a.set_remote(ufrag, pwd, cands)` and `a.add_remote(c)` (trickle).
   - `a.connect(timeout_ms) Pair ! IceError` runs connectivity checks over the
     checklist (priority by RFC 8445 §5.1.2) and nominates one pair
     (regular nomination); `Pair` carries the `Udp` and the remote text address,
     and `a.send(bytes)` / `a.recv() bytes` carry the data after.
   - Demultiplexing STUN from DTLS/RTP on the shared socket follows RFC 7983
     (first-byte ranges), so ADR 1000 and 1001 share the socket.
3. No TURN in this ADR: relay candidates are a later ADR (it needs long-term
   credentials and TCP framing); `IceError` kind `no_path` says so when only a
   relay would work.
4. Tests run on loopback: a local STUN server task (written in the test) and
   two agents checking each other, plus the RFC 5769 test vectors for the
   message encoder, integrity and fingerprint.

## Consequences

- New modules must be registered in `load.ail`, `lower_mod.ail`, `lower_err.ail`
  and `names.ail`, each pub fn needs a tests/cases case, each rule an example.
- Windows and Linux use the same code over `Udp`; interface listing needs a
  small runtime call per OS (`getifaddrs`, `GetAdaptersAddresses`).

## Rejected alternatives

- ICE lite only: rejected, browsers behind NATs need full checks.
- Text-only STUN messages: rejected, the format is binary.
- Aggressive nomination: rejected, it is deprecated by RFC 8445.
- Folding STUN into `std.net`: rejected, it keeps `std.net` small and the
  token cost of unrelated programs unchanged.
