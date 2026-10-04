# 12. Modules and programs

Chapter key: `mod`

> **Status:** Drafted (partial). Only the program entry point is specified.
> Modules, visibility and packages are still being designed.

## Program entry point

**mod.main.1** — A program begins executing at a function named `main` that
takes no parameters. Its return type is either absent, or an error type in the
form `!` (inferred) or `! E`. *(ADR 0088)*

**mod.main.2** — If `main` returns an error, the program writes the error
report (`err.ctx.3`) to standard error and exits with status 1. If `main`
completes normally, the exit status is 0. Traps follow `err.trap.3`.
*(ADR 0088)*

**mod.main.3** — `print(value)` is available without import. It writes the
value's text form followed by a newline to standard output. *(ADR 0088)*

```ail
fn main()
    print("Hello, world!")
```
