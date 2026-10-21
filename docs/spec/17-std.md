# 17. Standard modules

Chapter key: `std`

> **Status:** Drafted (partial). Specified: importing the standard modules,
> and `std.compress`, `std.fs`, `std.env`, `std.hash`, `std.io`, `std.mem`, `std.net`, `std.path`, `std.proc`, `std.random`, `std.regex` (in [std-regex.md](std-regex.md)), `std.tensor`, `std.time`, `std.time.date`, `std.tz` and `std.ui`. Not yet
> designed: the rest of the standard library, and capability effects
> (plan §3.2), which another lane adds.

The standard modules reach the machine through small, typed functions whose
failures are values (ADR 0157). There is no string-typed shell.

## Imports

**std.use.1** — `use std.NAME` imports the standard module `NAME`, available
under the name `NAME`, or under `d` with `use std.NAME as d` (`mod.use.1`).
The standard modules are `env`, `fs`, `io`, `net`, `path`, `proc`, `random`, `regex`, `tensor`, `time` and `ui`. Importing
`std` itself, a submodule of a standard module, or any other `std.NAME` is a
compile-time error that lists the modules. *(ADR 0157)*

**std.use.2** — A standard module's functions and types are referred to by
qualified name, `fs.read_text(path)` and `fs.IoError` (`mod.use.2`). A
module is not a value, and naming a function or type the
module doesn't have is a compile-time error. *(ADR 0157)*

**std.use.3** — Two imports in one file MUST NOT make modules available under
the same name. A local MUST NOT reuse the name of a module imported with
`use` (`expr.bind.5`). *(ADR 0157)*

**std.use.4** — Every standard module is in scope under its own name without
a `use` line, unless the program declares a type, trait, function or constant
with that name, or imports something else under it. A local with the name
hides the module in its scope: calling one of the module's functions through
it is a compile-time error that says so. `use std.NAME` stays valid and
behaves as `std.use.1`. *(ADR 0277)*

```ail rules=std.use.4
fn main()
    print(path.ext("notes.txt"))   # txt
```

## Errors

**std.err.1** — A failure of `std.fs` is the error struct `fs.IoError`, with
fields `path str` (the path the function was given) and `kind str`. A
failure to start a process is the error struct `proc.ProcError`, with
fields `cmd str` and `kind str`. `kind` is one of `not_found`,
`permission`, `exists`, `is_dir`, `not_dir`, `not_empty`, `invalid_utf8`,
`timed_out` (a process killed at its time limit, `std.proc.2`),
`cancelled` (`proc.run` in a cancelled task, `conc.cancel.2`) and `other`,
and is the same for the same failure on every platform. `std.net` fails
with `net.NetError` (`std.net.4`). *(ADR 0158, ADR 0282, ADR 0321)*

```ail rules=std.err.1
use std.fs

fn load(path str) str ! fs.IoError
    fs.read_text(path)? "loading {path}"

fn main()
    text := fs.read_text("notes.txt") else err
        print("{err.kind}: {err.path}")
        ""
```

## Files: `std.fs`

**std.fs.1** — `read_text(path str) str ! IoError` returns the whole file as
text. A directory fails with `is_dir`; a file that isn't UTF-8 fails with
`invalid_utf8`. *(ADR 0157)*

**std.fs.2** — `write_text(path str, text str) ! IoError` creates the file or
replaces its contents. *(ADR 0157)*

**std.fs.3** — `append_text(path str, text str) ! IoError` creates the file
or adds to its end. *(ADR 0157)*

**std.fs.4** — `exists(path str) bool` is whether anything exists at the
path. *(ADR 0157)*

**std.fs.5** — `list_dir(path str) List[str] ! IoError` returns the names
(not paths) of the directory's entries, sorted by their bytes. *(ADR 0157)*

**std.fs.6** — `remove(path str) ! IoError` removes a file or an empty
directory. A directory with entries fails with `not_empty`; nothing removes
a tree. *(ADR 0157)*

**std.fs.7** — `make_dir(path str) ! IoError` creates a directory and any
missing parents. An existing directory is not a failure. *(ADR 0157)*

**std.fs.8** — `walk(dir str) List[str] ! IoError` returns every file
below `dir` at any depth, each as `dir` joined with its path below it
(`path.join`), sorted by bytes. Directories are not listed, names that start
with `.` are, and a symbolic link to a directory is listed, not followed.
Any directory that can't be read fails with that directory's path.
*(ADR 0281)*

