# compiler: watch

## watch
```
watch.SERVE := "ail_serve"
pub fn watch.shim(text str) str  ## `text` with `app.run(` calls made `std.attach.serve.run(` calls and an
fn watch.retarget(line str) str  ## A line with its `app.run(` calls retargeted; comments stay as they are.
fn watch.has_clash(text str) bool  ## Whether the program already uses the name `attach` or `ail_serve` for
```
