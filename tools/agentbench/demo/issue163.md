# Issue 163: `x := f() else err` panics when `f` returns `T? ! E` and gives `none`

Reported against the old bootstrap interpreter (now removed). `ail check` accepted this program and
`ail run` panicked when the call succeeded with `none`:

```ail
error struct Bad "bad"

fn f(x int) int? ! Bad
    if x > 5
        return Bad()
    if x > 2
        return none
    x

fn main()
    v := f(3) else err
        print("failed {err}")
        return
    print(v)
```

Expected: `v` is `none` (the `else` handles only the failure) or the checker rejects `else err` on a
`T? ! E` value. `f(1)` and `f(9)` behaved as expected; only success-with-`none` panicked.