**std.fs.9** — `glob(pattern str) List[str]` returns the existing paths
that match `pattern`, sorted by bytes, without duplicates. The pattern's
components are split at `/` (and at `\` on Windows). On Windows a pattern
that starts with a drive and a separator (`C:\`) is absolute, as one that
starts with a separator is everywhere. In a component, `*` matches any run of
characters and `?` exactly one, and neither matches a leading `.` unless
the component starts with `.`; any other character matches itself. A whole
component `**` matches zero or more directories (not starting with `.`),
and a trailing `**` matches everything below. A pattern starting with `/`
is absolute; otherwise paths are relative to the working directory and are
returned without a leading `./`. Directories that can't be read are
skipped, so `glob` never fails. *(ADR 0281)*

**std.fs.10** — `metadata(path str) Metadata ! IoError` returns
`struct Metadata { size int, modified_ms int, is_dir bool }`: the size in
bytes (`0` for a directory), the last modification time in milliseconds
since 1970-01-01T00:00:00Z, and whether it is a directory. Symbolic links
are followed. *(ADR 0281)*

**std.fs.11** — `rename(from str, to str) ! IoError` moves a file or
directory, replacing a file at `to`. *(ADR 0281)*

**std.fs.12** — `copy(from str, to str) ! IoError` copies the file `from`
to `to`, replacing a file there. A directory `from` fails with `is_dir`;
nothing copies a tree. For `rename` and `copy`, a failure's `path` is
`from` when `from` is missing (or, for `copy`, a directory), else `to`.
*(ADR 0281)*

**std.fs.13** — `read_bytes(path str) bytes ! IoError` reads the whole file
as `bytes`, whatever it holds. It fails as `read_text` does, and never because
of the contents. *(ADR 0415)*

**std.fs.14** — `write_bytes(path str, data bytes) ! IoError` writes `data`
to the file, creating it or replacing its contents; `append_bytes(path str,
data bytes) ! IoError` adds `data` at the end, creating the file if needed.
They fail as `write_text` and `append_text` do. *(ADR 0415)*

```ail norun rules=std.fs.9
use std.fs

fn main() !
    for p in fs.glob("examples/*.ail")
        m := fs.metadata(p)?
        print("{p} {m.size}")
```

## Environment: `std.env`

**std.env.1** — `args() List[str]` returns the program's arguments: the
words after the program's file, so `ail run tool.ail a b` gives
`["a", "b"]`. *(ADR 0159)*

**std.env.2** — `get(name str) str?` returns an environment variable, or
`none` when it isn't set or isn't UTF-8. *(ADR 0157)*

**std.env.3** — `cwd() str` returns the working directory. If it can't be
read, the program traps. *(ADR 0157)*

**std.env.4** — `program() str` returns the program's file path as it was
given to `ail run`. *(ADR 0159)*

**std.env.5** — `vars() Map[str, str]` returns the environment variables
whose names and values are UTF-8, in order of their names' bytes.
*(ADR 0282)*

**std.env.6** — `home() str?` returns the user's home directory: `HOME`,
else `USERPROFILE` (Windows), or `none` when neither is set or one is
empty. *(ADR 0296)*

## Paths: `std.path`

**std.path.1** — `std.path` works on path text only: it never touches the
file system, so its functions have no effects (`eff.infer.4`). *(ADR 0253)*

**std.path.2** — A separator is `/`, and also `\` on Windows.
`dir(p str) str` is the text before the last separator: `.` when there is
none, and the separator itself for a path in the root (`dir("/x")` is `/`).
On Windows a drive's root keeps its separator (`dir("C:\x")` is `C:\`).
`base(p str) str` is the text after the last separator. *(ADR 0253, ADR 0504)*

**std.path.3** — `ext(p str) str` is the text after the last `.` of
`base(p)`, without the dot, and `""` when there is none or the only `.` is the
first character (`ext(".bashrc")` is `""`). *(ADR 0253)*

**std.path.4** — `join(a str, b str) str` is `b` when `b` starts with a
separator or `a` is empty; on Windows also when `b` starts with a drive and
a separator (`C:\`). Otherwise `a` and `b` with one `/` between.
*(ADR 0253, ADR 0504)*

**std.path.5** — `stem(p str) str` is `base(p)` without its extension: the
text before the last `.`, or all of `base(p)` when there is none or the only
`.` is the first character (`stem("a/archive.tar.gz")` is `archive.tar`).
*(ADR 0630)*

```ail rules=std.path.1,std.path.2,std.path.3,std.path.4,std.path.5
use std.path

fn main()
    print(path.dir("examples/logscan.ail"))    # examples
    print(path.base("examples/logscan.ail"))   # logscan.ail
    print(path.ext("archive.tar.gz"))          # gz
    print(path.join("examples", "data"))       # examples/data
    print(path.stem("archive.tar.gz"))         # archive.tar
```

## Hashing: `std.hash`

**std.hash.1** — `std.hash` works on values only, so its functions have no
effects (`eff.infer.4`). Each data argument is `bytes` or `str`; a `str` is
hashed as its UTF-8 bytes (`type.bytes.10`). *(ADR 0442)*

**std.hash.2** — `sha256(s str) str`, `sha384(s str) str` and
`sha512(s str) str` return the digest of FIPS 180-4 as lower-case hex.
`hmac_sha256(key str, s str) str` returns the HMAC of RFC 2104 and RFC 4231
as lower-case hex. *(ADR 0442)*

**std.hash.3** — `crc32(s str) int` returns the CRC-32 of RFC 1952
(polynomial 0xEDB88320) as an `int` from 0 to 2^32 - 1. *(ADR 0445)*

**std.hash.4** — `hash64(s str) int` is a fast 64-bit hash for tables and
checksums, not for security. Its value for a given input is the same on
every platform and version, and may be any `int`. *(ADR 0444)*

```ail
use std.hash

fn main()
    print(hash.sha256("abc"))    # ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
    print(hash.crc32("123456789"))   # 3421780262
```

## Compression: `std.compress`

**std.compress.1** — `deflate(data bytes) bytes` and `gzip(data bytes) bytes`
compress `data` (a `str` is also accepted, as its UTF-8 bytes) at a fixed
level (6) into a raw DEFLATE stream (RFC 1951) and a gzip stream (RFC 1952).
They never fail, and the same input gives the same output on every platform.
They have no effects.

**std.compress.2** — `inflate(data bytes) bytes ! compress.CompressError`
and `gunzip(data bytes) bytes ! compress.CompressError` reverse them; use
`.to_str()` (`type.bytes.8`) when the result is text.
`CompressError { kind str, message str }`; `kind` is `corrupt` (malformed
data, including trailing bytes after a DEFLATE stream), `checksum` (gzip
CRC-32 or size mismatch), `too_big` (std.compress.4), `unsafe_path`, `unsupported`, or an I/O kind of
`std.err.1` for file functions. *(ADR 0443)*

**std.compress.3** — `gunzip` accepts the optional header fields of
RFC 1952 and several members in a row, and checks every member's CRC-32 and
size. `gzip` writes no timestamp or name. *(ADR 0445)*

**std.compress.4** — A decompression whose output would pass 2^30 bytes
fails with kind `too_big` before it allocates more. *(ADR 0445)*

**std.compress.5** — `gzip_file(src str, dst str)`,
`gunzip_file(src str, dst str)`, `zip_dir(dir str, dst str)` and
`unzip(archive str, dir str)` return `Unit ! compress.CompressError` and
have the effects `fs` and `io`. `zip_dir` stores every file under `dir`
(symbolic links skipped) with `/`-separated relative names sorted by name,
a fixed 1980-01-01 timestamp, and each file stored or deflated, whichever is
smaller; an empty directory is an entry ending in `/`. The archive is
deterministic. No zip64: more than 65535 files or any size over 4 GiB fails
with `too_big`. `unzip` creates `dir`, checks every CRC-32, and fails with
`unsafe_path`, before writing anything, when any name is absolute, has a
`..`, `.` or empty component, or contains `\`, `:` or a NUL. Encrypted
entries and methods other than store and deflate fail with `unsupported`.
*(ADR 0446)*

**std.compress.7** — `zlib(data bytes) bytes` and
`unzlib(data bytes) bytes ! compress.CompressError` use the zlib wrapper of
RFC 1950 (header `78 9c`, DEFLATE, big-endian Adler-32), as PDF Flate
streams and PNG do. `unzlib` fails with `checksum` on an Adler-32 mismatch
and `unsupported` for a preset dictionary. *(ADR 0443)*

**std.compress.6** — Output is read by the system `gzip` and `unzip` and
their output is read back; the project's tests check this when the tools
exist and say so when they don't. *(ADR 0445)*

```ail
use std.compress

fn main() !
    z := compress.gzip("abc abc abc abc")
    print(compress.gunzip(z)?.to_str() else "binary")
```

## Standard input and output: `std.io`

**std.io.1** — `read_line() str?` reads one line from standard input,
without its line ending (`\n` or `\r\n`), or returns `none` at the end of
input. Bytes that aren't UTF-8 become U+FFFD. *(ADR 0157)*

**std.io.2** — `read_all() str` reads the rest of standard input.
*(ADR 0157)*

**std.io.3** — `eprint(value)` writes the value's text form and a newline to
standard error, as `print` does to standard output (`mod.main.3`). Output
already printed reaches standard output first. *(ADR 0157)*

**std.io.4** — `exit(code int)` ends the program with exit status `code`,
after writing any pending standard output. It never returns, so the code
after it is unreachable (ADR 0139). A status outside 0 to 255 traps.
*(ADR 0157)*

**std.io.5** — `read_bytes() bytes` reads the rest of standard input as
`bytes`, without decoding it. *(ADR 0415)*

## Processes: `std.proc`

**std.proc.1** — `run(cmd str, args List[str]) Output ! ProcError` runs the
program `cmd` with the arguments `args`, directly and without a shell, and
waits for it. `cmd` is found on `PATH` unless it contains a path separator.
On Windows the search also tries each extension of `PATHEXT`, so `"npm"`
starts `npm.cmd` (ADR 0504). A batch file (`.cmd`, `.bat`) runs through the
system's command interpreter and fails as a `ProcError` when an argument has
characters the system won't pass to it. The process inherits the working directory and the environment, reads an
empty standard input, and its standard output and standard error are
captured. The result is `struct Output { status int, out str, err str }`:
the exit status (128 plus the signal number if a signal ended it, as shells
report it) and the captured text, with bytes that aren't UTF-8 as U+FFFD. A
process that runs and fails is a successful `run` with a nonzero
`status`; only a process that can't start is a `ProcError`. *(ADR 0160)*

```ail norun rules=std.proc.1
use std.proc

fn main() !
    r := proc.run("git", ["status", "--short"])?
    if r.status != 0
        print("git failed: {r.err}")
```

**std.proc.2** — `run` takes two optional parameters after `args`, which
may be left off from the end (ADR 0284): `input str?` and
`timeout_ms int?`. With `input`, the process reads that text as its
standard input, which is then closed; `none` gives an empty input. With
`timeout_ms`, a process still running after that many milliseconds is
killed and `run` fails with `ProcError` of kind `timed_out`; its output is
discarded. A negative `timeout_ms` traps. Standard output already printed
is written before the process starts. *(ADR 0282)*

**std.proc.3** — `which(name str) str?` returns the path of the program
that `run(name, ...)` would start, or `none` when there is none: `name`
itself when it contains a separator and is an executable file, otherwise
the first executable `DIR/name` for the directories of `PATH` in order
(on Windows, also with each extension of `PATHEXT`). *(ADR 0282)*

```ail rules=std.proc.2,std.proc.3
use std.proc

fn main() !
    if proc.which("sort") == none
        return
    r := proc.run("sort", [], "b\na\n", 2000)?
    print(r.out)
```

**std.proc.4** — `start(cmd str, args List[str], timeout_ms int?) Child !
ProcError` starts the program as `run` does (`std.proc.1`) and returns at once
with a `proc.Child`, `unique struct Child { pid int, stdin Channel[str], out
Channel[str], err Channel[str] }`. (`spawn` is a keyword, so the function is
`start`.) `timeout_ms` may be left off; a process still running after that
many milliseconds is killed (`std.proc.6`); a negative value traps. A process
that can't start is a `ProcError`, a cancelled task starts none (kind
`cancelled`, `conc.cancel.2`). *(ADRs 0424–0425)*

**std.proc.5** — The streams of a child are channels. `c.out` and `c.err`
hold the lines the child writes to standard output and standard error, in
order, without the line end (`\n` or `\r\n`; a last line with no end
counts), with bytes that aren't UTF-8 as U+FFFD. Each is closed when the
stream ends, so `recv` gives `none`, `for line in c.out` ends, and
`task.select` (`conc.select.1`) can wait on both. A child that writes more
than a task reads is never blocked: lines queue without a limit. `c.stdin`
takes text: `c.stdin.send(text)` writes it to the child's standard input
at once, without waiting, and `c.stdin.close()` ends the input (a child that
reads until the end of its input needs the `close`). Sending to a child that
has exited is not an error. All three are `Channel[str]`, so a task can be
passed `c.out` and read it while another task writes `c.stdin`. *(ADR 0425)*

**std.proc.6** — `wait(move c Child) int ! ProcError` waits for the child to
exit and returns its status as `run` does (`std.proc.1`). It fails with
`ProcError` of kind `timed_out` when the time limit of `start` killed the
child, and of kind `cancelled` (after killing the child) in a cancelled task.
While it waits, other tasks run, so a task can write the child's input or read
its output meanwhile. Lines the child wrote before it exited stay readable
from `out` and `err`. *(ADR 0428)*

**std.proc.7** — A `Child` is a unique value (`own.uniq.1`): it can't be
copied, and `wait` and `kill` take it with `move` (`own.move.2`), so a child
is waited or killed once. `kill(move c Child) int` ends the child at once
(with `SIGKILL`; `TerminateProcess` on Windows), waits for it and returns its
status (137 on Unix). A child that is neither waited nor killed when the
program ends is killed then, so no program leaves a process running. *(ADR 0425)*

**std.proc.8** — `run_bytes(cmd str, args List[str], input bytes?, timeout_ms
int?) RawOutput ! ProcError` is `run` with `bytes` in and out: `input` is
written to the process's standard input as it is, and `struct RawOutput {
status int, out bytes, err bytes }` holds what it wrote, undecoded. Everything
else, including the trailing optional arguments, the time limit, the failure
kinds and the effects, is that of `run`. `Output` is unchanged, with
`out` and `err` as `str`. *(ADR 0417)*

```ail
use std.proc

fn main() !
    c := proc.start("sort", [])?
    c.stdin.send("b\na\n")
    c.stdin.close()
    for line in c.out
        print(line)
    print(proc.wait(move c)?)
```

```ail rules=std.proc.8
use std.proc

fn main() !
    r := proc.run_bytes("sort", [], b"b\na\n")?
    print("{r.status} {r.out.len()}")
```

## Memory for C: `std.mem`

**std.mem.1** — `std.mem` has `cstr(s str) *u8`, which returns a new
NUL-terminated copy of `s`; `from_cstr(p *u8) str`, which reads a
NUL-terminated string (invalid UTF-8 becomes U+FFFD); `alloc(n, example) *T`,
`cast(p, example) *T`, `null() *u8` and `free(p)` (`unsafe.mem.3`). Each is an unsafe operation (`unsafe.op.1`), and a block that is
not freed leaks. The rules for bad pointers are `unsafe.mem.1-2`. *(ADR 0434)*

```ail fragment rules=std.mem.1
use std.mem

extern c fn strlen(s *u8) int

fn c_length(text str) int
    unsafe
        p := mem.cstr(text)
        n := strlen(p)
        mem.free(p)
        n
```

## Network: `std.net`

**std.net.1** — Every function of `std.net` has the effects `net` and
`io` (`eff.infer.4`), so `--deny net`, a package's capability grants and
`--deterministic` (`eff.det.1`) cover it; `serve` and `listen` add `net.serve`
(`std.net.14`). *(ADR 0283, ADR 0422)*

**std.net.2** — `get(url str) Response ! NetError` sends an HTTP/1.1 `GET`
to an `http://` or `https://` URL (`std.net.5`)
and returns `struct Response { status int, headers Map[str, str], body str }`:
the status code, the headers with names in lowercase in the order they
arrived (a repeated header's values joined with `, `), and the body as
text, with bytes that aren't UTF-8 as U+FFFD. A response with any status is
a success; only a request that gets no response fails. Redirects (`301`,
`302`, `303`, `307`, `308` with a `Location`) are followed, at most five.
Connecting, and each read and write, time out after 30 seconds.
*(ADR 0283)*

**std.net.3** — `post(url str, body str, content_type str) Response ! NetError`
sends `body` as an HTTP/1.1 `POST` with that `Content-Type`, and returns
the response as `get` does, without following redirects. *(ADR 0283)*

**std.net.4** — A failure is `error struct NetError { url str, kind str,
message str }`: the URL given, a short kind, and a sentence for people.
`kind` is `bad_url` (not `http://` or `https://HOST[:PORT][/PATH]`, or not
an address `HOST:PORT`), `bad_request` (a method, header name or header value
the request or response may not carry, `std.net.6`), `tls` (the TLS handshake
or connection failed, `std.net.5`), `not_found` (the host has no address),
`refused`, `timed_out`, `closed` (the connection or server was closed, or the
waiting task was cancelled, `std.net.9`), `bind` (the address is in use or
not permitted, `std.net.9`), `protocol` (a malformed response, or too many
redirects) or `other`. *(ADR 0283, ADR 0369, ADR 0418, ADR 0419)*

**std.net.5** — An `https` URL (default port 443) is fetched over TLS 1.3.
The server's certificate chain **MUST** lead to a trusted root, be within
its validity dates and allow server authentication, and its names
(subjectAltName) **MUST** include the URL's host; otherwise the request
fails with kind `tls` and a message naming the cause (for example, an
expired certificate, the names it does cover, or an untrusted issuer). The
trusted roots are the platform's, or the PEM file named by the environment
variable `SSL_CERT_FILE` when it is set. A server that doesn't speak TLS
1.3 fails with kind `tls`. Redirects may move between `http` and `https`.
*(ADR 0360, ADR 0366–0369)*

```ail norun rules=std.net.2,std.net.5
use std.net

fn main() !
    r := net.get("https://example.com/health") else err
        print("{err.kind}: {err.message}")
        return
    print("{r.status} {r.headers["content-type"]}")
```

**std.net.6** — `request(method str, url str, opts Options) Response ! NetError`
sends any method with options, where `struct Options { headers Map[str, str],
query Map[str, str], body str, timeout_ms int, max_redirects int }`.
`options()` returns the defaults: no headers, no query, an empty body, a
30000 ms timeout and 5 redirects; write `mut o := net.options()` and set
fields. The method is made uppercase and must be an HTTP token. `headers`
are sent after the defaults `Host`, `User-Agent: ail` and `Accept: */*`, and
replace a default of the same name (case-insensitively); `Connection`,
`Content-Length` and `Transfer-Encoding` are the client's own and are
ignored. A header name must be an HTTP token and a value must not contain
CR, LF or NUL: otherwise the request fails with kind `bad_request` before
anything is sent. `query` is appended to the URL, percent-encoded, after any
query the URL has. `body` is sent with a `Content-Length` (always for `POST`,
`PUT` and `PATCH`). `timeout_ms` bounds connecting and each read and write,
and must be at least 1; `max_redirects` must be at least 0 (otherwise the
program traps). `get(url)` and `post(url, body, content_type)` are
`request` with the defaults. *(ADR 0418)*

**std.net.7** — Redirects (`std.net.2`) are followed only for `GET` and
`HEAD`, at most `max_redirects` times; with `max_redirects` 0 the redirect
response itself is returned, and one more redirect than allowed fails with
kind `protocol`. When a redirect leaves the scheme, host or port it started
at, the headers `Authorization` and `Cookie` are dropped from the requests
that follow. Response bodies are read whole and returned as `str`: streaming
bodies are deferred until `bytes` (lane 123) and incremental reads can be
designed together, and a request that needs one uses a `Conn` (`std.net.12`)
and writes the HTTP itself. *(ADR 0418)*

**std.net.8** — A network call waits cooperatively. In a program with
tasks, a call that must wait for the network (connecting, reading, writing,
`accept`) lets the other ready tasks run, so a client and a server in one
program, or tasks that serve and ask, make progress. Without tasks it waits
and times out the same way. A wait ends with kind `closed` when its task is
cancelled (`conc.cancel.1`) or the program is shutting down. The set of
tasks that run, and the order, is the scheduler's (`conc.task.*`, ADR 0186),
not the network's. *(ADR 0420)*

**std.net.9** — `serve(addr str) Server ! NetError` listens for HTTP on
`HOST:PORT` (port 0 picks a free one) and returns a `Server`; failing to bind
is kind `bind`. `srv.addr() str` is the address it listens on, with the real
port. `srv.accept() Exchange ! NetError` waits for the next complete
request, from whichever connection finishes sending first, so a slow client
doesn't hold up others, and any number of tasks may call it on one server (a
pool of workers). `srv.close()` stops listening: every waiting `accept`
fails with kind `closed`, connections still sending a request are dropped,
and an `Exchange` accepted earlier can still be answered (a graceful stop).
`struct Exchange { request Request, handle int }` and `struct Request {
method str, path str, query Map[str, str], headers Map[str, str], body str,
peer str }`. The handler form is `std.net.http` (`std.net.18`); otherwise a server is a
loop over `accept`, usually in tasks. *(ADR 0419)*

**std.net.10** — The server accepts only well-formed HTTP/1.0 and HTTP/1.1
requests and answers the rest itself, with a plain-text body, without
showing them to the program: `400` for a malformed request line or header
(including whitespace before a colon, a folded line, a missing `Host` in
HTTP/1.1, a non-numeric or repeated `Content-Length`, both `Content-Length`
and `Transfer-Encoding`, or a bad chunk), `413` for a body over 16 MiB,
`431` for a head over 64 KiB, `408` when a connection takes more than 10
seconds to send a whole request, `501` for a transfer coding other than
`chunked`, and `505` for another HTTP version. `Expect: 100-continue` is
answered with `100 Continue`. `Request.path` is the target before `?`, as
sent; `query` is the rest, split at `&` and `=`, with `%XX` and `+`
decoded (a repeated key keeps its last value); header names are lowercase
and a repeated header's values are joined with `, `; `body` is the decoded
body as text, bytes that aren't UTF-8 as U+FFFD; `peer` is the client's
address. *(ADR 0419)*

**std.net.11** — `ex.respond(r Response) ! NetError` sends the response and
closes the connection: the server sends `Connection: close` and does not
keep connections alive. The status must be 100 to 599; header names must be
tokens and values must not contain CR, LF or NUL; otherwise `respond` fails
with kind `bad_request` and the exchange stays open to be answered again.
`Content-Length`, `Connection` and `Transfer-Encoding` in `r.headers` are
ignored (the server frames the response). A body is sent with a
`Content-Type` of `text/plain; charset=utf-8` unless `r.headers` has one;
`1xx`, `204` and `304` responses have no body; a reply to `HEAD` has the
headers a `GET` would have and no body. Answering twice, or after the
server's peer is gone, fails with kind `other` or `closed`. An exchange that
is never answered keeps its connection until the program ends. *(ADR 0419)*

**std.net.12** — `listen(addr str) Listener ! NetError` and `connect(host
str, port int) Conn ! NetError` are raw TCP. `l.accept() Conn ! NetError`
(any number of tasks may wait), `l.addr() str` and `l.close()` match
`Server`. A `Conn` has `c.read(max int) str ! NetError`: waits for at
least one byte and returns up to `max` bytes as text, never splitting a
character across two reads (so it may return up to three bytes more), and
`""` at the end of the stream; `c.read_line() str? ! NetError`: a line
without its `\n` or `\r\n`, or absent at the end of the stream;
`c.write(text str) ! NetError`; `c.set_timeout(ms int)` (default 30000):
how long a read, write or connect may make no progress before kind
`timed_out`; `c.peer() str`; and `c.close()`. A `Conn` and a `Server`
handle are used from one task at a time (`Server.accept` excepted). After
`close`, every operation fails with kind `closed`. `read_bytes` and
`write_bytes` are the undecoded forms (`std.net.16`). *(ADR 0421)*

**std.net.13** — `connect_tls(host str, port int) Conn ! NetError` connects
and runs the TLS 1.3 handshake of `std.net.5` for `host`, with the same
certificate rules, and returns a `Conn` that reads and writes the
decrypted stream. Failures are kind `tls`, or the transport's kind during
the handshake. There is no TLS server. *(ADR 0421, ADR 0360)*

**std.net.14** — Serving needs a capability of its own. `serve` and
`listen` have the effects `net`, `net.serve` and `io`; every other `std.net`
function and the methods of `Server`, `Exchange`, `Listener` and `Conn` have
`net` and `io`. `--deny net.serve` refuses a program that can start a
server and still lets it be a client; `--deny net` and `--deterministic`
refuse both, and a package's `caps` list names `net.serve` separately
(`eff.cap.4–5`). *(ADR 0422)*

**std.net.15** — Targets: Linux x86-64 and macOS ARM64 run the same code
over the platform's sockets. On Windows the sockets are the Rust standard
library's (Winsock); the code uses nothing else, and it is untested there
(ADR 0004, lane 153). *(ADR 0423)*

**std.net.16** — Bodies as `bytes`. *(ADR 0416)*
- `get_bytes(url str)`, `post_bytes(url str, data bytes, content_type str)`
  and `request_bytes(method str, url str, opts Options)` act as `get`, `post`
  and `request` but return `struct RawResponse { status int, headers
  Map[str, str], data bytes } ! NetError`: the body undecoded. `Response.body`
  stays a `str`, the same bytes with each invalid UTF-8 sequence as U+FFFD.
- `Options` has a last field `data bytes`, empty from `options()`. A request
  sends `data` as its body when it isn't empty, and `body` otherwise.
- `Request` has a last field `data bytes`: the body the server received, as
  it arrived (`body` is its lossy text).
- `ex.respond_bytes(status int, headers Map[str, str], data bytes) !
  NetError` answers an `Exchange` with `data` as the body; it checks the
  status and headers as `respond` does.
- `Conn.read_bytes(max int) bytes ! NetError` waits for at least one byte and
  returns up to `max`, empty at the end of the stream; it fails as `read`
  does, and `Conn.write_bytes(data bytes) ! NetError` writes all of `data`.

**std.net.17** — Repeated headers. `Response` and `RawResponse` have a last
field `all_headers Map[str, List[str]]`: each header name in lowercase with
every value it arrived with, one by one and in order (a `Set-Cookie` whose
`Expires` holds a comma stays whole); `headers` still joins them with `, `
(`std.net.2`). `net.Response(...)` may leave `all_headers` out: it is then
empty. When a server answers with `ex.respond(r)`, each name in
`r.all_headers` is sent once per value in place of its entry in `r.headers`,
which is how to send several `Set-Cookie` headers. *(ADR 0418)*

**std.net.18** — The handler form. `use std.net.http` gives
`http.serve(addr str, handler fn(net.Request) net.Response) ! net.NetError`,
which listens on `addr` and answers every request with `handler`, and
`http.run(srv net.Server, handler) !`, which does the same on a server the
program holds and returns when `srv.close()` has stopped it. Four tasks
answer requests at once, each one at a time; a handler is a declared
function (`conc.task.*`, function values), and a client that goes away
before its answer is not a failure. *(ADR 0419)*

```ail rules=std.net.17,std.net.18
use std.net
use std.net.http

fn page(r net.Request) net.Response
    cookies := {"set-cookie": ["a=1; Expires=Wed, 21 Oct 2026 07:28:00 GMT", "b=2"]}
    net.Response(status: 200, headers: {}, body: "hi {r.path}", all_headers: cookies)

fn main() !
    srv := net.serve("127.0.0.1:0")?
    t := spawn http.run(srv, page)
    r := net.get("http://{srv.addr()}/x")?
    print("{r.body} {r.all_headers["set-cookie"].len()} {r.headers["set-cookie"]}")
    srv.close()
    t.wait()?
```

```ail
use std.net

fn answer(srv net.Server) !
    ex := srv.accept()?
    q := ex.request.query.get("name") else "world"
    ex.respond(net.Response(status: 200, headers: {}, body: "hello {q}"))?

fn main() !
    srv := net.serve("127.0.0.1:0")?
    t := spawn answer(srv)
    r := net.get("http://{srv.addr()}/?name=ail")?
    print(r.body)
    t.wait()?
```

```ail rules=std.net.16
use std.net

fn echo(srv net.Server) !
    ex := srv.accept()?
    ex.respond_bytes(200, {}, ex.request.data + b"!")?

fn main() !
    srv := net.serve("127.0.0.1:0")?
    t := spawn echo(srv)
    r := net.post_bytes("http://{srv.addr()}/", b"\x01\x02", "application/octet-stream")?
    print("{r.status} {r.data.hex()}")
    t.wait()?
```

## Clock: `std.time`

**std.time.1** — `now_ms() int` returns the time in milliseconds since
1970-01-01T00:00:00Z. *(ADR 0157)*

**std.time.2** — `sleep_ms(ms int)` pauses the program for at least `ms`
milliseconds. A negative `ms` traps. *(ADR 0157)*

## Calendar: `std.time.date`

A standard module written in AIL (`use std.time.date`, ADR 0447). A
**duration** is an `int` of milliseconds; the constants `date.SEC`, `MIN`,
`HOUR` and `DAY` name the usual ones. A **zone** is a `str`: `"UTC"` (or
`"Z"`), a fixed offset such as `"+05:30"` (`±HH`, `±HHMM` or `±HH:MM`), or an
IANA name such as `"America/New_York"` read from the platform's tz database
(`std.tz`, ADR 0448).

**std.date.1** — `Date(year, month, day)` is a day of the proleptic Gregorian
calendar. `Date.new(y, m, d)` is `none` unless the day exists.
`is_leap(y)` and `days_in_month(y, m)` follow the Gregorian rules. A `Date`
compares by year, then month, then day, and prints as its `iso()` text.

**std.date.2** — `d.days()` is the number of days since 1970-01-01, negative
before it; `Date.from_days(n)` is its inverse.

**std.date.3** — `d.weekday()` is 1 for Monday to 7 for Sunday; `d.yday()`
is 1 for January 1.

**std.date.4** — `d.add_days(n)` and `a.since(b)` (days from `b` to `a`) are
exact for any `n`.

**std.date.5** — `d.add_months(n)` moves the month and keeps the day, except
that a day the target month lacks becomes that month's last day (January 31
plus one month is February 28 or 29). `d.add_years(n)` is `add_months(12 * n)`,
so February 29 becomes February 28 in a common year. *(ADR 0449)*

