# 17. Standard modules

Chapter key: `std`

> **Status:** Specified. Every standard module below has numbered rules and
> a runnable example. Modules with a chapter of their own are linked in the
> index under Imports.

The standard modules reach the machine through small, typed functions whose
failures are values (ADR 0157). There is no string-typed shell.

## Imports

**std.use.1** — `use std.NAME` imports the standard module `NAME`, available
under the name `NAME`, or under `d` with `use std.NAME as d` (`mod.use.1`).
The modules built into the compiler are `compress`, `crypto`, `env`, `fs`,
`hash`, `hw`, `io`, `mem`, `net`, `os`, `path`, `proc`, `random`, `regex`,
`task`, `tensor`, `time`, `tz` and `ui`. The modules written in AIL are
`std.app`, `std.args`, `std.attach`, `std.audio`, `std.bigint`, `std.csv`,
`std.file`, `std.gfx`, `std.kv`, `std.log`, `std.mcp`, `std.net.http`,
`std.screen`, `std.term`, `std.test`, `std.time.date`, `std.toml`,
`std.yaml` and the submodules in the index below; they are imported by their
whole path (`use std.time.date`) and used under the last name (`date`).
`use std` itself, or any `std.NAME` that is not in the index, is a
compile-time error that lists the modules. *(ADR 0157)*

### Index of the standard modules

One line per module: what it is for, and the rules that define it. `ail ask
std WORDS` finds a function by what it does, and `ail card std.NAME` prints
its signatures.

| Module | For | Rules |
|---|---|---|
| `fs`, `file`, `kv` | files, streaming files, a key-value file | `std.fs`, `std.file`, `std.kv` |
| `env`, `path`, `os`, `proc` | environment, paths, signals, processes | `std.env`, `std.path`, `std.os`, `std.proc` |
| `hash`, `crypto`, `compress`, `bigint` | digests, keys, zip and gzip, big integers | `std.hash`, `std.crypto`, `std.compress`, `std.bigint` |
| `io`, `log`, `args`, `term`, `test` | standard streams, logging, flags, terminal, test fixtures | `std.io`, `std.log`, `std.args`, `std.term`, `test.stub` ([19](19-testing.md)) |
| `net`, `net.http`, `mcp` | TCP, TLS, HTTP, tool servers | `std.net`, [std-net-http.md](std-net-http.md), `std.mcp` |
| `time`, `time.date`, `tz`, `random` | clock, calendar, zones, randomness | `std.time`, `std.date`, `std.tz`, `std.random` |
| `csv`, `toml`, `yaml`, `regex`, `Json` | text formats and patterns | `std.csv`, `std.toml`, `std.yaml`, [std-regex.md](std-regex.md), [std-json.md](std-json.md) |
| `tensor`, `mem`, `hw` | arrays of numbers, memory for C, fences | `std.tensor`, `std.mem`, [13](13-unsafe.md), [14](14-ffi.md) |
| `ui`, `app`, `gfx`, `screen`, `audio`, `attach` | windows, apps, graphics, sound, agent control | `std.ui`, [std-app.md](std-app.md), [std-gfx.md](std-gfx.md), [std-screen.md](std-screen.md), [std-audio.md](std-audio.md), [std-attach.md](std-attach.md) |
| `webm`, `vp9`, `opus` | media formats | [std-webm.md](std-webm.md), [std-vp9.md](std-vp9.md), [std-opus.md](std-opus.md) |
| `Heap`, `Deque`, `Grid` | collections in the prelude, no `use` | `type.heap`, `type.deque` ([04](04-types.md)) |

**std.use.2** — A standard module's functions and types are referred to by
qualified name, `fs.read_text(path)` and `fs.IoError` (`mod.use.2`). A
module is not a value, and naming a function or type the
module doesn't have is a compile-time error. *(ADR 0157)*

**std.use.3** — Two imports in one file MUST NOT make modules available under
the same name. A local may reuse the name of a module imported with `use`; it
hides the module until its block ends (`expr.bind.5`). *(ADR 0157, ADR 1121)*

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
fields `path str` (the path the function was given) and `kind fs.IoKind`,
the enum `NotFound`, `Permission`, `Exists`, `IsDir`, `NotDir`, `NotEmpty`,
`InvalidUtf8`, `Other`. A failure to start a process is the error struct
`proc.ProcError`, with fields `cmd str` and `kind proc.ProcKind`: the
variants of `fs.IoKind` in the same order, then `TimedOut` (a process killed
at its time limit, `std.proc.2`) and `Cancelled` (`proc.run` in a cancelled
task, `conc.cancel.2`). A kind is the same for the same failure on every
platform, and is told apart with `==` or an exhaustive `match`
(`expr.match.2`). Every kind of the standard library is an enum
(`type.enum.7`). `std.net` fails with `net.NetError` (`std.net.4`).
*(ADR 0158, ADR 0282, ADR 0321, ADR 1068)*

```ail rules=std.err.1
use std.fs

fn load(path str) str ! fs.IoError
    fs.read_text(path)? "loading {path}"

fn main()
    text := fs.read_text("notes.txt") else err
        match err.kind
            NotFound => print("no {err.path} yet")
            Permission => print("{err.path} is locked")
            Exists | IsDir | NotDir | NotEmpty | InvalidUtf8 | Other => print("{err.kind}")
        ""
    print(text)
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
to `to`, replacing a file there. When `from` is a directory, `copy` copies
the tree: `to` and its parents are made when missing, files already there are
replaced, other files there are kept, and entries are copied in name order.
A symbolic link is copied as the file it points to; a link to a directory
fails with `other` at that path, and so does copying a directory into itself.
For `rename` and `copy`, a failure's
`path` is `from` when `from` is missing, else the path that failed.
*(ADR 0281, ADR 0751)*

**std.fs.13** — `read_bytes(path str) bytes ! IoError` reads the whole file
as `bytes`, whatever it holds. It fails as `read_text` does, and never because
of the contents. *(ADR 0415)*

**std.fs.14** — `write_bytes(path str, data bytes) ! IoError` writes `data`
to the file, creating it or replacing its contents; `append_bytes(path str,
data bytes) ! IoError` adds `data` at the end, creating the file if needed.
They fail as `write_text` and `append_text` do. *(ADR 0415)*

**std.fs.15** — A path that doesn't exist fails with kind `not_found`.
This is so for `read_text`, `list_dir` and `remove`, and for every other call
on a missing path. *(ADR 0158)*

**std.fs.16** — Reading a directory as a file, such as `read_text(".")`, fails
with kind `is_dir`. *(ADR 0158)*

**std.fs.17** — `list_dir` of a file, not a directory, fails with kind
`not_dir`. *(ADR 0158)*

**std.fs.18** — `remove` of a directory that still holds entries fails with
kind `not_empty`; remove the entries first. *(ADR 0158)*

**std.fs.19** — `read_text` of a file whose bytes aren't UTF-8 fails with kind
`invalid_utf8`; `read_bytes` reads the same file as it is. *(ADR 0158)*

**std.fs.20** — `exists` is the kind when a path names an existing file where
the call needs a new directory: `make_dir` of a path that is a file fails with
`exists`. (`make_dir` of an existing directory succeeds.) *(ADR 0783)*

**std.fs.21** — `permission` is the kind when the OS refuses the access: a read,
write or remove the process has no right to. It is the same on every platform.
*(ADR 0783)*

**std.fs.22** — `other` is the kind for an OS failure that has none of the kinds
above. A program that reads `other` gets the path and the kind, not the OS text.
*(ADR 0783)*

**std.fs.23** — `scan(root str, same_device bool) Scan ! IoError` walks the
tree below `root` once and returns `struct Scan { entries List[Entry], skipped
List[str] }`, with `struct Entry { path str, size int, alloc int, modified_ms
int, inode int, device int, links int, is_dir bool, is_link bool }`. It lists
every file, directory and link below `root` (not `root` itself) in pre-order,
each directory's names sorted by bytes, each `path` being `root` joined with
the path below it (`path.join`). Links are not followed: an entry that is a
link has `is_link` true, and `size` is that of the link. `size` is the length
in bytes (`0` for a directory), `alloc` the bytes allocated on disk for the
entry itself, which is what deleting it frees (a sparse, compressed or cloned
file may differ from `size`), `inode` and `device` identify the file so a hard
link seen twice can be counted once, and `links` is its hard-link count.
`modified_ms` is as in `std.fs.10`. With `same_device` true, a directory on a
different device than `root` is listed but not entered. A directory that can't
be read is listed, entered no further, and its path added to `skipped`; only a
`root` that can't be read fails. The result does not depend on how the walk is
scheduled, so an implementation may read subtrees in parallel. *(ADR 0927)*

**std.fs.24** — `volume(path str) Volume ! IoError` returns `struct Volume {
total int, free int }`: the size in bytes of the volume holding `path` and the
bytes a program without privilege can still write to it. *(ADR 0928)*

**std.fs.25** — `read_range(path str, offset int, len int) bytes ! IoError`
reads up to `len` bytes starting at byte `offset`. It returns fewer at the end
of the file and an empty `bytes` when `offset` is at or past it. A negative
`offset` or `len` traps. It fails as `read_bytes` does (`std.fs.15`,
`std.fs.16`). *(ADR 0929)*

**std.fs.26** — `trash(path str) ! IoError` moves a file or a whole directory
to the user's trash, so the user can put it back: `~/.Trash` (or the volume's
`.Trashes`) on macOS, the freedesktop trash with its `.trashinfo` on Linux, the
Recycle Bin on Windows. On Linux the `.trashinfo` records the deletion time in UTC. A name already in the trash gets a unique suffix. It
never copies or deletes: when no trash on the volume can be written, it fails
with kind `other`. *(ADR 0929)*

**std.fs.27** — `scan_count() ScanCount` returns `struct ScanCount { entries
int, bytes int }`: how many entries every `fs.scan` (std.fs.23) in the program
has listed so far, and the sum of their `alloc`. Both only grow, a directory's
entries at a time, from `0` before the first scan; another task may read them
while a scan runs, to show its progress. The two counts are read one after the
other, not as one snapshot, so `bytes` may already include entries that
`entries` does not yet. *(ADR 0992)*

**std.fs.36** — `map(path str) bytes ! IoError` returns the whole file as a
read-only `bytes` view onto the file's pages, with no copy and no read up
front: the system pages in only what the program touches, so a file larger
than memory can be searched or indexed. The view reads exactly like
`read_bytes` (`len`, `b[i]`, slices, `find`, `==`) and fails like it, with kind
`not_found`, `permission` or `is_dir`. The mapping is private and lasts to the
end of the process; the collector never scans or frees it. A file changed by
another process while mapped may show the change, and one truncated while
mapped may stop the program on access (like C `mmap`), so map files that
other programs are not writing. On Windows the system refuses to truncate or
delete a file while it is mapped. An empty file gives an empty `bytes`. If the
system refuses the mapping, the result is an ordinary copy with the same
contents. *(ADR 1016)*

```ail norun rules=std.fs.36
use std.fs

fn main() !
    log := fs.map("big.log")?                 # no read yet
    print(log.find(b"ERROR"))                 # pages in what the search touches
```

**std.fs.35** — `temp_dir() str` returns the directory for temporary files: the
value of `TMPDIR` when it is set and not empty, else `TEMP` when it is set and
not empty, else `/tmp`. It does not check that the directory exists and never
fails. *(ADR 1059)*

**std.fs.37** — `make_temp_dir(prefix str) str ! IoError` creates a new empty
directory in `temp_dir()` and returns its path. Its name is `prefix` followed by
12 random lowercase hexadecimal digits. The directory is created exclusively:
a name already taken is never reused, and the call tries new digits until one is
free, so two callers never share a directory. Only the owner can read it where
the system has modes (`0700`). It fails as `make_dir` does when `temp_dir()`
can't be written (kind `permission`, `not_found`, ...), with the path it tried.
The caller removes the directory: there is no call that removes a tree, as
`remove` (std.fs.6) removes an empty directory only. *(ADR 1059)*

```ail norun rules=std.fs.35,std.fs.37
use std.fs

