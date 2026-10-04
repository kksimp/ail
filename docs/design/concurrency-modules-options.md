# Concurrency and modules: options explained

> **Non-normative design aid,** 2026-10-04. These are serious decisions under
> [ADR 0076](../decisions/0076-delegated-decisions.md), because they're hard
> to reverse once Stage 2 starts. Examples are in `tools/tokbench/corpus/r5/`.
> Token figures are the mean of five tokenizers.

## Part 1: Concurrency

What's already decided: safe code can't have data races (ADR 0025), values
are independent (ADR 0060), `ref` cells stay inside one task (ADR 0063), each
task can have its own heap (ADR 0012), and a trap ends only its task (ADR 0047).

### 1a. How do functions wait for I/O and other tasks?

**Option A: colorless tasks (Go, Java virtual threads).** Any function can
block. The runtime parks the task and runs others. No `async` keyword.

```
fn load_profile(id int) Profile ! NetError
    user := fetch_user(id)?          # may block; other tasks keep running
    posts := fetch_posts(id)?
    Profile(user: user, posts: posts)

fn main() !
    a := spawn load_profile(1)
    b := spawn load_profile(2)
    print(a.wait()?)
    print(b.wait()?)
```

**Option B: `async`/`await` (Rust, JavaScript, C#).** Functions that can
suspend are marked `async`, and every call to one needs `await`.

```
async fn load_profile(id int) Profile ! NetError
    user := await fetch_user(id)?
    ...
```

| | Colorless | async/await |
|---|---|---|
| Tokens (example) | **76.8** | 78.0 (+1.5%, a tie) |
| Function "coloring" | None: any function calls any function | Two worlds. Async spreads up the call graph, and sync code can't easily call async code. |
| Where waiting can happen | Not marked (any call might block) | Marked at every `await` |
| Common AI bug | Rare | Forgetting `await`, or calling async from sync code |
| Per-task memory | Needs a stack per task. AIL's whole-program analysis can often compute the exact stack size, avoiding Go-style growable stacks. | Smallest: a state machine sized at compile time |
| Runtime complexity | M:N scheduler plus stack management | Executor plus state-machine transform |

### 1b. Can tasks outlive the code that started them?

- **Scope-bound (structured concurrency):** every task spawned in a block is
  waited for before that block exits. Tasks can't leak, traps and errors
  always reach someone, and cancellation flows naturally. Fire-and-forget
  needs an explicit long-lived task group.
- **Free-running (Go):** a spawned task runs until it finishes, regardless of
  who started it. Simple, but goroutine leaks and lost errors are a classic
  bug class.

## Part 2: Modules

### 2a. What is a module?

- **A directory (Go-style):** all `.ail` files in a directory form one module
  and share names, with no imports between files of the same module. The
  module's name is the directory name. Files can be split or merged freely
  without changing any code.
- **A file (Rust/Python-style):** each file is its own module. Moving a
  function between files changes how other code imports it.

### 2b. How is something made visible outside its module?

| Form | Example | Declaration-heavy snippet | Representative code |
|---|---|---|---|
| **`pub` keyword** | `pub fn Config.load(...)` | 44.6 | **195.8** |
| Capitalisation (Go) | `fn Config.Load(...)`; exported names start uppercase | 40.6 (−9%) | 194.4 (−0.7%) |

On representative code the difference is under 1%, a tie under ADR 0076.
Capitalisation then costs AI understanding and readability: changing
visibility means renaming every use (larger diffs and structural edits),
private types must be lowercase (so casing no longer tells types from
values), and exported fields look different from locals.

### 2c. How are other modules used?

| Form | Example | Tokens |
|---|---|---|
| **Import the module, use qualified names** | `use net.http` then `http.get(url)` | **35.6** |
| Import individual names | `use net.http.{get}` then `get(url)` | 38.0 |

Qualified names are both cheaper and self-describing at the point of use:
`http.get` says where `get` comes from. That fits ADR 0084 (one name, one
meaning) and is a routine decision unless you prefer otherwise.
