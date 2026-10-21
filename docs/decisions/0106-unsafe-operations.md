# 0106. The set of unsafe operations

- **Status:** Accepted
- **Date:** 2026-10-04
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** 15 — Unsafe semantics
- **Spec rules:** [unsafe.op.1–2](../spec/13-unsafe.md)

## Deciding criterion

The brief (§16) lists the required capabilities. **AI understanding** decides
the rule: the set is closed and enumerated, so "is this unsafe?" has a
definite answer. The raw-pointer spelling `*T` (1 token) was chosen on
**token usage**.

## Decision

The following are unsafe operations, allowed only in `unsafe` blocks:
1. dereferencing a raw pointer `*T` (reading or writing through it), and pointer arithmetic;
2. converting a managed reference to a raw pointer (`plat.ptr.3`), and pinning;
3. manual allocation and deallocation;
4. reinterpreting memory as another type;
5. calling an `unsafe fn` or a C function (ADR 0104);
6. inline assembly and hardware-specific intrinsics with preconditions.

Creating or copying a raw pointer value is **not** unsafe. Using it is.

## Consequences

- The detailed semantics (aliasing rules for raw pointers, what is undefined
  behaviour inside `unsafe`) are to be specified before the backend relies
  on them.

## Rejected alternatives

- Marking the creation or copying of a raw pointer as unsafe as well as its use: rejected because creating a pointer value touches no memory, so the unsafe marker belongs on the use. (reconstructed 2026-10-21: the record states the rule without a debate.)
- A longer spelling of the raw pointer type: `*T` was chosen for its one-token cost (token usage).