fn main() !
    dir := fs.make_temp_dir("ail-spec-")?
    other := fs.make_temp_dir("ail-spec-")?
    print(dir != other)                       # true: never shared
    print(dir.starts_with(fs.temp_dir()))     # true
    fs.write_text("{dir}/a.txt", "x")?
    print(fs.list_dir(dir)?)                  # [a.txt]
    fs.remove("{dir}/a.txt")?
    fs.remove(dir)?
    fs.remove(other)?
```

```ail norun rules=std.fs.23,std.fs.24,std.fs.25,std.fs.26,std.fs.27
use std.fs

fn main() !
    s := fs.scan("/Users/me/Library/Caches", true)?
    seen := fs.scan_count()                  # from another task, for a progress bar
    mut total := 0
    for e in s.entries
        if not e.is_dir and not e.is_link
            total += e.alloc
    v := fs.volume("/")?
    print("{total} of {v.total} bytes, {v.free} free, {s.skipped.len()} skipped, {seen.bytes} seen")
    head := fs.read_range("a.bin", 0, 4096)?
    fs.trash("old.bin")?
```

**std.fs.28** — `mode(path str) int ! IoError` returns the permission bits of
the file (0 to 4095, the low twelve bits of `st_mode`), links followed. On
Windows a read-only file is 292 (`0o444`) and any other file 438 (`0o666`).
*(ADR 0977)*

**std.fs.29** — `set_mode(path str, mode int) ! IoError` sets the permission
bits; a `mode` outside 0 to 4095 traps. On Windows only the owner's write bit
(128) acts: without it the file becomes read-only, with it that attribute is
cleared. *(ADR 0977)*

**std.fs.30** — `set_modified(path str, ms int) ! IoError` sets the
modification time to `ms` milliseconds since 1970-01-01T00:00:00Z, so that
`metadata(path).modified_ms` reads it back; the access time is unchanged.
*(ADR 0977)*

**std.fs.31** — `read_link(path str) str ! IoError` returns the text a symbolic
link holds, as it was written and not resolved. A path that is not a link
fails with kind `other`. On Windows the text has `\` where `/` was written.
*(ADR 0977)*

**std.fs.32** — `symlink(target str, path str) ! IoError` makes a symbolic link
at `path` holding `target` as written; `path` existing fails with kind
`exists`. On Windows it needs the privilege or developer mode and otherwise
fails with kind `permission`. *(ADR 0977)*

**std.fs.33** — `truncate(path str, len int) ! IoError` makes the file end at
byte `len`: bytes past it are dropped and a shorter file grows with zero
bytes. A negative `len` traps. *(ADR 0977)*

**std.fs.34** — `write_at(path str, offset int, data bytes) ! IoError` writes
`data` over the bytes of the file from `offset`, making the file when it is
missing and growing it with zero bytes when it is shorter than `offset`; the
rest of the file is unchanged. A negative `offset` traps. *(ADR 0977)*

**std.fs.38** — `sync(path str) ! IoError` asks the operating system to put the
file's data on the disk, so it survives a power failure: `fsync` on Linux,
`fcntl(F_FULLFSYNC)` on macOS (plain `fsync` there stops at the drive's cache),
`FlushFileBuffers` on Windows. The file must exist and be writable, and is not
changed. It does not sync the directory that holds the file, so a file made a
moment before may still be missing after a power failure. It needs `fs` or
`fs.write` ([18-effects](18-effects.md)). *(ADR 1098)*

```ail norun rules=std.fs.28,std.fs.29,std.fs.30,std.fs.31,std.fs.32,std.fs.33,std.fs.34,std.fs.38
use std.fs

fn main() !
    m := fs.mode("tool.sh")?
    fs.set_mode("tool.sh", m.bit_or(73))?
    fs.set_modified("tool.sh", 1700000000000)?
    fs.symlink("tool.sh", "run")?
    print(fs.read_link("run")?)
    fs.truncate("archive.bin", 1048576)?
    fs.write_at("out.bin", 4096, b"tail")?
    fs.sync("out.bin")?
```

```ail rules=std.fs.20
use std.fs

fn main() !
    fs.write_text("fs_exists.txt", "x")?
    fs.make_dir("fs_exists.txt") else err
        print("{err.kind}")                  # exists
    fs.remove("fs_exists.txt")?
```

```ail norun rules=std.fs.21
use std.fs

fn main() !
    text := fs.read_text("/root/no-access.txt") else err
        print("{err.kind}")                  # permission, for a user without access
        ""
    print(text.len())
```

```ail norun rules=std.fs.22
use std.fs

fn main() !
    text := fs.read_text("fs_other.txt") else err
        print("{err.kind}")                  # other, for an OS failure outside the list
        ""
    print(text.len())
```

```ail rules=std.fs.15
use std.fs

fn main() !
    text := fs.read_text("fs_no_such_file.txt") else err
        print("{err.kind}")                  # not_found
        ""
    print(text.len())
```

```ail rules=std.fs.16
use std.fs

fn main() !
    text := fs.read_text(".") else err
        print("{err.kind}")                  # is_dir
        ""
    print(text.len())
```

```ail rules=std.fs.17
use std.fs

fn main() !
    fs.write_text("fs_plain.txt", "x")?
    names := fs.list_dir("fs_plain.txt") else err
        print("{err.kind}")                  # not_dir
        []
    print(names.len())
    fs.remove("fs_plain.txt")?
```

```ail rules=std.fs.18
use std.fs

fn main() !
    fs.make_dir("fs_full")?
    fs.write_text("fs_full/a.txt", "a")?
    fs.remove("fs_full") else err
        print("{err.kind}")                  # not_empty
    fs.remove("fs_full/a.txt")?
    fs.remove("fs_full")?
```

```ail rules=std.fs.19
use std.fs

fn main() !
    fs.write_bytes("fs_bytes.bin", b"\xff\xfe")?
    text := fs.read_text("fs_bytes.bin") else err
        print("{err.kind}")                  # invalid_utf8
        ""
    print(text.len())
    print(fs.read_bytes("fs_bytes.bin")?)
    fs.remove("fs_bytes.bin")?
```
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

```ail rules=std.env.1
use std.env

fn main()
    print(env.args().len())
```

**std.env.2** — `get(name str) str?` returns an environment variable, or
`none` when it isn't set or isn't UTF-8. *(ADR 0157)*

```ail rules=std.env.2
use std.env

fn main()
    home := env.get("AIL_NO_SUCH_VAR_X") else "unset"
    print(home)
```

**std.env.3** — `cwd() str` returns the working directory. If it can't be
read, the program traps. *(ADR 0157)*

```ail rules=std.env.3
use std.env

fn main()
    print(env.cwd().len() > 0)
```

**std.env.4** — `program() str` returns the program's file path as it was
given to `ail run`. *(ADR 0159)*

```ail rules=std.env.4
use std.env

fn main()
    print(env.program().len() > 0)
```

**std.env.5** — `vars() Map[str, str]` returns the environment variables
whose names and values are UTF-8, in order of their names' bytes. On Windows, where names ignore case, they are ASCII capitals.
*(ADR 0282)*

```ail rules=std.env.5
use std.env

fn main()
    print(env.vars().len() > 0)
```

**std.env.6** — `home() str?` returns the user's home directory: `HOME`,
else `USERPROFILE` (Windows), or `none` when neither is set or one is
empty. *(ADR 0296)*

```ail rules=std.env.6
use std.env

fn main()
    h := env.home() else "none"
    print(h.len() > 0)
```

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

**std.hash.5** — `hmac_sha384(key str, s str) str` and
`hmac_sha512(key str, s str) str` return the HMAC of RFC 2104 and RFC 4231
(128-byte blocks) as lower-case hex, 96 and 128 digits. A key longer than the
block is hashed first. They have no effects. *(ADR 0959)*

**std.hash.6** — `sha3_256(s str) str` and `sha3_512(s str) str` (FIPS 202)
return the digest as lower-case hex (64 and 128 digits). `sha1_legacy(s str) str`
(FIPS 180-4) and `md5_legacy(s str) str` (RFC 1321) return 40 and 32 digits.
**SHA-1 and MD5 are broken against collisions and are not for security**: never
for signatures, certificates, password storage or integrity against an attacker.
They exist for file formats and protocols that require them (git objects, ETags,
legacy checksums), and the `_legacy` suffix is the only way to name them. They
have no effects. *(ADR 0959)*

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
`CompressError { kind compress.CompressKind, message str }`; the kind is
`Corrupt` (malformed data, including trailing bytes after a DEFLATE stream),
`Checksum` (gzip CRC-32 or size mismatch), `TooBig` (std.compress.4),
`UnsafePath`, `Unsupported`, or, for file functions, an I/O kind of
`std.err.1` (`NotFound` to `InvalidUtf8`, and `Other`). *(ADR 0443,
ADR 1068)*

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

**std.compress.8** — `gunzip` of bytes without the gzip magic `1f 8b` fails
with kind `corrupt` and message `not gzip data`; other malformed input fails
with `corrupt` and a message that names the defect. *(ADR 0443)*

```ail rules=std.compress.8
use std.compress

fn main()
    data := compress.gunzip(b"not gzip at all") else err
        print("{err.kind}: {err.message}")   # corrupt: not gzip data
        b""
    print(data.len())
```
**std.compress.9** — DEFLATE failures: `inflate` (and the DEFLATE part of
`gunzip`, `unzlib` and `unzip`) fails with kind `corrupt` and exactly one of
these messages: `unexpected end of compressed data`, `invalid block type`,
`stored block length check failed`, `too many length or distance codes`,
`over-subscribed Huffman code`, `invalid Huffman code`, `repeat with no
previous length`, `code length repeat overruns the table`, `block has no
end-of-block code`, `invalid length symbol`, `invalid distance symbol`,
`distance reaches before the start of the output`, or `extra bytes after the
end of the compressed data`. *(ADR 0784)*

```ail rules=std.compress.9
use std.compress

fn main()
    a := compress.inflate(b"\x07") else err
        print("{err.kind}: {err.message}")   # corrupt: invalid block type
        b""
    b := compress.inflate(b"\x01") else err
        print("{err.kind}: {err.message}")   # corrupt: unexpected end of compressed data
        b""
    print(a.len() + b.len())
```

**std.compress.10** — Wrapper failures: besides `std.compress.8`, `gunzip`
fails with `corrupt` and `unknown gzip compression method`, `reserved gzip
flags set`, `truncated gzip header` or `truncated gzip trailer`, and with
`checksum` and `gzip CRC-32 mismatch` or `gzip size mismatch`. `unzlib`
fails with `corrupt` and `not zlib data` or `truncated zlib trailer`, with
`checksum` and `zlib Adler-32 mismatch`, and with `unsupported` and `zlib
preset dictionaries are not supported`. *(ADR 0784)*

```ail rules=std.compress.10
use std.compress

fn main()
    z := compress.unzlib(b"x\x9c\x03\x00\x00\x00\x00\x02") else err
        print("{err.kind}: {err.message}")   # checksum: zlib Adler-32 mismatch
        b""
    print(z.len())
