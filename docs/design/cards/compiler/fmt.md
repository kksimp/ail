# compiler: fmt
tests: fmt.fmt.keeps_the_space_before_a_variadic_dots

## fmt
```
pub fn fmt.command(args List[str]) int [fs io]
pub fn fmt.diff(path str, old str, new str) str  ## A unified diff of `old` against `new`, labelled with `path`; empty when equal.
pub struct fmt.Formatted  ## The result of `format`: `status` 0 is done and `text` is the canonical
    pub status int
    pub text str
    pub diags List[syntax.Diag]
    pub why str
pub fn fmt.format(src str) fmt.Formatted  ## The canonical form of `src`.
pub fn fmt.format_with(original str, write_self bool) fmt.Formatted  ## `format`, with `write_self` choosing whether bare members get `self.` (fmt.self).
```

trimmed to fit 2000 tokens (~231); dropped: private items