**std.date.6** — `DateTime` is an instant seen in a zone: `date`, `hour`,
`minute`, `second`, `ms`, `offset` (seconds east of UTC at that instant) and
`zone`. `==` compares these fields, so the same instant in two zones is not
equal; `a.unix_ms() == b.unix_ms()` and `a.before(b)` compare instants.

**std.date.7** — `date.utc(ms)` and `date.at(ms, zone)` give the instant `ms`
milliseconds after 1970-01-01T00:00:00Z; `at` fails with `DateError` for an
unknown zone. `dt.in_zone(zone)` is the same instant in another zone. Reading
a local time in a zone (`parse_in`, `add_days`, `add_months`, `add_years`)
resolves it to an instant: a local time that happens twice (the clock goes
back) takes the first, and one that is skipped (the clock goes forward) takes
the offset in force before the change, so it lands just after the gap.
*(ADR 0449)*

**std.date.8** — `dt.unix_ms()`, `a.before(b)` and `a.since(b)` (milliseconds
from `b` to `a`) work on instants.

**std.date.9** — `dt.iso()` is `YYYY-MM-DDTHH:MM:SS`, then `.mmm` unless the
milliseconds are zero, then `Z` for offset 0 or `±HH:MM`. `d.iso()` is
`YYYY-MM-DD`. Years below 1000 are padded to four digits. *(ADR 0450)*

