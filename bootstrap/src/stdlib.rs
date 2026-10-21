//! The standard modules `std.fs`, `std.env`, `std.io`, `std.proc`,
//! `std.net`, `std.path`, `std.random`, `std.tensor`, `std.time` and `std.ui` (spec 17-std, ADRs 0157-0160, 0206, 0177, 0281-0283, 0328): their names, the functions
//! each provides, and the declarations of their types. The checker types
//! calls to them in `check/stdlib.rs`; the interpreter runs them in
//! `interp/stdlib.rs`.

use crate::ast::{PRELUDE_LINE, Program};
use crate::lexer::lex;

/// Each standard module and its functions (std.use.1-2), sorted.
pub const MODULES: &[(&str, &[&str])] = &[
    ("compress", &["deflate", "gunzip", "gunzip_file", "gzip", "gzip_file", "inflate", "unzip", "unzlib", "zip_dir", "zlib"]),
    ("env", &["args", "cwd", "get", "home", "program", "vars"]),
    (
        "fs",
        &[
            "append_bytes", "append_text", "copy", "exists", "glob", "list_dir", "make_dir", "metadata", "read_bytes",
            "read_text", "remove", "rename", "walk", "write_bytes", "write_text",
        ],
    ),
    ("hash", &["crc32", "hash64", "hmac_sha256", "sha256", "sha384", "sha512"]),
    ("mem", &["alloc", "cast", "cstr", "free", "from_cstr", "null"]),
    ("io", &["eprint", "exit", "read_all", "read_bytes", "read_line"]),
    ("net", &["connect", "connect_tls", "get", "get_bytes", "listen", "open", "options", "post", "post_bytes", "request", "request_bytes", "serve"]),
    ("path", &["base", "dir", "ext", "join", "stem"]),
    ("proc", &["kill", "run", "run_bytes", "start", "wait", "which"]),
    ("random", &["float", "int", "seed", "shuffle"]),
    ("regex", &["captures", "compile", "escape", "find", "find_all", "group_all", "is_match", "replace", "replace_first", "split"]),
    ("tensor", &["from", "zeros"]),
    ("task", &["cancelled", "check", "select"]),
    ("time", &["now_ms", "sleep_ms"]),
    ("tz", &["offset", "system"]),
    ("ui", &["clipboard_get", "clipboard_set", "open"]),
];

