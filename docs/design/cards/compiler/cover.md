# compiler: cover

## cover
```
fn cover.is_test(f ast.FnDecl) bool
fn cover.starts_line(lines List[str], pos ast.Pos) bool  ## True when the statement at `pos` starts its line (so a call can be put on the line before it).
fn cover.points(lines List[str], b List[ast.Stmt], mut out List[int]) [mutates]
pub fn cover.command(args List[str]) int [fs io]
```