**std.date.10** — `dt.format(pattern)` replaces these tokens and copies the
rest: `%Y` year, `%y` two-digit year, `%m` month, `%d` day, `%e` day padded
with a space, `%j` day of year, `%H` hour, `%I` 12-hour, `%p` AM or PM, `%M`
minute, `%S` second, `%f` milliseconds, `%a` and `%A` weekday name, `%b` and
`%B` month name (English), `%z` `+0530`, `%:z` `+05:30`, `%Z` the zone text,
`%s` Unix seconds, `%%` a percent sign. An unknown token such as `%q` is kept
as written. *(ADR 0451)*

**std.date.11** — `date.now()` is the current instant in UTC and
`date.now_local()` the current instant in the system zone (`tz.system()`, UTC
when that zone isn't installed). Both read the clock, so `--deterministic`
rejects a program that reaches them (`eff.det.1`); the rest of the module is
pure.

**std.date.12** — `dt.add_ms(n)` adds elapsed time. `dt.add_days(n)`,
`add_months(n)` and `add_years(n)` keep the wall-clock time in the same zone
(std.date.7), so across a clock change a day may last 23 or 25 hours.

**std.date.13** — `date.parse_date(text)` reads `YYYY-MM-DD`.

**std.date.14** — `date.parse_iso(text)` reads `YYYY-MM-DD` and `T` (or a
space) and `HH:MM[:SS[.fraction]]` and an offset `Z`, `±HH`, `±HHMM` or
`±HH:MM`; digits of the fraction past milliseconds are dropped. The result is
in the text's own offset. Text with no offset is an error; `date.parse_in(text,
zone)` reads it in `zone` instead (an offset in the text still wins). Seconds
go to 59: leap seconds are not supported. *(ADR 0450)*

**std.date.15** — Refused text, zones and dates fail with `DateError(text,
why)`, which displays as `why: \`text\``.

## Zones: `std.tz`

**std.tz.1** — `tz.offset(zone, secs)` is the UTC offset in seconds of IANA
zone `zone` at the instant `secs` (seconds since the epoch), or `none` when the
zone isn't installed, the name is empty or unsafe (it may hold only letters,
digits and `/ _ + -`, no `..`, no leading `/`). It reads TZif files (RFC 9636)
from `/usr/share/zoneinfo` and the usual alternatives, and applies the file's
POSIX rule for instants after its last transition. It has the effects `fs` and
`env`. *(ADR 0448)*

**std.tz.2** — `tz.system()` is the system zone: `TZ` if it names a safe zone,
else the zone `/etc/localtime` links to, else `"UTC"`.

**std.tz.3** — On Windows there are no TZif files: `tz.offset` is `none` for
every name and `tz.system()` is `"UTC"`, until the registry reader of ADR 0448
lands; UTC and fixed offsets work everywhere.

## Randomness: `std.random`

**std.random.1** — `use std.random` provides `int`, `float`, `seed` and
`shuffle`. Each has the capability `random` (`eff.infer.4`). *(ADR 0206)*

**std.random.2** — `int(lo int, hi int) int` returns a uniformly distributed
integer with `lo <= n <= hi`; both bounds are inclusive. It traps if `lo >
hi`. `float() float` returns a uniformly distributed float with `0 <= x <
1`. *(ADR 0206)*

**std.random.3** — The functions draw from one generator. `seed(s int)`, or
`ail run --seed S`, sets its state from `s`: the draws that follow are then a
function of `s` and of the calls made, so a run repeats. Without a seed the
generator starts from the clock. The sequence for a given seed is not
specified across releases. `--seed` needs an integer (exit status 2
otherwise). *(ADR 0206, ADR 0207)*

**std.random.4** — `shuffle(mut xs)` reorders any `List[T]` in place so that
every order is equally likely (Fisher-Yates). The argument is marked `mut`
(`own.param.2`) and must be a list. *(ADR 0206)*

**std.random.5** — `ail run --deterministic` rejects every program that
reaches `random` (`eff.det.2`). *(ADR 0207)*

```ail rules=std.random.1,std.random.3,std.random.4
use std.random

fn main()
    random.seed(1)
    mut deck := [1, 2, 3, 4, 5]
    random.shuffle(mut deck)
    print(deck.len())
    print(random.int(1, 6) <= 6)
```

## Tables of text: `std.csv`

**std.csv.1** — `use std.csv` provides `parse(text str)` and
`parse_sep(text str, sep char)`, each giving `List[List[str]] ! CsvError`:
one list of fields per row. Rows end at `\n` or `\r\n`; a blank line adds no
row; a final newline adds no empty row. A field in double quotes may hold the
separator, line breaks, and `""` for one quote. *(ADR 0626)*

**std.csv.2** — `records(text str) List[Map[str, str]] ! CsvError` reads the
first row as the header and gives each later row as a map from header name to
field. A row whose field count differs from the header's is a `CsvError`.
*(ADR 0626)*