/// One sentence of documentation for each function of `MODULES`, keyed
/// `module.name`, sorted. The functions are built into the interpreter, so there
/// is no AIL source to carry a `##` line; `builtin_docs_cover_modules` fails when
/// a function of `MODULES` has no entry here or an entry names no function.
#[allow(dead_code)] // read by the test until the card lists builtins
pub const BUILTIN_DOCS: &[(&str, &str)] = &[
    ("compress.deflate", "Compresses `data` into a raw DEFLATE stream at level 6."),
    ("compress.gunzip", "Decompresses a gzip stream made by `gzip`."),
    ("compress.gunzip_file", "Writes the decompressed contents of the gzip file `src` to `dst`."),
    ("compress.gzip", "Compresses `data` into a gzip stream at level 6."),
    ("compress.gzip_file", "Writes the gzip compression of the file `src` to `dst`."),
    ("compress.inflate", "Decompresses a raw DEFLATE stream made by `deflate`."),
    ("compress.unzip", "Extracts the zip archive `archive` into the directory `dir`."),
    ("compress.unzlib", "Decompresses a zlib stream made by `zlib`."),
    ("compress.zip_dir", "Writes a zip archive of the directory `dir` to `dst`."),
    ("compress.zlib", "Compresses `data` with the zlib wrapper of RFC 1950, as PDF Flate streams use."),
    ("env.args", "Returns the program's arguments, the words after its file."),
    ("env.cwd", "Returns the working directory; the program traps if it can't be read."),
    ("env.get", "Returns the environment variable `name`, or `none` when it is unset or not UTF-8."),
    ("env.home", "Returns the home directory from `HOME` (or `USERPROFILE` on Windows), or `none`."),
    ("env.program", "Returns the program's file path as given to `ail run`."),
    ("env.vars", "Returns the environment variables whose names and values are UTF-8, in name order."),
    ("fs.append_bytes", "Adds `data` to the end of the file at `path`, creating the file if needed."),
    ("fs.append_text", "Adds `text` to the end of the file at `path`, creating the file if needed."),
    ("fs.copy", "Copies the file or directory tree `from` to `to`, replacing a file already at `to`."),
    ("fs.exists", "Returns whether anything exists at `path`."),
    ("fs.glob", "Returns the existing paths that match `pattern` (with `*`, `?` and `**`), sorted by bytes."),
    ("fs.list_dir", "Returns the names, not paths, of a directory's entries, sorted by bytes."),
    ("fs.make_dir", "Creates a directory at `path` and any missing parents; an existing directory is not a failure."),
    ("fs.metadata", "Returns the size, the modification time in milliseconds since 1970, and whether `path` is a directory."),
    ("fs.read_bytes", "Returns the whole file at `path` as `bytes`, whatever it holds."),
    ("fs.read_text", "Returns the whole file at `path` as text."),
    ("fs.remove", "Removes a file or an empty directory at `path`."),
    ("fs.rename", "Moves the file or directory `from` to `to`."),
    ("fs.walk", "Returns every file below `dir`."),
    ("fs.write_bytes", "Creates or replaces the file at `path` with `data`."),
    ("fs.write_text", "Creates the file at `path` with `text`, or replaces its contents."),
    ("hash.crc32", "Returns the CRC-32 of `s` as an int from 0 to 2^32 - 1."),
    ("hash.hash64", "Returns a fast 64-bit hash of `s`, the same on every platform, not for security."),
    ("hash.hmac_sha256", "Returns the HMAC-SHA-256 of `s` under `key` as lower-case hex."),
    ("hash.sha256", "Returns the SHA-256 digest of `s` as lower-case hex."),
    ("hash.sha384", "Returns the SHA-384 digest of `s` as lower-case hex."),
    ("hash.sha512", "Returns the SHA-512 digest of `s` as lower-case hex."),
    ("io.eprint", "Writes the value's text and a newline to standard error."),
    ("io.exit", "Ends the program with exit status `code` after writing pending output."),
    ("io.read_all", "Reads the rest of standard input."),
    ("io.read_bytes", "Reads the rest of standard input as `bytes`, undecoded."),
    ("io.read_line", "Reads one line from standard input without its line ending, or returns `none` at end of input."),
    ("mem.alloc", "Returns a pointer to `n` elements of the example's type, each set to the example's value."),
    ("mem.cast", "Returns the address `p` as a pointer to the example's type."),
    ("mem.cstr", "Returns a new NUL-terminated copy of `s` that the program frees itself."),
    ("mem.free", "Frees a block that `mem.alloc` made, and traps on a null pointer or a double free."),
    ("mem.from_cstr", "Reads the NUL-terminated string at `p`; invalid UTF-8 becomes U+FFFD."),
    ("mem.null", "Returns the null `*u8` pointer."),
    ("net.connect", "Opens a raw TCP connection to `host` on `port`."),
    ("net.connect_tls", "Opens a TCP connection to `host` on `port` and runs a TLS 1.3 handshake."),
    ("net.get", "Sends an HTTP GET to `url` and returns the status, headers and body."),
    ("net.get_bytes", "Is `get` with the body as `bytes`."),
    ("net.listen", "Starts a raw TCP listener on `addr`."),
    ("net.open", "Sends a request and returns the response body as a stream."),
    ("net.options", "Returns a default `Options` value for `request` and `open`."),
    ("net.post", "Sends `body` as an HTTP POST with that content type, without following redirects."),
    ("net.post_bytes", "Is `post` with the body as `bytes`."),
    ("net.request", "Sends any method to `url` with the headers, query, body and limits in `opts`."),
    ("net.request_bytes", "Is `request` with the body as `bytes`."),
    ("net.serve", "Listens for HTTP on `addr`, where port 0 picks a free port."),
    ("path.base", "Returns the last component of `p`."),
    ("path.dir", "Returns the text before the last separator, `.` when there is none."),
    ("path.ext", "Returns the extension of the base name, without the dot, or `\"\"`."),
    ("path.join", "Joins `a` and `b` with one `/`, or returns `b` when `b` is absolute or `a` is empty."),
    ("path.stem", "Returns the base name without its extension."),
    ("proc.kill", "Ends the child and returns its status."),
    ("proc.run", "Runs `cmd` with `args` and the optional `input` and `timeout_ms`, without a shell, waits for it, and returns its status and output."),
    ("proc.run_bytes", "Is `run` with `bytes` as input and output."),
    ("proc.start", "Starts `cmd` as `run` does and returns at once with a `Child`."),
    ("proc.wait", "Waits for the child to exit and returns its status."),
    ("proc.which", "Returns the path of the program `run` would start, or `none`."),
    ("random.float", "Returns a uniformly distributed float."),
    ("random.int", "Returns a uniform integer `n` with `lo <= n <= hi`, and traps if `lo > hi`."),
    ("random.seed", "Sets the generator state from `s`, so later draws repeat for the same calls."),
    ("random.shuffle", "Reorders `xs` in place so every order is equally likely."),
    ("regex.captures", "Returns the first match of `re` in `text` with its groups."),
    ("regex.compile", "Checks the pattern `pat` and returns a `Regex` that holds it in its field `pat`."),
    ("regex.escape", "Puts a backslash before each character that is special in a pattern."),
    ("regex.find", "Returns the leftmost match of `re` in `text`, or `none`."),
    ("regex.find_all", "Returns every match of `re` in `text`, left to right."),
    ("regex.group_all", "Returns the text of group `group` in each match of `re` in `text`."),
    ("regex.is_match", "Returns whether `re` matches anywhere in `text`."),
    ("regex.replace", "Replaces every match in `text` with `rep`, where `$1`, `${name}` and similar stand for groups."),
    ("regex.replace_first", "Replaces only the first match in `text` with `rep`."),
    ("regex.split", "Returns the pieces of `text` between the matches of `re`."),
    ("task.cancelled", "Returns whether the running task was cancelled; `false` in the main task."),
    ("task.check", "Returns an error when the running task was cancelled, so `task.check()?` stops it at that point."),
    ("task.select", "Waits until one of the channels in `chans` is ready and returns its index."),
    ("tensor.from", "Makes a tensor from `xs`, 1-D, or with `shape` when given."),
    ("tensor.zeros", "Returns a tensor of `shape` with every element zero."),
    ("time.now_ms", "Returns the current time in milliseconds since 1970-01-01T00:00:00Z."),
    ("time.sleep_ms", "Pauses the program for at least `ms` milliseconds."),
    ("tz.offset", "Returns the UTC offset in seconds of the IANA zone at `secs`, or `none` when the zone is unknown."),
    ("tz.system", "Returns the system zone name, from `TZ` when it names a safe zone."),
    ("ui.clipboard_get", "Returns the clipboard text, or `none` when the clipboard holds no text."),
    ("ui.clipboard_set", "Puts `text` on the clipboard."),
    ("ui.open", "Opens a window of `cols` by `rows` cells, every cell blank."),
];

