# 0419. The HTTP server is `net.serve(addr)` with `accept` and `respond`; the handler form waits on function values

- **Status:** Accepted
- **Date:** 2026-10-20
- **Decided by:** AI (delegated, ADR 0076; the designer delegated this batch's decisions on 2026-10-20)
- **Topic:** standard library
- **Spec rules:** [std.net.9–11](../spec/17-std.md#network-stdnet)
- **Criterion that decided:** AI understanding (no ambiguity, good diagnostics), then tokens; speed is a tie

## Context

The brief asks for `net.serve(addr, handler)` with a `fn(Request) Response`
handler, concurrent connections on the task scheduler, and a graceful stop.
AIL has no function values yet (lane 130, not merged), and every task runs
the interpreter on its own thread, taking turns (ADR 0186).

## Options considered

### Option A — `serve(addr)` returns a `Server`; `accept()` returns an `Exchange`; `respond` answers (chosen now)
- **Pros:** needs no function values; the program chooses its concurrency
  with `spawn` (one task per request, or a pool of workers on one `Server`);
  the accept loop and cancellation are ordinary AIL; a slow client doesn't
  hold others up because `accept` returns whichever request completes
  first; it is the primitive the handler form is a ten-line wrapper over.
- **Cons:** a loop and a `spawn` where the handler form has one call.

### Option B — `serve(addr, handler)` only
- **Pros:** the fewest tokens for the simple server.
- **Cons:** impossible without function values; hides the loop, so a
  program can't stop, log or limit.

### Option C — `serve` takes the name of a function as text
- **Pros:** works today.
- **Cons:** untyped, not checked, and replaced by Option B's form soon;
  an API that would have to be removed.

### Option D — one OS thread per connection that runs the handler
- **Pros:** real parallelism.
- **Cons:** handlers share nothing with the scheduler; values can't cross;
  `Rc` values in the interpreter are sound only because one task runs at
  a time.

## Decision

Option A now; the handler form `net.serve(addr, handler)` is added when
function values land (lane 130), as an AIL wrapper over Option A in
`std.net`, recorded as a follow-up. `Server.accept` is safe to call from
many tasks. Strictness (std.net.10) is the server's job: it answers
malformed requests itself with `400`, `413`, `431`, `408`, `501` or `505`
and never hands them to the program, rejecting the framing shapes behind
request smuggling (both `Content-Length` and `Transfer-Encoding`, repeated
or non-numeric lengths, whitespace before a colon, folded lines) and
bounding head, body and time. Responses always carry `Connection: close`:
keep-alive adds a per-connection state machine and a second timeout, and
the measured use (an agent's local tool server, tests) doesn't need it;
HTTP/2 and server-side TLS are out of scope (ADR 0360 is a client).

## Consequences

- A server is `srv := net.serve(addr)?`, workers that loop on
  `srv.accept()`, and `srv.close()`; tests run a client in the same program.
- `Server` and `Exchange` are handles in a process-wide table; closed
  handles are never reused.
- Follow-up: the handler wrapper, keep-alive, and request streaming.

## Addendum (2026-10-20): the handler form

With function values (lane 130, ADRs 0455-0456), the handler form is an AIL
module, `std.net.http`: `http.serve(addr, handler)` and `http.run(srv,
handler)` over `Server.accept` with four worker tasks (`std.net.18`). It is
a module of its own because `net.serve(addr)` already exists with one
argument and AIL has no overloading by arity; a name such as `serve_with`
would not say what it adds. A pool, not a task per request, because each
task is an OS thread with a large stack. Closures as handlers wait on
lane 130's closures.

## Rejected alternatives

- `serve(addr, handler)` only (option B): rejected because it is impossible without function values, and it hides the loop, so a program cannot stop, log or limit.
- `serve` taking the name of a function as text (option C): rejected because it is untyped and unchecked, and would have to be removed once the handler form landed.
- One OS thread per connection that runs the handler (option D): rejected because handlers would share nothing with the scheduler and values could not cross, and `Rc` values are sound only because one task runs at a time.