**std.csv.3** — `quote(field str) str` wraps a field in double quotes, and
doubles the quotes inside it, when it holds a comma, a quote or a line break;
otherwise it returns the field. `write(rows List[List[str]]) str` quotes each
field and ends each row with `\n`. *(ADR 0626)*

**std.csv.4** — `write_records(header List[str], recs List[Map[str, str]])
str` writes the header and then each map's values in header order; a key a map
lacks writes as an empty field. *(ADR 0626)*

**std.csv.5** — `CsvError` is an error with `why str` and `row int` (1-based).
An unterminated quote and a record of the wrong width are its causes.
*(ADR 0626)*

```ail rules=std.csv.1,std.csv.2,std.csv.3,std.csv.4
use std.csv

fn main() !
    rows := csv.parse("a,b\n1,\"x, y\"\n")?
    print(rows[1][1])
    recs := csv.records("name,qty\nbolt,4\n")?
    print(recs[0]["qty"])
    print(csv.write(rows))
```

```ail rules=std.csv.5
use std.csv

fn main()
    rows := csv.parse("a,\"b\n") else err
        print("{err.why}, row {err.row}")
        []
    print(rows.len())
```

## Command-line arguments: `std.args`

**std.args.1** — `use std.args` provides the spec makers `flag(name, short,
help)` for a switch and `opt(name, short, help, default)` for an option that
takes a value. `short` is one letter or `""`. *(ADR 0627)*

**std.args.2** — `pos(name, help)` is a required positional word; the
positionals are taken in the order the specs list them. `rest(name, help)`
allows any number of further positionals, which `Args.rest` returns. Without a
`rest` spec, an extra word is an error. *(ADR 0627)*

**std.args.3** — `parse(name str, about str, specs List[Spec], words
List[str]) Args ! ArgsError` reads `words` (usually `env.args()`). `--NAME` and
`-S` set a switch; `--NAME V`, `--NAME=V` and `-S V` set an option; a bare `--`
makes every later word a positional; a lone `-` is a positional. *(ADR 0627)*

**std.args.4** — `Args.flag(name) bool` says whether the switch was given;
`Args.get(name) str` gives an option's value (its default when absent) or a
positional's word; `Args.rest` is the list of words past the named positionals.
*(ADR 0627)*

**std.args.5** — `--help` and `-h` print the usage text and exit with status 0.
`usage(name, about, specs) str` returns that text: a `usage:` line, the
`about` line, and one line per argument with its help and default. *(ADR 0627)*

**std.args.6** — `ArgsError` is an error with `why str` and `usage str`; its
display text is both. Causes: an unknown option, an option without its value,
a value given to a switch, a missing positional, an extra positional.
*(ADR 0627)*

```ail rules=std.args.1,std.args.2,std.args.3,std.args.4
use std.args

fn main() !
    spec := [args.flag("verbose", "v", "more output"), args.opt("out", "o", "output", "a.txt"), args.pos("input", "file")]
    a := args.parse("tool", "demo", spec, ["-v", "in.txt"])?
    print("{a.flag("verbose")} {a.get("out")} {a.get("input")}")
```

```ail rules=std.args.5,std.args.6
use std.args

fn main()
    spec := [args.opt("out", "o", "output", "a.txt"), args.pos("input", "file")]
    print(args.usage("tool", "demo", spec))
    a := args.parse("tool", "demo", spec, ["--out"]) else err
        print(err.why)
        return
    print(a.get("input"))
```

## Logging: `std.log`

**std.log.1** — The environment variable `LOG_LEVEL` (`debug`, `info`, `warn`,
`error` or `off`, case-insensitive; default `info`) says which messages show.
An unknown value shows nothing. *(ADR 0628)*

**std.log.2** — `use std.log` provides `debug`, `info`, `warn` and `err`, each
taking a `str`. A message at or above the level is written to standard error as
`LEVEL message` and a line break; the others are dropped. `err` is not named
`error` because that is a keyword. *(ADR 0628)*

```ail rules=std.log.1,std.log.2
use std.log

fn main()
    log.debug("hidden at the default level")
    log.info("shown on standard error")
    print("done")
```

## Windows: `std.ui`

A window showing a grid of monospace text cells, with keyboard and resize
events: enough for a text editor or a dashboard. Colors are `int`s
`0xRRGGBB`; only the low 24 bits count. *(ADR 0177)*

```ail skip=context
struct Window              # a handle; at most one window is open
struct Size
    cols int
    rows int
struct Modifiers
    ctrl bool
    alt bool
    shift bool
enum Event
    Key(key str, ctrl bool, alt bool, shift bool)
    Resize(cols int, rows int)
    Close
    Mouse(col int, row int, button str, kind str)
    Scroll(lines int)
error struct UiError
    kind str               # "no_display", "closed" or "other"
    message str
```

**std.ui.1** — `open(title str, cols int, rows int) Window ! UiError` opens
a window `cols` by `rows` cells in size, with every cell blank (a space,
light grey on black). It fails with `kind` `no_display` when no window can
be shown (no display, or a run without a window host), and `other` when
`cols` or `rows` is outside 1 to 1000 or a window is already open. *(ADR 0177)*

**std.ui.2** — The methods of `Window` are `size`, `clear`, `put`, `cursor`,
`present`, `next_event`, `poll_event`, `modifiers`, `set_title` and `close`; naming another is a
compile-time error. All but `size(self) Size` and `modifiers(self) Modifiers`
take `mut self`. `size` is the
grid's size in cells, which follows `Resize` events; it is 0 by 0 for a
window that was never opened or has been closed with `close`. *(ADR 0177)*

**std.ui.3** — `clear(mut self, bg int)` makes every cell a space on `bg`.
*(ADR 0177)*

**std.ui.4** — `put(mut self, col int, row int, text str, fg int, bg int)`
writes `text` into the cells from (`col`, `row`) rightwards, one character
per cell; columns and rows count from 0 at the top left. What falls outside
the grid on any side is dropped, so any `col` and `row` are allowed. A
character outside printable Latin-1 (U+0020–U+007E, U+00A0–U+00FF),
including a tab, shows as `?`. *(ADR 0177)*

**std.ui.5** — Drawing is buffered: `clear` and `put` change only the grid,
and `present(mut self)` shows it. `cursor(mut self, col int, row int)` sets
where the caret, a thin bar in white or black, whichever shows on the cell's
background, is drawn at the next `present`; a position outside the grid,
such as (-1, -1), hides it. *(ADR 0177)*

**std.ui.6** — `next_event(mut self) Event` waits for the next event;
`poll_event(mut self) Event?` returns `none` at once if there is none.
`Resize` reports a new size in cells, and the grid has that size before the
event is returned: cells that still fit are kept, new cells are blank.
`Close` means the window was closed (std.ui.8). *(ADR 0177)*

**std.ui.7** — In `Key(key, ctrl, alt, shift)`, `key` is either the typed
character (`"a"`, `"A"`, `" "`, `"é"`, following the keyboard layout and
Shift) or one of the names `enter`, `tab`, `backspace`, `delete`, `escape`,
`up`, `down`, `left`, `right`, `home`, `end`, `pageup`, `pagedown` and `f1`
to `f12`. While Ctrl is held, a text key gives its unshifted character
(Ctrl-S is `Key("s", true, false, false)`, Ctrl-Shift-S has `shift` true),
and no typed character is reported. On macOS, Command reports as `ctrl`.
Modifier keys alone give no event. *(ADR 0177)*

**std.ui.8** — When the user closes the window, the next event is `Close`.
After that, and after `close(mut self)`, the window is closed: `next_event`
and `poll_event` return `Close`, and drawing does nothing. A program that
ends closes its window. *(ADR 0177)*

**std.ui.9** — `ui.open`, `ui.clipboard_get`, `ui.clipboard_set` and every
`Window` method give the capability `ui`
and the effect `io` (`eff.infer.4`), so `ail run --deny ui` refuses a program
that can open a window (`eff.cap.1`). *(ADR 0177)*

```ail norun rules=std.ui.1,std.ui.2,std.ui.5,std.ui.6,std.ui.7,std.ui.8
use std.ui

fn main()
    mut w := ui.open("Hello", 40, 5) else err
        print("no window: {err.kind}")
        return
    while true
        w.clear(0x202020)
        w.put(1, 1, "Hello, AIL (Esc quits)", 0xFFFFFF, 0x202020)
        w.present()
        match w.next_event()
            Key(key, _, _, _) =>
                if key == "escape"
                    break
            Resize(_, _) | Mouse(_, _, _, _) | Scroll(_) => continue
            Close => break
    w.close()
```

**std.ui.10** — `Mouse(col, row, button, kind)` reports a mouse button over
cell (`col`, `row`): `button` is `left`, `right` or `middle`, and `kind` is
`down` when it is pressed, `up` when it is released, and `drag` when the
mouse moves to another cell while it is held. Movement with no button held
gives no event. *(ADR 0240)*

**std.ui.11** — `Scroll(lines)` reports the wheel or a trackpad: `lines` is
the number of text lines to scroll, positive towards the end of a document.
A wheel notch is about 3 lines. *(ADR 0240)*

**std.ui.12** — `clipboard_set(text str)` puts `text` on the clipboard, and
`clipboard_get() str?` returns its text, or `none` when it holds no text.
Under `ail run` this is the system clipboard where the platform's tools
reach it (`pbcopy` on macOS, `wl-copy`, `xclip` or `xsel` on Linux,
PowerShell on Windows); otherwise, and always under `ail test`, it is a
clipboard of the program's own, so tests never read or change the user's.
Neither needs an open window. *(ADR 0240)*

**std.ui.13** — `set_title(mut self, text str)` changes the window's title;
on a closed window it does nothing. *(ADR 0240)*

**std.ui.14** — `modifiers(self) Modifiers` returns which of Ctrl, Alt and
Shift were held when the last event that `next_event` or `poll_event`
returned happened (as std.ui.7 reports them, Command counting as Ctrl on
macOS), so a program can tell a Shift-click from a click. Before any event,
and on a closed window, all three are false. *(ADR 0242)*

