# Standard library: `std.throttle`

Chapter key: `std`

> **Status:** Drafted (ADR 1235). Written in AIL (`std/throttle/throttle.ail`).

Rate limiting on the monotonic clock `time.mono_ns` (std.time.3): a token
bucket, a sliding-window limiter and a channel that forwards its values at a
bounded rate. Rates and sizes are whole numbers. A bad one gives `none` or
`false`, never a trap.

## The module

**std.throttle.1** — `throttle` is in scope without `use` (`std.use.4`);
`use std.throttle` stays valid. A `Bucket` is a token bucket: it holds up to
`burst` tokens and gains `rate` tokens a second. Its fields `rate` and `burst`
are public. The balance is kept in billionths of a token, so a refill is
exact and a rate that does not divide a second loses nothing. *(ADR 1235)*

```ail rules=std.throttle.1
fn main()
    b := throttle.bucket_at(5, 10, 0) else
        return
    print("{b.rate} {b.burst}")   # 5 10
```

**std.throttle.2** — `bucket(per_sec, burst) Bucket?` is a full bucket whose
clock starts now; `bucket_at(per_sec, burst, now_ns)` starts it at the reading
`now_ns`. `none` unless `per_sec` and `burst` are each from 1 to
1_000_000_000. *(ADR 1235)*

```ail rules=std.throttle.2
fn main()
    zero := throttle.bucket(0, 5) else
        print("none")                          # none
        return
    print(zero.rate)
```

**std.throttle.3** — `take(n) bool` and `take_at(n, now_ns) bool` add the tokens
earned since the last reading (never above `burst`), then take `n` and return
true when the bucket holds them; otherwise nothing is taken. They return false
for an `n` below 1 or above `burst`. A reading older than the last one earns
nothing and leaves the clock where it was. *(ADR 1235)*

```ail rules=std.throttle.3
fn main()
    mut b := throttle.bucket_at(2, 1, 0) else
        return
    print(b.take_at(1, 0))               # true
    print(b.take_at(1, 100_000_000))     # false: 0.2 of a token
    print(b.take_at(1, 500_000_000))     # true
```

**std.throttle.4** — `available()` and `available_at(now_ns)` give the whole
tokens in the bucket, after the refill. *(ADR 1235)*

```ail rules=std.throttle.4
fn main()
    mut b := throttle.bucket_at(10, 4, 0) else
        return
    print(b.take_at(3, 0))                    # true
    print(b.available_at(250_000_000))        # 3
```

**std.throttle.5** — `wait_ms_at(n, now_ns) int?` is the milliseconds, rounded
up, until `n` tokens are in the bucket (0 if they are); `none` for an `n` below
1 or above `burst`, which no wait can satisfy. `wait(n) bool` takes `n` tokens,
sleeping (`time.sleep_ms`) until they are there, and returns true; it returns
false at once, with nothing taken, for the same bad `n`. *(ADR 1235)*

```ail rules=std.throttle.5
fn main()
    mut b := throttle.bucket_at(2, 3, 0) else
        return
    print(b.take_at(3, 0))                    # true
    w := b.wait_ms_at(2, 0) else -1
    print(w)                                  # 1000
    print(b.wait(4))                          # false
```

**std.throttle.6** — A `Limiter` allows at most `limit` events in any window of
`window_ms` milliseconds. Its fields `limit` and `window_ms` are public. It
keeps the times of the last `limit` allowed events and nothing else, so its
size is fixed. *(ADR 1235)*

```ail rules=std.throttle.6
fn main()
    l := throttle.limiter(3, 1000) else
        return
    print("{l.limit} {l.window_ms}")   # 3 1000
```

**std.throttle.7** — `limiter(limit, window_ms) Limiter?`: `none` unless `limit`
is from 1 to 1_000_000 and `window_ms` from 1 to 1_000_000_000. *(ADR 1235)*

```ail rules=std.throttle.7
fn main()
    l := throttle.limiter(0, 10) else
        print("none")                    # none
        return
    print(l.limit)
```

**std.throttle.8** — `allow()` and `allow_at(now_ns)` return true and record the
event when fewer than `limit` allowed events happened in the `window_ms` up to
now; otherwise they return false and record nothing. An event exactly
`window_ms` after an earlier one no longer counts it. *(ADR 1235)*

```ail rules=std.throttle.8
fn main()
    mut l := throttle.limiter(1, 1000) else
        return
    print(l.allow_at(0))                 # true
    print(l.allow_at(999_000_000))       # false
    print(l.allow_at(1_000_000_000))     # true
```

**std.throttle.9** — `pace[T](input Channel[T], output Channel[T], per_sec, burst)
bool` receives each value of `input` until it is closed and empty, waits for a
token (std.throttle.5), sends the value to `output`, and closes `output` at the
end. It is meant for a task of its own (`spawn` takes a call of a declared
function, so wrap it). It returns true when it ran to the end, and false, having
moved nothing and left `output` open, for a bad `per_sec` or `burst`. *(ADR 1235)*

```ail rules=std.throttle.9
fn run(input Channel[int], output Channel[int]) bool
    throttle.pace(input, output, 1000, 4)

fn main()
    input := Channel[int](4)
    output := Channel[int](4)
    t := spawn run(input, output)
    input.send(1)
    input.send(2)
    input.close()
    for v in output
        print(v)                         # 1 then 2
    done := t.wait() else false
    print(done)                          # true
```

**std.throttle.10** — The functions that read the clock (`bucket`, `take`,
`available`, `wait`, `allow`, `pace`) need the `clock` capability (std.time.3);
the `_at` forms and the fields are pure. After `test.clock(ms)` the clock is
`ms` (test.stub.4), so a test can use either. *(ADR 1235)*

```ail rules=std.throttle.10
fn main()
    mut b := throttle.bucket_at(1, 1, 0) else
        return
    print(b.take_at(1, 0))               # true
```
