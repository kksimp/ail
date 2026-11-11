# 1012. `std.term`: terminal control for text programs

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.term`)
- **Spec rules:** std.term.1-8 ([std-term.md](../spec/std-term.md))

## Context

AIL has no way to write a terminal program: raw mode, key and mouse input, the alternate screen, colours and the size are all missing. The only text grid is the `std.ui` window (ADR 0177), which is a window, not a terminal. Rust has crossterm and C has ncurses; htop-style tools, editors and TUIs need the same.

## Options considered

### Option A — a std module written in AIL over libc calls (chosen)
`std/term/term.ail` declares `tcgetattr`, `tcsetattr`, `cfmakeraw`, `poll`, `read` and `write` with `extern c fn`, as `std.gfx.host` does for `mmap`. The decoder and styles are pure AIL.
- **Pros:** no runtime or lowering change, so macOS and Linux, arm64 and x86-64 share one source (ADR 0008); the decoder is a pure function that `tests/cases` checks without a terminal.
- **Cons:** the FFI cannot call variadic functions (`ffi.decl.3`), so `ioctl(TIOCGWINSZ)` is out; Windows needs a different set of calls (below).

### Option B — runtime functions in `compiler/rt/native`
- **Pros:** `ioctl` and the Windows console API are reachable.
- **Cons:** three OS arms in the runtime, a lowering entry, a record/replay entry and a runtime global slot for a feature that C already provides on two of the three targets.

### Option C — terminfo-driven, like ncurses
- **Pros:** works on terminals nobody has used since 1995.
- **Cons:** a database to read and a capability layer for sequences every supported terminal sends the same way.

## Decision

Option A. The size comes from the terminal itself: save the cursor, move to row and column 9999, ask for the cursor position (`ESC [ 6 n`), restore it, and read `ESC [ row ; col R`. `Term.poll` repeats that at most every 250 ms while it waits, and a changed size is a `Resize` event. When `std.os` delivers SIGWINCH (ADR 1011) `poll` can wake on that instead; the event does not change. Mouse reports use the SGR form (`?1006`); the wheel is `Scroll(-3)` and `Scroll(3)` as `std.ui.11`. `Event` mirrors `std.ui` (`Key`, `Mouse`, `Scroll`, `Resize`), so a program can share its handler.

`Term` is a plain value closed with `close()` (idempotent), used with `defer`. A `unique` type with `Drop` would restore the terminal on every path, but `pub unique struct` is not accepted by the parser's `pub` rule (`Parser.pub_marker`), and changing the parser needs a new seed before the std source can use it. That is a follow-up.

Windows: the same module calls kernel32 (`GetConsoleMode`, `SetConsoleMode`, `WaitForSingleObject`, `ReadFile`, `WriteFile`, `GetConsoleScreenBufferInfo`, `SetConsoleOutputCP`), bound lazily by `ffi.load.3` (ADR 0980), so a Unix run never loads them. Input is set to virtual-terminal input (0x200), which makes the console deliver the same escape sequences as a Unix terminal, so `parse` is shared. Output is set to processed plus virtual-terminal processing (1 | 4 | 8). The size is read from the console window, so Windows needs no cursor-position query. Both code pages are set to UTF-8, and `close` restores the saved modes (the code pages stay). It needs a console that supports virtual-terminal output (Windows 10 1809 and later); `open` fails with `TermError` otherwise. Unverified on ail-win until the gate runs there (ADR 0925).

## Gap found: variadic C functions

`ioctl`, `fcntl` and `open` are variadic in C, and the FFI cannot call variadic functions (`ffi.decl.3`, ADR 0926 skips them; on arm64 macOS variadic arguments also go on the stack, which a plain call would not match). `ioctl(TIOCGWINSZ)` is the clean way to get a terminal size and was out of reach, as are device `ioctl`s generally. For the systems ABI lane: specify variadic calls (`extern c fn ioctl(fd i32, req int, ...)`) with the per-target convention (macOS arm64 stack, Linux and Win64 register rules).

## Rationale

Criteria in order: tokens (a short module, no new lowering), AI understanding (one pure decoder, events as in `std.ui`), readability, then speed (input is human-rate). Option A is the smallest that is complete on the first-class Unix targets.

## Consequences

- Programs can build TUIs today on macOS and Linux; `ail-edit` can drop its window host for a terminal front end if wanted.
- While a `Term` is open the program must not read standard input another way, and Ctrl-C arrives as a key.
- A cursor-position reply from row 1 reads as F3 with a modifier (`ESC [ 1 ; m R`) and is dropped as a size; a terminal at least two rows tall is not affected.
- Native only (ADR 0865).

## Rejected alternatives

Option B costs three runtime arms for what libc already does. Option C reads a database to learn what the terminal would have said. Reading the size from `LINES` and `COLUMNS` was rejected because shells do not export them.
