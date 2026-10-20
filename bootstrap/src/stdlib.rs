//! The standard modules `std.fs`, `std.env`, `std.io`, `std.proc`,
//! `std.net`, `std.path`, `std.random`, `std.tensor`, `std.time` and `std.ui` (spec 17-std, ADRs 0157-0160, 0206, 0177, 0281-0283, 0328): their names, the functions
//! each provides, and the declarations of their types. The checker types
//! calls to them in `check/stdlib.rs`; the interpreter runs them in
//! `interp/stdlib.rs`.

use crate::ast::{PRELUDE_LINE, Program};
use crate::lexer::lex;

/// Each standard module and its functions (std.use.1-2), sorted.
pub const MODULES: &[(&str, &[&str])] = &[
    ("env", &["args", "cwd", "get", "home", "program", "vars"]),
    (
        "fs",
        &[
            "append_text", "copy", "exists", "glob", "list_dir", "make_dir", "metadata", "read_text", "remove", "rename",
            "walk", "write_text",
        ],
    ),
    ("io", &["eprint", "exit", "read_all", "read_line"]),
    ("mem", &["cstr", "free", "from_cstr"]),
    ("net", &["get", "post"]),
    ("path", &["base", "dir", "ext", "join"]),
    ("proc", &["run", "which"]),
    ("random", &["float", "int", "seed", "shuffle"]),
    ("tensor", &["from", "zeros"]),
    ("task", &["cancelled", "check"]),
    ("time", &["now_ms", "sleep_ms"]),
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

/// The methods of `tensor.Tensor[T]` (std.tensor.3-8), sorted. None
/// changes its receiver: a tensor is a value (ADRs 0060-0063).
pub const TENSOR_METHODS: &[&str] =
    &["add", "dot", "len", "map", "matmul", "max", "mean", "mul", "reshape", "shape", "sum", "to_list", "transpose"];

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
    /// `m.NAME`: a constant of a module, such as `ui.RED` (std.ui.17).
    Const(i64),
}

/// Each type of a standard module, declared in `std/types.ail`: its name
/// there, and its module.
const TYPES: &[(&str, &str)] = &[
    ("IoError", "fs"),
    ("Metadata", "fs"),
    ("Response", "net"),
    ("NetError", "net"),
    ("Output", "proc"),
    ("ProcError", "proc"),
    ("Window", "ui"),
    ("Size", "ui"),
    ("Modifiers", "ui"),
    ("Event", "ui"),
    ("Style", "ui"),
    ("Cursor", "ui"),
    ("UiError", "ui"),
    ("Tensor", "tensor"),
];

/// The declarations of the standard modules' types, read with the prelude.
const SOURCE: &str = include_str!("std/types.ail");

/// The lines of `std/types.ail` are numbered from here, inside the prelude's
/// range (`ast::is_prelude`) and after the prelude itself.
const FIRST_LINE: usize = PRELUDE_LINE + (1 << 20);

/// The standard modules written in AIL (std.ui.19): their dotted path, the
/// file name they are read under, and their source.
const AIL_MODULES: &[(&str, &str, &str)] =
    &[("std.ui.widgets", "<std>/ui/widgets.ail", include_str!("std/ui/widgets.ail"))];

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
/// `ProcError.kind` carry it (std.err.1).
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

/// std.path.2: a separator is `/`, and also `\\` on Windows.
fn is_sep(c: char) -> bool {
    c == '/' || (cfg!(windows) && c == '\\')
}

/// std.path.2: the text before and after the last separator; the part
/// before is `None` when there is no separator.
pub fn path_split(p: &str) -> (Option<&str>, &str) {
    match p.rfind(is_sep) {
        Some(i) => (Some(&p[..i]), &p[i + 1..]),
        None => (None, p),
    }
}

/// std.path.2: the directory part: `.` with no separator, the separator
/// itself for a file in the root.
pub fn path_dir(p: &str) -> String {
    match path_split(p) {
        (None, _) => ".".into(),
        (Some(""), _) => p[..1].into(),
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

/// std.path.4: `b` under `a` with one separator between; `b` alone when it
/// is absolute or `a` is empty.
pub fn path_join(a: &str, b: &str) -> String {
    if a.is_empty() || b.starts_with(is_sep) {
        b.into()
    } else if a.ends_with(is_sep) {
        format!("{a}{b}")
    } else {
        format!("{a}/{b}")
    }
}