**std.ui.15** — `set_font_size(mut self, px int)` changes the font to the
available size nearest `px` (the smaller on a tie); `font_size(self) int`
returns it. A size is a cell height in pixels, one of 10, 12, 13, 14, 15,
18, 20, 24, 26, 28, 30, 36 and 40; a window opens at 20. The window keeps
its size in pixels, so when the number of cells that fit changes, a
`Resize` event follows (std.ui.7) and the grid takes the new size when the
program receives it. On a window that is not open, `set_font_size` does
nothing visible. *(ADR 0243)*

**std.ui.16** — `put_styled(mut self, col int, row int, text str, fg int,
bg int, style Style)` is `put` (std.ui.4) with a `ui.Style(bold, italic,
underline)` drawn into each cell; `put` draws plain text. The backend
synthesises the styles from the one font: bold thickens strokes, italic
slants them, underline marks the row below the baseline. *(ADR 0294)*

**std.ui.17** — `ui.BLACK`, `BLUE`, `CYAN`, `GRAY`, `GREEN`, `MAGENTA`,
`ORANGE`, `RED`, `WHITE` and `YELLOW` are `int` colors (0xRRGGBB) usable
wherever a color is. Any other `ui.NAME` that isn't a function or type is
an error naming the colors. *(ADR 0294)*

**std.ui.18** — `set_cursor(mut self, col int, row int, shape Cursor)`
places the caret like `cursor` (std.ui.5) and sets its shape: `Bar` (a
thin line at the cell's left, `cursor`'s shape), `Block` (the cell's colors
swapped) or `Underline` (a line at the cell's foot). *(ADR 0294)*

### Widgets: `std.ui.widgets`

**std.ui.19** — `use std.ui.widgets` imports a standard module written in
AIL and shipped inside the toolchain. It is an ordinary module (spec 16):
its items are used qualified (`widgets.ListView`), its source is shown as
`<std>/ui/widgets.ail` in diagnostics, and it needs the `ui` capability
only where it draws. It is the one standard module with a submodule path.
*(ADR 0314)*

**std.ui.20** — A widget's colors come from a theme, a `Map[str, int]` keyed
by the kinds the editor's themes use (`bg`, `fg`, `pane_bg`, `line_bg`,
`select_bg`, `status_bg`, `status_fg`, `gutter_fg`); `widgets.color(theme,
kind)` gives a kind's color, with the dark default for a kind the theme
leaves out. *(ADR 0315)*

**std.ui.21** — Every widget has `render(self, r Rect, theme) List[Run]`,
which is pure and fills exactly the cells of `r`; `draw(self, mut w, r,
theme)`, which puts those runs on a window; and `on_event(mut self, ev
ui.Event, ...) Action`, where `Action` is `Ignored` (not for this widget),
`Changed`, `Submit` (Enter, a press, or a click on a choice) or `Cancel`
(Escape). Widgets that take mouse events also take their `Rect`.
`widgets.text_rows(runs, cols, rows)` gives the text a window would show,
so a program can print or test a screen without one. *(ADR 0315)*

**std.ui.22** — `ListView` shows the items that `widgets.fuzzy` keeps for
its `filter`: those holding the filter's characters in order, ignoring
case. Typing edits the filter; Up, Down, PageUp and PageDown move the
selection, which stays visible; `chosen()` is the selected item. With no
match it shows its `empty` text. *(ADR 0315)*

**std.ui.23** — `TextField` edits one line: printable keys insert at the
cursor, and Backspace, Delete, Left, Right, Home and End do what they
name. `caret_col(r)` is the caret's column, and `draw` places the caret
there. *(ADR 0315)*

**std.ui.24** — `Button` is pressed by a click, or by Enter or Space while
`focused`. `Dialog` shows a message above its choices; Left, Right and Tab
move between them. `Pane` scrolls lines with a cursor line (a `cursor`
outside the lines highlights none) and takes the wheel. `Tree` shows
`Node`s in depth-first order, hiding the children of a closed node; Right
opens, Left closes, Enter opens or closes a parent and submits a leaf.
*(ADR 0315)*

## Tensors: `std.tensor`

Dense arrays of numbers whose operations run as whole-array kernels: on the
CPU's vector unit and, for large products, on several cores. This is the
CPU slice of ADR 0247; device backends are allowed by ADR 0381 and not
yet implemented. *(ADR 0328)*

**std.tensor.1** — `tensor.Tensor[T]` is a value (ADRs 0060–0063) holding
a shape and `int` or `float` elements, row-major. It has no fields and no
struct literal; it is built by std.tensor.2 and read by its methods, none
of which changes the receiver. Its functions and methods have no effects
(`eff.infer.4`). A tensor of any other element type is a compile-time
error. *(ADR 0328)*

**std.tensor.2** — `tensor.from(xs List[T])` is the 1-D tensor of `xs`;
`tensor.from(xs List[T], shape List[int])` gives it `shape`.
`tensor.zeros(shape List[int])` is all zeros, `Tensor[float]` unless the
expected type is `Tensor[int]`. A negative size, or a shape whose element
count differs from `xs.len()`, traps. *(ADR 0328)*

**std.tensor.3** — The methods are `add`, `dot`, `len`, `map`, `matmul`,
`max`, `mean`, `mul`, `reshape`, `shape`, `sum`, `to_list` and `transpose`
(ADR 0345). `shape() List[int]`,
`len() int` (the element count), `to_list() List[T]` (row-major), and
`reshape(shape List[int]) Tensor[T]` (the same elements; traps like
std.tensor.2 when the count differs). Any other method is a compile-time
error that lists them. *(ADR 0328)*

**std.tensor.4** — `a.add(b)` and `a.mul(b)` are elementwise `+` and `*`;
`a.dot(b) T` is the sum of the elementwise products. `a` and `b` have the
same element type (checked). `add`, `mul` and the operators broadcast
NumPy-style (std.tensor.10): the shapes line up from the right, and per axis
the sizes are equal or one is 1, which stretches to the other; a missing
leading axis counts as 1. The result has the broadcast shape. `dot` needs
equal shapes. Shapes that don't broadcast trap. *(ADR 0328, amended by ADR 0522)*

**std.tensor.5** — On `float` tensors, `add`, `mul` and `/` round each element
as `+`, `*` and `/` do. `sum` and `dot` add in this order: element `i` (or
product `i`) goes to partial sum `i % 16` for the first `16 * (n / 16)`
elements; the partial sums then halve, `s[i] += s[i + h]` for `h` = 8, 4,
2, 1; the remaining elements are added to `s[0]` in order. Products are
rounded before they are added (no fused multiply-add). So a result is the
same on every target and vector unit (ADR 0008), though it can differ in
the last bits from a plain loop that adds left to right. *(ADR 0329)*

**std.tensor.6** — On `int` tensors, `sum`, `dot` and `matmul` add in
index order and trap when a product or a running sum doesn't fit `int`;
`add` and `mul` trap when an element's result doesn't fit. *(ADR 0329)*

**std.tensor.7** — `a.matmul(b) Tensor[T]` multiplies an `m` by `k` tensor
by a `k` by `n` one. Element `(i, j)` adds `a(i, p) * b(p, j)` for `p` from
0 to `k - 1`, in that order, starting from zero: exactly the plain triple
loop. Tensors that aren't both 2-D, or whose inner sizes differ, trap. When
`m * k * n` is at least 64³ the rows are split across the thread pool
(`conc.par.9`); the result doesn't change. *(ADR 0329)*

**std.tensor.8** — `t.map(|x| e)` evaluates `e` once per element, in
row-major order, with `x` bound to the element, and gives a tensor of the
same shape. `e` is `int` or `float` and decides the new element type.
*(ADR 0328)*

**std.tensor.9** — Two tensors are `==` when their shapes and their
elements are equal. A tensor's text form is the call that rebuilds it:
`tensor.from([1.0, 2.0])`, with the shape after the list when the tensor
isn't 1-D. *(ADR 0328)*

**std.tensor.10** — `a + b`, `a - b`, `a * b` and `a / b` with a tensor
operand are elementwise, with no other arithmetic operator defined for
tensors. Both operands are tensors with one element type and broadcastable
shapes (std.tensor.4; `a + b` is `a.add(b)`, `a * b` is `a.mul(b)`), or one
is a tensor and the
other a scalar of its element type, on either side, which is taken with
every element: `t * 2.0`, `1.0 - t`, `8.0 / t`. `/` is defined on `float`
tensors only; `/` on a `Tensor[int]` is a compile-time error. An `int`
literal beside a `Tensor[float]` is a `float` (`type.num.10`). Rounding is
std.tensor.5's and overflow std.tensor.6's; other operand types are a
compile-time error, and shapes that don't broadcast trap. `+=`, `-=` and `*=` follow.
Matrix product stays `a.matmul(b)`. *(ADR 0344, ADR 0507)*

**std.tensor.11** — `t.sum()`, `t.max()` and `t.mean()` reduce every element
to a `T`; `t.sum(axis int)`, `t.max(axis)` and `t.mean(axis)` reduce along
one axis to a `Tensor[T]` whose shape drops that axis. Each result adds its
elements, taken along the axis in index order, in std.tensor.5's order, so
the axis sum of a row equals `sum()` of that row bit for bit; `int` sums are
std.tensor.6's. `max` keeps the first element and replaces it by each later
one that is `>` it: a NaN never replaces it, and ties keep the earlier one.
`mean` is the sum divided by the count, and is a compile-time error on
`Tensor[int]`. An axis outside `0` to rank − 1 traps, and so do `max` and
`mean` of zero elements. *(ADR 0345)*

**std.tensor.12** — `t.transpose()` of an `m` by `n` tensor is the `n` by
`m` tensor whose element `(j, i)` is `t`'s `(i, j)`. A tensor that isn't
2-D traps. *(ADR 0345)*

**std.tensor.13** — `t.relu()`, `t.sigmoid()` and `t.tanh()` apply an
activation to each element of a `float` tensor and give a tensor of the
same shape. `relu(x)` is `x` when `x > 0.0` and `0.0` otherwise, except
that a NaN stays NaN. `sigmoid(x)` is `1.0 / (1.0 + exp(-x))` and `tanh` is
the hyperbolic tangent, both with the platform's `exp` and `tanh`, so their
last bit may differ between operating systems. Each element is computed by
the same scalar code on every ISA, so std.tensor.5's vector paths don't
apply to them.
These methods on a `Tensor[int]` are a compile-time error. *(ADR 0507)*

**std.tensor.14** — `-t` on a tensor negates each element and gives a tensor
of the same shape and element type. A `float` is negated exactly, so `-0.0`
is a result. An `int` element that is the least `int` traps as in
std.tensor.6. *(ADR 0508)*

