# Standard library: `std.term`

Chapter key: `std`

> **Status:** Drafted (ADR 1012). Written in AIL over the C library on macOS
> and Linux and over kernel32's console calls on Windows (virtual-terminal
> input and output, so the sequences and `parse` are the same).

Terminal control for text programs: styles, the cursor, raw input with key,
mouse and resize events, and the alternate screen. The window grid of
`std.ui` (ADR 0177) is not a terminal; this is for programs run in one.

**std.term.1** — `use std.term` provides the style and cursor functions. Each
returns a control sequence as a `str`, for the program to print or to give to
`Term.write`: `reset()`, `bold()`, `underline()`, `reverse()`; `fg(n)` and
`bg(n)` for colour `n` of the 256-colour palette; `rgb(r, g, b)` and
`bg_rgb(r, g, b)` for true colour; `goto(col, row)` for a cell counted from 0
at the top left; `clear()`, `clear_line()`, `hide_cursor()`, `show_cursor()`,
`enter_alt()` and `leave_alt()`. A colour channel or palette index outside 0
to 255 is brought to that range, and a negative `col` or `row` to 0. They are
pure and need no capability. *(ADR 1012)*

**std.term.2** — `parse(data bytes) List[Event]` decodes bytes read from a
terminal into events, in order. `Event` has the variants `Key(key, ctrl, alt,
shift)`, `Mouse(col, row, button, kind)`, `Scroll(lines)` and `Resize(cols,
rows)`. `key` is a `term.KeyCode`, with the variants, methods and
`key_code(name)` of `ui.KeyCode` (`std.ui.7`): `Text` of the typed character
(UTF-8 is decoded), or `Enter`, `Tab`, `Backspace`, `Delete`, `Insert`,
`Escape`, `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`
and `F(1)` to `F(12)`. A control byte is its letter with `ctrl` (byte 3 is `Text("c")`
with Ctrl); Shift-Tab is `Tab` with `shift`; a modifier parameter in a
sequence (`ESC [ 1 ; 5 C`) sets `ctrl`, `alt` and `shift`; ESC before a key
sets `alt`; a lone ESC is `Escape`. A sequence cut off at the end of the
data, or one `parse` doesn't know, gives no event. *(ADR 1012, ADR 1068)*

**std.term.3** — A mouse report in the SGR form (`ESC [ < b ; x ; y M` or
`m`, which `open(true)` asks for) is `Mouse(col, row, button, kind)` with
`col` and `row` counted from 0, `button` a `term.MouseButton` (`Left`,
`Middle`, `Right`) and `kind` a `term.MouseKind` (`Down`, `Up`, or `Drag` for
movement with a button held). Movement with no button gives no event. The
wheel is `Scroll(-3)` upwards and `Scroll(3)` downwards, as `std.ui.11`.
*(ADR 1012, ADR 1068)*

**std.term.4** — `is_tty() bool` is true when standard input and standard
output are both a terminal (a console, on Windows). *(ADR 1012)*

```ail rules=std.term.1,std.term.2,std.term.3,std.term.4 norun
use std.term

fn main()
    print(term.is_tty())
    for e in term.parse(b"\x1b[1;5Cq\x1b[<0;10;5M\x1b[<64;1;1M")
        print(e)
    print("{term.fg(196)}red{term.reset()}{term.goto(4, 2)}")
```

**std.term.5** — `open(mouse bool) Term ! TermError` puts the terminal in raw
mode, switches to the alternate screen, hides the cursor and, if `mouse` is
true, asks for mouse reports; then it reads the terminal's size. On Windows it sets the console to virtual-terminal input and output and both code pages to UTF-8, and `close` restores the old modes. In raw mode
a typed key reaches the program at once and is not shown, Ctrl-C and Ctrl-Z
are the keys `c` and `z` with `ctrl` (no signal is raised), and output
processing is off, so a line ends with `"\r\n"`. `t.close()` puts all of that
back, and closing twice does nothing. A `Term` is a plain value: write
`defer t.close()` after `open`. The program must not read standard input by
any other means while it holds a `Term`. *(ADR 1012)*

**std.term.6** — `t.poll(ms int) Event?` waits up to `ms` milliseconds (a
negative `ms` waits for ever) for the next event, in the order they were
typed, and gives `none` when none came or standard input has ended. A change
of the terminal's size is a `Resize` event, seen within 250 ms while `poll`
waits (on Unix it asks the terminal for the cursor position after moving it to the
far corner and compares; on Windows it reads the console window; a signal-driven notice waits for `std.os`, ADR
1011). `t.size() Size` is the size, `cols` and `rows`, as of the last `open`
or `poll`. *(ADR 1012)*

**std.term.7** — `t.write(s str)` writes `s` to standard output at once,
without waiting for a newline, after anything the program has printed.
*(ADR 1012)*

**std.term.8** — `open` fails with a `TermError` whose `why` begins `std.term.`
when standard input or output is not a terminal, or when the terminal's or console's settings
can't be read or set (a Windows console without virtual-terminal output). `TermError` has the field `why`.
`open`, `poll`, `write` and `close` give the effects `io`, `env` and `clock`.
*(ADR 1012)*

```ail rules=std.term.5,std.term.6,std.term.7,std.term.8 norun
use std.term

fn main() !
    mut t := term.open(true) else err
        print("not a terminal: {err}")
        return
    defer t.close()
    t.write("{term.clear()}{term.goto(2, 1)}{term.bold()}Press q\r\n")
    while true
        ev := t.poll(1000) else continue
        match ev
            Key(k, _, _, _) =>
                if k == Text("q")
                    break
            Resize(c, r) => t.write("{term.goto(0, 0)}{c}x{r}")
            _ => continue
```

Rejected: `ioctl(TIOCGWINSZ)` for the size (variadic, which the FFI leaves out,
`ffi.decl.3`, and its request number differs by OS); a runtime piece in the
native backend (the C library already has every call); `terminfo` lookup
(every terminal the targets run on speaks the sequences above).
