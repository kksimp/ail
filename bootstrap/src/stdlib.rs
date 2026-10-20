//! The standard modules `std.fs`, `std.env`, `std.io`, `std.proc`,
//! `std.random`, `std.time` and `std.ui` (spec 17-std, ADRs 0157-0160, 0206, 0177): their names, the functions
//! each provides, and the declarations of their types. The checker types
//! calls to them in `check/stdlib.rs`; the interpreter runs them in
//! `interp/stdlib.rs`.

use crate::ast::{PRELUDE_LINE, Program};
use crate::lexer::lex;

/// Each standard module and its functions (std.use.1-2), sorted.
pub const MODULES: &[(&str, &[&str])] = &[
    ("env", &["args", "cwd", "get", "program"]),
    ("fs", &["append_text", "exists", "list_dir", "make_dir", "read_text", "remove", "write_text"]),
    ("io", &["eprint", "exit", "read_all", "read_line"]),
    ("proc", &["run"]),
    ("random", &["float", "int", "seed", "shuffle"]),
    ("time", &["now_ms", "sleep_ms"]),
    ("ui", &["clipboard_get", "clipboard_set", "open"]),
];

/// The methods of `ui.Window` (std.ui.2-8), sorted, each with whether its
/// receiver is `mut self`.
pub const WINDOW_METHODS: &[(&str, bool)] = &[
    ("clear", true),
    ("close", true),
    ("cursor", true),
    ("next_event", true),
    ("poll_event", true),
    ("modifiers", false),
    ("present", true),
    ("put", true),
    ("set_title", true),
    ("size", false),
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
}

/// Each type of a standard module, declared in `std/types.ail`: its name
/// there, and its module.
const TYPES: &[(&str, &str)] = &[
    ("IoError", "fs"),
    ("Output", "proc"),
    ("ProcError", "proc"),
    ("Window", "ui"),
    ("Size", "ui"),
    ("Modifiers", "ui"),
    ("Event", "ui"),
    ("UiError", "ui"),
];

/// The declarations of the standard modules' types, read with the prelude.
const SOURCE: &str = include_str!("std/types.ail");

/// The lines of `std/types.ail` are numbered from here, inside the prelude's
/// range (`ast::is_prelude`) and after the prelude itself.
const FIRST_LINE: usize = PRELUDE_LINE + (1 << 20);

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
