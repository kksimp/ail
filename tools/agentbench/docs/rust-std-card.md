# Rust reference card

> Rust 2021 edition, standard library only (no crates, no Cargo, no network). Non-normative:
> the Rust Reference and the `std` documentation are authoritative. Size-matched to the AIL
> language card for the agent benchmark ([ADR 0916](../../../docs/decisions/0916-agent-end-to-end-benchmark.md)):
> signatures and pitfalls, no solutions.

## Building and running programs

```sh
rustc --edition 2021 -O -C debug-assertions=on -C overflow-checks=on -o prog src/main.rs   # the benchmark's build line
./prog ARGS < input.txt        # arguments from std::env::args(), input on stdin
echo $?                        # exit status of the last command
RUST_BACKTRACE=1 ./prog        # a stack trace when the program panics
```

One crate is one file tree rooted at `src/main.rs`. `mod parse;` loads `src/parse.rs`
(or `src/parse/mod.rs`); items in it are private unless marked `pub`, and are reached
as `parse::name` or brought in with `use crate::parse::name;`. Warnings (unused
variables, unused `Result`, dead code) are printed but do not fail the build; read them.

A program needs `fn main()`. Its exit status is 0 when `main` returns, 101 when the
program panics, and `n` after `std::process::exit(n)`. `fn main() -> Result<(), E>`
also works: an `Err` prints its `Debug` form and exits 1.

## Layout

- Blocks use `{ }`; statements end in `;`. The last expression of a block without a
  `;` is the block's value. Four-space indentation is the convention (`rustfmt`).
- `//` line comments, `/* */` block comments, `///` doc comments.
- Items (functions, structs, `use`) may appear in any order in a module.
- Names: `snake_case` for functions and variables, `CamelCase` for types and traits,
  `SCREAMING_CASE` for constants. A leading `_` silences the unused warning.

## Functions

```rust
fn add(a: i64, b: i64) -> i64 {     // parameter types are required; return type after ->
    a + b                            // no semicolon: this is the return value
}

fn greet(name: &str) {               // no return type: returns ()
    println!("hello, {name}");
}

fn split_pair(s: &str) -> Option<(&str, &str)> {   // several results: a tuple
    s.split_once('=')
}
```

`return x;` leaves early. Arguments are moved or copied in: a `String` passed by value
is moved and the caller can no longer use it; pass `&s` (a borrow) or `&mut s` (a
mutable borrow) instead. Closures: `|x| x + 1`, `|a: &i64, b: &i64| a.cmp(b)`; they
capture variables from the enclosing scope by reference unless written `move |x| ...`.

## Variables and types

```rust
let x = 1;               // immutable; type inferred (i32 for a bare integer literal)
let mut n: i64 = 0;      // `mut` to reassign
let (a, b) = (1, 2);     // destructuring
const LIMIT: usize = 500;
let x = x * 2;           // shadowing: a new binding with the same name
```

- Integers: `i8 i16 i32 i64 i128 isize`, `u8 u16 u32 u64 u128 usize`. `usize` is for
  lengths and indexes. Literals take a suffix or `_` separators: `10u64`, `1_000_000`.
- `f32 f64`, `bool`, `char` (a Unicode scalar, 4 bytes), `()` (unit).
- No implicit conversions between numeric types: `a as i64`, `n as usize`. `as`
  truncates or wraps silently (`300i32 as u8 == 44`, `-1i64 as usize` is huge); use
  `i64::try_from(x)` or `u32::try_from(n).ok()` when the value may not fit.