```

**std.compress.11** — Zip failures: `unzip` fails with `corrupt` and `not a
zip archive`, `truncated zip data`, `bad zip central directory`, `bad zip
local header` or `zip name is not UTF-8`; with `unsafe_path` and `unsafe
name in archive: NAME` (`std.compress.5`); with `unsupported` and
`encrypted zip entries are not supported` or `zip method N is not
supported`; with `checksum` and `zip CRC-32 or size mismatch in NAME`; and
with `too_big` and `output exceeds the size limit`, `archive expands past
the size limit`, `more than 65535 files; zip64 is not supported` or `over 4
GiB; zip64 is not supported` (`std.compress.4`). *(ADR 0784)*

```ail norun rules=std.compress.11
use std.compress

fn main()
    compress.unzip("notes.txt", "out") else err
        print("{err.kind}: {err.message}")   # corrupt: not a zip archive
```

**std.compress.6** — Output is read by the system `gzip` and `unzip` and
their output is read back; the project's tests check this when the tools
exist and say so when they don't. *(ADR 0445)*

```ail
use std.compress

fn main() !
    z := compress.gzip("abc abc abc abc")
    print(compress.gunzip(z)?.to_str() else "binary")
```

## Cryptography: `std.crypto`

**std.crypto.1** — `ct_eq(a bytes, b bytes) bool` is true when the two byte
strings are equal. Its running time depends on the lengths only, never on where
the bytes differ, so a program compares MACs, tags and password hashes with it
and never with `==`. It has no effects. *(ADR 0959)*

**std.crypto.2** — `random_bytes(n int) bytes` returns `n` bytes (0 to
1073741824) from the operating system's generator (`/dev/urandom`, or
ProcessPrng on Windows). It never uses the `std.random` generator, so `seed`
and `--seed` do not affect it. When the system has no source it stops the
program with a trap, never falling back to a weaker one. An `n` outside the
range traps. It has the effect `random`. *(ADR 0959)*

**std.crypto.3** — `hkdf_sha256(ikm bytes, salt bytes, info bytes, len int)
bytes` is HKDF (RFC 5869) with HMAC-SHA-256: `len` bytes, 0 to 8160. An empty
`salt` stands for 32 zero bytes. A `len` outside the range traps. It has no
effects. *(ADR 0959)*

**std.crypto.4** — `pbkdf2_sha256(password bytes, salt bytes, iterations int,
len int) bytes` is PBKDF2 (RFC 8018) with HMAC-SHA-256. `iterations` is at
least 1 and `len` is 0 to 1048576, or it traps. It has no effects, and a call
with a large `iterations` is slow by design. *(ADR 0959)*

```ail rules=std.crypto.1,std.crypto.2,std.crypto.3,std.crypto.4
use std.crypto

fn main()
    print(crypto.ct_eq(b"abc", b"abc"))                         # true
    print(crypto.random_bytes(16).len())                        # 16
    print(crypto.hkdf_sha256(b"key", b"salt", b"info", 8).len())   # 8
    print(crypto.pbkdf2_sha256(b"password", b"salt", 1, 32).hex())
    # 120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b
```

## Standard input and output: `std.io`

**std.io.1** — `read_line() str?` reads one line from standard input,
without its line ending (`\n` or `\r\n`), or returns `none` at the end of
input. Bytes that aren't UTF-8 become U+FFFD. *(ADR 0157)*

```ail rules=std.io.1
use std.io

fn main()
    line := io.read_line() else "eof"
    print(line)
```

**std.io.2** — `read_all() str` reads the rest of standard input.
*(ADR 0157)*

```ail rules=std.io.2
use std.io

fn main()
    print(io.read_all().len())
```

**std.io.3** — `eprint(value)` writes the value's text form and a newline to
standard error, as `print` does to standard output (`mod.main.3`). Output
already printed reaches standard output first. *(ADR 0157)*

```ail rules=std.io.3
use std.io

fn main()
    io.eprint("note")
    print("done")
```

**std.io.4** — `exit(code int)` ends the program with exit status `code`,
after writing any pending standard output. It never returns, so the code
after it is unreachable (ADR 0139). A status outside 0 to 255 traps.
*(ADR 0157)*

```ail norun rules=std.io.4
use std.io

fn main()
    print("bye")
    io.exit(3)
```

**std.io.5** — `read_bytes() bytes` reads the rest of standard input as
`bytes`, without decoding it. *(ADR 0415)*

```ail rules=std.io.5
use std.io

fn main()
    print(io.read_bytes().len())
```

**std.io.6** — `write_bytes(data bytes)` writes the bytes of `data` to
standard output, undecoded and with no newline added, and on Windows too with no `\n` to
`\r\n` translation. Output from `print` and
`write_bytes` reaches standard output in the order the calls ran; `exit`
flushes it (`std.io.4`). `io.read_bytes` (`std.io.5`) is the matching read, so
a filter reads all of standard input as bytes and writes bytes back. *(ADR 1007)*

```ail rules=std.io.6
use std.io

fn main()
    io.write_bytes(b"he")
    io.write_bytes(b"llo\n")
    print("mid")
```

**std.io.7** — `read_exact(n int) bytes?` reads exactly `n` bytes of standard
input, undecoded, through the buffer `read_line`, `read_all` and `read_bytes`
share, so the calls mix in order. It returns `none` when the input ends before
`n` bytes; the bytes read are consumed. `n` of `0` returns empty bytes. A
negative `n`, or one above 1073741824, traps. A protocol that frames messages
by length (`Content-Length: N`) reads each body with it. *(ADR 1071)*

```ail rules=std.io.7
use std.io

fn main()
    body := io.read_exact(4) else b""
    print(body.len())
```

## Processes: `std.proc`

**std.proc.1** — `run(cmd str, args List[str], input str?, timeout_ms int?) Output ! ProcError`
runs the
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

**std.proc.9** — `Output` (the result of `proc.run`) has the methods `text()`
and `ok()`, and no others. Each takes no arguments, and neither changes the
value: a call with an argument, or an unknown name, is an error. *(ADR 0733)*

**std.proc.10** — `o.text() str` is the standard output followed by the standard
error, with nothing between them. `o.ok() bool` is `o.status == 0`. *(ADR 0733)*

```ail rules=std.proc.9,std.proc.10
use std.proc

fn main() !
    r := proc.run("echo", ["hello"])?
    print(r.ok())                         # true
    print(r.text().len() > 0)             # true
```

## Signals: `std.os`

**std.os.1** — `signals(names List[str]) Channel[str]` returns a channel that
receives the name of each listed signal as the operating system delivers it to
the program. The names are `"HUP"`, `"INT"`, `"QUIT"`, `"TERM"` and `"WINCH"`;
the value is the same name. Receiving uses the ordinary channel methods
(`recv`, `try_recv`, `recv_for`, `task.select`) and, like a child's output
(std.proc.5), needs no task: a receiver that finds the channel empty waits for
a signal. Signals of one kind that arrive before a receiver looks are delivered as one
value. Every
channel that names a signal gets its own copy. While a channel is open, the
named signals no longer end the program. Needs the `proc` capability.
*(ADR 1011)*

**std.os.2** — `signals` traps when the list is empty or names a signal other
than those of std.os.1. *(ADR 1011)*

**std.os.3** — `ch.close()` on a signal channel ends it: queued names can
still be received, then `recv` gives `none`. When the last open channel naming
a signal is closed, that signal's default action returns. `ch.send(v)` on a
signal channel has no effect. `signals` traps if the system refuses the
handler. *(ADR 1011)*

**std.os.4** — On Windows, `"INT"` is Ctrl-C and Ctrl-Break, and `"TERM"` is
the console closing, the user logging off and the system shutting down (Windows
ends the program a few seconds later whatever the program does). `"HUP"`,
`"QUIT"` and `"WINCH"` are accepted and never delivered there. *(ADR 1011)*

```ail norun rules=std.os.1,std.os.2,std.os.3,std.os.4
use std.os

fn main()
    stop := os.signals(["INT", "TERM"])
    name := stop.recv() else return
    print("{name}: shutting down")
    stop.close()
```

## Counting: `Map[K, int]`

**std.count.1** — `m[k] += 1` adds one to the count at `k`, and starts the count
at zero when `k` is absent. The receiver must be `mut`, and the map's values must
be `int`. *(ADR 0732)*

**std.count.2** — `m.top(n int) List[K]` is the `n` keys with the largest counts,
largest first. Keys with equal counts keep their insertion order. A count of zero
or less gives `[]`. `m` is not changed. *(ADR 0732)*

```ail rules=std.count.1,std.count.2
fn main()
    mut counts := Map[str, int]()
    for word in ["b", "a", "b", "c", "b", "a"]
        counts[word] += 1
    print(counts.top(2))                  # ["b", "a"]
    print(counts.top(0))                  # []
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

