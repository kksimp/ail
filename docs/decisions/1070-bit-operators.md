# 1070. Bit operators `& | ^ ~ << >>` as sugar for the `int` methods

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, ADR 0076; the designer asked on 2026-11-12 that every recommendation of the pre-release language review go into 0.1.0)
- **Topic:** 11 — Operators; integer arithmetic
- **Spec rules:** `expr.bit.3`–`expr.bit.5` (new); `expr.bit.1`–`expr.bit.2` unchanged
- **Supersedes:** [ADR 0465](0465-bit-operations-as-int-methods.md) (the methods stay); amends [ADR 0077](0077-operator-traits.md) (the operator set) and [ADR 0695](0695-fix-bitwise-not.md) (`~a` is no longer a mistake)

## Context

ADR 0465 measured the operators at 8% to 23% fewer tokens on bit code and
chose methods only because they were cheaper to implement. The compiler's own
hashes, codecs and object writers are nearly all bit code, and the
designer's order puts tokens first (ADR 0076). The review ranked this a
reversal worth making.

## Decision

1. Infix `&`, `|`, `^`, `<<`, `>>` and prefix `~` are operators. Each is
   exactly the method call it names: `a & b` is `a.bit_and(b)`, `a | b` is
   `a.bit_or(b)`, `a ^ b` is `a.bit_xor(b)`, `a << k` is `a.shl(k)`,
   `a >> k` is `a.shr(k)`, `~a` is `a.bit_not()`. The parser builds the method
   call, so checking, lowering and the trap of `expr.bit.2` are those of the
   methods. The methods stay.
2. **No bit operator mixes with another operator without parentheses
   (`expr.bit.4`).** C ranks `&` below `==` and a model that remembers Rust or
   Python writes the opposite, so no ranking is defined at all. An operand of a
   bit operator is a unary expression (a name, literal, call, `-x`, `~x`,
   a parenthesised expression). A chain repeats one operator (`a & b & c`);
   `a | b & c`, `a << 1 << 2`, `a & b == c`, `a + b & c` and `a & b + c` are
   errors whose help gives the parenthesised spelling. A shift never chains.
3. `x op= e` for `op` in `& | ^ << >>` is `x = x op e` for a plain name `x`
   (`expr.bit.5`). A place with an index or a field is written out in full.
4. `&x` as a prefix keeps its old diagnostic (there are no references).
5. A trailing `|` does not continue the line (the other operators do), because
   `|` also closes a key function's `|x|`; wrap in parentheses to break a long
   `|` chain.
6. The compiler's own source does not use the operators in this release: it is
   built by the pinned seed. A later seed lets it.

## Measurement

The table of ADR 0465 stands (operators 8% to 23% fewer tokens on bit-only
code). Parentheses add two tokens where a comparison or arithmetic meets a bit
operator, which the same rule makes unambiguous for a reader and for a model.

## Options considered

- **Copy C's table.** Rejected: the C ranking is the best-known operator trap.
- **Copy Rust's table** (bit operators above comparison, below arithmetic).
  Rejected for AI understanding: a mis-remembered ranking gives a silent wrong
  program; a required parenthesis gives a diagnostic and costs two tokens.
- **Keep the methods only** (ADR 0465). Rejected: it left the measured token
  cost on the code AIL's own tools are written in.

## Rejected alternatives

- Compound assignment on indexed places: rejected, it would evaluate the place once and needs checker work; the written-out form is clear.
- A new AST node: rejected, the method call gives identical checking and lowering with no new match arms.
