//! The standard modules `std.fs`, `std.env`, `std.io`, `std.proc`,
//! `std.net`, `std.path`, `std.random`, `std.tensor`, `std.time` and `std.ui` (spec 17-std, ADRs 0157-0160, 0206, 0177, 0281-0283, 0328): their names, the functions
//! each provides, and the declarations of their types. The checker types
//! calls to them in `check/stdlib.rs`; the interpreter runs them in
//! `interp/stdlib.rs`.

use crate::ast::{PRELUDE_LINE, Program};
use crate::lexer::lex;

/// Each standard module and its functions (std.use.1-2), sorted.
pub const MODULES: &[(&str, &[&str])] = &[
    (
        "compress",
        &["deflate", "gunzip", "gunzip_file", "gzip", "gzip_file", "inflate", "unzip", "unzlib", "zip_dir", "zlib"],
    ),
    ("env", &["args", "cwd", "get", "home", "program", "vars"]),
    (
        "fs",
        &[
            "append_bytes",
            "append_text",
            "copy",
            "exists",
            "glob",
            "list_dir",
            "make_dir",
            "metadata",
            "read_bytes",
            "read_text",
            "remove",
            "rename",
            "walk",
            "write_bytes",
            "write_text",
        ],
    ),
    ("hash", &["crc32", "hash64", "hmac_sha256", "sha256", "sha384", "sha512"]),
    ("mem", &["alloc", "cast", "cstr", "free", "from_cstr", "null"]),
    ("io", &["eprint", "exit", "read_all", "read_bytes", "read_line"]),
    (
        "net",
        &[
            "connect",
            "connect_tls",
            "get",
            "get_bytes",
            "listen",
            "open",
            "options",
            "post",
            "post_bytes",
            "request",
            "request_bytes",
            "serve",
        ],
    ),
    ("path", &["base", "dir", "ext", "join", "stem"]),
    ("proc", &["kill", "run", "run_bytes", "start", "wait", "which"]),
    ("random", &["float", "int", "seed", "shuffle"]),
    (
        "regex",
        &[
            "captures",
            "compile",
            "escape",
            "find",
            "find_all",
            "group_all",
            "is_match",
            "replace",
            "replace_first",
            "split",
        ],
    ),
    ("tensor", &["from", "zeros"]),
    ("task", &["cancelled", "check", "select"]),
    ("time", &["now_ms", "sleep_ms"]),
    ("tz", &["offset", "system"]),
    ("ui", &["clipboard_get", "clipboard_set", "open"]),
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
pub const TENSOR_METHODS: &[&str] = &[
    "add",
    "argmax",
    "argmin",
    "clamp",
    "conv2d",
    "dot",
    "item",
    "len",
    "map",
    "matmul",
    "max",
    "mean",
    "mul",
    "relu",
    "reshape",
    "round",
    "shape",
    "sigmoid",
    "sum",
    "tanh",
    "to_f32",
    "to_float",
    "to_list",
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