**std.net.4** — A failure is `error struct NetError { url str,
kind net.NetKind, message str }`: the URL given, a kind, and a sentence for
people. `net.NetKind` is `BadUrl` (not `http://` or
`https://HOST[:PORT][/PATH]`, or not an address `HOST:PORT`), `BadRequest`
(a method, header name or header value the request or response may not
carry, `std.net.6`), `Tls` (the TLS handshake or connection failed,
`std.net.5`), `NotFound` (the host has no address), `Refused`, `TimedOut`,
`Closed` (the connection or server was closed, or the waiting task was
cancelled, `std.net.9`), `Bind` (the address is in use or not permitted,
`std.net.9`), `Protocol` (a malformed response, or too many redirects) or
`Other`. *(ADR 0283, ADR 0369, ADR 0418, ADR 0419, ADR 1068)*

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
query Map[str, str], body str, timeout_ms int, max_redirects int, data bytes,
connect_ms int, total_ms int }`.
`options()` returns the defaults: no headers, no query, an empty body, a
30000 ms timeout and 5 redirects; write `mut o := net.options()` and set
fields. The method is made uppercase and must be an HTTP token. `headers`
are sent after the defaults `Host`, `User-Agent: ail` and `Accept: */*`, and
replace a default of the same name (case-insensitively); `Content-Length`
is the client's own and is ignored, and `Connection: close` and
`Transfer-Encoding: chunked` are honoured (`std.net.19`). A header name must be an HTTP token and a value must not contain
CR, LF or NUL: otherwise the request fails with kind `bad_request` before
anything is sent. `query` is appended to the URL, percent-encoded, after any
query the URL has. `body` is sent with a `Content-Length` (always for `POST`,
`PUT` and `PATCH`). `timeout_ms` bounds connecting and each read and write,
and must be at least 1; `connect_ms` and `total_ms` (default 0, no separate
limit) bound connecting alone and the whole request (`std.net.19`); `max_redirects` must be at least 0 (otherwise the
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
answers it. The server keeps the connection open for the next request when
the client allows it (`std.net.22`), and otherwise sends `Connection: close`
and closes it. The status must be 100 to 599; header names must be
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
over the platform's sockets. On Windows the sockets are Winsock, called
by the native runtime (`win_net`); the code uses nothing else (ADR 0004). *(ADR 0423)*

**std.net.16** — Bodies as `bytes`. *(ADR 0416)*
- `get_bytes(url str)`, `post_bytes(url str, data bytes, content_type str)`
  and `request_bytes(method str, url str, opts Options)` act as `get`, `post`
  and `request` but return `struct RawResponse { status int, headers
  Map[str, str], data bytes } ! NetError`: the body undecoded. `Response.body`
  stays a `str`, the same bytes with each invalid UTF-8 sequence as U+FFFD.
- `Options` has a last field `data bytes`, empty from `options()`. A request
  sends `data` as its body when it isn't empty, and `body` otherwise.
- `Request` has a field `data bytes`: the body the server received, as
  it arrived (`body` is its lossy text), and a last field `server int`, the
  handle `req.stop()` uses (`http.serve.1`).
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

**std.net.19** — Persistent connections and limits. The client keeps a
connection that a response left open (HTTP/1.1, a body framed by length or
chunks, no `Connection: close`, nothing buffered beyond the response) in a
pool, up to 4 idle per scheme, host and port and 32 in all, for at most 30
seconds (less if the server's `Keep-Alive: timeout=` says so). The next
request to the same origin, redirects included, uses it; a pooled connection
the server has closed is dropped when found, and a request that fails on one
before any response byte arrived (a reset or end of file) is sent again, once, on a new
connection, whatever its method, as browsers do (ADR 0915); a failure after a response
byte, or on a new connection, is not. The header
`Connection: close` in `opts.headers` sends that header and keeps nothing.
The header `Transfer-Encoding: chunked` sends the body of `POST`, `PUT` and
`PATCH` as chunks of at most 8 KiB, with no `Content-Length`. Interim `1xx`
responses (except `101`) are skipped. `connect_ms` (0: use `timeout_ms`)
bounds the connection attempt and `total_ms` (0: no limit) bounds the whole
request across redirects: past it a read or write fails with kind
`timed_out`, and `timeout_ms` still bounds each wait. `connect_ms` or
`total_ms` below 0 traps. *(ADR 0637)*

**std.net.20** — `use std.net.jar` gives a cookie jar, `jar.new() Jar`
(RFC 6265 basics). `j.set_cookie(url, line) bool` stores one `Set-Cookie`
line received from `url`: the `Domain` must be the host or a parent of it
(else the line is dropped and the result is `false`; there is no public
suffix list), no `Domain` makes the cookie host-only, `Path` defaults to the
request path's directory, `Max-Age` wins over `Expires`, a cookie whose time
has passed (or `Max-Age` 0 or less) removes any cookie of the same name,
domain and path, and a cookie of the same name, domain and path replaces the
earlier one. `j.store(url, r)` stores every `set-cookie` value of a
`Response`. `j.header(url) str` is the `Cookie` header for a request to
`url`: cookies whose domain and path match, `Secure` ones only for `https`,
unexpired, the longest path first and then the oldest; `j.matching(url)`
gives them as `Cookie` values and `j.value(url, name) str?` one value.
`j.for_script(url)` gives the ones a script may read (not `HttpOnly`) by
name, and `j.set_from_script(url, line)` is `set_cookie` refusing `HttpOnly`.
`j.len()`, `j.clear()` and `j.drop_expired()`. *(ADR 0638)*

**std.net.21** — `j.request(method, url, opts) Response ! NetError` is
`net.request` with the jar's cookies in the `Cookie` header (added to one the
program set) and every response's cookies stored, following the redirects of
`GET` and `HEAD` itself, up to `opts.max_redirects`, so each hop carries and
stores its own cookies; a redirect leaving the origin takes the program's own
`Cookie` header along only if the jar has none for it. `j.get(url)` is
`j.request("GET", url, net.options())`. *(ADR 0638)*

**std.net.22** — Server keep-alive. After answering a request that is
HTTP/1.1 without `Connection: close`, with nothing sent beyond it, and a
final (not `1xx`) response whose headers don't ask to close, the server
sends `Connection: keep-alive` and waits for the next request on the same
connection, up to 15 seconds; a connection that stays quiet is closed
without a word. Each request on it is an `Exchange` as before, the 10 second
limit of `std.net.10` applies per request, and `srv.close()` drops idle
connections. Any other request is answered with `Connection: close`.
*(ADR 0639)*

**std.net.23** — SameSite. A cookie has `same_site`: `lax` (the default, and
any unknown value), `strict` or `none`; `none` without `Secure` drops the
line (`set_cookie` gives `false`). `j.header_from(url, from, navigation)` and
`j.matching_from(...)` give the cookies for a request to `url` made from the
page `from` (`""` for one the user typed) with `navigation` true for a
top-level `GET`: within one site every cookie goes; across sites `none` ones
go, `lax` ones only on a navigation, `strict` ones never. A site is a host's
last two labels, since there is no public suffix list. `header` and
`matching` are the same-site case. *(ADR 0640)*

**std.net.24** — `srv.shutdown()` stops a `Server` listening and closes its
idle connections. A request that has already started to arrive is still
accepted and answered, with `Connection: close`; when none is left `accept`
fails with kind `closed`, as after `close`. *(ADR 0640)*

**std.net.25** — Streaming bodies. `net.open(method, url, opts) Body !
NetError` sends the request like `net.request` (redirects of `GET` and `HEAD`
included, their bodies read out) and returns after the response head, as
`struct Body { status int, headers Map[str, str], all_headers Map[str,
List[str]], handle int }`. `b.read_bytes(max) bytes ! NetError` gives the
next 1 to `max` bytes of the body, and no bytes once it has all been read;
whatever the framing (length, chunks, or up to the end of the connection).
`b.close()` ends it early and closes the connection; a body read to its end
returns its connection to the pool (`std.net.19`). Reading after `close`
fails with kind `closed`. `timeout_ms` bounds each wait for the next piece
and `total_ms` the whole request including the reading. *(ADR 0716)*

**std.net.26** — Content encodings. `get`, `post` and `request` (and the
`_bytes` forms) send `Accept-Encoding: gzip, deflate` unless the program sets
an `Accept-Encoding` header, and undo a `Content-Encoding` of `gzip`,
`x-gzip` or `deflate` (zlib or raw) on the final response, up to 256 MiB,
dropping `Content-Encoding` and `Content-Length` from its headers. A body
that does not decode fails with kind `protocol`. A program that sets
`Accept-Encoding` gets the body as sent, with its headers, and decodes it
itself with `std.compress`. `net.open` sends no `Accept-Encoding` of its own
and decodes nothing. *(ADR 0717)*

**std.net.27** — connection, DNS and timeout failures: A host that doesn't resolve, or an address with no address, fails with kind `not_found`; a refused or reset connection fails with `refused` or `closed`; a request past `connect_ms` or `total_ms` fails with `timed_out`. A `timeout_ms` or `max_redirects` out of range traps. *(ADR 0637)*

**std.net.28** — HTTP shape failures: more than `max_redirects` redirects fails with kind `protocol`, and a body that does not decode under its `Content-Encoding` fails with kind `protocol` (std.net.26). *(ADR 0717)*

**std.net.29** — Request input failures: a URL that is not `http://` or `https://HOST[:PORT][/PATH]` fails with kind `bad_url`; a method that is not an HTTP token, or a header name or value that `std.net.6` forbids, fails with kind `bad_request`. Both fail before any connection is opened. *(ADR 0784)*

```ail rules=std.net.29
use std.net

fn main()
    a := net.get("ftp://example.com") else err
        print(err.kind)   # bad_url
        net.Response(status: 0, body: "", headers: {})
    mut o := net.options()
    o.headers["a b"] = "x"
    b := net.request("GET", "http://localhost:1/", o) else err
        print(err.kind)   # bad_request
        net.Response(status: 0, body: "", headers: {})
    print(a.status + b.status)
```

**std.net.30** — Proxies: when `HTTP_PROXY` (for `http://`) or `HTTPS_PROXY` (for `https://`) is set (the lowercase name wins) and the host doesn't match `NO_PROXY` (comma-separated hosts and domain suffixes, an optional `:port`, `*` for all), the request goes through that `http://[user:pass@]host[:port]` proxy: `http` as an absolute-form request with a `Proxy-Authorization: Basic` header when the proxy URL has a user, `https` through a `CONNECT` tunnel with TLS to the origin. Loopback hosts never use a proxy. A proxy that can't be reached fails with the `std.net.27` kinds; one that answers `CONNECT` with anything but 2xx fails with kind `protocol`; a variable that isn't an `http://` proxy URL fails with kind `bad_url`. The native client only. *(ADR 0869)*

```ail norun rules=std.net.30
use std.net

fn main()
    # With HTTP_PROXY=http://127.0.0.1:3128 in the environment, this asks the
    # proxy for http://example.test/a; with NO_PROXY=example.test it doesn't.
    r := net.get("http://example.test/a") else err
        print(err.kind)
        return
    print(r.status)
```

**std.net.31** — HTTP/2: on `https://` a new connection offers ALPN `h2` and `http/1.1` (unless the body streams, `net.open`, or `AIL_HTTP2=0` is set). When the server picks `h2` the request is sent as HTTP/2 with HPACK, the pseudo-headers `:method`, `:authority`, `:scheme`, `:path` first and the other headers in a stable browser-like order, honouring flow control; the result is the same `Response` as over HTTP/1.1. A server that picks `http/1.1` or names no protocol gets HTTP/1.1. A protocol the client did not offer fails with kind `tls`; any other HTTP/2 failure (a malformed frame or header block, GOAWAY before the request is processed, RST_STREAM) fails with kind `protocol`. An idle HTTP/2 connection is kept for the next request to its origin as in std.net.19. The native client only. *(ADR 0913)*

```ail norun rules=std.net.31
use std.net

fn main()
    # Over HTTP/2 when the server picks `h2`; the program can't tell.
    r := net.get("https://example.test/") else err
        print(err.kind)
        return
    print(r.status)
```

**std.net.32** — Readiness sets: the server waits for its listener and its parked keep-alive connections (`std.net.22`) in an `epoll` set on Linux, a `kqueue` set on macOS and a completion port on Windows, so the cost of a wait does not grow with the number of idle connections. Nothing a program can observe changes: the same requests get the same answers in any order, a quiet connection holds up no other, and the deadlines of `std.net.10` and `std.net.22` still apply. A wait on a single connection (a client call, `read`) stays `poll`. If the system gives no set, the server falls back to a `poll` loop over its connections. The native runtime only. *(ADR 1009)*

```ail norun rules=std.net.32
use std.net

fn main() !
    srv := net.serve("127.0.0.1:0")?
    # Thousands of idle connections cost the next request nothing extra.
    ex := srv.accept()?
    ex.respond(net.Response(status: 200, headers: {}, body: "ok"))?
```

**std.tls.1** — TLS handshake and record failures: A record that is too long, a Finished message that does not verify, or any other handshake or record error fails with kind `tls` as a `TlsError` of kind `Protocol`, and its message names the defect. *(ADR 0360)*

**std.tls.2** — certificate check failures: A certificate that is malformed, has a bad time, a trailing byte, a non-minimal length or an unexpected element, or that does not match the server, fails with kind `tls` as a `TlsError` of kind `Certificate`. *(ADR 0360)*

**std.tls.3** — protocol and root-store failures: A protocol error that is not a record error, and a root-store error passed through, fail with kind `tls` as a `TlsError` of kind `Protocol`. *(ADR 0360)*

**std.tls.4** — crypto primitive failures: a primitive's failure surfaces only as a TLS failure of kind `tls`, never as a separate error of its own. *(ADR 0360)*

```ail norun rules=std.net.27
use std.net

fn kind_of(url str) str
    r := net.get(url) else err
        return "{err.kind}"
    return "ok"

fn main()
    print(kind_of("https://no-such-host.invalid"))
```

```ail norun rules=std.net.28
use std.net

fn kind_of(url str) str
    r := net.get(url) else err
        return "{err.kind}"
    return "ok"

fn main()
    print(kind_of("https://no-such-host.invalid"))
```

```ail norun rules=std.tls.1
use std.net

fn kind_of(url str) str
    r := net.get(url) else err
        return "{err.kind}"
    return "ok"

fn main()
    print(kind_of("https://no-such-host.invalid"))
```

```ail norun rules=std.tls.2
use std.net

fn kind_of(url str) str
    r := net.get(url) else err
        return "{err.kind}"
    return "ok"

fn main()
    print(kind_of("https://no-such-host.invalid"))
```

```ail norun rules=std.tls.3
use std.net

fn kind_of(url str) str
    r := net.get(url) else err
        return "{err.kind}"
    return "ok"

fn main()
    print(kind_of("https://no-such-host.invalid"))
```

```ail norun rules=std.tls.4
use std.net

fn kind_of(url str) str
    r := net.get(url) else err
        return "{err.kind}"
    return "ok"

fn main()
    print(kind_of("https://no-such-host.invalid"))
```

```ail rules=std.net.26
use std.net
use std.compress

fn once(srv net.Server) !
    ex := srv.accept()?
    ex.respond_bytes(200, {"content-encoding": "gzip"}, compress.gzip("zipped".to_bytes()))?

fn main() !
    srv := net.serve("127.0.0.1:0")?
    t := spawn once(srv)
    print(net.get("http://{srv.addr()}/")?.body)
    t.wait()?
```

```ail rules=std.net.23,std.net.24,std.net.25
use std.net
use std.net.http
use std.net.jar

fn hi(r net.Request) net.Response
    net.Response(status: 200, headers: {}, body: "hi")

fn main() !
    mut j := jar.new()
    j.set_cookie("https://shop.test/", "l=1; SameSite=Lax")
    j.set_cookie("https://shop.test/", "s=2; SameSite=Strict")
    print(j.header_from("https://shop.test/", "https://other.test/", true))
    print(j.header_from("https://shop.test/", "https://other.test/", false) == "")
    srv := net.serve("127.0.0.1:0")?
    t := spawn http.run(srv, hi)
    print(net.get("http://{srv.addr()}/")?.body)
    b := net.open("GET", "http://{srv.addr()}/", net.options())?
    print(b.read_bytes(100)?.len())
    b.close()
    srv.shutdown()
    t.wait()?
```

```ail rules=std.net.19,std.net.20,std.net.21,std.net.22
use std.net
use std.net.http
use std.net.jar

fn site(r net.Request) net.Response
    if r.path == "/login"
        return net.Response(status: 302, headers: {"location": "/home"}, body: "", all_headers: {"set-cookie": ["sid=abc; Path=/"]})
    net.Response(status: 200, headers: {}, body: "{r.peer} {r.headers.get("cookie") else "-"}")

fn main() !
    srv := net.serve("127.0.0.1:0")?
    t := spawn http.run(srv, site)
    base := "http://{srv.addr()}"
    mut j := jar.new()
    a := j.get("{base}/login")?.body.split(" ")
    b := j.get("{base}/again")?.body.split(" ")
    print("{a[1]} {a[0] == b[0]}")
    mut o := net.options()
    o.total_ms = 5000
    o.headers["Transfer-Encoding"] = "chunked"
    o.body = "data"
    print(net.request("POST", "{base}/x", o)?.status)
    srv.close()
    t.wait()?
```

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

```ail rules=std.time.1
use std.time

fn main()
    print(time.now_ms() > 0)
```

**std.time.2** — `sleep_ms(ms int)` pauses the program for at least `ms`
milliseconds. A negative `ms` traps. *(ADR 0157)*

```ail rules=std.time.2
use std.time

fn main()
    time.sleep_ms(1)
    print("woke")
```

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

**std.date.16** — `date.parse_date(text)` and `date.parse_iso(text)` fail
with `DateError` whose `why` is `no such day` when the text has the right shape
but names no day, such as `2026-02-30`. *(ADR 0449)*

```ail rules=std.date.16
use std.time.date

fn main()
    d := date.parse_date("2026-02-30") else err
        print("{err}")                       # no such day: `2026-02-30`
        date.Date(year: 2000, month: 1, day: 1)
    print(d.days())
```

**std.date.17** — The shape of the date fails with `DateError` and why
`expected YYYY-MM-DD`, `expected a 4-digit year`, `expected a 2-digit month` or
`expected a 2-digit day`, for `parse_date` and for the date part of `parse_iso`.
*(ADR 0783)*

**std.date.18** — The time part of `parse_iso` fails with why `expected \`T\`
between the date and the time`, `expected a time after the date`, `expected a
2-digit hour`, `expected a 2-digit minute`, `expected HH:MM`, `expected a 2-digit
second`, `expected digits after \`.\``, or `seconds go to 59 (leap seconds
aren't supported)`. *(ADR 0783)*

**std.date.19** — The offset fails with why `no offset (write Z or +HH:MM, or use
parse_in)` when a `parse_iso` text has none, and `expected Z, +HH:MM or -HH:MM
after the time` when it is malformed. *(ADR 0783)*

**std.date.20** — A zone name the zone data doesn't know fails with why `unknown
zone` and the name as the text, for `date.at` and for a `parse_in` zone.
*(ADR 0783)*

```ail rules=std.date.17
use std.time.date

fn date_why(text str) str
    d := date.parse_date(text) else err
        return err.why
    return "ok {d.iso()}"

fn main()
    print(date_why("2026-2-3"))    # expected YYYY-MM-DD
    print(date_why("2026-02-30"))  # no such day
```

```ail rules=std.date.18
use std.time.date

fn iso_why(text str) str
    t := date.parse_iso(text) else err
        return err.why
    return "ok"

fn main()
    print(iso_why("2026-01-02 1:00"))   # expected a time after the date
    print(iso_why("2026-01-02T10:61Z")) # expected HH:MM
```

```ail rules=std.date.19
use std.time.date

fn iso_why(text str) str
    t := date.parse_iso(text) else err
        return err.why
    return "ok"

fn main()
    print(iso_why("2026-01-02T10:00"))    # no offset (write Z or +HH:MM, or use parse_in)
    print(iso_why("2026-01-02T10:00+5"))  # expected Z, +HH:MM or -HH:MM after the time
```

```ail rules=std.date.20
use std.time.date

fn zone_why(zone str) str
    t := date.at(0, zone) else err
        return err.why
    return "ok"

fn main()
    print(zone_why("Mars/Base"))  # unknown zone
```
## Zones: `std.tz`

**std.tz.1** — `tz.offset(zone, secs)` is the UTC offset in seconds of IANA
zone `zone` at the instant `secs` (seconds since the epoch), or `none` when the
zone isn't installed, the name is empty or unsafe (it may hold only letters,
digits and `/ _ + -`, no `..`, no leading `/`). It reads TZif files (RFC 9636)
from the copy of the IANA database shipped with AIL (`std.tz.3`), so a zone
gives the same offsets on every OS; when that copy is missing it falls back to
`/usr/share/zoneinfo` and the usual alternatives (none on Windows). It applies
the file's POSIX rule for instants after its last transition. It has the
effects `fs` and `env`. *(ADR 0448, ADR 0924)*

```ail rules=std.tz.1
use std.tz

fn main()
    off := tz.offset("Europe/Paris", 0) else 0
    print(off)
```

**std.tz.2** — `tz.system()` is the system zone: `TZ` if it names a safe zone,
else the zone `/etc/localtime` links to (on Windows, the active zone mapped to
its IANA name, `std.tz.3`), else `"UTC"`.

```ail rules=std.tz.2
use std.tz

fn main()
    print(tz.system().len() > 0)
```

**std.tz.3** — AIL ships the IANA tz database as slim TZif files in
`tools/tzdata/zones` under `AIL_ROOT` (the tree `ail` runs from); `tz.offset`
looks there first. When `AIL_ROOT` is unset or the file is missing, a name is
looked up in the system directories, and on Windows it is `none`.
`tz.system()` is `TZ` if it names a safe zone, else the zone `/etc/localtime`
links to, else `"UTC"` (on Windows, `TZ` if it names a safe zone, else the active Windows zone ID mapped to its IANA name by `tools/tzdata/windows-zones.txt`, else `"UTC"`; ADR 0972). UTC and fixed offsets
work everywhere. *(ADR 0924)*

```ail rules=std.tz.3
use std.tz

fn main()
    off := tz.offset("UTC", 0) else 0
    print(off)
```

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

## Streaming files: `std.file`

`use std.file` opens a file as a handle read or written a piece at a time;
`std.fs` reads and writes whole files. The handles are written over
`std.fs` (`read_range`, `write_at`, `append_bytes`), so they hold no operating-system
resource and act the same on macOS, Linux and Windows. Both are `unique`
(`own.uniq.1`). A failure is the `fs.IoError` of `std.fs` (`std.err.1`);
misuse by the program (a negative offset or count, `seek` on an appending
writer) traps. *(ADR 1071)*

**std.file.1** — `open(path str) Reader ! fs.IoError` opens `path` for reading
at byte 0 and fails as `fs.read_range` does (`not_found`, `permission`,
`is_dir`). `r.close()` ends the reader; a reader that goes out of scope ends
too. No file is held open, so the file may be renamed or removed meanwhile.
*(ADR 1071)*

**std.file.2** — `r.read(n int) bytes ! fs.IoError` returns up to `n` bytes
from the position and moves it past them. It returns fewer than `n` only when
the file ends, and empty bytes at the end. A negative `n` traps. *(ADR 1071)*

**std.file.3** — `r.read_exact(n int) bytes? ! fs.IoError` returns exactly `n`
bytes, or `none` when fewer remain; then the position is unchanged, so a
reader following a growing file may try again. (`io.read_exact`,
`std.io.7`, cannot do that: standard input has no position to restore.)
*(ADR 1071)*

**std.file.4** — `r.read_line() str? ! fs.IoError` returns the next line without
its `\n` or `\r\n` (invalid UTF-8 becomes U+FFFD), or `none` at the end of
the file; a last line with no ending is returned. `r.read_complete_line()`
returns a line only when its `\n` is already in the file and otherwise returns
`none` and leaves the position, so a program following a log that is being
written never sees half a line. A reader at the end of the file sees what is
added later on its next read. *(ADR 1071)*

**std.file.5** — `r.tell() int` is the offset of the next byte; `r.seek(pos
int)` moves it (a negative `pos` traps; a position past the end reads nothing
until the file grows); `r.size() int ! fs.IoError` is the file's size now.
*(ADR 1071)*

```ail norun rules=std.file.1,std.file.2,std.file.3,std.file.4,std.file.5
use std.file

fn main() !
    mut r := file.open("app.log")?
    r.seek(r.size()?)
    while true
        line := r.read_complete_line()? else break
        print(line)
    head := r.read_exact(16)? else b""
    print(head.len())
    r.close()
```

**std.file.6** — `create(path str)`, `append(path str)` and `update(path str)`
each give `Writer ! fs.IoError`. `create` makes the file or empties it and
writes from byte 0; `append` makes it when missing and adds at the end, each
flush being one append, so several writers may add to one file; `update`
makes it when missing, empties nothing and writes from byte 0 so that `seek`
can patch it. They fail as `fs.write_bytes` does. *(ADR 1071)*

**std.file.7** — `w.write(text str)`, `w.write_line(text str)` (adds `\n`) and
`w.write_bytes(data bytes)`, each `! fs.IoError`, add to a buffer. At 65536
bytes held the buffer is written, and `w.flush() ! fs.IoError` writes it at
once; after a failed flush the bytes stay buffered. A reader of the file sees
only what has been flushed. *(ADR 1071)*

**std.file.8** — `w.tell() int` is the offset the next byte will have,
counting what is buffered; `w.seek(pos int) ! fs.IoError` flushes and moves the
write position (bytes between the old end and a position past it are zero). A
negative `pos`, or `seek` on a writer made by `append`, traps. *(ADR 1071)*

**std.file.9** — `w.close() ! fs.IoError` flushes and ends the writer, and is
the way to see a failure. A writer that goes out of scope is flushed by its
`drop` (`own.uniq.3`) on every path, a trap included, but a `drop` cannot
report a failure, so write `w.close()?` where the data matters. *(ADR 1071)*

**std.file.10** — `w.sync() ! fs.IoError` flushes the writer and then asks the
operating system to put the file on the disk (`fs.sync`, `std.fs.38`), so the
data written so far survives a power failure. `close` and `drop` flush but do not
sync. *(ADR 1098)*

```ail norun rules=std.file.6,std.file.7,std.file.8,std.file.9,std.file.10
use std.file

fn main() !
    mut w := file.create("report.csv")?
    w.write_line("id,name")?
    for i in 0..100000
        w.write_line("{i},row {i}")?
    w.flush()?
    print(w.tell())
    w.close()?
    mut log := file.append("report.log")?
    log.write("done\n")?
    mut patch := file.update("report.csv")?
    patch.seek(3)?
    patch.write("ID")?
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
field and ends each row with `\n`; a row of one empty field is written as
`""`, because a blank line is no row. *(ADR 0626)*

**std.csv.4** — `write_records(header List[str], recs List[Map[str, str]])
str` writes the header and then each map's values in header order; a key a map
lacks writes as an empty field. *(ADR 0626)*

**std.csv.5** — `CsvError` is an error with `why str` and `row int` (1-based).
An unterminated quote and a record of the wrong width are its causes.
*(ADR 0626)*

**std.csv.6** — A quoted field with no closing quote before the end of the text
fails with `CsvError` and why `unterminated quote`, at the row where it starts.
*(ADR 0783)*

**std.csv.7** — A record whose field count differs from the header's fails with
`CsvError` and why `N fields, the header has M`, where `row` is that record's row.
*(ADR 0783)*

```ail rules=std.csv.6
use std.csv

fn main()
    recs := csv.records("name,age\n\"ann,3\n") else err
        print("{err.why} (row {err.row})")   # unterminated quote (row 2)
        []
    print(recs.len())
```

```ail rules=std.csv.7
use std.csv

fn main()
    recs := csv.records("name,age\nann,3,x\n") else err
        print("{err.why} (row {err.row})")   # 3 fields, the header has 2 (row 2)
        []
    print(recs.len())
```

```ail rules=std.csv.2,std.csv.5
use std.csv

fn main()
    rows := csv.parse("a,\"b") else err
        print("{err.why} (row {err.row})")   # unterminated quote (row 1)
        []
    print(rows.len())
    recs := csv.records("name,age\nann,3,x\n") else err
        print("{err.why} (row {err.row})")   # 3 fields, the header has 2 (row 2)
        []
    print(recs.len())
```
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

## Configuration files: `std.toml`

**std.toml.1** — `use std.toml` provides `parse(text str) Json ! TomlError`,
which reads a TOML 1.0 document into the `Json` value ([std-json](std-json.md)):
tables are `Obj` (members in document order), arrays are `Arr`, strings are
`Str` and booleans are `Bool`, so `get`, `at`, `as_str` and the other `Json`
accessors read it. *(ADR 0901)*

**std.toml.2** — Basic strings (`"..."`, `"""..."""`) take the escapes `\b \t
\n \f \r \" \\ \uXXXX \UXXXXXXXX`; in a `"""` string a backslash at the end
of a line drops the break and the blanks after it. Literal strings (`'...'`,
`'''...'''`) take no escapes. A line break right after an opening `"""` or
`'''` is dropped. *(ADR 0901)*

**std.toml.3** — Integers (decimal, `0x`, `0o`, `0b`, `_` between digits, a
leading `+` or `-`) and floats read as `Num`, so an integer beyond 2^53 loses
precision. A date, time or date-time (`1979-05-27`, `07:32:00`,
`1979-05-27 07:32:00Z`) is kept as the `Str` of its TOML spelling. *(ADR 0901)*

**std.toml.4** — A key is bare (letters, digits, `_`, `-`), `"quoted"` or
`'literal'`; dots between parts name nested tables (`a.b = 1`). *(ADR 0901)*

**std.toml.5** — `[a.b]` opens a table, creating the tables on its path.
`[[a]]` appends a new table to the array of tables `a`; a path through an array
of tables goes into its last table. *(ADR 0901)*

**std.toml.6** — Arrays may span lines, hold comments and end in a comma, and
may mix types. An inline table `{ k = v, ... }` is a table on one line.
*(ADR 0901)*

**std.toml.7** — A key set twice in a table, a table opened on a name that
holds a value, and a key path through a value that is not a table fail with
`TomlError`. *(ADR 0901)*

**std.toml.8** — `TomlError` is an error with `why str` and `line int`
(1-based, the line where the parser stopped); its text is `WHY (line N)`. An
unterminated string, an invalid escape, an invalid number and a missing value
are further causes. *(ADR 0901)*

```ail rules=std.toml.1,std.toml.2,std.toml.3,std.toml.4,std.toml.5,std.toml.6
use std.toml

fn main() !
    src := "name = \"a\\tb\"\nport = 8080\nlit = 'c:\\x'\nx.y = [1,\n 2, ]\npt = \{ k = true }\n[db]\nhost = \"h\"\n[[db.node]]\nid = 1\n"
    doc := toml.parse(src)?
    print(doc.str("name") else "")
    print(doc.str("lit") else "")
    print(doc.num("port") else 0.0)
    print((doc.get("x") else Json.Null).to_text())
    print((doc.get("pt") else Json.Null).to_text())
    db := doc.get("db") else Json.Null
    print(db.str("host") else "")
    print((db.get("node") else Json.Null).to_text())
```

```ail rules=std.toml.7,std.toml.8
use std.toml

fn main()
    doc := toml.parse("a = 1\na = 2\n") else err
        print(err)   # duplicate key 'a' (line 2)
        Json.Null
    print(doc.to_text())
```

## Configuration files: `std.yaml`

**std.yaml.1** — `use std.yaml` provides `parse(text str) Json ! YamlError`,
which reads one YAML 1.2 document, in the subset these rules list, into the
`Json` value ([std-json](std-json.md)): maps are `Obj` (members in document
order, keys always `Str`), lists are `Arr`. An empty document, or one that is
only comments, is `Null`. A leading byte order mark and `\r\n` line ends are
accepted. *(ADR 1090)*

**std.yaml.2** — A block map is `key: value` lines at one indentation; a block
list is `- item` lines. A nested block is indented deeper than its parent; a
list may sit at the same indentation as its key. `- key: v` starts a map
inside a list item, and `- - x` a list. A key is plain, `'single'` or
`"double"` quoted. Flow lists `[a, b]` and maps `{k: v}` may span lines and
nest. Double-quoted strings take the escapes `\0 \a \b \t \n \v \f \r \e \"
\/ \\ \xXX \uXXXX \UXXXXXXXX`; single-quoted strings take `''` for `'`.
*(ADR 1090)*

**std.yaml.3** — A plain scalar is, by the YAML 1.2 core schema: `null`, `Null`,
`NULL`, `~` or empty is `Null`; `true`/`True`/`TRUE` and `false`/`False`/`FALSE`
are `Bool`; decimal integers (a leading `+` or `-`), `0x` and `0o` integers
and floats (`.5`, `1e3`) are `Num`; anything else is `Str`. `yes`, `no`, `on`,
`off` and `1.2.3` are strings. A quoted scalar is always a `Str`. An integer
beyond 2^53 loses precision. *(ADR 1090)*

**std.yaml.4** — `#` starts a comment at the start of a line or after a blank,
outside quotes; `a#b` is not a comment. *(ADR 1090)*

**std.yaml.5** — `key: |` reads the indented lines below it literally; `key:
>` folds a single line break between text lines into a space and keeps a blank
line as a line break. The block ends at the first line indented no deeper than
the key. Chomping: `-` removes the final line break, none keeps one, `+` keeps
the trailing blank lines too. *(ADR 1090)*

**std.yaml.6** — A key set twice in one map fails with `YamlError`. *(ADR 1090)*

**std.yaml.7** — Outside the subset the parser fails with a message that says
what to write instead; it never reads such text another way. Refused: anchors
(`&a`), aliases (`*a`), tags (`!t`), merge keys (`<<`), complex keys (`?`),
directives (`%`), explicit block indentation indicators (`|2`), several
documents in the text, multi-line plain or quoted scalars, and a map that
starts on its key's line (`a: b: c`). *(ADR 1090)*

**std.yaml.8** — Indentation is spaces. A tab in the indentation fails.
*(ADR 1090)*

**std.yaml.9** — `---` may start the document and `...` may end it; nothing but
comments and blank lines may follow `...`. *(ADR 1090)*

**std.yaml.10** — `YamlError` is an error with `why str` and `line int`
(1-based, the line where the parser stopped); its text is `WHY (line N)`.
*(ADR 1090)*

```ail rules=std.yaml.1,std.yaml.2,std.yaml.3,std.yaml.4,std.yaml.5,std.yaml.9
use std.yaml

fn main() !
    src := "---\nname: demo   # a comment\nport: 8080\nflag: no\ntags: [a, \"b c\"]\nbuild:\n  - name: test\n    run: |\n      make\n      make test\n...\n"
    doc := yaml.parse(src)?
    print(doc.str("name") else "")
    print(doc.num("port") else 0.0)
    print(doc.str("flag") else "")
    print((doc.get("tags") else Json.Null).to_text())
    print((doc.get("build") else Json.Null).to_text())
```

```ail rules=std.yaml.6,std.yaml.7,std.yaml.8,std.yaml.10
use std.yaml

fn main()
    for src in ["a: 1\na: 2\n", "base: &b 1\n", "a:\n\tb: 1\n"]
        doc := yaml.parse(src) else err
            print(err)   # duplicate key 'a' (line 2), then the anchor and tab errors
            Json.Null
        print(doc.to_text())
```

## Key-value files: `std.kv`

**std.kv.1** — `use std.kv` provides `open(path str) Store ! fs.IoError | KvError`,
which opens the store kept in the file `path` and makes it when the file is
missing or empty. A `Store` is a `unique struct` (`own.uniq.1`): a file has one
handle at a time. The keys are `str` and the values `bytes`; `close(move self)`
ends the use, and dropping the store does the same, because nothing is
buffered. *(ADR 1098)*

**std.kv.2** — `get(key) bytes?`, `has(key) bool`, `keys() List[str]` and
`len() int` read the store from memory and do no file access. `keys` lists
each key in the order it was first set since it was last removed. Any `str`
is a key, the empty string too; any `bytes` is a value, empty ones too.
*(ADR 1098)*

**std.kv.3** — `set(mut self, key, value bytes) ! fs.IoError | KvError` gives
the key the value, and `remove(mut self, key) bool ! fs.IoError | KvError`
deletes it and is true when the key had a value (nothing is written when it
had none). The change is in the file when the call returns; when the call
fails the store is as it was. A value of 4 GiB or more fails with `KvError`.
*(ADR 1098)*

**std.kv.4** — A change is made with an ordinary file write: it survives the
program stopping, crashing or being killed, but is not yet on the disk, so a
power failure may lose the last changes. `sync(self) ! fs.IoError` asks the
operating system to put the file on the disk (`fs.sync`, `std.fs.38`); after it
returns, every change made so far survives a power failure. Call it after the
changes that matter, not after each one: it is slow. `compact` syncs the new log
before it renames it into place. *(ADR 1098)*

**std.kv.5** — The file is a log: a 7-byte header `AILKV1\n`, then one record
per change (key length and value length as 4-byte little-endian integers, the
key, the value, and the CRC-32 of those bytes; a value length of `0xFFFFFFFF`
and no value mean removal). `open` replays the records in order and stops at
the first record that is cut short or fails its checksum; it cuts the file back
to the end of the last good record, so a stop in the middle of a change loses
that change and nothing before it. *(ADR 1098)*

**std.kv.6** — A non-empty file that does not start with the header, or with
the start of it, is not a store: `open` fails with `KvError` and leaves the
file alone. *(ADR 1098)*

**std.kv.7** — `compact(mut self) ! fs.IoError | KvError` rewrites the log
with one record per key. It writes `PATH.compact` and renames it over the
file (`std.fs.11`), so a stop leaves the old log or the new one whole. `set` and
`remove` compact by themselves when the log holds more than 1 MiB of replaced
or removed records and more of them than live ones. *(ADR 1098)*

**std.kv.8** — A store is held in memory: its memory is the keys and values
together, so it suits small data. *(ADR 1098)*

**std.kv.9** — `KvError` is an error with `why str`; its text is `why`.
*(ADR 1098)*

```ail rules=std.kv.1,std.kv.2,std.kv.3,std.kv.4,std.kv.8
use std.kv

fn main() !
    dir := fs.make_temp_dir("ail-kv-doc-")?
    path := "{dir}/settings.kv"
    mut s := kv.open(path)?
    s.set("name", "demo".to_bytes())?
    s.set("blob", b"\x01\x02")?
    s.sync()?                               # on the disk, not only in the file cache
    print(s.len())                          # 2
    print(s.remove("blob")?)                # true
    s.close()
    mut again := kv.open(path)?             # the change is still there
    print((again.get("name") else b"").text())   # demo
    print(again.keys().join(","))           # name
    again.close()
    fs.remove(path)?
    fs.remove(dir)?
```

```ail rules=std.kv.5,std.kv.7
use std.kv

fn main() !
    dir := fs.make_temp_dir("ail-kv-doc-")?
    path := "{dir}/log.kv"
    mut s := kv.open(path)?
    s.set("k", b"1")?
    s.set("k", b"22")?
    s.close()
    fs.append_bytes(path, b"\x05\x00")?     # a change cut short by a stop
    mut again := kv.open(path)?             # the unfinished record is dropped
    print((again.get("k") else b"").text()) # 22
    again.compact()?
    print(fs.metadata(path)?.size)          # 22: header and one record
    again.close()
    fs.remove(path)?
    fs.remove(dir)?
```

```ail rules=std.kv.6,std.kv.9
use std.kv

fn try_open(path str)
    s := kv.open(path) else err
        print(err)                          # not a std.kv file (bad header)
        return
    print(s.len())

fn main() !
    dir := fs.make_temp_dir("ail-kv-doc-")?
    path := "{dir}/notes.txt"
    fs.write_text(path, "just text")?
    try_open(path)
    fs.remove(path)?
    fs.remove(dir)?
```

## Integers of any size: `std.bigint`

**std.bigint.1** — `use std.bigint` provides `struct BigInt`, an integer of
any size. Its operations never overflow; they are limited by memory only.
*(ADR 0902)*

**std.bigint.2** — `from_int(n int) BigInt` and `parse(text str) BigInt !
BigIntError`. `parse` takes decimal digits with an optional leading `-` or
`+`; leading zeros are allowed and `-0` is zero. *(ADR 0902)*

**std.bigint.3** — `BigInt.to_text() str` is the decimal text with a leading
`-` for a negative value. `BigInt.as_int() int?` is `none` when the value does
not fit in an `int`. *(ADR 0902)*

**std.bigint.4** — `add`, `sub` and `mul` take a `BigInt` and give a `BigInt`.
`cmp(o)` gives -1, 0 or 1. `negate()` and `abs()` give a `BigInt`. *(ADR 0902)*

**std.bigint.5** — `divmod(o) QuoRem ! BigIntError` divides toward zero:
`quo` is the quotient and `rem` has the sign of the dividend, so
`quo * o + rem` is the dividend. *(ADR 0902)*

**std.bigint.6** — `pow(n int) BigInt ! BigIntError` is `self` to the power
`n >= 0`; `pow(0)` is 1. *(ADR 0902)*

**std.bigint.7** — `factorial(n int) BigInt ! BigIntError` is `n!` for `n >= 0`.
*(ADR 0902)*

**std.bigint.8** — `is_zero()` and `is_negative()` give `bool`; zero is never
negative. *(ADR 0902)*

**std.bigint.9** — `BigIntError` is an error with `why str`. Its causes are `no
digits`, `invalid digit 'C'`, `division by zero`, `negative exponent` and
`negative factorial`. *(ADR 0902)*

```ail rules=std.bigint.1,std.bigint.2,std.bigint.3,std.bigint.4,std.bigint.5,std.bigint.6,std.bigint.7,std.bigint.8,std.bigint.9
use std.bigint

fn main() !
    a := bigint.parse("123456789012345678901234567890")?
    b := bigint.from_int(-7)
    print(a.add(b).to_text())
    print(a.mul(b).to_text())
    print(a.cmp(b))
    qr := a.divmod(b)?
    print("{qr.quo.to_text()} {qr.rem.to_text()}")
    print(bigint.from_int(2).pow(70)?.to_text())
    print(bigint.factorial(25)?.to_text())
    print(a.as_int() else 0)
    print(b.is_zero() or not b.is_negative())
    bad := bigint.parse("1x") else err
        print(err)   # invalid digit 'x'
        b
    print(bad.to_text())
```

## Command-line arguments: `std.args`

**std.args.1** — `use std.args` provides the spec makers `flag(name, short,
help)` for a switch and `opt(name, short, help, default)` for an option that
takes a value. `short` is one letter or `""`. *(ADR 0627)*

**std.args.2** — `pos(name, help)` is a required positional word; the
positionals are taken in the order the specs list them. `rest(name, help)`
allows any number of further positionals, which `Args.rest` returns. Without a
`rest` spec, an extra word is an error. A `Spec`'s `kind` is the `SpecKind`
of the maker that made it: `Flag`, `Opt`, `Pos` or `Rest`. *(ADR 0627,
ADR 1068)*

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

```ail rules=std.args.6
use std.args

fn why(words List[str]) str
    spec := [args.flag("verbose", "v", "more output"), args.opt("out", "o", "output", "a.txt"), args.pos("input", "file")]
    a := args.parse("tool", "demo", spec, words) else err
        return err.why
    print(a.get("input"))
    return "ok"

fn main()
    print(why(["--nope", "in.txt"]))       # unknown option --nope
    print(why(["--verbose=yes", "in.txt"]))  # --verbose takes no value
    print(why(["in.txt", "--out"]))        # --out needs a value
    print(why(["in.txt", "extra"]))        # unexpected argument extra
    print(why([]))                         # missing INPUT
    print(why(["in.txt"]))                 # in.txt, then ok
```

**std.args.7** — `impl FromArgs for Opts` on a struct derives
`Opts.parse(name, about, words) Opts ! ArgsFailure`. A field is a switch
(`bool`), an option (`str?`, `int?` or `float?`; `none` when absent), the rest
(`List[str]`) or a required positional (`str`, `int` or `float`, in field
order). The field's `##` comment (`ct.derive.5`) is its help. A switch or
option has the short name of its first letter unless an earlier one has it or
it is `h`. An `int` or `float` word that doesn't parse, and everything
`args.parse` rejects, is an `ArgsFailure` (an error with `why str` and
`usage str`, like `ArgsError`); any other field type is a `DeriveError` at the
`impl`. *(ADR 0777)*

**std.args.8** — `Args.has(name)` says whether an option or positional was
given; `Args.opt(name) str?`, `Args.opt_int(name) int?` and
`Args.opt_float(name) float?` give `none` when it was absent (or, for the
numbers, isn't one); `Args.int(name) int ! ArgsError` and
`Args.float(name) float ! ArgsError` fail on text that isn't a number. The
derive calls them. *(ADR 0777)*

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

```ail rules=std.args.7,std.args.8,ct.derive.6
use std.args

struct Opts
    ## more output
    verbose bool
    ## worker count
    jobs int?
    ## file to read
    input str

impl FromArgs for Opts

fn main()
    o := Opts.parse("tool", "demo", ["-v", "-j", "4", "in.txt"]) else e
        print(e.why)
        return
    print("{o.verbose} {o.jobs} {o.input}") # true 4 in.txt
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
    Key(key KeyCode, ctrl bool, alt bool, shift bool)
    Resize(cols int, rows int)
    Close
    Mouse(col int, row int, button MouseButton, kind MouseKind)
    Scroll(lines int)
enum KeyCode
    Text(text str)
    Enter
    Tab
    Backspace
    Delete
    Insert
    Escape
    Up
    Down
    Left
    Right
    Home
    End
    PageUp
    PageDown
    F(n int)
enum MouseButton
    Left
    Middle
    Right
enum MouseKind
    Down
    Up
    Drag
error struct UiError
    kind UiKind
    message str
enum UiKind
    NoDisplay
    Closed
    Other
```

**std.ui.1** — `open(title str, cols int, rows int) Window ! UiError` opens
a window `cols` by `rows` cells in size, with every cell blank (a space,
light grey on black). It fails with `kind` `NoDisplay` when no window can
be shown (no display, or a run without a window host), and `Other` when
`cols` or `rows` is outside 1 to 1000 or a window is already open. In a native build the window is drawn by the
`std.app` host (`std.app.10`) as one canvas of the cell grid: `NoDisplay` when
the host can't start, `click` is a `down` mouse event (no `up` or `drag` yet),
and the clipboard is the program's own. *(ADR 0177, ADR 0863, ADR 1068)*

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

**std.ui.7** — In `Key(key, ctrl, alt, shift)`, `key` is a `KeyCode`:
either `Text(text)`, the typed character (`"a"`, `"A"`, `" "`, `"é"`,
following the keyboard layout and Shift), or a named key `Enter`, `Tab`,
`Backspace`, `Delete`, `Insert`, `Escape`, `Up`, `Down`, `Left`, `Right`,
`Home`, `End`, `PageUp`, `PageDown` or `F(n)` for `n` from 1 to 12. While
Ctrl is held, a text key gives its unshifted character (Ctrl-S is
`Key(Text("s"), true, false, false)`, Ctrl-Shift-S has `shift` true), and no
typed character is reported. On macOS, Command reports as `ctrl`. Modifier
keys alone give no event. `key.name()` is the key's text or its name in lower
case (`enter`, `pageup`, `f5`); `key.text()` is the text, or `none` for a
named key; `key_code(name)` is the inverse of `name()`. *(ADR 0177,
ADR 1068)*

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
                if key == Escape
                    break
            Resize(_, _) | Mouse(_, _, _, _) | Scroll(_) => continue
            Close => break
    w.close()
```

**std.ui.10** — `Mouse(col, row, button, kind)` reports a mouse button over
cell (`col`, `row`): `button` is a `MouseButton` (`Left`, `Right` or
`Middle`), and `kind` is a `MouseKind`: `Down` when it is pressed, `Up` when
it is released, and `Drag` when the mouse moves to another cell while it is
held. Movement with no button held gives no event. *(ADR 0240, ADR 1068)*

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

**std.ui.25** — Layout is pure arithmetic on `Rect`. `left(n)` and `top(n)`
keep the first `n` columns or rows, clamped to the rectangle; `inset(n)`
shrinks every side by `n`, never below zero; `has(c, r)` tests a cell.
`cols_of(sizes)` and `rows_of(sizes)` split a rectangle side by side or
stacked: a size of 0 or more is fixed, and -1 shares what is left evenly,
the first such sizes taking any remainder. *(ADR 0973)*

**std.ui.26** — `Label` shows its `text` (bold if `bold`) cut or padded to
its rectangle. It takes no events. *(ADR 0973)*

**std.ui.27** — `Checkbox` toggles `checked` on a click, or on Space or
Enter while `focused`, and answers `Changed`. `RadioGroup` shows one choice
per row; Up and Down move `selected`, a click on a row picks it (both
`Changed`), Enter answers `Submit`, and `chosen()` is the selected choice.
*(ADR 0973)*

**std.ui.28** — `ProgressBar` shows `done` of `total` as a bar and a
percentage (`percent()`, 0 to 100; an empty total is 0). `Tabs` shows one
row of titles; Left, Right and Tab move `selected`, and a click on a title
picks it (all `Changed`). *(ADR 0973)*

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
and NaN, and the tree walker and the bytecode tier agree. A chain over
`f32` tensors runs the same operations at `f32` (std.tensor.21). A chain whose
inputs don't share one shape, or that has an `int` input, runs unfused.
Fusion is not visible in the program. *(ADR 0524)*

```ail rules=std.tensor.20
fn main()
    a := tensor.from([1.0, -2.0, 3.0, 0.5], [2, 2])
    b := tensor.from([4.0, 8.0, -1.0, 2.0], [2, 2])
    print((a * b + a).relu())     # tensor.from([5.0, 0.0, 0.0, 1.5], [2, 2])
    print(-(a - b) / b)           # tensor.from([0.75, 1.25, 4.0, 0.75], [2, 2])
    print((a * 2.0 - b).sigmoid().tanh().sum() > 0.0)   # true
```

**std.tensor.24** — `t.softmax()` on a `float` or `f32` tensor of at least one
axis gives a tensor of the same shape in which each row along the last axis is
`exp(x - m)` divided by the sum of the row's `exp(x - m)`, where `m` is the
row's largest element. The sum runs in element order. A row that holds a NaN
or an infinity gives NaN for that row. An empty last axis traps. An `f32`
tensor is computed in `float` and rounded once, as std.tensor.21 does for the
other activations. On a `Tensor[int]` it is a compile-time error. Fusion
(std.tensor.20) does not cover it. *(ADR 0827)*

```ail rules=std.tensor.24
fn main()
    print(tensor.from([0.0, 0.0]).softmax())    # tensor.from([0.5, 0.5])
    a := tensor.from([1.0, 3.0, 2.0, 2.0, 2.0, 2.0], [2, 3])
    print(a.softmax().map(|x| x * 100.0).round())   # per row: 9 67 24, then 33 33 33
```

**std.tensor.25** — `tensor.from_fn(shape, |i| e)` builds a tensor whose element
at flat (row-major) index `i` is `e`, computed in index order. `shape` is an
`int` (a 1-D tensor of that length) or a `List[int]` as in `tensor.zeros`; a
negative size or too many elements traps with std.tensor.2. The function takes
the `int` index and gives an `int`, a `float` or an `f32`, and that is the
element type. A call with another argument count, a function that is not
written `|i| e`, a shape of another type, or a function that gives another type
is a compile-time error. *(ADR 0831)*

```ail rules=std.tensor.25
fn main()
    print(tensor.from_fn(4, |i| i.to_float() * 0.5))   # tensor.from([0.0, 0.5, 1.0, 1.5])
    print(tensor.from_fn([2, 3], |i| i * i))           # tensor.from([0, 1, 4, 9, 16, 25], [2, 3])
```

**std.tensor.21** — `tensor.Tensor[f32]` holds `f32` elements (ADR 0796). It has
the methods and operators of `Tensor[float]`, and each result is rounded to
`f32` once: the operations are single-precision IEEE operations, and `sum`,
`dot`, `max`, `mean` and `matmul` accumulate in `f32` in the order of
std.tensor.5, on every path. A `float` scalar in an expression is rounded to
`f32` at the boundary. `tensor.from(xs, shape)` and `tensor.zeros(shape)` build
a `Tensor[f32]` when the expected type is one; `xs` is a `List[float]`, rounded
element by element. A `Tensor[f32]` and a `Tensor[float]` in one expression are
a compile-time error. `to_list()` gives a `List[float]`. *(ADR 0796, ADR 0797)*

**std.tensor.22** — `t.to_f32()` rounds each element of a `Tensor[float]` to
`f32`, and `t.to_float()` widens each element of a `Tensor[f32]` exactly to
`float`. Either on the other element type, or on an `int` tensor, is a
compile-time error. *(ADR 0798)*

```ail rules=std.tensor.21,std.tensor.22
fn half() tensor.Tensor[f32]
    tensor.from([1.0, 2.0, 3.0, 4.0], [2, 2])

fn main()
    a := half()
    print(a * 2.0 + a)             # tensor.from([3.0, 6.0, 9.0, 12.0], [2, 2]).to_f32()
    print(a.to_list())             # [1.0, 2.0, 3.0, 4.0]
    print(a.to_float().sum())      # 10.0
    print(a.map(|x| x * x).sum())  # 30.0
```

**std.tensor.23** — `t.top2(axis)` on a `float` or `f32` tensor gives three
tensors of the shape without `axis`: the index of each row's best element, the
index of its second-best, and the margin (best minus second) in `t`'s element
type. The best is the first largest element, and a tie with it is the second,
so its margin is `0`; a NaN never replaces either. A NaN best, or a row without
a second value, traps. An `int` tensor is a compile-time error, and an `axis`
with fewer than two elements traps. *(ADR 0813)*

```ail rules=std.tensor.23
fn main()
    s := tensor.from([0.1, 0.9, 0.4, 0.7, 0.7, 0.3], [2, 3])
    best, second, margin := s.top2(1)
    print(best)      # tensor.from([1, 0])
    print(second)    # tensor.from([2, 1])
    print(margin)    # the margins 0.5 and 0.0, up to rounding
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

```ail rules=std.ui.25,std.ui.26,std.ui.27,std.ui.28
use std.ui
use std.ui.widgets

fn press(k str) ui.Event
    Key(key: ui.key_code(k), ctrl: false, alt: false, shift: false)

fn main()
    theme := Map[str, int]()
    area := widgets.Rect(col: 0, row: 0, cols: 20, rows: 4)
    rows := area.rows_of([1, -1, 1])
    print(rows.len())                              # 3: header, body, footer
    halves := rows[1].cols_of([-1, -1])
    print(halves[0].cols + halves[1].cols)         # 20
    print(area.left(30).cols)                      # 20: clamped
    print(area.inset(1).rows)                      # 2
    print(widgets.Label(text: "hi", bold: true).render(rows[0], theme).len())
    mut box := widgets.Checkbox(label: "on", checked: false, focused: true)
    space := press(" ")
    print(box.on_event(space, rows[2]))            # Changed
    print(box.checked)                             # true
    mut radio := widgets.RadioGroup.new(["a", "b"])
    radio.selected = 1
    print(radio.chosen() else "")                  # b
    print(widgets.ProgressBar(done: 1, total: 4).percent())   # 25
    tabs := widgets.Tabs.new(["one", "two"])
    print(widgets.text_rows(tabs.render(rows[0], theme), 20, 1)[0])
```

```ail rules=std.tensor.6
fn main()
    xs := tensor.from([1, 2, 3], [3])
    print(xs.sum())                 # 6: int sums add in index order
    print(xs.dot(xs))               # 14
    print(xs.add(xs).to_list())     # [2, 4, 6]
```

## Examples for the hash, compression, date, zone, net and process rules

```ail rules=std.hash.1,std.hash.2,std.hash.3,std.hash.4,std.hash.5,std.hash.6
use std.hash

fn main()
    print(hash.sha256("abc"))         # ba7816bf...: the FIPS 180-4 digest
    print(hash.sha384("abc").len())   # 96 hex digits
    print(hash.sha512("abc").len())   # 128 hex digits
    print(hash.hmac_sha256("key", "The quick brown fox jumps over the lazy dog").len())   # 64
    print(hash.hmac_sha384("Jefe", "what do ya want for nothing?").len())   # 96 (RFC 4231 case 2)
    print(hash.hmac_sha512("Jefe", "what do ya want for nothing?").len())   # 128
    print(hash.sha1_legacy("abc"))           # a9993e364706816aba3e25717850c26c9cd0d89d
    print(hash.md5_legacy("abc"))            # 900150983cd24fb0d6963f7d28e17f72
    print(hash.sha3_256("abc").len())   # 64
    print(hash.sha3_512("abc").len())   # 128
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
    print(tz.system().len() > 0)                     # "UTC" on Windows unless TZ names a zone
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

## Tool servers: `std.mcp`

**std.mcp.1** — `use std.mcp` provides `connect(cmd str, args List[str]) Client !
McpError`. It starts the program as `proc.start` does (`std.proc.4`) and speaks
MCP (the Model Context Protocol) to it as JSON-RPC 2.0 over its standard input
and output, one JSON message per line. A `Client` is a unique value
(`own.uniq.1`) that owns the server process; the program is killed when the
client is released (`std.proc.7`). Tests need no network: the server is any
program that speaks the protocol. *(ADR 1091)*

**std.mcp.2** — `connect` sends `initialize` with protocol revision
`2025-03-26`, waits for the reply, then sends the `notifications/initialized`
notification. A server that can't start fails with kind `Start`. *(ADR 1091)*

**std.mcp.3** — `c.tools() List[Tool] ! McpError` is the server's `tools/list`,
following `nextCursor` until there is none. `struct Tool { name str,
description str, schema Json }`; `description` is `""` and `schema` is `Null`
when the server gives none. *(ADR 1091)*

**std.mcp.4** — `c.call(name str, args Json) Json ! McpError` is `tools/call`
with `args` (a `Json` object) as the arguments. It returns the reply's
`structuredContent` when it has one, and otherwise the whole `result` object.
*(ADR 1091)*

**std.mcp.5** — `mcp.text(result Json) str` joins the `text` blocks of a result's
`content` with newlines. A result whose `isError` is true fails `call` with kind
`Tool` and that text as the message. *(ADR 1091)*

**std.mcp.6** — A line from the server that is not JSON, has no `id` or has an
`id` other than the pending request's is skipped, so notifications and log
lines don't disturb a call. A reply with an `error` member fails with kind
`Rpc`; the message is the server's text and `(code N)`. *(ADR 1091)*

**std.mcp.7** — `McpError` is an error with `kind McpKind` and `message str`
(ADR 1068). `enum McpKind` has the variants `Start`, `Rpc`, `Tool` and
`Timeout`; a server that has exited fails with `Timeout`, not a kind of its
own. A request
waits at most 30 seconds for its reply (`c.set_timeout(ms int)` changes it);
when the time runs out or the server has exited it fails with kind `Timeout`,
and the message ends with ` (server stderr: ...)`, the first ten lines the
server wrote to its standard error joined with ` | `, when there are any.
*(ADR 1091)*

**std.mcp.8** — `c.close()` takes the client with `move` and closes the server's
standard input; the server is then killed as in `std.mcp.1`. *(ADR 1091)*

```ail norun rules=std.mcp.1,std.mcp.2,std.mcp.3,std.mcp.4,std.mcp.5,std.mcp.6,std.mcp.7,std.mcp.8
use std.mcp

fn main() !
    mut c := mcp.connect("my-mcp-server", ["--stdio"])?
    for t in c.tools()?
        print("{t.name}: {t.description}")
    args := Json.Obj([JsonMember(key: "text", value: Json.Str("hi"))])
    r := c.call("echo", args) else err
        print("{err.kind}: {err.message}")
        return
    print(mcp.text(r))
    c.close()
```
