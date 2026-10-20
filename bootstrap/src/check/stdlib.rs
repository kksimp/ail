//! The standard modules `std.fs`, `std.env`, `std.io`, `std.proc`,
//! `std.net` and `std.time` (spec 17-std): their imports (std.use.1-3), the
//! types of their functions (std.fs.1-12, std.env.1-5, std.io.1-4,
//! std.proc.1-3, std.net.1-4, std.time.1-2), and their types `fs.IoError`,
//! `proc.Output`, `proc.ProcError` and `net.NetError` (std.err.1).

use std::collections::HashMap;

use super::{Checker, Def, Errs, Sig, Ty, list_names, suggest};
use crate::ast::{Arg, Expr, TypeRef};
use crate::diag::{Diagnostic, Edit, Pos};
use crate::stdlib::{self, StdCall};

impl<'p> Checker<'p> {
    /// std.use.1, std.use.3: each `use std.NAME` names a standard module, and
    /// no two imports share a name.
    pub(super) fn check_uses(&mut self) {
        // Imports apply to their file (mod.use.3).
        let mut seen: HashMap<(&str, &str), Pos> = HashMap::new();
        for u in &self.prog.uses {
            match u.path.as_slice() {
                [(root, _), (module, mpos)] if root == "std" => {
                    if !stdlib::is_std_module(module) {
                        let names = stdlib::module_names();
                        let mut d =
                            Diagnostic::new("std.use.1", *mpos, format!("there is no standard module `std.{module}`"));
                        d = match suggest(module, &names) {
                            Some(s) => d.help(format!("did you mean `use std.{s}`?")),
                            None => d.help(format!("the standard modules are {}", list_names(&names))),
                        };
                        self.diags.push(d);
                        continue;
                    }
                }
                [(root, rpos)] if root == "std" => {
                    self.error_help(
                        "std.use.1",
                        *rpos,
                        "`std` is a collection of modules; import one of them",
                        format!("e.g. `use std.fs`; the standard modules are {}", list_names(&stdlib::module_names())),
                    );
                    continue;
                }
                [(root, _), _, _, ..] if root == "std" && stdlib::ail_module_path(&u.dotted()).is_some() => continue,
                [(root, _), _, _, ..] if root == "std" => {
                    let (_, last) = u.path.last().expect("a path");
                    self.error_help(
                        "std.use.1",
                        *last,
                        "the standard modules have no submodules",
                        format!("the standard modules are {}", list_names(&stdlib::module_names())),
                    );
                    continue;
                }
                // `use a.b`: the program's own modules (`modules.rs`).
                _ => continue,
            }
            let (name, pos) = u.name();
            let file = self.prog.file_at(u.pos).map_or("", |f| f.path.as_str());
            if let Some(first) = seen.get(&(file, name.as_str())) {
                self.diags.push(Diagnostic::new("std.use.3", *pos, format!("`{name}` is already imported")).help(
                    format!(
                        "it is first imported on line {}; remove one import, or import it under another name with `as`",
                        first.line
                    ),
                ));
            } else {
                seen.insert((file, name), *pos);
            }
        }
    }