/// The methods of `ui.Window` (std.ui.2-8), sorted, each with whether its
/// receiver is `mut self`.
pub const WINDOW_METHODS: &[(&str, bool)] = &[
    ("clear", true),
    ("close", true),
    ("cursor", true),
    ("font_size", false),
    ("modifiers", false),
    ("next_event", true),
    ("poll_event", true),
    ("present", true),
    ("put", true),
    ("put_styled", true),
    ("set_cursor", true),
    ("set_font_size", true),
    ("set_title", true),
    ("size", false),
];

/// The methods of the `std.net` handle types (std.net.9-12): the type, the
/// method, and the name `StdCall::Method` carries to the interpreter.
pub const NET_METHODS: &[(&str, &str, &str)] = &[
    ("Server", "accept", "net.Server.accept"),
    ("Server", "addr", "net.Server.addr"),
    ("Server", "close", "net.Server.close"),
    ("Server", "shutdown", "net.Server.shutdown"),
    ("Request", "stop", "net.Request.stop"),
    ("Exchange", "respond", "net.Exchange.respond"),
    ("Exchange", "respond_bytes", "net.Exchange.respond_bytes"),
    ("Listener", "accept", "net.Listener.accept"),
    ("Listener", "addr", "net.Listener.addr"),
    ("Listener", "close", "net.Listener.close"),
    ("Conn", "close", "net.Conn.close"),
    ("Body", "close", "net.Body.close"),
    ("Body", "read_bytes", "net.Body.read_bytes"),
    ("Conn", "peer", "net.Conn.peer"),
    ("Conn", "read", "net.Conn.read"),
    ("Conn", "read_bytes", "net.Conn.read_bytes"),
    ("Conn", "read_line", "net.Conn.read_line"),
    ("Conn", "set_timeout", "net.Conn.set_timeout"),
    ("Conn", "write", "net.Conn.write"),
    ("Conn", "write_bytes", "net.Conn.write_bytes"),
];

/// The methods of `proc.Output` (std.proc.9), sorted. Neither changes its receiver.
pub const OUTPUT_METHODS: &[&str] = &["ok", "text"];

