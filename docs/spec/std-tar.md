# Standard library: `std.tar`

Chapter key: `std`

> **Status:** Drafted. Specified and implemented in AIL (ADR 1251). Reads and writes tar archives, plain or gzipped, in
> memory and as files. It does not write sparse files, devices or multi-volume archives, and it does not read them
> (`std.tar.4`).

## The module

**std.tar.1** — `std.tar` provides `read`, `write`, `list`, `extract`, `create`, the constructors `new_file`,
`new_dir` and `new_symlink`, and the types `Entry`, `EntryKind`, `TarError` and `TarKind`. Details: [below](#more-on-stdtar1). *(ADR 1251)*

**std.tar.2** — An archive in memory is a `List[Entry]`. Details: [below](#more-on-stdtar2). *(ADR 1251)*

```ail rules=std.tar.1,std.tar.2
fn main() !
    es := [
        tar.new_dir("docs"),
        tar.new_file("docs/a.txt", "hello\n".to_bytes()),
        tar.new_symlink("docs/latest", "a.txt"),
    ]
    for e in es
        print("{e.name} {e.kind.name()} {e.mode} [{e.link}]")
    # docs/ Dir 493 []
    # docs/a.txt File 420 []
    # docs/latest Symlink 511 [a.txt]
```

**std.tar.3** — `write(entries List[Entry], gzip bool = false) bytes ! TarError` gives the archive of `entries` in
POSIX ustar: for each entry a 512-byte header, then a file's content padded with zero bytes to a multiple of 512;
after the last entry, two zero blocks. The result depends on the entries alone: user and group ids are 0, their names
empty, and the checksum is the sum of the header's bytes with the checksum field read as spaces. A name of more than
100 bytes is cut at a `/` into the header's `prefix` (at most 155 bytes) and `name` (at most 100) when that leaves
both non-empty; otherwise, and for a link target of more than 100 bytes, a pax extended header (type `x`) comes first
and holds the whole `path` or `linkpath`. A `mode` is cut to its low twelve bits and an `mtime` is held between 0 and
8589934591. `write` does not check a name: `../x` and `/x` are written as given, and `extract` refuses them
(`std.tar.9`). It fails with `BadEntry` for an empty name or a link with no target, with `Unsupported` for a `Special`
entry, and with `TooBig` for a file of 8 GiB or more or more than 2^30 bytes of content in all. *(ADR 1251)*

```ail rules=std.tar.3
fn main() !
    raw := tar.write([tar.new_file("a.txt", "hi\n".to_bytes())])?
    print(raw.len())                 # 2048: header, content block, two end blocks
    print(raw[0..5].text())          # a.txt
    print(raw[257..262].text())      # ustar
    long := tar.new_file("{"d".repeat(60)}/{"e".repeat(60)}/f.txt", b"x")
    print(tar.write([long])?.len())  # 2048: the name is cut at a `/`
    back := tar.read(tar.write([long])?)?
    print(back[0].name.len())        # 127
```

**std.tar.4** — `read(data bytes) List[Entry] ! TarError` reads ustar (with `prefix`), pax (`x` records `path`,
`linkpath`, `size` and `mtime`, the time's fraction dropped; `g` records are skipped), the GNU long name and long
link records (`L` and `K`) and old V7 archives (a name ending in `/` is a directory). A number may be octal or the GNU
base-256 form. Every header's checksum is checked (the unsigned or the signed sum). The archive ends at the first zero
block; an archive that ends with no zero block is read as well. A name or link that is not UTF-8 fails with
`Unsupported`. Types `0`, `7` and NUL are files, `5` a directory, `2` a symbolic link and `1` a hard link; any other
type is a `Special` entry with no content. *(ADR 1251)*

```ail rules=std.tar.4
fn main() !
    raw := tar.write([tar.new_file("a.txt", "hi\n".to_bytes()), tar.new_dir("d")])?
    for e in tar.read(raw)?
        print("{e.name} {e.kind.name()} {e.data.len()}")
    # a.txt File 3
    # d/ Dir 0
```

**std.tar.5** — `read` takes gzip data as well: when `data` starts with the gzip magic `1f 8b` it is gunzipped first
(`std.compress.2`), and a failure there is a `TarError` of kind `TooBig` (`std.compress.4`) or `Corrupt`, with the
message `gzip: ` and the compression error's. `write(entries, gzip: true)` returns the archive gzipped (`std.compress.1`,
so a `.tar.gz`). *(ADR 1251)*

```ail rules=std.tar.5
fn main() !
    es := [tar.new_file("a.txt", "hi\n".to_bytes())]
    packed := tar.write(es, gzip: true)?
    print(packed[0..2].hex())        # 1f8b
    print(tar.read(packed)? == es)   # true
```

**std.tar.6** — `TarError` has the fields `kind TarKind` and `why str`; its text is `why`. Details: [below](#more-on-stdtar6). *(ADR 1251)*

```ail rules=std.tar.6
fn main()
    es := tar.read(b"this is not an archive, and it is not even 512 bytes long") else err
        print("{err.kind.name()}: {err}")
        return
    print(es.len())
```

**std.tar.7** — A read or a write takes at most 2^30 bytes of content in all and fails with `TooBig` past that, before
it allocates more. An entry whose content runs past the end of the data, and a header cut short, fail with `Corrupt`
(`the archive ends inside an entry`, `the archive ends inside a header`). *(ADR 1251)*

```ail rules=std.tar.7
fn main() !
    raw := tar.write([tar.new_file("big", "y".repeat(600).to_bytes())])?
    _ := tar.read(raw[0..700]) else err
        print(err)                   # the archive ends inside an entry
        return
```

**std.tar.8** — `list(archive str) List[str] ! TarError | fs.IoError` returns the entry names of the archive file, in
order (a directory's ends with `/`), gzipped or not. It needs the `fs` effect. *(ADR 1251)*

```ail rules=std.tar.8
fn main() !
    base := fs.make_temp_dir("ail-tar-doc-")?
    arch := "{base}/a.tar"
    fs.write_bytes(arch, tar.write([tar.new_dir("d"), tar.new_file("d/f", b"x")])?)?
    print(tar.list(arch)?)           # ["d/", "d/f"]
    fs.remove(arch)?
    fs.remove(base)?
```

**std.tar.9** — `extract(archive str, dir str) ! TarError | fs.IoError` unpacks the archive file into `dir`, which it
creates with its parents. Before it writes anything it checks every entry, and one that fails stops the call with no
file made: a name that is absolute or has a `..`, `.` or empty component, or contains `\` or `:`, fails with
`UnsafePath`; a leading `./` and a trailing `/` are dropped, and the entry `.` or `./` is the directory itself and is
skipped. An entry of kind `Special` fails with `Unsupported`. *(ADR 1251)*

```ail rules=std.tar.9
fn main() !
    base := fs.make_temp_dir("ail-tar-doc-")?
    arch := "{base}/a.tar"
    fs.write_bytes(arch, tar.write([tar.new_file("../evil.txt", b"x")])?)?
    tar.extract(arch, "{base}/out") else err
        print(err)                   # unsafe path "../evil.txt"
        print(fs.exists("{base}/out"))   # false
    fs.remove(arch)?
    fs.remove(base)?
```

**std.tar.10** — A symbolic link is extracted only when its target is written as a relative path that, read as text
from the link's directory (`path.clean`), stays below `dir`; an absolute target, or one that climbs out, fails with
`UnsafePath`, and so does an entry whose path runs through a symbolic link of the same archive. A hard link entry
becomes a copy of the file it names, which must be an earlier entry with a safe name (`NotFound` from `fs.copy`
otherwise). Where the platform cannot make symbolic links the call fails with the I/O kind `permission`
(`std.fs.32`). *(ADR 1251)*

```ail rules=std.tar.10
fn main() !
    base := fs.make_temp_dir("ail-tar-doc-")?
    arch := "{base}/a.tar"
    fs.write_bytes(arch, tar.write([tar.new_symlink("up", "../secret")])?)?
    tar.extract(arch, "{base}/out") else err
        print(err)                   # the link "up" points outside the directory
    fs.remove(arch)?
    fs.remove(base)?
```

**std.tar.11** — `extract` sets a file's permission bits to the low nine bits of its `mode` (no setuid, setgid or
sticky bit) and its modification time to `mtime` seconds when that is above 0; a directory gets the default mode. A
file that exists is overwritten. An entry repeated in the archive is written again, so the last one wins. *(ADR 1251)*

```ail rules=std.tar.11
fn main() !
    base := fs.make_temp_dir("ail-tar-doc-")?
    arch := "{base}/a.tar"
    f := tar.Entry(name: "run.sh", kind: tar.EntryKind.File, mode: 493, mtime: 1700000000, data: b"echo hi\n", link: "")
    fs.write_bytes(arch, tar.write([f])?)?
    tar.extract(arch, "{base}/out")?
    print(fs.mode("{base}/out/run.sh")?)             # 493
    print(fs.metadata("{base}/out/run.sh")?.modified_ms)   # 1700000000000
    fs.remove("{base}/out/run.sh")?
    fs.remove("{base}/out")?
    fs.remove(arch)?
    fs.remove(base)?
```

**std.tar.12** — `create(dir str, out str) ! TarError | fs.IoError` packs everything below `dir` (not `dir` itself)
into the file `out`: a directory as an entry whose name ends with `/`, a file with its content, a symbolic link
as a link entry (not followed). Details: [below](#more-on-stdtar12). *(ADR 1251)*

```ail rules=std.tar.12
fn main() !
    base := fs.make_temp_dir("ail-tar-doc-")?
    fs.make_dir("{base}/tree/sub")?
    fs.write_text("{base}/tree/a.txt", "alpha\n")?
    fs.write_text("{base}/tree/sub/b.txt", "beta\n")?
    tar.create("{base}/tree", "{base}/t.tar")?
    print(tar.list("{base}/t.tar")?)     # ["a.txt", "sub/", "sub/b.txt"]
    for p in ["{base}/tree/sub/b.txt", "{base}/tree/sub", "{base}/tree/a.txt", "{base}/tree", "{base}/t.tar"]
        fs.remove(p)?
    fs.remove(base)?
```

**std.tar.13** — `create` writes the archive gzipped when `out` ends with `.gz` or `.tgz`, and plain otherwise.
`list` and `extract` read either form, by the magic of `std.tar.5` and not by the name. *(ADR 1251)*

```ail rules=std.tar.13
fn main() !
    base := fs.make_temp_dir("ail-tar-doc-")?
    fs.make_dir("{base}/tree")?
    fs.write_text("{base}/tree/a.txt", "alpha\n")?
    tar.create("{base}/tree", "{base}/t.tar.gz")?
    print(fs.read_bytes("{base}/t.tar.gz")?[0..2].hex())   # 1f8b
    tar.extract("{base}/t.tar.gz", "{base}/out")?
    print(fs.read_text("{base}/out/a.txt")?.trim())         # alpha
    for p in ["{base}/out/a.txt", "{base}/out", "{base}/tree/a.txt", "{base}/tree", "{base}/t.tar.gz"]
        fs.remove(p)?
    fs.remove(base)?
```

**std.tar.14** — `read`, `write` and the constructors are pure (`eff.infer.1`) and need no capability; `list`,
`extract` and `create` have the effect `fs` (`eff.infer.4`). *(ADR 1251)*

### `std.tar` reference

Details moved out of the numbered rules above, so `ail explain` stays short.

#### More on std.tar.1

The module is written in AIL over `std.fs`, `std.path` and `std.compress`, and is in scope without `use` (`std.use.4`); `tar.read(...)` names it.

#### More on std.tar.12

Names are relative to `dir`, written with `/`, in the order of `fs.scan` (`std.fs.23`: each directory's names sorted by bytes, a directory before what it holds). Each entry has the mode and the modification time (whole seconds) of the file, user and group ids 0 and no names, so the same tree gives the same archive. The tree is read into memory, so it is held to the limit of `std.tar.7`; `out` should not lie below `dir`.

#### More on std.tar.2

`struct Entry` has the fields `name str` (with `/`; a directory's name ends with `/`), `kind EntryKind`, `mode int` (permission bits, 0 to 4095), `mtime int` (seconds since 1970), `data bytes` (a file's content) and `link str` (the target of a link). `enum EntryKind` has `File`, `Dir`, `Symlink`, `Hardlink` and `Special` (a device, a fifo or anything else). `data` is empty unless the kind is `File`, and `link` is empty unless it is `Symlink` or `Hardlink`. `new_file(name, data)` makes an entry with mode 420 and time 0, `new_dir(name)` one with mode 493 and a `/` added to the name when it has none, and `new_symlink(name, target)` one with mode 511.

#### More on std.tar.6

The kinds are `Corrupt` (not a tar archive, a wrong checksum, a bad number, a truncated archive, bad gzip data), `UnsafePath` (`std.tar.9`, `std.tar.10`), `Unsupported` (an entry or a name the module does not handle), `TooBig` (`std.tar.7`) and `BadEntry` (an entry `write` cannot make).