**std.tensor.15** — `t[i]` (an `int` `i`) is the tensor of `t`'s rows after
the first, with that first axis dropped: a 1-D tensor's `t[i]` has shape
`[]`. `t[lo..hi]` and `t[lo..=hi]` (a missing bound is the start or the end)
are the rows of that range, with every axis kept. The index is typed as a
`tensor.Tensor[T]` in both cases, so the checker doesn't need a rank. A
row index outside `0` to the row count, or a range outside it, traps. An
index that is not an `int` is a compile-time error. `t.item()` is the one
element of a tensor with exactly one element and traps otherwise. Assigning
to an element, `t[i] = v`, is a compile-time error: a tensor is a value, so
build a new one. *(ADR 0509)*

**std.tensor.16** — `t.conv2d(k)` is the valid 2-D convolution of an `h` by
`w` `float` image `t` with an `kh` by `kw` kernel `k`, stride 1 and no
padding: the result is `(h - kh + 1)` by `(w - kw + 1)`. Each output adds
its products in the kernel's row-major order, one at a time, so the sum is
the same on every target. Both operands must be 2-D, and the kernel must
fit the image; otherwise the call traps. `conv2d` on a `Tensor[int]` is a
compile-time error. *(ADR 0521)*

**std.tensor.17** — `t.argmax(axis)` and `t.argmin(axis)` give the `int`
index of the largest or smallest element along `axis`, as a `Tensor[int]`
whose shape drops that axis; with no argument they give the index in the
flattened tensor, an `int`. The first of equal elements wins, and a NaN never
replaces the running best, as `max` does (std.tensor.11). An empty axis
traps. *(ADR 0523)*

**std.tensor.18** — `t.clamp(lo, hi)` on a `float` tensor limits each element
to `[lo, hi]`; a NaN stays NaN. Bounds that are NaN, or with `lo > hi`, trap.
On a `Tensor[int]` it is a compile-time error. *(ADR 0523)*

**std.tensor.19** — `t.round()` on a `float` tensor rounds each element half
away from zero to an `int`, giving a `Tensor[int]` of the same shape. A NaN,
an infinity or a value outside `int` traps. On a `Tensor[int]` it is a
compile-time error. *(ADR 0523)*

**std.tensor.20** — A chain of two or more elementwise operations on tensors
(`+ - * /`, unary `-`, and `relu()`, `sigmoid()`, `tanh()` with no arguments)
over names and float literals may run as one pass over the elements. The
result is the same as the unfused calls, element by element, including traps
and NaN, and the tree walker and the bytecode tier agree. A chain whose
inputs don't share one shape, or that has a non-`float` input, runs unfused.
Fusion is not visible in the program. *(ADR 0524)*

```ail rules=std.tensor.20
fn main()
    a := tensor.from([1.0, -2.0, 3.0, 0.5], [2, 2])
    b := tensor.from([4.0, 8.0, -1.0, 2.0], [2, 2])
    print((a * b + a).relu())     # tensor.from([5.0, 0.0, 0.0, 1.5], [2, 2])
    print(-(a - b) / b)           # tensor.from([0.75, 1.25, 4.0, 0.75], [2, 2])
    print((a * 2.0 - b).sigmoid().tanh().sum() > 0.0)   # true
```

```ail rules=std.tensor.1,std.tensor.2,std.tensor.3,std.tensor.4,std.tensor.5,std.tensor.7,std.tensor.8,std.tensor.9,std.tensor.10,std.tensor.11,std.tensor.12,std.tensor.13,std.tensor.14
fn main()
    a := tensor.from([1.0, 2.0, 3.0, 4.0], [2, 2])
    print(a * 2.0 - a)        # tensor.from([1.0, 2.0, 3.0, 4.0], [2, 2])
    print(a.sum(0))           # tensor.from([4.0, 6.0])
    print(a.transpose().max(1))   # tensor.from([3.0, 4.0])
    print(a.matmul(a))        # tensor.from([7.0, 10.0, 15.0, 22.0], [2, 2])
    print(a.map(|x| x * 2.0).sum())   # 20.0
    print(tensor.from([1, 2, 3]).dot(tensor.from([4, 5, 6])))   # 32
```

```ail rules=std.tensor.15,std.tensor.16,std.tensor.17,std.tensor.18,std.tensor.19
fn main()
    a := tensor.from([1.0, 2.0, 3.0, 4.0, 5.0, 6.0], [3, 2])
    print(a[1])                    # tensor.from([3.0, 4.0])
    print(a[0..2].sum())           # 10.0
    print(a + tensor.from([10.0, 20.0]))   # broadcast over the last axis
    img := tensor.from([1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0], [3, 3])
    print(img.conv2d(tensor.from([1.0, 0.0, 0.0, -1.0], [2, 2])))
    scores := tensor.from([0.1, 0.9, 0.4, 0.7, 0.2, 0.3], [2, 3])
    print(scores.argmax(1))        # tensor.from([1, 0])
    print(tensor.from([-1.5, 0.5, 2.5]).clamp(0.0, 1.0))
    print(tensor.from([2.5, -2.5, 0.49]).round())
```

## Examples

Conformance examples (ADR 0002) for the rules above. File examples run in a
scratch directory and remove what they make. `norun` examples are checked
but not run: they need a window, a network, or a clipboard the harness
doesn't have.

```ail rules=std.use.1,std.use.2
use std.path as pth
use std.env

fn main()
    print(pth.ext("notes.txt"))       # txt: `as` names the module
    print(env.args().len())           # qualified function access
```

```ail error=std.use.3
use std.io
use std.fs as io                      # error: `io` is already the name of a module
```

```ail rules=std.use.3
use std.fs
use std.path as path_mod

fn main()
    print(fs.exists("no-such-file-here"))
    print(path_mod.base("a/b.txt"))   # b.txt
```

```ail rules=std.fs.1,std.fs.2,std.fs.3,std.fs.4
use std.fs

fn main() !
    fs.write_text("fs_a.txt", "one\n")?
    fs.append_text("fs_a.txt", "two\n")?
    print(fs.read_text("fs_a.txt")?)  # one, two
    print(fs.exists("fs_a.txt"))      # true
    fs.remove("fs_a.txt")?
    print(fs.exists("fs_a.txt"))      # false
```

```ail rules=std.fs.1
use std.fs

fn main()
    text := fs.read_text(".") else err
        print("{err.kind}")          # is_dir
        ""
    print(text.len())                # 0
```

```ail rules=std.fs.5,std.fs.6,std.fs.7
use std.fs

fn main() !
    fs.make_dir("fs_b/a/b")?         # creates the missing parents
    fs.make_dir("fs_b/a/b")?         # an existing directory is not a failure
    fs.write_text("fs_b/z.txt", "")?
    fs.write_text("fs_b/a.txt", "")?
    print(fs.list_dir("fs_b")?)      # [a, a.txt, z.txt]: sorted by bytes
    fs.remove("fs_b") else err
        print(err.kind)              # not_empty: nothing removes a tree
    fs.remove("fs_b/a/b")?
    fs.remove("fs_b/a")?
    fs.remove("fs_b/a.txt")?
    fs.remove("fs_b/z.txt")?
    fs.remove("fs_b")?
```

```ail rules=std.fs.8,std.fs.9
use std.fs

fn main() !
    fs.make_dir("fs_c/sub")?
    fs.write_text("fs_c/b.ail", "")?
    fs.write_text("fs_c/sub/a.ail", "")?
    fs.write_text("fs_c/.hidden", "")?
    print(fs.walk("fs_c")?)          # the files below fs_c, dot names included
    print(fs.glob("fs_c/*.ail"))     # [fs_c/b.ail]
    print(fs.glob("fs_c/**/*.ail"))  # [fs_c/b.ail, fs_c/sub/a.ail]
    fs.remove("fs_c/sub/a.ail")?
    fs.remove("fs_c/sub")?
    fs.remove("fs_c/.hidden")?
    fs.remove("fs_c/b.ail")?
    fs.remove("fs_c")?
```

```ail rules=std.fs.10,std.fs.11,std.fs.12
use std.fs

fn main() !
    fs.write_text("fs_d.txt", "hello")?
    m := fs.metadata("fs_d.txt")?
    print(m.size)                    # 5
    print(m.is_dir)                  # false
    fs.copy("fs_d.txt", "fs_d2.txt")?
    fs.rename("fs_d2.txt", "fs_d3.txt")?
    print(fs.read_text("fs_d3.txt")?)  # hello
    fs.remove("fs_d.txt")?
    fs.remove("fs_d3.txt")?
```

```ail rules=std.env.1,std.env.2,std.env.3,std.env.4,std.env.5,std.env.6
use std.env

fn main()
    print(env.args().len())                      # 0: the harness passes no words
    print(env.get("AIL_SPEC_NOT_SET") else "unset")   # unset
    print(env.cwd().len() > 0)                   # true
    print(env.program().len() > 0)               # true
    print(env.vars().len() >= 0)                 # true
    print(env.home() else "no home")
```

```ail rules=std.path.2,std.path.4
use std.path

fn main()
    print(path.dir("examples/logscan.ail"))      # examples
    print(path.dir("/x"))                        # /
    print(path.dir("plain"))                     # .
    print(path.join("", "data"))                 # data
    print(path.join("a", "/abs"))                # /abs
```

```ail rules=std.io.1,std.io.2,std.io.3,std.io.4
use std.io

fn main()
    line := io.read_line() else "no input"
    print(line)                      # no input: the harness gives an empty standard input
    print(io.read_all().len())       # 0
    io.eprint("to standard error")
    io.exit(0)
```

```ail rules=std.proc.1,std.proc.2
use std.proc

fn main() !
    r := proc.run("sh", ["-c", "printf hi"])?
    print(r.status)                  # 0
    print(r.out)                     # hi
    s := proc.run("sh", ["-c", "exit 3"])?
    print(s.status)                  # 3: a process that fails is still a successful run
    echoed := proc.run("sh", ["-c", "cat"], "b\na\n", 2000)?
    print(echoed.out)                # the input is the standard input
    t := proc.run("sh", ["-c", "sleep 5"], none, 100) else err
        print(err.kind)              # timed_out
        return
    print(t.status)
    u := proc.run("no-such-program-for-ail", []) else err
        print(err.kind)              # not_found
        return
    print(u.status)
```

