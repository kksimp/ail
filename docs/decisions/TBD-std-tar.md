# TBD. `std.tar`: tar archives, read and written, with gzip

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md)); the designer asked for the std library to be filled out on 2026-11-26
- **Topic:** 17 — Standard library; AI understanding
- **Spec rules:** std.tar.1 to std.tar.14 in [std-tar.md](../spec/std-tar.md)
- **Extends:** [ADR 1126](1126-std-modules-in-ail-without-use.md), [ADR 0446](0446-zip-archives.md)
- The number is assigned by the integrator; the file and the index row are renamed with it.

## Context

`std.compress` has gzip and zip but no tar, so a program that reads a `.tar.gz` (a release, a package, a backup) or
makes one has to write the 512-byte header format itself, and the checksum, the octal fields, the long-name rules
and the unsafe-path checks are where such code goes wrong. `compress.unzip` already refuses unsafe names
(std.compress.5); tar needs the same rule, and more, because a tar entry can also be a link. The std plan
(`std-gap-plan.md`, lane 17) lists it as a pure AIL module.

## Decision

A new module written in AIL, `std/tar/tar.ail`, in scope without `use` (std.use.4). No syntax change, no runtime
or checker work.

- An archive in memory is a `List[Entry]` (`name`, `kind`, `mode`, `mtime`, `data`, `link`). `read(bytes)` and
  `write(entries, gzip: bool = false)` are pure; `list`, `extract` and `create` work on archive files. The names
  `list`, `extract` and `create` are the ones an agent types for tar (`ail ask std`).
- `write` produces POSIX ustar, with a `prefix` split for long names, a pax `x` header when no split fits or a link
  target is long, and the same bytes for the same entries (ids 0, no names, two end blocks and no padding to
  10240 bytes). `read` also takes pax records, GNU `L`/`K` long names, V7 archives, base-256 numbers and checks
  every checksum. Other tar variants (sparse, multi-volume) are `Special` entries with no content.
- Gzip is detected on read by its magic, not by a name or a flag; `write(gzip: true)` and `create` to a `.gz` or `.tgz`
  file make it.
- `extract` validates every entry before it writes anything, with the rule of `unzip` (no absolute path, no `..`, `.`
  or empty component, no `\` or `:`) plus: a link may point only below the directory (judged by `path.clean` on the
  text), no entry may run through a link of the same archive, a `Special` entry is refused, and a hard link is a copy.
  The permission bits are the low nine (no setuid), so an archive cannot make a setuid file.
- Failures are `TarError` with an enum `kind` (`Corrupt`, `UnsafePath`, `Unsupported`, `TooBig`, `BadEntry`) and a
  text (type.enum.7). Content is held to 2^30 bytes, as `std.compress.4` does.

## Rejected alternatives

- **Built-in module in the runtime.** Costs checker, effects and lowering work per function, collides in
  `check/stdlib.ail`, and parsing 512-byte headers is not a speed problem.
- **Reading and writing streams of entries.** Needs a handle type with a lifetime; the in-memory list covers
  the common uses (config bundles, packages, test fixtures) and `std.compress` has the same limit.
- **Extracting what is safe and skipping the rest.** A hostile archive then half-extracts with no error; refusing
  before any write is what the zip rule already does.
- **Following the system's `tar` for links (any target, written as is).** The classic attack is a link to `..`
  followed by a file below it; a lexical check on the target plus the no-write-through-a-link rule closes it without
  reading the file system.
- **A name that clashes with common locals.** `tar` was measured in the std plan at 0 clashes in the corpus, examples
  and ail-train.
- **Padding the archive to 10240 bytes.** An old tape habit; GNU and bsdtar read archives without it.
- **A leniency flag on `extract` for unsafe names.** The check is the module's point; a caller that wants other
  behaviour can use `read` and write the files itself.