/// The methods of `tensor.Tensor[T]` (std.tensor.3-8), sorted. None
/// changes its receiver: a tensor is a value (ADRs 0060-0063).
pub const TENSOR_METHODS: &[&str] =
    &[
        "add", "argmax", "argmin", "clamp", "conv2d", "dot", "item", "len", "map", "matmul", "max", "mean", "mul", "relu",
        "reshape", "round", "shape", "sigmoid", "sum", "tanh", "to_f32", "to_float", "to_list", "top2", "transpose",
        "reshape", "round", "shape", "sigmoid", "softmax", "sum", "tanh", "to_f32", "to_float", "to_list",
        "transpose",
    ];

/// A call the checker resolved to a standard module (std.use.2), by the
/// position of the function's name.
#[derive(Debug, Clone, Copy)]
pub enum StdCall {
    /// `m.f(...)`: module and function.
    Fn(&'static str, &'static str),
    /// `m.Type(...)`: a struct literal of a module's type, by struct index.
    Type(usize),
    /// `w.m(...)` on a `ui.Window` (std.ui.2-8): the method's name.
    Method(&'static str),
    /// `t.m(...)` on a `tensor.Tensor` (std.tensor.3-8): the method's name.
    Tensor(&'static str),
    /// `o.m()` on a `proc.Output` (std.proc.9-10): the method's name.
    Output(&'static str),
    /// `m.NAME`: a constant of a module, such as `ui.RED` (std.ui.17).
    Const(i64),
    /// `p.m(...)` on a raw pointer (unsafe.ptr.3-5): the method's name and
    /// the type the pointer points to.
    Ptr(&'static str, PtrElem),
}

/// What a raw pointer points to (unsafe.ptr.1), as memory holds it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PtrElem {
    /// `int`: 8 bytes.
    Int,
    /// `float`: 8 bytes.
    Float,
    /// `bool`: 1 byte.
    Bool,
    /// A sized number: its width.
    Num(crate::num::Num),
    /// Another pointer: 8 bytes.
    Ptr,
}

/// Each type of a standard module, declared in `std/types.ail`: its name
/// there, and its module.
const TYPES: &[(&str, &str)] = &[
    ("CompressError", "compress"),
    ("IoError", "fs"),
    ("Metadata", "fs"),
    ("Response", "net"),
    ("RawResponse", "net"),
    ("RawOutput", "proc"),
    ("NetError", "net"),
    ("Options", "net"),
    ("Request", "net"),
    ("Server", "net"),
    ("Exchange", "net"),
    ("Body", "net"),
    ("Listener", "net"),
    ("Conn", "net"),
    ("Output", "proc"),
    ("Child", "proc"),
    ("ProcError", "proc"),
    ("Window", "ui"),
    ("Size", "ui"),
    ("Modifiers", "ui"),
    ("Event", "ui"),
    ("Style", "ui"),
    ("Cursor", "ui"),
    ("UiError", "ui"),
    ("Tensor", "tensor"),
    ("Regex", "regex"),
    ("RegexError", "regex"),
    ("Match", "regex"),
    ("Captures", "regex"),
];

/// The declarations of the standard modules' types, read with the prelude.
const SOURCE: &str = include_str!("std/types.ail");

/// The lines of `std/types.ail` are numbered from here, inside the prelude's
/// range (`ast::is_prelude`) and after the prelude itself.
const FIRST_LINE: usize = PRELUDE_LINE + (1 << 20);

/// The standard modules written in AIL (std.ui.19): their dotted path, the
/// file name they are read under, and their source.
const AIL_MODULES: &[(&str, &str, &str)] = &[
    ("std.args", "<std>/args/args.ail", include_str!("std/args/args.ail")),
    ("std.csv", "<std>/csv/csv.ail", include_str!("std/csv/csv.ail")),
    ("std.log", "<std>/log/log.ail", include_str!("std/log/log.ail")),
    ("std.net.http", "<std>/net/http.ail", include_str!("std/net/http.ail")),
    ("std.net.jar", "<std>/net/jar.ail", include_str!("std/net/jar.ail")),
    ("std.time.date", "<std>/time/date.ail", include_str!("std/time/date.ail")),
    ("std.ui.widgets", "<std>/ui/widgets.ail", include_str!("std/ui/widgets.ail")),
];

/// The file name of the standard module in AIL at `dotted`, if there is one.
pub fn ail_module_path(dotted: &str) -> Option<&'static str> {
    AIL_MODULES.iter().find(|(d, _, _)| *d == dotted).map(|(_, p, _)| *p)
}

/// The source of a standard module in AIL, by its file name.
pub fn ail_module_source(path: &str) -> Option<&'static str> {
    AIL_MODULES.iter().find(|(_, p, _)| *p == path).map(|(_, _, s)| *s)
}

/// Whether `std.NAME` is a standard module (std.use.1).
pub fn is_std_module(name: &str) -> bool {
    MODULES.iter().any(|(m, _)| *m == name)
}

/// The functions of standard module `module`.
pub fn functions(module: &str) -> &'static [&'static str] {
    MODULES.iter().find(|(m, _)| *m == module).map_or(&[], |(_, f)| f)
}

