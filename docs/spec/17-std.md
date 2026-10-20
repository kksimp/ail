# 17. Standard modules

Chapter key: `std`

> **Status:** Drafted (partial). Specified: importing the standard modules,
> and `std.fs`, `std.env`, `std.io`, `std.proc`, `std.random`, `std.time` and `std.ui`. Not yet
> designed: the rest of the standard library, and capability effects
> (plan §3.2), which another lane adds.

The standard modules reach the machine through small, typed functions whose
failures are values (ADR 0157). There is no string-typed shell.

## Imports

**std.use.1** — `use std.NAME` imports the standard module `NAME`, available
under the name `NAME`, or under `d` with `use std.NAME as d` (`mod.use.1`).
The standard modules are `env`, `fs`, `io`, `proc`, `random`, `time` and `ui`. Importing
`std` itself, a submodule of a standard module, or any other `std.NAME` is a
compile-time error that lists the modules. *(ADR 0157)*

**std.use.2** — A standard module's functions and types are referred to by
qualified name, `fs.read_text(path)` and `fs.IoError` (`mod.use.2`). Using
one without importing its module is a compile-time error whose help gives
the `use` line. A module is not a value, and naming a function or type the
module doesn't have is a compile-time error. *(ADR 0157)*

**std.use.3** — Two imports in one file MUST NOT make modules available under
the same name. A local MUST NOT reuse an imported module's name
(`expr.bind.5`). *(ADR 0157)*

## Errors

**std.err.1** — A failure of `std.fs` is the error struct `fs.IoError`, with
fields `path str` (the path the function was given) and `kind str`. A
failure to start a process is the error struct `proc.ProcError`, with
fields `cmd str` and `kind str`. `kind` is one of `not_found`,
`permission`, `exists`, `is_dir`, `not_dir`, `not_empty`, `invalid_utf8`
and `other`, and is the same for the same failure on every platform.
*(ADR 0158)*

```ail
use std.fs

fn load(path str) str ! fs.IoError
    fs.read_text(path)? "loading {path}"

fn main()
    text := fs.read_text("notes.txt") else err
        print("{err.kind}: {err.path}")
        ""
```

## Files: `std.fs`

**std.fs.1** — `read_text(path str) str ! IoError` returns the whole file as
text. A directory fails with `is_dir`; a file that isn't UTF-8 fails with
`invalid_utf8`. *(ADR 0157)*

**std.fs.2** — `write_text(path str, text str) ! IoError` creates the file or
replaces its contents. *(ADR 0157)*

**std.fs.3** — `append_text(path str, text str) ! IoError` creates the file
or adds to its end. *(ADR 0157)*

**std.fs.4** — `exists(path str) bool` is whether anything exists at the
path. *(ADR 0157)*

**std.fs.5** — `list_dir(path str) List[str] ! IoError` returns the names
(not paths) of the directory's entries, sorted by their bytes. *(ADR 0157)*

**std.fs.6** — `remove(path str) ! IoError` removes a file or an empty
directory. A directory with entries fails with `not_empty`; nothing removes
a tree. *(ADR 0157)*

**std.fs.7** — `make_dir(path str) ! IoError` creates a directory and any
missing parents. An existing directory is not a failure. *(ADR 0157)*

## Environment: `std.env`

**std.env.1** — `args() List[str]` returns the program's arguments: the
words after the program's file, so `ail run tool.ail a b` gives
`["a", "b"]`. *(ADR 0159)*

**std.env.2** — `get(name str) str?` returns an environment variable, or
`none` when it isn't set or isn't UTF-8. *(ADR 0157)*

**std.env.3** — `cwd() str` returns the working directory. If it can't be
read, the program traps. *(ADR 0157)*

**std.env.4** — `program() str` returns the program's file path as it was
given to `ail run`. *(ADR 0159)*

## Standard input and output: `std.io`

**std.io.1** — `read_line() str?` reads one line from standard input,
without its line ending (`\n` or `\r\n`), or returns `none` at the end of
input. Bytes that aren't UTF-8 become U+FFFD. *(ADR 0157)*

**std.io.2** — `read_all() str` reads the rest of standard input.
*(ADR 0157)*

**std.io.3** — `eprint(value)` writes the value's text form and a newline to
standard error, as `print` does to standard output (`mod.main.3`). Output
already printed reaches standard output first. *(ADR 0157)*

