# 1072. `--allow`: scoped capabilities checked at run time

- **Status:** Accepted
- **Date:** 2026-11-12
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)), on the designer's instruction to ship every item of the language review in 0.1.0
- **Topic:** 20 — AI semantic API; 13 — Package management (capabilities)
- **Spec rules:** [eff.allow.1–9](../spec/18-effects.md)
- **Amends:** [ADR 0165](0165-capability-denial.md) (which deferred allow-lists)

## Context

`--deny` ([ADR 0165](0165-capability-denial.md)) refuses at compile time a
program that reaches a capability. It is all or nothing: `--deny fs` stops an
agent-written tool from reading its own input, so it is rarely usable, and a
paths or hosts argument is not known until the program runs. People who run
code an agent wrote want "this program may read `./data` and call
`api.example.com`, nothing else", and no mainstream language has that built in.
The deciding criteria are AI understanding (a refusal an agent can act on) and
safety.

## Options considered

### Option A — allow-list on the command line, a static half and a run-time half (chosen)
`--allow LIST` names what the program may reach. Capabilities the list leaves out
are refused at compile time exactly as `--deny` does (`eff.cap.1`); the ones it
scopes (`fs`, `net`, `proc`, `env`) are checked by the runtime when the call
happens, against the canonical path, host and port, program name or variable.
- **Pros:** one flag, nothing to install; the run-time check sees real arguments
  (paths built from input); denial is a trap that names the entry to add.
- **Cons:** a check inside the standard library is not an operating-system
  sandbox (`eff.allow.9` lists the gaps).

### Option B — operating-system sandbox (seccomp, Seatbelt, AppContainer)
- **Pros:** covers FFI, unsafe code and children.
- **Cons:** three unrelated mechanisms with different power (no path-scoped
  rules in AppContainer without a broker, no host rules in Seatbelt or
  seccomp), root or entitlements on some, and a regression in "Mac, Linux and
  Windows at parity". It can be added under `--allow` later.

### Option C — scopes in the package manifest only
- **Pros:** reviewable, versioned.
- **Cons:** the person running the code is not always the author of the
  manifest; the flag is what an agent types. A manifest default can come later
  (it would feed the same list).

### Option D — return an error value on denial
- **Pros:** the program can recover.
- **Cons:** untrusted code is the case this serves, so it could ignore it and
  carry on with half its work done; the failure then depends on the program's
  error handling. A trap is simple and certain.

## Decision

`ail run --allow LIST FILE` and `ail check --allow LIST FILE` (`eff.allow.1`).
Entries are `NAME` or `NAME=SCOPE` (`eff.allow.2`): `fs`, `fs.read`, `fs.write`
(path prefixes), `net` (`HOST[:PORT]`, `*.DOMAIN`), `net.serve`, `proc` (program
names), `env` (names, `NAME*` prefixes), and the unscoped `clock`, `random`,
`ui`, `app` and `unsafe`. Unsafe code is refused unless `unsafe` is listed, because
a C call or a raw pointer ignores every scope.

At run time `ail run` sets `AIL_ALLOW` for the program only (not for the compiler
that ran first). `compiler/rt/native/policy.ail` reads it at each check; there is
no global slot, and no cost beyond one scan of the environment when it is not set.
Checks sit in the `fs_*`, `compress_*`, `net_connect*`, `http_exchange_on`,
`net_listen`, `proc_run_with`, `proc_start`, `proc_which`, `env_get`, `env_home` and
`env_vars` entry points the lowering calls, not in the system-call wrappers, because
the runtime itself reads files (DNS and TLS roots, time zones) and variables (proxies)
on a program's behalf. `env_raw` is the unchecked variable read for those.

Paths are made absolute, links resolved and `.`/`..` applied as the kernel does
(a `readlink` walk on Linux and macOS; `GetFinalPathNameByHandleW` on Windows),
then compared by whole components, folding case on macOS and Windows.
Windows refuses UNC and device paths, streams, reserved names and drive-relative
paths. A denial is `trap[eff.allow.N]` with exit status 1 and one line naming the
capability, the canonical argument and the entry that would allow exactly that use.

## Rationale

The same list drives the compile-time and the run-time check, so an agent learns
the allowed set once. Trapping on denial is fail closed; the message is the repair
instruction. Matching hosts by the written name and never by address keeps "DNS
name versus IP" simple and safe: a scope says which spelling is allowed.

## Consequences

- `proc` is as strong as the program allowed (`git -c`, an interpreter): the
  documentation says to allow only programs that cannot run others.
- Another process or task that can change links can race the check; the spec says so.
- `ail test --allow`, a manifest default and an operating-system backstop are open
  follow-ups; none changes this rule set.
- `ui`, `app`, `clock`, `random`, `env.cwd`, `env.args` and `env.program` are allowed or
  denied, not scoped.
- Seed: the runtime and checker change need a new seed before the stage-1 build relies on them.

## Rejected alternatives

- **OS sandbox (B):** unequal power across three systems; kept as a possible addition.
- **Manifest only (C):** the flag is what agents and people type.
- **Error value (D):** untrusted code can ignore it.
- **Hooking `os_open` and the other system-call wrappers:** the runtime's own reads
  (resolver, TLS roots, `tz`) would be refused with the program's.
- **Matching a host by resolving it:** DNS can answer differently each time; a name
  is what the program wrote and what the policy can check.