/// The names of the standard modules, for diagnostics.
pub fn module_names() -> Vec<&'static str> {
    MODULES.iter().map(|(m, _)| *m).collect()
}

/// The standard module that `name` refers to in `prog`: the module imported
/// under that name with `use std.MODULE` or `use std.MODULE as name`.
pub fn imported(prog: &Program, name: &str) -> Option<&'static str> {
    prog.uses.iter().find_map(|u| match u.path.as_slice() {
        [(root, _), (module, _)] if root == "std" && u.name().0 == name => {
            MODULES.iter().find(|(m, _)| m == module).map(|(m, _)| *m)
        }
        _ => None,
    })
}

/// Add the standard modules' types to `prog`, each named `module.Type`
/// (std.use.2). Their names can't clash with a program's, which has no `.`.
pub fn add_declarations(prog: &mut Program) {
    let (mut tokens, diags) = lex(SOURCE);
    debug_assert!(diags.is_empty(), "{diags:?}");
    for t in &mut tokens {
        t.pos.line += FIRST_LINE;
    }
    let (std, diags) = crate::parser::parse(tokens);
    debug_assert!(diags.is_empty(), "{diags:?}");
    for mut s in std.structs {
        let module = TYPES.iter().find(|(n, _)| *n == s.name).map(|(_, m)| *m).expect("each std type has a module");
        s.name = format!("{module}.{}", s.name);
        prog.structs.push(s);
    }
    for mut e in std.enums {
        let module = TYPES.iter().find(|(n, _)| *n == e.name).map(|(_, m)| *m).expect("each std type has a module");
        e.name = format!("{module}.{}", e.name);
        prog.enums.push(e);
    }
}

/// The short `kind` of an I/O failure, as `IoError.kind` and
/// `ProcError.kind` carry it (std.err.1). The causes are std.fs.15
/// (`not_found`), std.fs.16 (`is_dir`), std.fs.17 (`not_dir`), std.fs.18
/// (`not_empty`) and std.fs.19 (`invalid_utf8`).
/// std.fs.20 (`exists`), std.fs.21 (`permission`) and std.fs.22 (`other`) close the list.
pub fn error_kind(e: &std::io::Error) -> &'static str {
    use std::io::ErrorKind as K;
    match e.kind() {
        K::NotFound => "not_found",
        K::PermissionDenied => "permission",
        K::AlreadyExists => "exists",
        K::IsADirectory => "is_dir",
        K::NotADirectory => "not_dir",
        K::DirectoryNotEmpty => "not_empty",
        K::InvalidData => "invalid_utf8",
        _ => "other",
    }
}

/// std.path.2: a separator is `/`, and also `\` on Windows. The platform is
/// the caller's choice, so tests check both rules on any host.
pub fn is_sep_in(c: char, windows: bool) -> bool {
    c == '/' || (windows && c == '\\')
}

/// std.path.2, std.path.4: a Windows drive and its colon at the start of `p`
/// (`C:`), which on Windows is absolute only when a separator follows it.
fn drive_absolute_in(p: &str, windows: bool) -> bool {
    windows && matches!(p.as_bytes(), [d, b':', s, ..] if d.is_ascii_alphabetic() && is_sep_in(*s as char, true))
}

/// std.path.2: the text before and after the last separator; the part
/// before is `None` when there is no separator.
pub fn path_split(p: &str) -> (Option<&str>, &str) {
    path_split_in(p, cfg!(windows))
}

fn path_split_in(p: &str, windows: bool) -> (Option<&str>, &str) {
    match p.rfind(|c| is_sep_in(c, windows)) {
        Some(i) => (Some(&p[..i]), &p[i + 1..]),
        None => (None, p),
    }
}

