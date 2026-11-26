# 1217. `std.ini`: INI files into `Json`, `write`, and `.env` files into a map

- **Status:** Accepted
- **Date:** 2026-11-26
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for a fuller std on 2026-11-26
- **Topic:** 17 — Standard library
- **Spec rules:** [std.ini.1-12](../spec/std-ini.md)

## Context

Programs read INI files (git, php, systemd units, Windows tools) and `.env` files (every container and web
project). The ail-train tasks `ini_sections` and `t4_ini_query` and the corpus's four ini tasks each hand-write the
parser. `std.toml` and `std.yaml` already read into `Json`; INI had no module and `.env` had no home.

## Decision

- `std.ini` is an AIL module (`std/ini/ini.ail`, pure, no runtime or checker work). `ini.parse(text) Json ! IniError`
  gives the shape of `toml.parse`: keys before a section are object members, a `[name]` section is an object member.
- Every value is a `Str`; section names are kept as written (`[a.b]` is the member `a.b`).
- Comments start with `;` or `#`; `=` and `:` both separate; values may be `"..."` with escapes or `'...'` literal;
  a bare value ends at a `;` or `#` after a blank.
- A duplicate key or section is an `IniError` with its line, in `parse` and in `parse_env`. This is a decision, not
  a default: see the rejected alternatives.
- `ini.write(j) str` is the inverse for objects of strings; other values are written as their JSON text.
- `ini.parse_env(text) Map[str, str] ! IniError` reads `.env` text: `export`, `#` comments, bare, `"` and `'`
  values (quoted values may span lines), and `${NAME}` / `${NAME:-default}` expanded from the file's earlier
  keys only. It never reads the process environment, so it stays pure.
- The module name `ini` clashes with no local name in the corpus, examples or ail-train (measured 0 in the std-gap
  plan), so it is a safe implicit module under std.use.4.
- No syntax changes.

## Rejected alternatives

- **Typed values (numbers, booleans).** INI has no types; a guess reads `007` and `yes` wrongly. The program calls
  `to_int()` where it expects a number.
- **Last value wins for a repeated key.** Hides a mistake in a file that looks fine, against AI understanding, the
  first criterion of ADR 1077. Tools that repeat keys on purpose (git config lists) are not served by this module.
- **`[a.b]` as nested tables.** The dot is part of the name in the files that exist; splitting is one call for the
  program that wants it.
- **`parse_env` returning a map with no error.** A line with no `=` would be dropped silently, the same mistake class
  as a duplicate. The error carries the line.
- **`${VAR}` from the process environment.** Makes `parse_env` impure and dependent on the machine; a program that
  wants it merges `env.get` itself.
- **Continuation lines and `$NAME` without braces.** Not portable across INI readers and `.env` readers; refused
  rather than guessed.
- **Runtime built-ins.** Costs checker, effects and lowering work per function and collides in `check/stdlib.ail`
  for text that AIL parses as fast.