```ail rules=std.proc.3
use std.proc

fn main()
    print(proc.which("sh") != none)  # true on Linux and macOS
    print(proc.which("no-such-program-for-ail"))   # none
```

```ail norun rules=std.net.1,std.net.3,std.net.4,std.net.5
use std.net

fn main() !
    r := net.get("https://example.com/health") else err
        print("{err.kind}: {err.message}")
        return
    print(r.status)
    p := net.post("https://example.com/api", "[]", "application/json") else err
        print(err.kind)
        return
    print(p.status)
```

```ail rules=std.net.2,std.net.4
use std.net

fn main()
    r := net.get("ftp://example.com/") else err
        print(err.kind)              # bad_url: the scheme is not http or https
        return
    print(r.status)
```

```ail rules=std.time.1,std.time.2
use std.time

fn main()
    start := time.now_ms()
    time.sleep_ms(1)
    print(time.now_ms() >= start)    # true
```

```ail rules=std.random.2,std.random.5
use std.random

fn main()
    random.seed(7)
    print(random.int(3, 3))          # 3: both bounds are inclusive
    print(random.float() < 1.0)      # true
```

```ail norun rules=std.random.5
use std.random

fn main()
    print(random.int(1, 6))          # ail run --deterministic rejects this program
```

```ail rules=std.ui.9,std.ui.14,std.ui.15,std.ui.17
use std.ui

fn main()
    print(ui.RED)                    # the colors are ints
    print(ui.BLACK)
```

```ail norun rules=std.ui.3,std.ui.4,std.ui.10,std.ui.11,std.ui.12,std.ui.13,std.ui.16,std.ui.18
use std.ui

fn main()
    mut w := ui.open("Demo", 40, 5) else err
        print("no window: {err.kind}")
        return
    w.clear(0x202020)
    w.put(1, 1, "Hi", 0xFFFFFF, 0x202020)
    w.put_styled(1, 2, "bold", 0xFFFFFF, 0x202020, ui.Style(bold: true, italic: false, underline: false))
    w.set_cursor(3, 1, Block)
    w.set_title("Demo 2")
    w.set_font_size(18)
    print(w.font_size())
    ui.clipboard_set("copied")
    print(ui.clipboard_get() else "")
    w.present()
    e := w.poll_event() else Close
    match e
        Mouse(_, _, _, _) => print("click")
        Scroll(_) => print("wheel")
        _ => print("other")
    print(w.modifiers().shift)
    w.close()
```

```ail rules=std.ui.19,std.ui.20,std.ui.21,std.ui.22,std.ui.23,std.ui.24
use std.ui
use std.ui.widgets

fn main()
    theme := {"bg": 0x101010, "fg": 0xEEEEEE}
    print(widgets.color(theme, "fg"))
    print(widgets.fuzzy("config.toml", "cfg"))        # true: letters in order
    print(widgets.fit("abcdef", 3))
    list := widgets.ListView.new(["alpha", "beta", "gamma"])
    field := widgets.TextField.new("name", "ann")
    print(field.caret_col(widgets.Rect(col: 0, row: 0, cols: 20, rows: 1)))
    print(list.shown())
    print(list.chosen() else "")
    button := widgets.Button(label: "OK", focused: true)
    print(button.render(widgets.Rect(col: 0, row: 0, cols: 6, rows: 1), theme).len())
    lines := widgets.text_rows(list.render(widgets.Rect(col: 0, row: 0, cols: 12, rows: 3), theme), 12, 3)
    print(lines.len())
```

```ail rules=std.tensor.6
fn main()
    xs := tensor.from([1, 2, 3], [3])
    print(xs.sum())                 # 6: int sums add in index order
    print(xs.dot(xs))               # 14
    print(xs.add(xs).to_list())     # [2, 4, 6]
```

## Examples for the hash, compression, date, zone, net and process rules

```ail rules=std.hash.1,std.hash.2,std.hash.3,std.hash.4
use std.hash

fn main()
    print(hash.sha256("abc"))         # ba7816bf...: the FIPS 180-4 digest
    print(hash.sha384("abc").len())   # 96 hex digits
    print(hash.sha512("abc").len())   # 128 hex digits
    print(hash.hmac_sha256("key", "The quick brown fox jumps over the lazy dog").len())   # 64
    print(hash.crc32("abc"))          # 891568578
    print(hash.hash64("abc") == hash.hash64("abc"))   # true: the same on every run
```

```ail rules=std.compress.1,std.compress.2,std.compress.3,std.compress.7
use std.compress

fn main() !
    packed := compress.gzip("hello hello hello")
    print(packed.len() > 0)
    print(compress.gunzip(packed)?.to_str() else "")        # hello hello hello
    raw := compress.deflate("abc")
    print(compress.inflate(raw)?)                      # b"abc"
    z := compress.zlib("abc")
    print(z[0])                                        # 120: the zlib header 78 9c
    print(compress.unzlib(z)?)                         # b"abc"
```

```ail rules=std.compress.2
use std.compress

fn main()
    text := compress.inflate(b"\x01\x02\x03") else err
        print(err.kind)               # corrupt: malformed data
        b""
    print(text.len())
```

```ail norun rules=std.compress.4,std.compress.5,std.compress.6
use std.compress

fn main() !
    compress.gzip_file("notes.txt", "notes.txt.gz")?
    compress.gunzip_file("notes.txt.gz", "notes2.txt")?
    compress.zip_dir("site", "site.zip")?
    compress.unzip("site.zip", "unpacked")?
    print("done")                     # the system gzip and unzip read this output
```

```ail norun rules=std.net.6,std.net.7
use std.net

fn main() !
    mut o := net.options()
    o.headers["accept"] = "text/plain"
    o.max_redirects = 0               # a redirect is returned, not followed
    r := net.request("GET", "https://example.com/", o)?
    print(r.status)
```

```ail norun rules=std.net.8,std.net.9,std.net.10,std.net.11
use std.net

fn main() !
    srv := net.serve("127.0.0.1:0")?
    print(srv.addr())
    ex := srv.accept()?
    ex.respond(net.Response(status: 200, headers: {}, body: "ok"))?
```

```ail norun rules=std.net.12,std.net.13,std.net.14,std.net.15
use std.net

fn main() !
    l := net.listen("127.0.0.1:0")?
    c := net.connect("127.0.0.1", 80)?
    t := net.connect_tls("example.com", 443)?
    print(l.addr())
```

```ail norun rules=std.proc.4,std.proc.5,std.proc.6,std.proc.7
use std.proc

fn main() !
    c := proc.start("sh", ["-c", "cat"])?
    c.stdin.send("hello")
    c.stdin.close()
    for line in c.out
        print(line)
    status := proc.wait(move c)?
    print(status)
    d := proc.start("sh", ["-c", "sleep 60"])?
    print(proc.kill(move d))          # 137 on Unix
```

```ail norun rules=std.proc.5
use std.proc

fn main() !
    c := proc.start("sh", ["-c", "echo err 1>&2"])?
    msg := c.err.recv()               # `none` once the stream ends
    print(msg)
    status := proc.wait(move c)?
    print(status)
```

```ail rules=std.date.1,std.date.2,std.date.3,std.date.4,std.date.5
use std.time.date

fn main()
    d := date.Date(year: 2024, month: 2, day: 29)
    print(d.iso())                                   # 2024-02-29
    print(d.days())                                  # 19782
    print(d.weekday())                               # 4: a Thursday
    print(d.yday())                                  # 60
    print(date.Date.from_days(0).iso())              # 1970-01-01
    print(date.Date.new(2023, 2, 29) == none)        # true: no such day
    print(d.add_days(365).iso())                     # 2025-02-28
    print(d.since(date.Date(year: 2024, month: 1, day: 1)))   # 59
    print(date.Date(year: 2024, month: 1, day: 31).add_months(1).iso())   # 2024-02-29
    print(d.add_years(1).iso())                      # 2025-02-28
    print(date.is_leap(1900))                        # false
    print(date.days_in_month(2023, 2))               # 28
```

```ail rules=std.date.6,std.date.7,std.date.8,std.date.9,std.date.10,std.tz.1,std.tz.2
use std.time.date
use std.tz

fn main() !
    a := date.utc(1700000000000)
    print(a.iso())                                   # 2023-11-14T22:13:20Z
    b := date.at(1700000000000, "Europe/Paris")?
    print(b.iso())
    print(a.unix_ms() == b.unix_ms())                # true: the same instant
    print(a == b)                                    # false: the fields differ
    print(a.before(b))                               # false
    print(a.in_zone("UTC")?.hour)
    print(a.format("%Y-%m-%d %H:%M:%S %a %b"))
    print(tz.offset("Europe/Paris", 1700000000) else 0)
    print(tz.system().len() > 0)
```

```ail rules=std.date.7,std.date.15
use std.time.date

fn main()
    z := date.at(0, "No/Such_Zone") else err
        print(err)                                   # DateError: the zone is unknown
        return
    print(z.iso())
```

```ail norun rules=std.date.11,std.date.12
use std.time.date

fn main()
    now := date.now()                                # reads the clock: --deterministic rejects it
    later := now.add_ms(1000)
    print(later.before(now))
```

```ail rules=std.date.13,std.date.14,std.date.15
use std.time.date

fn main() !
    d := date.parse_date("2024-03-10")?
    print(d.iso())                                   # 2024-03-10
    t := date.parse_iso("2024-03-10T08:30:00.123+02:00")?
    print(t.iso())                                   # 2024-03-10T08:30:00.123+02:00
    local := date.parse_in("2024-03-10 08:30", "Europe/Paris")?
    print(local.iso())
    bad := date.parse_iso("2024-03-10T08:30") else err
        print(err)                                   # no offset: an error
        return
    print(bad.iso())
```

```ail rules=std.tz.3
use std.tz

fn main()
    print(tz.system().len() > 0)                     # "UTC" on Windows until the registry reader lands
    print(tz.offset("UTC", 0) else 0)                # 0
```

## Examples for bytes I/O

```ail rules=std.fs.13,std.fs.14
use std.fs

fn main() !
    fs.write_bytes("spec-bytes.bin", b"\x00\xffhi")?
    print(fs.read_bytes("spec-bytes.bin")?)
    fs.append_bytes("spec-bytes.bin", b"!")?
    print(fs.read_bytes("spec-bytes.bin")?)
```

```ail norun rules=std.io.5
fn main()
    data := io.read_bytes()                 # reads the rest of standard input, undecoded
    print(data.len())
```
