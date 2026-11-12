# compiler: watch

## watch
```
watch.SERVE := "ail_serve"
watch.GFX := "ail_gfx"
pub fn watch.shim(text str) str  ## A program that calls `host.run(` after `use std.gfx.host` gets `std.gfx.watch.run(`
fn watch.retarget(line str, apps bool, games bool) str  ## A line with its `app.run(` and `app.run_with(` calls retargeted; comments stay as they are.
fn watch.has_clash(text str) bool  ## Whether the program already uses the name `attach` or `ail_serve` for
```
