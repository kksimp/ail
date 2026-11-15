# Task: split the program into modules

`stock` (spec in `DOCS/spec.md`) is one long source file, `src/main`. Split it into modules without changing its behaviour:

- one module that parses a line into an event,
- one that holds the stock state and applies the spec's rules,
- one that produces the three outputs.

`src/main` keeps only argument handling, reading the input and the wiring between modules: at most 90 non-blank lines. There must be at least three other source files of the language under `src/` (in C, `.c` files; headers do not count). The build command in `./build` stays as it is.

Hidden cases check that every behaviour is unchanged, stdout, stderr and exit code alike. Keep `./test` passing.
