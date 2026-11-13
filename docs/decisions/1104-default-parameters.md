# 1104. Default parameter values: `name Type = value`, passed by name

- **Status:** Accepted
- **Date:** 2026-11-13
- **Decided by:** AI (delegated, ADR 0076, order by ADR 1077)

## Context

`expr.call.3` said there are no default values. Functions with options
(`fetch(url, retries, timeout_ms)`) need every caller to spell every option,
or need a second function per combination. The AI-first review asked for
defaults (S10), with the field-default rules of ADR 1076.

## Decision

A parameter may be written `name Type = value` (`expr.call.4`). Defaulted
parameters come last. A call may leave them out and gives one only by name,
`fetch(u, retries: 5)`; a positional argument for a defaulted parameter is an
error that says to name it. The value is a plain constant with no names or
calls (the rule of `type.struct.8`) and is evaluated at each call. A `mut` or
`move` parameter, and a method of a trait or of an `impl Trait` block, have no
default. A function value's type has every parameter. `ail api` and
`ail context` print `= value` after the parameter, and `ail diff` does not
call a new trailing defaulted parameter breaking.

## Why (ADR 1077 order)

1. AI understanding: by-name-only means a call's meaning never depends on
   counting positions past the required ones; the default sits on the
   declaration line and has no scope or effect.
2. Tokens: the common call is shorter.

## Rejected alternatives

- **Positional use of defaulted parameters.** Two defaulted `int`s could be
  swapped silently, which `type.struct.7` already refuses for fields.
- **Defaults that call functions or name constants.** Hidden effects, and the
  caller's scope is not the declaration's.
- **Overloading.** A second mechanism, and a worse diagnostic for a wrong call.
- **Defaults in trait methods.** Which impl's default wins is unclear; later.

## Consequences

- Native lowering fills an omitted argument from the declaration at the call
  (`lower_label.ail`); a generic function's default is lowered per instance.
- The compiler's own source does not use the feature in this release.
