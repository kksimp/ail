# 0765. A `return` in a plain compiled body returns its value directly

- **Status:** Accepted
- **Date:** 2026-10-21
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** 13 — Standard library (the interpreter's speed)
- **Spec rules:** none (no language behaviour changes)
- **Builds on:** [ADR 0359](0359-bytecode-tier-in-the-bootstrap.md) (the bytecode tier)

## Context

`return e` in compiled code stored the value in `self.returned`, called `leave`
to end the blocks around it, and unwound with `Err(Exit::Return)` through
`exec`, `run_code` and `run_call`, which took the value back. A profile of
`binarytrees` (518,850 calls, half of them `return 1` from `check`) put about
50 instructions per return in that detour. Blocks need ending only for
tasks (`end_block` does nothing else without them), and `run_code` truncates the
environment itself.

## Decision

When the body is `plain` (no receiver, contract, extern body, inherited
contract or error wrapping, so nothing reads the position of the return) and
the program has no tasks or groups in play (`self.conc` is empty), `Ins::Ret`
returns `Ok(value)` from `exec` as `Ins::End` does. Every other body keeps the
unwinding path.

## Measured (callgrind, Linux x86-64)

binarytrees, depth 14 default size: 8,763.2 M to 8,613.7 M instructions
(-1.7%). The other 17 `tools/perfbench` programs are unchanged within 0.1%.
A trivial call costs about 487 instructions in total; the profile shows no
single item above 15% (operand decoding through the environment, the register
and environment pools, bounds checks), which is why this change is small.

`./ail selftest` passes. The bytecode fuzz (`tests/fuzz/bcprogs.rs`) agrees with
the walker on 8,000 programs.

## Rejected alternatives

- **Passing the arguments straight into one shared environment.** Trap
  snapshots (trace.snap.2) list the locals of each frame and would list the
  caller's too; lane 190 measured its frame redesign slower and dropped it.
- **Skipping the register clear at the end of a call.** The registers hold
  `Rc` clones that must drop with the frame, or the next write to a list the
  frame copied would copy it again (own.copy.1).
