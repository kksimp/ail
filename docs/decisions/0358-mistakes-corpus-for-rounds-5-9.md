# 0358. The mistakes corpus covers the features of rounds 5 to 9

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated tonight's serious decisions on 2026-10-20). Criterion: AI understanding (every typical slip with a new feature gets one diagnostic that names the rule and says how to repair it), then tokens.
- **Topic:** 20 AI semantic API (`gram.help.*`, ADR 0262)
- **Spec rules:** `conc.par.2`, `contract.ens.2`, `json.derive.1`, `json.derive.2`, `conc.share.6`, `own.move.2`, `expr.loop.4`

## Context

`bootstrap/tests/mistakes/` held 112 slips from the first batch of features.
None covered tasks, `with`, `par for`, ownership and `move`, contracts,
derives, tensors or std modules. Checking each candidate showed most already
had a good diagnostic. Five were wrong or missing, and one had no help.

## Decision

Add 28 programs, one slip each, with the rule IDs expected. Fix the checker
where the diagnostic was wrong, late or missing:

- A `par for` that writes through a `ref` declared outside the body is now a
  `conc.par.5` error citing `conc.par.2`. It ran, and raced.
- A derive over a field with no JSON form says so in the user's terms
  (`json.derive.1`, `json.derive.2`), not as a missing method of generated code.
- `result` in a `requires` condition is `contract.ens.2`, not "unknown name".
- `e.message()` (and `msg`, `description`) gets a help pointing to `"{e}"`.
- `for v in ch.recv()` gets a help (`expr.loop.4`).
- `ail fix` repairs `with m as x` (`as mut`) and `xs.push(f)` of a unique
  value (`move`): `conc.share.6`, `own.move.2`.

Slips whose repair needs a design choice (a dropped task handle, a wrong
channel type, an impure `requires`) carry help but no fix.

## Rationale

An agent that repairs a slip from one diagnostic spends fewer tokens than one
that reads a trap later. The mechanical fixes are the two where exactly one
edit is right.

## Consequences

- The corpus is 140 programs; `ail fix` resolves 88 (63%, target 60%).
- `ail explain` is unchanged: no rule was added, so the mean stays at the
  116 tokens of ADR 0336.
- Not covered, by decision: tensor shape mismatch is a trap (`std.tensor.4`)
  naming both shapes, since shapes are values; `fs.glob` under `--deny fs`
  is already `eff.cap.1` for any glob, and `ail check` takes no `--deny`.
  `invariant` reading a private field is legal in the struct's own module.

## Rejected alternatives

- **Check tensor shapes statically:** shapes are runtime values; a literal-only
  check would catch few real slips.
- **Fix `e.message()` by rewriting to `"{e}"`:** the receiver is often a longer
  expression, and the anchored rewrite would need its own template.
