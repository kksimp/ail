# Standard library: `std.resilience`

Chapter key: `std`

> **Status:** Drafted (ADR 1270). Written in AIL (`std/resilience/resilience.ail`).

Retry with exponential backoff and jitter, a circuit breaker and deadlines.
Rate limiting is `std.throttle`. A breaker keeps its state behind a `Mutex`, so
the same value handed to several tasks is one shared breaker (`conc.share.5`). Time is the
monotonic clock `time.mono_ns()` (`std.time.3`) in whole milliseconds, so
setting the wall clock moves nothing; after `test.clock` it follows the test
clock (`test.stub.4`). The module needs the `clock` capability, and a jittered
delay needs `random`. It is in scope without `use` (`std.use.4`).

## Retry with backoff

**std.resilience.1** — `backoff(base_ms, max_ms, max_tries) Backoff` is a retry
plan: up to `max_tries` attempts, and after failed attempt `n` a wait of
`base_ms * 2^(n-1)` capped at `max_ms`, then jittered by its `jitter`
(`Jitter`): `Full` (the default) picks anywhere from 0 to the wait, `Equal` from
half the wait to the wait, `NoJitter` keeps it. Details: [below](#more-on-stdresilience1). *(ADR 1270)*

```ail tool=run,FILE expect=Full,NoJitter rules=std.resilience.1
fn main()
    b := resilience.backoff(100, 1000, 5)
    p := b.with_jitter(resilience.Jitter.NoJitter)
    print("{b.jitter.name()},{p.jitter.name()}")
```

**std.resilience.2** — `b.ceiling_ms(attempt) int` is the wait after failed
attempt `attempt` (1 is the first) before jitter; it never exceeds `max_ms` and
cannot overflow. Details: [below](#more-on-stdresilience2). *(ADR 1270)*

```ail tool=run,FILE expect=100;200;400;1000,true rules=std.resilience.2
fn main()
    b := resilience.backoff(100, 1000, 9)
    cs := [b.ceiling_ms(n).to_str() for n in [1, 2, 3, 50]]
    d := b.delay_ms(3)
    print("{cs.join(";")},{d >= 0 and d <= 400}")
```

**std.resilience.3** — `retry(plan, f) Retried` calls `f(attempt)` (attempts
count from 1) until it returns true or `max_tries` attempts were made, sleeping
`delay_ms(n)` after failed attempt `n` but not after the last. Details: [below](#more-on-stdresilience3). *(ADR 1270)*

```ail tool=run,FILE expect=true,3,3 rules=std.resilience.3
use std.time

error struct Down "down {self.n}"
    n int

fn call(n int) int ! Down
    if n < 3
        return Down(n: n)
    n

fn main()
    r := resilience.retry(resilience.backoff(1, 2, 5), |n| n == 3)
    plan := resilience.backoff(1, 2, 5)
    mut n := 1
    mut got := 0
    while n <= plan.max_tries
        v := call(n) else err
            time.sleep_ms(plan.delay_ms(err.n))
            n += 1
            continue
        got = v
        break
    print("{r.ok},{r.attempts},{got}")
```

## Circuit breaker

**std.resilience.4** — `breaker(threshold, cooldown_ms) Breaker` is a circuit
breaker, closed at first. Details: [below](#more-on-stdresilience4). *(ADR 1270)*

```ail tool=run,FILE expect=Closed,Open rules=std.resilience.4
fn main()
    b := resilience.breaker(2, 60000)
    s0 := b.state().name()
    b.failure()
    b.failure()
    print("{s0},{b.state().name()}")
```

**std.resilience.5** — `b.allow() bool` is true when a call may go ahead:
always when closed, never when open, and when half open for the first caller
only, until it reports. Details: [below](#more-on-stdresilience5). *(ADR 1270)*

```ail tool=run,FILE expect=Failed,Rejected,false rules=std.resilience.5
fn bad() bool
    false

fn main()
    b := resilience.breaker(1, 60000)
    print("{b.call(bad).name()},{b.call(bad).name()},{b.allow()}")
```

## Deadlines

**std.resilience.6** — `deadline(ms) Deadline` is the moment `ms` from now. Details: [below](#more-on-stdresilience6). *(ADR 1270)*

```ail tool=run,FILE expect=false,true,false,true rules=std.resilience.6
fn main()
    d := resilience.deadline(30)
    print("{d.expired()},{d.sleep(1)},{d.sleep(1000)},{d.expired()}")
```

**std.resilience.7** — `within(ms, f) CallOutcome` calls `f(d)` with a
`Deadline` `ms` from now. Details: [below](#more-on-stdresilience7). *(ADR 1270)*

```ail tool=run,FILE expect=Done,TimedOut rules=std.resilience.7
fn quick(d resilience.Deadline) bool
    true

fn lazy(d resilience.Deadline) bool
    d.sleep(1000)

fn main()
    print("{resilience.within(1000, quick).name()},{resilience.within(10, lazy).name()}")
```

### `std.resilience` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.resilience.1

`b.with_jitter(j)` returns the plan with another jitter. `base_ms` and `max_tries` must be at least 1 and `max_ms` at least `base_ms`, else it traps.

#### More on std.resilience.2

`b.delay_ms(attempt) int` applies the jitter: 0 to the ceiling for `Full`, half the ceiling to the ceiling for `Equal`.

#### More on std.resilience.3

`Retried` has `ok` (an attempt returned true) and `attempts`. A trap in `f` is not caught. For failures with a type of their own, write the loop with `delay_ms`.

#### More on std.resilience.4

After `threshold` failures in a row it opens and refuses calls; `cooldown_ms` later it is half open, and lets one probe through: the probe's success closes it, its failure opens it again for another cooldown. `b.state()` is a `BreakerState` (`Closed`, `Open`, `HalfOpen`). `threshold` and `cooldown_ms` must be at least 1, else it traps.

#### More on std.resilience.5

`b.call(f) CallOutcome` does all three for `f`, a `fn() bool`: `Rejected` without calling `f`, else `Done` when `f` returned true and `Failed` when false. A trap in `f` reports nothing. `b.success()` closes the breaker and restarts the count; `b.failure()` counts one (opening at the threshold, or at once for a failed probe).

#### More on std.resilience.6

`d.at_ms` is it on the monotonic clock; `d.remaining_ms()` is the time left, never below 0; `d.expired()` is true once it has passed. `d.sleep(ms) bool` sleeps `ms`, or only until the deadline if that is sooner, and says whether the whole `ms` was slept.

#### More on std.resilience.7

It gives `TimedOut` when the deadline has passed by the time `f` returns, whatever `f` returned; otherwise `Done` or `Failed` as `f` said. It does not interrupt `f`: a long step checks `d.expired()` or sleeps with `d.sleep`. (Cancelling a running task is `g.cancel()`, `conc.cancel.*`.)