- **Overflow:** with `-C overflow-checks=on` (the benchmark's build) `i64::MAX + 1`
  panics with "attempt to add with overflow". Explicit forms: `a.checked_add(b)`
  returns `Option`, `a.wrapping_add(b)`, `a.saturating_add(b)`, `a.overflowing_add(b)`.
  `i64::MIN.abs()` and `-i64::MIN` overflow; `x.unsigned_abs()` gives a `u64` that always fits.
- Integer division truncates toward zero and `%` takes the sign of the dividend:
  `-7 / 2 == -3`, `-7 % 2 == -1`; `rem_euclid` and `div_euclid` give the
  non-negative forms. Division by zero panics.
- Arrays `[i64; 4]` have a fixed length; slices `&[i64]` are views; `Vec<i64>` grows.
- Tuples `(i64, String)` are accessed as `t.0`, `t.1`.

## Operators

`+ - * / %`, `== != < <= > >=`, `&& || !` (on `bool` only; integers are not truthy),
`& | ^ ! << >>` on integers (`!` is bitwise not), `+= -= *= /= %=`. No `++`/`--`.
Comparison works on any type that implements `PartialEq`/`PartialOrd`, including
`String`, `&str`, tuples, `Vec` and `Option`, comparing element by element; strings
compare by bytes. `==` on strings compares contents. Ranges: `0..n` (exclusive),
`0..=n` (inclusive). `?` propagates an error (below). `x as T` converts numbers.

## Structs, enums and traits

```rust
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Point { x: i64, y: i64 }

impl Point {
    fn new(x: i64, y: i64) -> Point { Point { x, y } }     // field init shorthand
    fn sum(&self) -> i64 { self.x + self.y }                // method: borrows self
    fn shift(&mut self, d: i64) { self.x += d; }            // needs a `mut` receiver
}

enum Shape { Circle { r: f64 }, Rect(f64, f64), Empty }

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => 3.14159 * r * r,
        Shape::Rect(w, h) => w * h,
        Shape::Empty => 0.0,
    }
}
```

- `#[derive(...)]` generates traits: `Debug` (`{:?}` printing), `Clone` (`.clone()`),
  `Copy` (implicit copies; only for types whose fields are all `Copy`), `PartialEq`/`Eq`
  (`==`), `PartialOrd`/`Ord` (comparison and sorting, field by field in declaration
  order), `Hash` (keys of `HashMap`), `Default`.
- `match` must cover every case; `_` matches anything. Guards: `Some(n) if n > 0 => ...`.
  `if let Some(x) = opt { ... }` and `let Some(x) = opt else { return; };` match one case.
- Traits declare shared behaviour: `trait Area { fn area(&self) -> f64; }` and
  `impl Area for Point { ... }`. Generic functions: `fn largest<T: Ord + Copy>(xs: &[T]) -> T`.
  `impl std::fmt::Display for Point { fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, "{},{}", self.x, self.y) } }`
  makes `{}` and `.to_string()` work.

## Ownership and borrowing

- Every value has one owner; it is dropped (freed) when the owner goes out of scope.
  Assigning or passing a non-`Copy` value (`String`, `Vec`, most structs) moves it;
  the old name is then unusable ("borrow of moved value"). `.clone()` makes a copy.
- `&T` is a shared borrow, `&mut T` an exclusive one. At any time a value has either
  any number of `&T` or exactly one `&mut T`, and no borrow may outlive the value.
- Common fixes for borrow errors: clone the small value you need; finish using one
  borrow before taking the next (borrows end at their last use); collect indexes or
  keys first and mutate afterwards; split work into a function that takes `&mut`.
- Holding `&str` slices into an input `String` avoids copies; the `String` must stay
  alive (and unmodified) while the slices are used. A struct that holds borrows
  needs a lifetime: `struct Row<'a> { name: &'a str }`. Owning `String` fields avoid that.
- Mutating a collection while iterating it is a compile error; iterate over indexes
  (`for i in 0..v.len()`) or build a new collection.
- `Rc<T>` shares ownership in one thread, `RefCell<T>` allows checked mutation through a
  shared reference (panics on conflicting borrows), `Box<T>` puts a value on the heap
  (needed for recursive types: `enum Tree { Leaf, Node(Box<Tree>, Box<Tree>) }`).

## Strings and characters

`String` is an owned, growable UTF-8 buffer; `&str` is a borrowed slice of one.
String literals are `&'static str`.

```rust
let s = String::from("a,b");  let t = s.to_string();  let u = "x".to_owned();
s.len()                         // bytes, not characters
s.is_empty()
s.push_str("c"); s.push('d');   // needs `mut s`
let joined = format!("{a}-{b}");          // building strings; `+` needs String + &str
s.split(',')                    // iterator of &str; keeps empty fields: "a,,b" -> "a", "", "b"
s.splitn(3, ',')                // at most 3 pieces
s.split_once(" -> ")            // Option<(&str, &str)>, at the first match
s.lines()                       // splits on \n, strips a trailing \r, drops a final empty line
s.trim() / trim_start() / trim_end()
s.starts_with("#")  s.ends_with(".txt")  s.contains("ab")  s.find(',')   // Option<usize> byte index
s.strip_prefix('-')             // Option<&str>
s.replace("a", "b")  s.to_uppercase()  s.repeat(3)
&s[2..5]                        // byte range; panics unless both ends are on char boundaries
s.as_bytes()                    // &[u8]
s.chars()                       // iterator of char; .count() is the character count
s.bytes()                       // iterator of u8
parts.join(",")                 // Vec<&str> or Vec<String> to String
String::from_utf8(bytes)        // Result; String::from_utf8_lossy(&bytes) replaces bad bytes
```

- Indexing `s[i]` is not allowed; use `s.as_bytes()[i]` or `s.chars().nth(i)`.
- `char` methods: `is_ascii_digit()`, `is_ascii_alphabetic()`, `is_ascii_alphanumeric()`,
  `is_whitespace()`, `to_ascii_uppercase()`, `to_digit(10)` (`Option<u32>`). `u8` has
  the same `is_ascii_*` methods; byte literals are `b'a'`, byte strings `b"abc"`.
- `split` on an empty string yields one empty piece; `split_whitespace()` skips empty
  pieces and any run of whitespace.
- `.lines()` treats `\r\n` as a line end, so it cannot detect a stray carriage return;
  use `split('\n')` when `\r` must be seen.
- `String` ordering (`<`, `sort`, `BTreeMap` keys) is by bytes, which is code-point order.

## Input and output

```rust
use std::io::{self, BufRead, Read, Write};
println!("{} {}", n, s);                 // stdout with a newline; print! without
eprintln!("error: {msg}");               // stderr
let mut input = String::new();
io::stdin().read_to_string(&mut input)?; // all of stdin; errors on invalid UTF-8
let mut raw = Vec::new();
io::stdin().read_to_end(&mut raw)?;      // all of stdin as bytes
for line in io::stdin().lock().lines() { let line = line?; }   // line by line, '\n' removed
let out = io::stdout(); let mut out = io::BufWriter::new(out.lock());
writeln!(out, "{}\t{}", k, v)?;          // fast buffered output; flushed on drop
std::fs::read_to_string(path)?  std::fs::write(path, text)?
```

- Format: `{}` uses `Display`, `{:?}` `Debug`, `{:5}` width, `{:<8}` left, `{:>8}` right,
  `{:05}` zero-padded, `{:.2}` two decimals, `{:x}` hex, `{{` and `}}` literal braces.
  Variables can be named inline: `{name}`, `{x:>5}`.
- Stdout is line-buffered even in a pipe and `println!` takes the lock on every call;
  for many lines use a `BufWriter` on `stdout().lock()`.
- `std::process::exit` does not run destructors: flush a `BufWriter` (`out.flush()?`)
  before calling it, or its contents are lost.
- `println!` panics if stdout is closed (for example piped into `head`); `writeln!`
  returns the error instead.

## Numbers from text

```rust
let n: i64 = "42".parse().unwrap();          // parse::<T>() returns Result<T, ParseIntError>
match s.parse::<i64>() { Ok(v) => ..., Err(e) => ... }
let v: Option<u64> = s.parse().ok();
let d: f64 = "2.5".parse()?;
i64::from_str_radix("ff", 16)
```

`parse::<i64>()` accepts an optional leading `+` or `-` and digits only: no spaces,
no `_`, no empty string, and it fails on overflow (`IntErrorKind::PosOverflow`).
Check the characters yourself when the format is stricter than that (no `+`, a fixed
number of digits). `"007".parse::<i64>()` is `Ok(7)`.

## Control flow

```rust
if a < b { ... } else if a == b { ... } else { ... }   // no parentheses; braces required
let kind = if n < 0 { "neg" } else { "pos" };         // if is an expression
while n > 0 { n -= 1; }
loop { if done() { break; } }                          // `break value` returns from loop
for i in 0..n { ... }                                  // ranges
for (i, x) in xs.iter().enumerate() { ... }
for x in &xs { ... }  for x in &mut xs { *x += 1; }  for x in xs { ... }   // borrow, mutate, consume
'outer: for a in 0..3 { for b in 0..3 { if b == a { continue 'outer; } } }
match cmd {
    "add" | "sum" => ...,
    s if s.starts_with('-') => ...,
    _ => ...,
}
match n { 0 => ..., 1..=9 => ..., _ => ... }
```

`match` on a `String` needs `.as_str()`. Arms are tried in order and must be
exhaustive. `match` on a tuple of values (`match (a, b)`) decides on several values at once.

## Errors and absence

```rust
fn parse_amount(s: &str) -> Result<i64, String> {
    let v: i64 = s.parse().map_err(|_| format!("bad amount {s:?}"))?;   // ? returns the Err early
    if v == 0 { return Err("zero".to_string()); }
    Ok(v)
}
let first: Option<&str> = args.get(1).map(|s| s.as_str());
```

- `Option<T>` is `Some(v)` or `None`; `Result<T, E>` is `Ok(v)` or `Err(e)`. There is no null.
- Taking the value: `match`, `if let`, `let ... else`, `?` (in a function returning
  `Option` or `Result`), `.unwrap_or(d)`, `.unwrap_or_default()`, `.unwrap_or_else(|| ...)`,
  `.ok_or("msg")?` (Option to Result), `.ok()` (Result to Option).
- Transforming: `.map(f)`, `.and_then(f)`, `.filter(p)`, `.is_some()`, `.is_none()`,
  `.is_ok()`, `.is_err()`, `.copied()` (Option<&T> to Option<T>).
- `.unwrap()` and `.expect("why")` panic on `None`/`Err`: fine when the value is
  proven present, a crash with exit status 101 otherwise. Indexing out of bounds
  (`v[10]`, `&s[..99]`) and `map[&key]` for a missing key also panic; `v.get(i)` and
  `map.get(&key)` return `Option`.
- An unused `Result` gives a warning ("unused `Result` that must be used"); handle it.
- Report errors on stderr and set the exit code: `eprintln!("error: ..."); std::process::exit(2);`.

## Collections

```rust
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};
let mut v: Vec<i64> = Vec::new();  let v2 = vec![1, 2, 3];  let v3 = vec![0; n];
v.push(4); v.pop(); v.len(); v.is_empty(); v.insert(0, x); v.remove(i); v.truncate(n);
v.contains(&x); v.iter().position(|y| *y == x); v.first(); v.last(); v.extend(other);
v.sort(); v.sort_unstable(); v.dedup(); v.reverse(); v.retain(|x| *x > 0);
v.binary_search(&x)                     // Ok(index) or Err(insertion point); v must be sorted
let mut m: HashMap<String, i64> = HashMap::new();
*m.entry(key).or_insert(0) += amount;   // insert or update
m.get(&k)  m.contains_key(&k)  m.insert(k, v)  m.remove(&k)  m.len()
for (k, v) in &m { ... }                // HashMap order is arbitrary
let mut b: BTreeMap<&str, i64> = BTreeMap::new();   // iterates in ascending key order
let mut s: BTreeSet<String> = BTreeSet::new();  s.insert(x);  s.contains(&x);  s.iter().next()   // smallest
let mut q = VecDeque::new();  q.push_back(x);  q.pop_front();
```

- `HashMap` and `HashSet` iterate in an unspecified order that changes between runs;
  use `BTreeMap`/`BTreeSet`, or sort the keys, whenever output order matters.
- A `HashMap<String, V>` is looked up with `&str`: `m.get("key")` works.
- `entry(k)` takes the key by value: `m.entry(name.to_string())` for a `String` key.
- A graph is a map from a node to its neighbours (`BTreeMap<&str, BTreeSet<&str>>`) or,
  after numbering the nodes, `Vec<Vec<usize>>`.

## Iterators

```rust
xs.iter()                    // &T items; .into_iter() moves; .iter_mut() gives &mut T
.map(|x| x * 2)  .filter(|x| **x > 0)  .filter_map(|s| s.parse().ok())
.enumerate()  .zip(other)  .rev()  .skip(1)  .take(n)  .chain(more)  .peekable()
.sum::<i64>()  .count()  .min()  .max()  .max_by_key(|r| r.size)  .any(p)  .all(p)  .find(p)
.collect::<Vec<_>>()  .collect::<String>()  .collect::<Result<Vec<_>, _>>()   // stops at the first Err
.fold(0, |acc, x| acc + x)
```

Iterators are lazy: nothing runs until a consuming call (`collect`, `sum`, `for`).
`filter` receives `&&T` when iterating `iter()`, hence `**x` or `|&&x|`.
`.max_by_key` returns the last maximum and `.min_by_key` the first on ties.

## Sorting

```rust
v.sort();                                    // stable, ascending, needs Ord
v.sort_by(|a, b| b.cmp(a));                  // descending
v.sort_by_key(|r| r.name.clone());           // stable; the key must be owned or Copy
v.sort_by(|a, b| b.size.cmp(&a.size).then_with(|| a.name.cmp(&b.name)));   // several keys
v.sort_by_key(|r| std::cmp::Reverse(r.size));
v.sort_unstable_by(...)                      // faster, NOT stable
```

`sort`, `sort_by` and `sort_by_key` are stable: equal elements keep their input order.
`f64` is not `Ord`; sort floats with `a.partial_cmp(b).unwrap()` or `a.total_cmp(b)`.
`Ordering` is `Less`, `Equal` or `Greater`; `.reverse()` and `.then(...)` combine them.

## Command-line arguments and exit status

```rust
let args: Vec<String> = std::env::args().collect();   // args[0] is the program name
match args.get(1).map(String::as_str) {
    Some("list") if args.len() == 2 => ...,
    Some("get") if args.len() == 3 => ...,
    _ => { eprintln!("usage: prog list|get NAME"); std::process::exit(1); }
}
```

- `std::env::args()` panics if an argument is not valid UTF-8; `args_os()` does not.
- `std::process::exit(code)` exits immediately without running destructors (buffered
  writers are not flushed). Returning from `main` exits 0.
- A panic prints `thread 'main' panicked at src/main.rs:LINE:COL:` and a message to
  stderr and exits 101, so a test that checks the exit code fails even when stdout matched.
- A usage error conventionally prints to stderr and exits 2 (or as the spec says);
  keep stdout for the program's output.

## Reading compiler errors

The compiler prints an error code, the location and usually a suggestion:

```
error[E0382]: borrow of moved value: `name`
 --> src/main.rs:12:20
  |
9 |     let owner = name;
  |                 ---- value moved here
12|     println!("{}", name);
  |                    ^^^^ value borrowed here after move
help: consider cloning the value if the performance cost is acceptable
```

- `rustc --explain E0382` prints a long explanation with examples.
- Frequent codes: E0382 use after move; E0499 two mutable borrows; E0502 mutable
  borrow while a shared one is live; E0505 move while borrowed; E0597 a borrow outlives
  its value; E0308 mismatched types (often a missing `;` or an `i32`/`i64`/`usize`
  mix); E0277 a trait is not implemented (`{}` on a type without `Display`, `sort` on
  floats); E0425 unknown name; E0599 no such method (often a missing `use`, such as
  `std::io::Read` for `read_to_string` or `std::io::Write` for `writeln!`); E0004 a
  `match` is not exhaustive; E0106 a missing lifetime on a struct holding a reference.
- Fix the first error first; later ones are often consequences.
- Warnings: `unused variable`, `unused import`, `unused Result`, `variable does not
  need to be mutable`, `unreachable pattern` (an arm that can never match: often a
  misspelt enum variant written as a binding name).

## Runtime panics

With the benchmark's build line these stop the program with exit status 101 and a
message on stderr naming the file and line:

- `attempt to add with overflow` (also subtract, multiply, negate, shift).
- `index out of bounds: the len is 3 but the index is 3`.
- `byte index 5 is not a char boundary` (string slicing inside a multibyte character).
- `called Option::unwrap() on a None value`, `called Result::unwrap() on an Err value: ParseIntError { kind: InvalidDigit }`.
- `attempt to divide by zero`, `already borrowed: BorrowMutError` (RefCell).
- `RUST_BACKTRACE=1` adds the call stack. The first frame in `src/` is usually the bug.

## Math

`f64` methods: `x.sqrt()`, `x.powi(2)`, `x.powf(0.5)`, `x.abs()`, `x.floor()`,
`x.ceil()`, `x.round()`, `x.trunc()`, `x.min(y)`, `x.max(y)`, `x.is_nan()`,
`std::f64::consts::PI`. Integers: `a.min(b)`, `a.max(b)`, `a.clamp(lo, hi)`,
`a.abs()`, `a.pow(3)`, `a.signum()`, `i64::MAX`, `u64::MIN`. Floating-point division
by zero gives infinity, not a panic. Compare floats with a tolerance.

## Time, environment and processes

```rust
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
let t = Instant::now(); let ms = t.elapsed().as_millis();
let secs = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
let home = std::env::var("HOME").ok();            // Option<String>
let out = std::process::Command::new("ls").arg("-l").output()?;   // out.status, out.stdout
std::thread::sleep(Duration::from_millis(10));
```

Files and directories: `std::fs::{read_to_string, write, read_dir, create_dir_all,
remove_file, metadata}`, `std::path::{Path, PathBuf}` (`path.join("x")`,
`path.extension()`), `File::open(p)?` with `BufReader::new(f).lines()`.
The standard library has no date parsing or formatting beyond these; check
calendar fields by hand.

## Threads

`std::thread::spawn(move || work())` returns a `JoinHandle`; `.join().unwrap()`
waits and gives the closure's result. `std::thread::scope(|s| { s.spawn(|| ...); })`
lets threads borrow local data. Shared mutable state needs `Arc<Mutex<T>>`
(`let g = m.lock().unwrap(); *g += 1;`) or atomics (`AtomicU64::fetch_add`).
Channels: `let (tx, rx) = std::sync::mpsc::channel();`. Data races do not compile.

## Tests

`#[cfg(test)] mod tests { use super::*; #[test] fn adds() { assert_eq!(add(2, 2), 4); } }`
runs with `rustc --test src/main.rs -o t && ./t` (prints each test, exits non-zero on
a failure). `assert!(cond)`, `assert_eq!(a, b)`, `assert_ne!(a, b)` panic with both
values. `debug_assert!` checks only when debug assertions are on (they are, in the
benchmark's build). Black-box tests run the built program with arguments and input
and compare stdout and the exit status: `printf 'a,b\n' | ./prog cmd > out.txt; echo $?; diff out.txt want.txt`.

## Slices and arrays

```rust
let a = [1, 2, 3, 4];            // [i32; 4], length fixed at compile time
let zeros = [0u8; 256];          // 256 zero bytes
let s: &[i32] = &a[1..3];        // a view: [2, 3]; panics if the range is out of bounds
s.len()  s.is_empty()  s.iter()  s.contains(&2)  s.first()  s.last()  s.get(9)
s.split_at(1)  s.chunks(2)  s.windows(2)  s.to_vec()
s.starts_with(&[1])  s.iter().rev()  s.binary_search(&3)
fn total(xs: &[i64]) -> i64 { xs.iter().sum() }   // take &[T]: accepts &Vec<T>, arrays and slices
```

- `&Vec<T>` coerces to `&[T]` and `&String` to `&str`: write parameters as `&[T]` and
  `&str` so callers can pass either.
- A 2-D grid is `Vec<Vec<T>>` (`vec![vec![0; w]; h]`) or one `Vec<T>` indexed `y * w + x`.
- `v.swap(i, j)` swaps two elements; borrowing `&mut v[i]` and `&mut v[j]` at once does
  not compile, but `let (a, b) = v.split_at_mut(j);` does.

## Generics, traits and lifetimes

```rust
fn largest<T: PartialOrd + Copy>(xs: &[T]) -> Option<T> {
    let mut best = *xs.first()?;
    for &x in xs { if x > best { best = x; } }
    Some(best)
}
fn print_all(items: &[impl std::fmt::Display]) { for i in items { println!("{i}"); } }
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str { if a.len() >= b.len() { a } else { b } }
struct Parser<'a> { input: &'a str, pos: usize }
impl<'a> Parser<'a> { fn rest(&self) -> &'a str { &self.input[self.pos..] } }
```

- A lifetime `'a` names how long a borrow lives; a function returning a reference
  derived from a parameter usually needs one when there are two reference parameters.
- `where` clauses move long bounds out of the signature: `fn f<T>(x: T) where T: Ord + Clone`.
- Useful standard traits: `Clone`, `Copy`, `Debug`, `Display`, `PartialEq`, `Eq`,
  `PartialOrd`, `Ord`, `Hash`, `Default`, `From`/`Into` (`String::from("x")`,
  `let s: String = "x".into()`), `FromStr` (what `.parse()` calls), `Iterator`
  (`fn next(&mut self) -> Option<Self::Item>`), `AsRef<str>`.
- Trait objects: `Box<dyn Fn(i64) -> i64>`, `Vec<Box<dyn Shape>>`; dynamic dispatch.
- Implementing `Ord` by hand: implement `Ord::cmp`, then `PartialOrd` as
  `Some(self.cmp(other))`, plus `PartialEq` and `Eq` consistent with it.

## Patterns

```rust
let Point { x, y } = p;                       // destructure a struct
match (cmd, args.len()) { ("top", 3) => ..., ("list", 2) => ..., _ => ... }
match bytes { [b'#', ..] => comment(), [] => blank(), _ => other() }   // slice patterns
match n { x @ 1..=9 => digit(x), _ => ... }   // bind and test
if let [a, b] = parts.as_slice() { ... }      // exactly two elements
while let Some(top) = stack.pop() { ... }
let Ok(v) = s.parse::<i64>() else { return Err("bad") };
```

- Patterns are checked at compile time for exhaustiveness; `#[non_exhaustive]` and
  integer ranges may need a final `_`.
- A lowercase name in a pattern is a new binding that matches anything, not a
  comparison with a variable of that name; compare with a guard: `x if x == limit`.
- `ref` and `ref mut` bind by reference inside a pattern on an owned value; matching
  on `&value` usually makes them unnecessary (default binding modes).

## Smart pointers and shared state

- `Box<T>`: one owner, heap storage, fixed size for recursive types and trait objects.
- `Rc<T>`: shared ownership, single thread; `Rc::clone(&r)` bumps a count. Cycles of
  `Rc` leak; break them with `Weak`.
- `RefCell<T>`: `borrow()` and `borrow_mut()` checked at run time; two live
  `borrow_mut()`s panic. `Cell<T>` for `Copy` values: `get` and `set`.
- `Arc<T>` and `Mutex<T>`/`RwLock<T>`: the thread-safe forms. A `Mutex` guard unlocks
  when dropped; keep the guarded region short and never hold two locks in different orders.
- Arena style avoids most borrow fights in graphs and trees: store nodes in a
  `Vec<Node>` and refer to them by `usize` index instead of by reference.

## Recipes: building blocks

Reading all of stdin and splitting it into lines that keep empty ones and `\r`:

```rust
let mut input = String::new();
std::io::stdin().read_to_string(&mut input).expect("stdin");
let mut lines: Vec<&str> = input.split('\n').collect();
if lines.last() == Some(&"") { lines.pop(); }      // a final newline ends the last line
for (i, line) in lines.iter().enumerate() { let lineno = i + 1; /* ... */ }
```

Splitting a record on a one-byte separator, keeping empty fields and checking the count:

```rust
let fields: Vec<&str> = line.split(',').collect();
let [a, b, c] = fields.as_slice() else { return Err("expected 3 fields") };
```

Checking that text is only ASCII digits (stricter than `parse`, which accepts `+`):

```rust
fn all_digits(s: &str) -> bool { !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) }
```

Counting or summing by key, then printing in key order:

```rust
let mut totals: BTreeMap<String, i64> = BTreeMap::new();
for (key, n) in items { *totals.entry(key.to_string()).or_insert(0) += n; }
for (k, v) in &totals { println!("{k}\t{v}"); }
```

A worklist over a graph stored as a map from a node to its neighbours:

```rust
let mut seen: BTreeSet<&str> = BTreeSet::new();
let mut work: Vec<&str> = vec![start];
while let Some(n) = work.pop() {
    for &m in &graph[n] { if seen.insert(m) { work.push(m); } }   // insert returns false if present
}
```

Building output in a `String` and writing it once:

```rust
use std::fmt::Write as _;          // for write! into a String
let mut out = String::new();
for (k, v) in &totals { writeln!(out, "{k}\t{v}").unwrap(); }
print!("{out}");
```

## Modules and visibility

```rust
// src/main.rs
mod parse;                         // src/parse.rs
mod report { pub fn print(rows: &[super::Row]) { /* ... */ } }   // inline module
use parse::{read_rows, Row};       // bring names into scope
pub struct Row { pub date: String, amount: i64 }   // `amount` is private to this module
```

- Everything is private to its module by default; `pub` exposes it to the parent,
  `pub(crate)` to the whole crate. Struct fields are private unless each is `pub`.
- `use` paths start from the crate root (`crate::`), the parent (`super::`) or the
  current module (`self::`). `use std::collections::BTreeMap;` for library items.
- With `rustc` and no Cargo, a module file must sit where `mod` expects it, next to
  `main.rs`; there is no build script and no dependency other than `std`.

## Formatting numbers

```rust
format!("{:.2}", 3.14159)        // "3.14"
format!("{:>8.2}", x)            // right-aligned in 8 columns
format!("{:08.3}", -1.5)         // "-001.500": zero padding after the sign
format!("{:+}", 5)               // "+5"
format!("{:e}", 1234.5)          // "1.2345e3"
format!("{:#x} {:#b} {:o}", 255, 5, 8)   // "0xff 0b101 10"
```

- Integer `Display` has no thousands separator; insert one by hand if needed.
- `{:.2}` on a float rounds the binary value, so `format!("{:.2}", 2.675)` gives `2.67`.
- `i64::MIN.to_string()` and `u128` formatting work like any other integer.

## Common mistakes

- Using `.lines()` when the spec treats `\r` or a final empty line as significant.
- `HashMap` iteration order in output; use `BTreeMap` or sort.
- `unwrap()` on input that can be malformed: a panic exits 101 instead of the
  required error message and code.
- `as` conversions that silently truncate or wrap (`i64 as i32`, a negative `as usize`).
- Overflow in sums or `abs` of extreme values: use `i128`, `checked_*`, or `unsigned_abs`.
- `std::process::exit` before flushing a `BufWriter`.
- Slicing strings at byte offsets inside UTF-8 characters.
- `sort_unstable` where ties must keep input order; `max_by_key` tie direction.
- `s.split(',').count()` counts empty fields; `split_whitespace` does not.
- Comparing `String` with `&str` works (`s == "x"`), but `match` needs `s.as_str()`.
- Holding a borrow from a collection while pushing to it: copy the value out first.