    /// The standard module that `name` names, if any: the one imported
    /// under `name` (std.use.1), or else the standard module called `name`,
    /// which is in scope without a `use` unless the program declares or
    /// imports something else under that name (std.use.4). Locals are
    /// checked first by the callers, so a local also hides it.
    pub(super) fn std_module(&self, name: &str) -> Option<&'static str> {
        stdlib::imported(self.prog, name).or_else(|| {
            let hidden = self.types.contains_key(name)
                || self.traits.contains_key(name)
                || self.consts.contains_key(name)
                || self.fns.contains_key(name)
                || self.prog.uses.iter().any(|u| u.name().0 == name);
            if hidden { None } else { stdlib::module_names().into_iter().find(|m| *m == name) }
        })
    }

    /// std.use.4: `n.f(...)` where the local `n` hides the standard module
    /// `n`, which has a function `f`. Reports it and returns true.
    pub(super) fn hidden_std_module(&mut self, recv: &Expr, name: &str, pos: Pos) -> bool {
        let Expr::Name(n, _) = recv else { return false };
        if self.local(n).is_none() || !stdlib::functions(n).contains(&name) {
            return false;
        }
        self.error_help(
            "std.use.4",
            pos,
            format!("the local `{n}` hides the standard module `std.{n}`, so `{n}.{name}` isn't found"),
            format!("rename the local, or import the module under another name: `use std.{n} as {n}_mod`"),
        );
        true
    }

    /// `m.f(...)` or `m.Type(...)` where `m` may be a standard module
    /// (std.use.2). `None` when `m` is not a module, so the caller goes on.
    pub(super) fn std_call(
        &mut self,
        head: &str,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let Some(module) = self.std_module(head) else {
            let undeclared = !self.types.contains_key(head)
                && !self.traits.contains_key(head)
                && !self.consts.contains_key(head)
                && !self.fns.contains_key(head);
            if undeclared && stdlib::is_std_module(head) {
                self.args_only(args);
                self.not_imported(head, pos);
                return Some(Ty::Err);
            }
            return None;
        };
        if (module, name) == ("tensor", "Tensor") {
            self.args_only(args);
            self.error_help(
                "std.tensor.1",
                pos,
                "a tensor isn't built from fields",
                format!("use `{head}.from(xs, shape)` or `{head}.zeros(shape)`"),
            );
            return Some(Ty::Err);
        }
        let qualified = format!("{module}.{name}");
        if let Some(Def::Struct(s)) = self.types.get(qualified.as_str()).copied() {
            self.resolved.std_calls.insert(pos, StdCall::Type(s));
            return Some(self.struct_literal(s, None, args, pos, expect));
        }
        let Some(&f) = stdlib::functions(module).iter().find(|f| **f == name) else {
            self.args_only(args);
            let fns = stdlib::functions(module);
            let mut d = Diagnostic::new("std.use.2", pos, format!("module `{head}` has no function `{name}`"));
            d = match suggest(name, fns) {
                Some(s) => d.help(format!("did you mean `{head}.{s}`?")),
                None => d.help(format!("`std.{module}` provides {}", list_names(fns))),
            };
            self.diags.push(d);
            return Some(Ty::Err);
        };
        self.resolved.std_calls.insert(pos, StdCall::Fn(module, f));
        let what = format!("{head}.{f}");
        if module == "mem" {
            self.note_unsafe_op(&format!("`{what}`"), pos);
        }
        if module == "tensor" {
            return Some(self.tensor_call(f, args, pos, expect));
        }
        if (module, f) == ("io", "eprint") {
            return Some(self.eprint_call(args, pos));
        }
        if (module, f) == ("random", "shuffle") {
            return Some(self.shuffle_call(args, pos));
        }
        if (module, f) == ("task", "select") {
            return Some(self.select_call(args, pos));
        }
        let (mut params, ret) = self.std_signature(module, f);
        if (module, f) == ("proc", "run") {
            // std.proc.2: `input str?` and `timeout_ms int?` may be left off
            // from the end.
            params.truncate(args.len().clamp(2, 4));
        }
        if (module, f) == ("proc", "start") {
            // std.proc.4: `timeout_ms int?` may be left off.
            params.truncate(args.len().clamp(2, 3));
        }
        let mut sig = Sig::plain(None, params.into_iter().map(|t| (false, t)).collect(), ret);
        if module == "proc" && matches!(f, "wait" | "kill") {
            // std.proc.7: the child is consumed.
            sig.moves = vec![true];
        }
        Some(self.check_call(&what, &sig, Vec::new(), args, None, pos, expect))
    }

    /// The parameter and result types of a standard function.
    fn std_signature(&self, module: &str, f: &str) -> (Vec<Ty>, Ty) {
        use Ty::{Bool, Int, Str, Unit};
        let list = |t: Ty| Ty::List(Box::new(t));
        let opt = |t: Ty| Ty::Opt(Box::new(t));
        let std_ty = |name: &str| match self.types.get(name) {
            Some(Def::Struct(s)) => Ty::Struct(*s, Vec::new()),
            _ => unreachable!("std/types.ail declares `{name}`"),
        };
        let io = |t: Ty| Ty::Res(Box::new(t), Errs::Set(vec![std_ty("fs.IoError")]));
        match (module, f) {
            ("fs", "read_text") => (vec![Str], io(Str)),
            ("fs", "write_text" | "append_text") => (vec![Str, Str], io(Unit)),
            ("fs", "exists") => (vec![Str], Bool),
            ("fs", "list_dir") => (vec![Str], io(list(Str))),
            ("fs", "remove" | "make_dir") => (vec![Str], io(Unit)),
            ("fs", "walk") => (vec![Str], io(list(Str))),
            ("fs", "glob") => (vec![Str], list(Str)),
            ("fs", "metadata") => (vec![Str], io(std_ty("fs.Metadata"))),
            ("fs", "rename" | "copy") => (vec![Str, Str], io(Unit)),
            ("env", "args") => (vec![], list(Str)),
            ("env", "get") => (vec![Str], opt(Str)),
            ("env", "home") => (vec![], opt(Str)),
            ("env", "cwd" | "program") => (vec![], Str),
            ("env", "vars") => (vec![], Ty::Map(Box::new(Str), Box::new(Str))),
            ("path", "dir" | "base" | "ext") => (vec![Str], Str),
            ("path", "join") => (vec![Str, Str], Str),
            ("io", "read_line") => (vec![], opt(Str)),
            ("io", "read_all") => (vec![], Str),
            // std.io.4: `exit` never returns (ADR 0139).
            ("io", "exit") => (vec![Int], Ty::Never),
            ("proc", "run") => (
                vec![Str, list(Str), opt(Str), opt(Int)],
                Ty::Res(Box::new(std_ty("proc.Output")), Errs::Set(vec![std_ty("proc.ProcError")])),
            ),
            ("proc", "which") => (vec![Str], opt(Str)),
            // std.proc.4-7 (ADR 0424)
            ("proc", "start") => (
                vec![Str, list(Str), opt(Int)],
                Ty::Res(Box::new(std_ty("proc.Child")), Errs::Set(vec![std_ty("proc.ProcError")])),
            ),
            ("proc", "wait") => (
                vec![std_ty("proc.Child")],
                Ty::Res(Box::new(Int), Errs::Set(vec![std_ty("proc.ProcError")])),
            ),
            ("proc", "kill") => (vec![std_ty("proc.Child")], Int),
            ("net", "get" | "post") => (
                if f == "get" { vec![Str] } else { vec![Str, Str, Str] },
                Ty::Res(Box::new(std_ty("net.Response")), Errs::Set(vec![std_ty("net.NetError")])),
            ),
            // unsafe.mem.1: C strings in memory the program frees itself.
            ("mem", "cstr") => (vec![Str], Ty::Ptr(Box::new(Ty::Num(crate::num::Num::U8)))),
            ("mem", "from_cstr") => (vec![Ty::Ptr(Box::new(Ty::Num(crate::num::Num::U8)))], Str),
            ("mem", "free") => (vec![Ty::Ptr(Box::new(Ty::Num(crate::num::Num::U8)))], Unit),
            ("random", "int") => (vec![Int, Int], Int),
            ("random", "float") => (vec![], Ty::Float),
            ("random", "seed") => (vec![Int], Unit),
            // conc.cancel.2
            ("task", "cancelled") => (vec![], Bool),
            ("task", "check") => (vec![], Ty::Res(Box::new(Unit), Errs::Set(vec![self.prelude_error("Cancelled")]))),
            ("time", "now_ms") => (vec![], Int),
            ("time", "sleep_ms") => (vec![Int], Unit),
            // std.tz.1-2
            ("tz", "offset") => (vec![Str, Int], opt(Int)),
            ("tz", "system") => (vec![], Str),
            ("ui", "clipboard_get") => (vec![], opt(Str)),
            ("ui", "clipboard_set") => (vec![Str], Unit),
            ("ui", "open") => (
                vec![Str, Int, Int],
                Ty::Res(Box::new(std_ty("ui.Window")), Errs::Set(vec![std_ty("ui.UiError")])),
            ),
            _ => unreachable!("stdlib::MODULES lists `{module}.{f}`"),
        }
    }

    /// A prelude error type, such as `Cancelled` (conc.cancel.2).
    fn prelude_error(&self, name: &str) -> Ty {
        match self.types.get(name) {
            Some(Def::Struct(s)) => Ty::Struct(*s, Vec::new()),
            _ => Ty::Err,
        }
    }

    /// conc.select.1 (ADR 0427): `task.select(chans)` takes a `List[Channel[T]]`
    /// and gives the index of a channel with a value waiting, or none.
    fn select_call(&mut self, args: &'p [Arg], pos: Pos) -> Ty {
        let result = Ty::Opt(Box::new(Ty::Int));
        if args.len() != 1 {
            self.args_only(args);
            self.error(
                "conc.select.1",
                pos,
                format!("`select` takes exactly one argument, but {} given", super::was_were(args.len())),
            );
            return result;
        }
        let t = self.expr(&args[0].value, None, true);
        match &t {
            Ty::List(e) if matches!(**e, Ty::Conc(super::conc::Conc::Channel, _) | Ty::Err) => {}
            Ty::Err => {}
            _ => self.error_help(
                "conc.select.1",
                args[0].value.pos(),
                format!("`select` needs a list of channels, not {}", self.ty_name(&t)),
                "write `task.select([a, b])`, then `recv` from the channel at the index it gives",
            ),
        }
        result
    }

    /// std.random.4: `random.shuffle(mut xs)` reorders any `List[T]` in place.
    fn shuffle_call(&mut self, args: &'p [Arg], pos: Pos) -> Ty {
        if args.len() != 1 {
            self.args_only(args);
            self.error(
                "std.random.4",
                pos,
                format!("`shuffle` takes exactly one argument, but {} given", super::was_were(args.len())),
            );
            return Ty::Unit;
        }
        let a = &args[0];
        let t = self.expr(&a.value, None, true);
        if !a.mutable {
            self.error_help(
                "own.param.2",
                a.value.pos(),
                "this parameter is `mut`, so the argument must be marked `mut`",
                "write `mut` before the argument",
            );
        } else {
            self.require_mutable(&a.value, Some("own.param.2"), "passed `mut`");
        }
        if !matches!(t, Ty::List(_) | Ty::Err) {
            self.error_help(
                "std.random.4",
                a.value.pos(),
                format!("`shuffle` needs a list, not {}", self.ty_name(&t)),
                "to pick one element, use `xs[random.int(0, xs.len() - 1)]`",
            );
        }
        Ty::Unit
    }

    /// std.io.3: `io.eprint(value)` takes one value with a text form, as
    /// `print` does (mod.main.3).
    fn eprint_call(&mut self, args: &'p [Arg], pos: Pos) -> Ty {
        for a in args {
            if let Some((_, lpos)) = &a.label {
                self.unsupported(*lpos, "named arguments are not supported by the bootstrap yet (ADR 0058)");
            }
            if a.mutable {
                self.error_help("own.param.2", a.value.pos(), "`eprint`'s parameter isn't `mut`", "remove the `mut`");
            }
        }
        if args.len() != 1 {
            self.args_only(args);
            self.error(
                "std.io.3",
                pos,
                format!("`eprint` takes exactly one argument, but {} given", super::was_were(args.len())),
            );
        } else {
            let t = self.expr(&args[0].value, None, true);
            if t == Ty::Unit {
                self.error("std.io.3", args[0].value.pos(), "this expression produces no value to print");
            } else {
                self.text_form(&t, args[0].value.pos(), "std.io.3");
            }
        }
        Ty::Unit
    }

    /// std.use.2: a standard module's name used without its import.
    fn not_imported(&mut self, module: &str, pos: Pos) {
        let d = Diagnostic::new("std.use.2", pos, format!("the standard module `{module}` is not imported"))
            .help(format!("add `use std.{module}` at the top of the file"))
            .fix(
                format!("add `use std.{module}`"),
                vec![Edit::insert(Pos { line: 1, col: 1 }, format!("use std.{module}\n"))],
            );
        self.diags.push(d);
    }

    /// A module's type written `m.Type` (std.use.2). `None` when `name` has
    /// no module part.
    pub(super) fn std_type(&mut self, name: &str, args: &[TypeRef], pos: Pos) -> Option<Ty> {
        let (head, item) = name.split_once('.')?;
        let Some(module) = self.std_module(head) else {
            if self.types.contains_key(name) && !stdlib::is_std_module(head) {
                // A type of the program's own module `head`, already
                // given its qualified name (`modules.rs`).
                return None;
            } else if stdlib::is_std_module(head) {
                self.not_imported(head, pos);
            } else {
                self.error_help(
                    "std.use.2",
                    pos,
                    format!("unknown type `{name}`: `{head}` is not an imported module"),
                    format!("the standard modules are {}", list_names(&stdlib::module_names())),
                );
            }
            return Some(Ty::Err);
        };
        match self.types.get(format!("{module}.{item}").as_str()).copied() {
            Some(def) => Some(self.generic_type(def, args, pos)),
            None => {
                self.error("std.use.2", pos, format!("module `{head}` has no type `{item}`"));
                Some(Ty::Err)
            }
        }
    }

    /// `ui.NAME`, a named color (std.ui.17); `None` when `recv` isn't an
    /// imported standard module.
    pub(super) fn std_constant(&mut self, recv: &Expr, name: &str, pos: Pos) -> Option<Ty> {
        let Expr::Name(n, _) = recv else { return None };
        if self.local(n).is_some() {
            return None;
        }
        let module = self.std_module(n)?;
        let consts: &[(&str, i64)] = if module == "ui" { crate::ui::COLORS } else { &[] };
        match consts.iter().find(|(c, _)| *c == name) {
            Some(&(_, v)) => {
                self.resolved.std_calls.insert(pos, StdCall::Const(v));
                Some(Ty::Int)
            }
            None if consts.is_empty() => None,
            None => {
                let names: Vec<&str> = consts.iter().map(|(c, _)| *c).collect();
                self.error_help(
                    "std.ui.17",
                    pos,
                    format!("module `{n}` has no color `{name}`"),
                    format!("the colors are {}", list_names(&names)),
                );
                Some(Ty::Err)
            }
        }
    }

    /// `m` used as a value where `m` is an imported module (std.use.2).
    pub(super) fn module_as_value(&mut self, n: &str, pos: Pos) -> bool {
        let Some(module) = self.std_module(n) else { return false };
        let example = stdlib::functions(module).first().copied().unwrap_or("f");
        self.error_help(
            "std.use.2",
            pos,
            format!("`{n}` is a module, not a value"),
            format!("call one of its functions, e.g. `{n}.{example}(...)`"),
        );
        true
    }
}
