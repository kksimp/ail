# compiler: (root)
entry: main in main.ail

## (root)
```
fn main() ! [env fail fs io]
    fails with JsonError | fs.IoError
fn check_dump(path str, many bool) ! [env fail fs io]  ## The checker's diagnostics for one dump, sorted by position as `ail check`
    fails with JsonError | fs.IoError
fn runtime(dir str) !ast.Program [fail fs io]  ## The native runtime in directory `dir`, parsed as one program.
    fails with fs.IoError
fn runtime_with(dir str, ms str, mb str) !ast.Program [fail fs io]  ## The runtime with `ail build`'s default time limit in milliseconds and
    fails with fs.IoError
fn lower_file(path str, rt ast.Program?) !mir.Module [fail fs io]  ## Parse a program and the modules it imports, and lower them to MIR
    fails with fs.IoError
fn parse_source(module str, path str) !mir.Source [fail fs io]  ## One file, parsed; syntax errors are printed and the run fails.
    fails with fs.IoError
```
