# Standard library: `std.jobs`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1281).

An in-process background job queue: a pool of workers that run jobs by
priority, delayed jobs, retries with exponential backoff, recurring jobs and a
snapshot of the waiting jobs in a `std.kv` file. The module is written in AIL
over `Mutex`, `Heap` and tasks and is in scope without `use` (`std.use.4`);
`jobs.pool(...)` names it. A job is data, a name and a text payload; the pool's
one handler runs it. Cron expressions are not part of this module: recurring
jobs take an interval (`std.jobs.7`).

## The pool

**std.jobs.1** — `pool(handler, workers, attempts = 3, retry_base_ms = 100,
retry_cap_ms = 30000) Pool` makes a pool of `workers` workers (at least 1)
that call `handler`, a `fn(Job) Outcome`. Details: [below](#more-on-stdjobs1). *(ADR 1281)*

**std.jobs.2** — `p.start()` starts the workers; jobs submitted earlier wait for
it, and starting a started pool does nothing. `p.stop()` makes the workers
finish the jobs they hold, then waits for them; it fails with `Trapped` only if a
worker itself trapped. Details: [below](#more-on-stdjobs2). *(ADR 1281)*

```ail tool=run,FILE expect=true rules=std.jobs.1,std.jobs.2
fn handle(j jobs.Job) jobs.Outcome
    print("{j.name} {j.payload}")
    jobs.done()

fn main() !
    mut p := jobs.pool(handle, 1)
    p.submit("a", "x")
    p.submit("b", "x", priority: 1)
    p.start()
    print(p.wait_idle(5000))
    p.stop()?
```

**std.jobs.3** — `p.submit(name, payload, priority = 0, delay_ms = 0,
attempts = 0) int` queues a job and returns its id, from 1 in order. Details: [below](#more-on-stdjobs3). *(ADR 1281)*

```ail tool=run,FILE expect=true rules=std.jobs.3
fn handle(j jobs.Job) jobs.Outcome
    jobs.done()

fn main() !
    mut p := jobs.pool(handle, 1)
    a := p.submit("a", "")
    b := p.submit("b", "", delay_ms: 100000)
    c := p.submit("c", "", priority: 9)
    print("{a} {b} {c}")
    print(p.cancel(b))
    print(p.cancel(b))
    p.stop()?
```

## Outcomes and retries

**std.jobs.4** — After a run that answers `retry`, the job is queued again once
the backoff has passed: `retry_base_ms` for the first retry, doubled for each
later one, never more than `retry_cap_ms`. Details: [below](#more-on-stdjobs4). *(ADR 1281)*

**std.jobs.5** — A handler answers `jobs.done()` (finished), `jobs.retry(why)`
(try again after the backoff, rule 4) or `jobs.abandon(why)` (fail for good
now). Details: [below](#more-on-stdjobs5). *(ADR 1281)*

**std.jobs.6** — A handler that traps fails the job as `abandon` does, with a
reason that starts `the handler trapped`; the worker and the pool go on. Details: [below](#more-on-stdjobs6). *(ADR 1281)*

```ail tool=run,FILE expect=true rules=std.jobs.4,std.jobs.5,std.jobs.6
fn handle(j jobs.Job) jobs.Outcome
    if j.name == "flaky" and j.attempt < 3
        return jobs.retry("not yet")
    print("{j.name} {j.attempt}")
    jobs.done()

fn main() !
    mut p := jobs.pool(handle, 1, retry_base_ms: 1)
    p.start()
    p.submit("flaky", "")
    print(p.wait_idle(5000))
    st := p.stats()
    print("{st.done} {st.retried} {st.dead}")
    p.stop()?
```

## Scheduling

**std.jobs.7** — `p.every(name, payload, interval_ms, priority = 0, delay_ms = -1)`
queues a job with that name every `interval_ms` (at least 1), the first after
`delay_ms` (the interval when below 0). Details: [below](#more-on-stdjobs7). *(ADR 1281)*

```ail tool=run,FILE expect=true rules=std.jobs.7
fn handle(j jobs.Job) jobs.Outcome
    jobs.done()

fn main() !
    mut p := jobs.pool(handle, 1)
    p.start()
    p.every("tick", "", 5)
    time.sleep_ms(60)
    n := p.unschedule("tick")
    print(p.wait_idle(5000))
    print(n)
    p.stop()?
```

## Inspection

**std.jobs.8** — `p.stats()` gives a `Stats` with `pending` (waiting or delayed),
`running`, `done`, `retried` (retries made) and `dead` (failures kept).
`p.dead()` lists the `Dead` jobs, oldest first, each with its `job` and `why`.
`p.wait_idle(timeout_ms)` waits until nothing is waiting or running and is
true when that happened; recurring entries do not count. *(ADR 1281)*

```ail tool=run,FILE expect=true rules=std.jobs.8,std.jobs.11,std.jobs.12
fn handle(j jobs.Job) jobs.Outcome
    if j.name == "bad"
        return jobs.abandon("no good")
    jobs.done()

fn main() !
    mut p := jobs.pool(handle, 1)
    p.start()
    p.submit("ok", "")
    p.submit("bad", "")
    print(p.wait_idle(5000))
    st := p.stats()
    print("{st.done} {st.dead} {p.dead()[0].why}")
    p.stop()?
```

## Persistence

**std.jobs.9** — `p.pending()` lists the waiting jobs, in the order queued,
with `delay_ms` set to the wait left. `p.resubmit(job)` queues such a job again,
keeping its name, payload, priority, attempts limit and wait, and returns the
new id. *(ADR 1281)*

**std.jobs.10** — `jobs.dump(path, js)` writes a list of jobs to the `std.kv`
file `path`, replacing what it held; `p.save(path)` saves `p.pending()`. Details: [below](#more-on-stdjobs10). *(ADR 1281)*

```ail tool=run,FILE expect=true rules=std.jobs.9,std.jobs.10
use std.file

fn handle(j jobs.Job) jobs.Outcome
    jobs.done()

fn main() !
    base := fs.make_temp_dir("ail-jobs-")?
    path := "{base}/q.kv"
    mut p := jobs.pool(handle, 1)
    p.submit("a", "x", priority: 1)
    p.submit("b", "y", delay_ms: 60000)
    p.save(path)?
    p.stop()?
    got := jobs.load(path)?
    print(got.len())
    print("{got[0].name} {got[0].priority} {got[1].delay_ms > 0}")
    fs.remove(path)?
    fs.remove(base)?
```

## Limits

**std.jobs.11** — [definitional] The module needs the `clock` capability (`time.mono_ns`,
`time.sleep_ms`, `eff.cap.1`) and `fs` for `save` and `load`. Under `ail run
--deterministic`, which denies `clock`, a pool cannot run. *(ADR 1281)*

**std.jobs.12** — [definitional] `jobs` is an in-process queue: jobs do not outlive the
program except through `save`, delivery is at least once across a restart
(a job that was running when the program stopped is in no snapshot), and the
queue is held in memory. Details: [below](#more-on-stdjobs12). *(ADR 1281)*

### `std.jobs` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.jobs.1

A `Job` has the fields `id`, `name`, `payload`, `priority`, `attempt`, `max_attempts` and `delay_ms`. `Pool` is a `unique` struct: it owns its worker tasks (`conc.task.10`).

#### More on std.jobs.10

`jobs.load(path)` reads them back in order: a missing file gives `[]` and an entry that is not a job is skipped. Failures are `fs.IoError` or `kv.KvError`. Running jobs are not in a snapshot: stop the pool first. Recurring entries are not saved: call `every` again after a restart.

#### More on std.jobs.12

Priorities and delays are ordered with a heap, so a queue of n jobs costs O(log n) for each submit and each run; `cancel` and `pending` look at every waiting job.

#### More on std.jobs.2

Waiting jobs stay in the pool (`p.pending()`). A pool is started once and stopped once, and a started pool that is dropped without `stop` waits for its workers for ever. An idle worker looks at the queue again after at most 20 ms or at the next due time.

#### More on std.jobs.3

The highest `priority` runs first and equal priorities run in the order they were queued (a retried job counts from when it was queued again). A job with `delay_ms` waits at least that long. `attempts` over 0 replaces the pool's limit for this job. `p.cancel(id)` removes a job that is waiting, delayed or waiting for a retry, and is true when it did; a running job is not stopped.

#### More on std.jobs.4

When `attempt` has reached `max_attempts` the job is not queued again: it fails for good with the reason `gave up after N attempts: WHY`. The first run has `attempt` 1.

#### More on std.jobs.5

The `Outcome` has the fields `kind` (`Ok`, `Retry`, `Abandon`) and `why`.

#### More on std.jobs.6

Each run is a task of its own, so a trap leaves no state of the pool half changed.

#### More on std.jobs.7

Each tick queues a new job with its own id, whether or not the last one has finished. Ticks missed while the pool had no free worker are skipped, not caught up. `p.unschedule(name)` stops the recurring entries with that name and gives how many there were. Cron expressions are future work: they would give the next due time of an entry.

**std.jobs.13** — A `priority` is clamped to -1,000,000 to 1,000,000 when the job is
queued: priorities beyond that range rank as the limit, so two jobs above 1,000,000
run in the order queued. `Job.priority` keeps the value given. *(ADR 1281)*

```ail tool=run,FILE expect=true rules=std.jobs.13
fn handle(j jobs.Job) jobs.Outcome
    print("{j.name} {j.priority}")
    jobs.done()

fn main() !
    mut p := jobs.pool(handle, 1)
    p.submit("low1", "", priority: -5000000)
    p.submit("low2", "", priority: -2000000)
    p.submit("high1", "", priority: 5000000)
    p.submit("high2", "", priority: 2000000)
    p.start()
    print(p.wait_idle(5000))
    p.stop()?
```
