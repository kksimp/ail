# 1219. `std.semver`: Semantic Versioning 2.0.0 versions and npm-style ranges

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.semver.1 to std.semver.12 in [std-semver.md](../spec/std-semver.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md)

## Context

Agents asked for version work (`semver_compare`, `semver_range_check` and `version_sorter` in the task corpus, `semver_ranges`
and `t4_semver_sort` in ail-train) write the same code each time: split on `.`, compare numbers, and get the prerelease
order wrong. The prerelease rules (a release outranks its prerelease; identifiers compare one by one) and the range
grammar (`^`, `~`, wildcards, `||`, hyphen ranges) are exactly the parts that go wrong. The std plan
(`std-gap-plan.md`, lane 11) lists it as a pure AIL module.

## Decision

A new module written in AIL, `std/semver/semver.ail`, in scope without `use` (std.use.4). No syntax change, no runtime
or checker work.

- `semver.parse(text) Version ! SemverError` reads SemVer 2.0.0 strictly (no `v` prefix, exactly three numbers, no
  leading zeros). `is_valid(text)` is the bool form.
- `Version` has `major`, `minor`, `patch`, `pre`, `build`. It implements `Ord` through `compare` (prerelease order of
  SemVer item 11, build ignored) and `ToStr`, so `<`, `sorted()`, `max()` and `print` work. `==` stays the automatic
  field-by-field equality, so it includes the build: `1.0.0+a == 1.0.0+b` is false while `compare` is 0.
- `bump(v, part)` takes `Part` (`Major`, `Minor`, `Patch`), an enum, and drops prerelease and build.
- `parse_range(text) VersionRange ! SemverError`, `r.matches(v)`, `satisfies(v, text) bool ! SemverError` and
  `max_satisfying(vs, r) Version?`. The grammar is the npm one: `=`, `>`, `>=`, `<`, `<=`, `~`, `^`, partial versions
  and `x`/`*` wildcards, `||`, `A - B`. A prerelease matches only an alternative that names a prerelease of the same
  `major.minor.patch` (npm's rule), so `^1.2.3` never picks `1.3.0-beta` by surprise.
- Failures are `SemverError` with an enum `kind` (`NoText`, `BadShape`, `BadNumber`, `BadPre`, `BadBuild`,
  `BadRange`) and a text (type.enum.7).
- Numeric prerelease identifiers compare by length, then text, so no `int` limit applies to them. Major, minor and
  patch must fit an `int`; a larger one is `BadNumber`.

Differences from the plan's sketch: `bump` takes an enum, not the text `"major"`, so a misspelt part is a compile
error; `max_satisfying` takes a parsed `VersionRange` and gives a plain `Version?`, since a `T? ! E` result has no
clear reading, and `satisfies` is the text form. `compiler/pkg/` is not switched to it here (follow-up).

## Rejected alternatives

- **Built-in module in the runtime.** Costs checker, effects and lowering work per function and collides in
  `check/stdlib.ail`; AIL is as fast for parsing this small.
- **`bump(v, "major")` with text.** A string where an enum is meant: a typo is found only at run time.
- **`==` that ignores the build.** It would agree with `compare`, but a replaced automatic `Eq` surprises a reader who
  expects field equality, and `Version.build` is data the caller may want to keep apart.
- **A lenient `parse` that accepts `v1.2` or `1.2`.** Hides mistakes; ranges take partial versions and a prefix, and
  coercion can be added as its own function when a program needs it.
- **Ranges only as text, with no `VersionRange` type.** `max_satisfying` over many versions would parse the range for
  each one.
- **A module named `version`.** A common local name (6 programs measured in the std plan), which hides the module
  (std.use.4); `semver` clashed with none.
