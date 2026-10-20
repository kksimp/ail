//! The standard modules `std.fs`, `std.env`, `std.io`, `std.proc` and
//! `std.time` (spec 17-std): their imports (std.use.1-3), the types of
//! their functions (std.fs.1-7, std.env.1-4, std.io.1-4, std.proc.1,
//! std.time.1-2), and their types `fs.IoError`, `proc.Output` and
//! `proc.ProcError` (std.err.1).

use std::collections::HashMap;

use super::{Checker, Def, Errs, Sig, Ty, list_names, suggest};
use crate::ast::{Arg, TypeRef};
use crate::diag::{Diagnostic, Edit, Pos};
use crate::lexer::UNSUPPORTED;
use crate::stdlib::{self, StdCall};

impl<'p> Checker<'p> {
    /// std.use.1, std.use.3: each `use std.NAME` names a standard module, and
    /// no two imports share a name.
    pub(super) fn check_uses(&mut self) {
        let mut seen: HashMap<&str, Pos> = HashMap::new();
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
                // TODO(lane 3, user modules): resolve `use a.b` for the
                // program's own modules (mod.use.1).
                _ => {
                    self.diags.push(Diagnostic::new(
                        UNSUPPORTED,
                        u.pos,
                        "importing a user module is not supported by the bootstrap yet (spec mod.use.1); \
                         only the standard modules `use std.NAME` are",
                    ));
                    continue;
                }
            }
            let (name, pos) = u.name();
            if let Some(first) = seen.get(name.as_str()) {
                self.diags.push(Diagnostic::new("std.use.3", *pos, format!("`{name}` is already imported")).help(
                    format!(
                        "it is first imported on line {}; remove one import, or import it under another name with `as`",
                        first.line
                    ),
                ));
            } else {
                seen.insert(name, *pos);
            }
        }
    }

    /// The standard module imported under `name`, if any (std.use.1).
    pub(super) fn std_module(&self, name: &str) -> Option<&'static str> {
        stdlib::imported(self.prog, name)
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
        if (module, f) == ("io", "eprint") {
            return Some(self.eprint_call(args, pos));
        }
        let (params, ret) = self.std_signature(module, f);
        let sig = Sig::plain(None, params.into_iter().map(|t| (false, t)).collect(), ret);
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
            ("env", "args") => (vec![], list(Str)),
            ("env", "get") => (vec![Str], opt(Str)),
            ("env", "cwd" | "program") => (vec![], Str),
            ("io", "read_line") => (vec![], opt(Str)),
            ("io", "read_all") => (vec![], Str),
            // std.io.4: `exit` never returns (ADR 0139).
            ("io", "exit") => (vec![Int], Ty::Never),
            ("proc", "run") => (
                vec![Str, list(Str)],
                Ty::Res(Box::new(std_ty("proc.Output")), Errs::Set(vec![std_ty("proc.ProcError")])),
            ),
            ("time", "now_ms") => (vec![], Int),
            ("time", "sleep_ms") => (vec![Int], Unit),
            _ => unreachable!("stdlib::MODULES lists `{module}.{f}`"),
        }
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
            if stdlib::is_std_module(head) {
                self.not_imported(head, pos);
            } else {
                // TODO(lane 3, user modules): types of the program's own modules.
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