**std.io.4** — `exit(code int)` ends the program with exit status `code`,
after writing any pending standard output. It never returns, so the code
after it is unreachable (ADR 0139). A status outside 0 to 255 traps.
*(ADR 0157)*

## Processes: `std.proc`

**std.proc.1** — `run(cmd str, args List[str]) Output ! ProcError` runs the
program `cmd` with the arguments `args`, directly and without a shell, and
waits for it. `cmd` is found on `PATH` unless it contains a path separator.
The process inherits the working directory and the environment, reads an
empty standard input, and its standard output and standard error are
captured. The result is `struct Output { status int, out str, err str }`:
the exit status (128 plus the signal number if a signal ended it, as shells
report it) and the captured text, with bytes that aren't UTF-8 as U+FFFD. A
process that runs and fails is a successful `run` with a nonzero
`status`; only a process that can't start is a `ProcError`. *(ADR 0160)*

```ail
use std.proc

fn main() !
    r := proc.run("git", ["status", "--short"])?
    if r.status != 0
        print("git failed: {r.err}")
```

## Clock: `std.time`

**std.time.1** — `now_ms() int` returns the time in milliseconds since
1970-01-01T00:00:00Z. *(ADR 0157)*

**std.time.2** — `sleep_ms(ms int)` pauses the program for at least `ms`
milliseconds. A negative `ms` traps. *(ADR 0157)*

## Randomness: `std.random`

**std.random.1** — `use std.random` provides `int`, `float`, `seed` and
`shuffle`. Each has the capability `random` (`eff.infer.4`). *(ADR 0206)*

**std.random.2** — `int(lo int, hi int) int` returns a uniformly distributed
integer with `lo <= n <= hi`; both bounds are inclusive. It traps if `lo >
hi`. `float() float` returns a uniformly distributed float with `0 <= x <
1`. *(ADR 0206)*

**std.random.3** — The functions draw from one generator. `seed(s int)`, or
`ail run --seed S`, sets its state from `s`: the draws that follow are then a
function of `s` and of the calls made, so a run repeats. Without a seed the
generator starts from the clock. The sequence for a given seed is not
specified across releases. `--seed` needs an integer (exit status 2
otherwise). *(ADR 0206, ADR 0207)*

**std.random.4** — `shuffle(mut xs)` reorders any `List[T]` in place so that
every order is equally likely (Fisher-Yates). The argument is marked `mut`
(`own.param.2`) and must be a list. *(ADR 0206)*

**std.random.5** — `ail run --deterministic` rejects every program that
reaches `random` (`eff.det.2`). *(ADR 0207)*

```ail
use std.random

fn main()
    random.seed(1)
    mut deck := [1, 2, 3, 4, 5]
    random.shuffle(mut deck)
    print(deck.len())
    print(random.int(1, 6) <= 6)
```

## Windows: `std.ui`

A window showing a grid of monospace text cells, with keyboard and resize
events: enough for a text editor or a dashboard. Colors are `int`s
`0xRRGGBB`; only the low 24 bits count. *(ADR 0177)*

```ail skip=context
struct Window              # a handle; at most one window is open
struct Size
    cols int
    rows int
struct Modifiers
    ctrl bool
    alt bool
    shift bool
enum Event
    Key(key str, ctrl bool, alt bool, shift bool)
    Resize(cols int, rows int)
    Close
    Mouse(col int, row int, button str, kind str)
    Scroll(lines int)
error struct UiError
    kind str               # "no_display", "closed" or "other"
    message str
```

**std.ui.1** — `open(title str, cols int, rows int) Window ! UiError` opens
a window `cols` by `rows` cells in size, with every cell blank (a space,
light grey on black). It fails with `kind` `no_display` when no window can
be shown (no display, or a run without a window host), and `other` when
`cols` or `rows` is outside 1 to 1000 or a window is already open. *(ADR 0177)*

**std.ui.2** — The methods of `Window` are `size`, `clear`, `put`, `cursor`,
`present`, `next_event`, `poll_event`, `modifiers`, `set_title` and `close`; naming another is a
compile-time error. All but `size(self) Size` and `modifiers(self) Modifiers`
take `mut self`. `size` is the
grid's size in cells, which follows `Resize` events; it is 0 by 0 for a
window that was never opened or has been closed with `close`. *(ADR 0177)*