/// std.path.2: the directory part: `.` with no separator, the separator
/// itself for a file in the root, and a drive's root with its separator
/// (`dir("C:\x")` is `C:\`).
pub fn path_dir(p: &str) -> String {
    path_dir_in(p, cfg!(windows))
}

fn path_dir_in(p: &str, windows: bool) -> String {
    match path_split_in(p, windows) {
        (None, _) => ".".into(),
        (Some(""), _) => p[..1].into(),
        (Some(d), _) if windows && d.len() == 2 && d.as_bytes()[1] == b':' && d.as_bytes()[0].is_ascii_alphabetic() => {
            p[..3].into()
        }
        (Some(d), _) => d.into(),
    }
}

/// std.path.3: the base name's text after its last `.`, or `""` when it has
/// none (a leading `.` doesn't count).
pub fn path_ext(p: &str) -> String {
    let base = path_split(p).1;
    match base.rfind('.') {
        Some(i) if i > 0 => base[i + 1..].into(),
        _ => String::new(),
    }
}

/// std.path.5: the base name without its extension (the text before its last
/// `.`; a leading `.` doesn't count).
pub fn path_stem(p: &str) -> String {
    let base = path_split(p).1;
    match base.rfind('.') {
        Some(i) if i > 0 => base[..i].into(),
        _ => base.into(),
    }
}

/// std.path.4: `b` under `a` with one separator between; `b` alone when it
/// is absolute (a drive path counts on Windows) or `a` is empty.
pub fn path_join(a: &str, b: &str) -> String {
    path_join_in(a, b, cfg!(windows))
}

fn path_join_in(a: &str, b: &str, windows: bool) -> String {
    if a.is_empty() || b.starts_with(|c| is_sep_in(c, windows)) || drive_absolute_in(b, windows) {
        b.into()
    } else if a.ends_with(|c| is_sep_in(c, windows)) {
        format!("{a}{b}")
    } else {
        format!("{a}/{b}")
    }
}

#[cfg(test)]
mod path_tests {
    use super::*;

    #[test]
    fn windows_drives_and_separators() {
        assert_eq!(path_dir_in("C:\\x\\y.ail", true), "C:\\x");
        assert_eq!(path_dir_in("C:\\x", true), "C:\\");
        assert_eq!(path_dir_in("C:\\", true), "C:\\");
        assert_eq!(path_dir_in("\\x", true), "\\");
        assert_eq!(path_dir_in("C:x", true), ".");
        assert_eq!(path_split_in("a\\b/c", true), (Some("a\\b"), "c"));
        assert_eq!(path_join_in("C:\\a", "C:\\b", true), "C:\\b");
        assert_eq!(path_join_in("C:\\a", "b", true), "C:\\a/b");
        assert_eq!(path_join_in("C:\\a", "\\b", true), "\\b");
        assert_eq!(path_join_in("C:\\a", "D:x", true), "C:\\a/D:x");
    }

    #[test]
    fn posix_is_unchanged() {
        assert_eq!(path_dir_in("C:\\x", false), ".");
        assert_eq!(path_split_in("a\\b", false), (None, "a\\b"));
        assert_eq!(path_join_in("a", "C:\\b", false), "a/C:\\b");
        assert_eq!(path_join_in("a/", "b", false), "a/b");
        assert_eq!(path_join_in("a", "/b", false), "/b");
    }
}

#[cfg(test)]
mod tests {
    use super::{BUILTIN_DOCS, MODULES};

    #[test]
    fn builtin_docs_cover_modules() {
        let mut want: Vec<String> =
            MODULES.iter().flat_map(|(m, fs)| fs.iter().map(move |f| format!("{m}.{f}"))).collect();
        want.sort();
        let have: Vec<&str> = BUILTIN_DOCS.iter().map(|(k, _)| *k).collect();
        assert!(have.windows(2).all(|w| w[0] < w[1]), "BUILTIN_DOCS is sorted by key, one entry each");
        let missing: Vec<&String> = want.iter().filter(|k| !have.contains(&k.as_str())).collect();
        let extra: Vec<&&str> = have.iter().filter(|k| !want.contains(&k.to_string())).collect();
        assert!(missing.is_empty(), "MODULES functions with no BUILTIN_DOCS entry: {missing:?}");
        assert!(extra.is_empty(), "BUILTIN_DOCS entries that are not in MODULES: {extra:?}");
        for (k, d) in BUILTIN_DOCS {
            assert!(d.ends_with('.') && !d.contains(" — "), "{k}: one plain sentence ending in a period");
        }
    }
}