**std.ui.3** — `clear(mut self, bg int)` makes every cell a space on `bg`.
*(ADR 0177)*

**std.ui.4** — `put(mut self, col int, row int, text str, fg int, bg int)`
writes `text` into the cells from (`col`, `row`) rightwards, one character
per cell; columns and rows count from 0 at the top left. What falls outside
the grid on any side is dropped, so any `col` and `row` are allowed. A
character outside printable Latin-1 (U+0020–U+007E, U+00A0–U+00FF),
including a tab, shows as `?`. *(ADR 0177)*

**std.ui.5** — Drawing is buffered: `clear` and `put` change only the grid,
and `present(mut self)` shows it. `cursor(mut self, col int, row int)` sets
where the caret, a thin bar in white or black, whichever shows on the cell's
background, is drawn at the next `present`; a position outside the grid,
such as (-1, -1), hides it. *(ADR 0177)*

**std.ui.6** — `next_event(mut self) Event` waits for the next event;
`poll_event(mut self) Event?` returns `none` at once if there is none.
`Resize` reports a new size in cells, and the grid has that size before the
event is returned: cells that still fit are kept, new cells are blank.
`Close` means the window was closed (std.ui.8). *(ADR 0177)*

**std.ui.7** — In `Key(key, ctrl, alt, shift)`, `key` is either the typed
character (`"a"`, `"A"`, `" "`, `"é"`, following the keyboard layout and
Shift) or one of the names `enter`, `tab`, `backspace`, `delete`, `escape`,
`up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown` and `f1`
to `f12`. While Ctrl is held, a text key gives its unshifted character
(Ctrl-S is `Key("s", true, false, false)`, Ctrl-Shift-S has `shift` true),
and no typed character is reported. On macOS, Command reports as `ctrl`.
Modifier keys alone give no event. *(ADR 0177)*

**std.ui.8** — When the user closes the window, the next event is `Close`.
After that, and after `close(mut self)`, the window is closed: `next_event`
and `poll_event` return `Close`, and drawing does nothing. A program that
ends closes its window. *(ADR 0177)*

**std.ui.9** — `ui.open`, `ui.clipboard_get`, `ui.clipboard_set` and every
`Window` method give the capability `ui`
and the effect `io` (`eff.infer.4`), so `ail run --deny ui` refuses a program
that can open a window (`eff.cap.1`). *(ADR 0177)*

```ail
use std.ui

fn main()
    mut w := ui.open("Hello", 40, 5) else err
        print("no window: {err.kind}")
        return
    while true
        w.clear(0x202020)
        w.put(1, 1, "Hello, AIL (Esc quits)", 0xFFFFFF, 0x202020)
        w.present()
        match w.next_event()
            Key(key, _, _, _) =>
                if key == "escape"
                    break
            Resize(_, _) | Mouse(_, _, _, _) | Scroll(_) => continue
            Close => break
    w.close()
```

**std.ui.10** — `Mouse(col, row, button, kind)` reports a mouse button over
cell (`col`, `row`): `button` is `left`, `right` or `middle`, and `kind` is
`down` when it is pressed, `up` when it is released, and `drag` when the
mouse moves to another cell while it is held. Movement with no button held
gives no event. *(ADR 0240)*

**std.ui.11** — `Scroll(lines)` reports the wheel or a trackpad: `lines` is
the number of text lines to scroll, positive towards the end of a document.
A wheel notch is about 3 lines. *(ADR 0240)*

**std.ui.12** — `clipboard_set(text str)` puts `text` on the clipboard, and
`clipboard_get() str?` returns its text, or `none` when it holds no text.
Under `ail run` this is the system clipboard where the platform's tools
reach it (`pbcopy` on macOS, `wl-copy`, `xclip` or `xsel` on Linux,
PowerShell on Windows); otherwise, and always under `ail test`, it is a
clipboard of the program's own, so tests never read or change the user's.
Neither needs an open window. *(ADR 0240)*

**std.ui.13** — `set_title(mut self, text str)` changes the window's title;
on a closed window it does nothing. *(ADR 0240)*

**std.ui.14** — `modifiers(self) Modifiers` returns which of Ctrl, Alt and
Shift were held when the last event that `next_event` or `poll_event`
returned happened (as std.ui.7 reports them, Command counting as Ctrl on
macOS), so a program can tell a Shift-click from a click. Before any event,
and on a closed window, all three are false. *(ADR 0242)*
