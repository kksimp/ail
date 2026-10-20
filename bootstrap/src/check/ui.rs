//! `std.ui`'s window methods (spec 17-std, std.ui.2-8): the types of
//! `w.size()`, `w.put(...)` and the rest on a `ui.Window`. `ui.open` is an
//! ordinary standard function (check/stdlib.rs).

use super::{Checker, Def, Sig, Ty, list_names, suggest};
use crate::ast::{Arg, Expr};
use crate::diag::{Diagnostic, Pos};
use crate::stdlib::{StdCall, WINDOW_METHODS};

impl<'p> Checker<'p> {
    /// `recv.name(args)` where `recv` is a `ui.Window`; `None` for any other
    /// receiver, so the caller goes on.
    pub(super) fn ui_method(
        &mut self,
        recv: &'p Expr,
        rt: &Ty,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let Ty::Struct(s, _) = rt else { return None };
        if self.prog.structs[*s].name != "ui.Window" {
            return None;
        }
        let Some(&(m, mutable)) = WINDOW_METHODS.iter().find(|(m, _)| *m == name) else {
            self.args_only(args);
            let names: Vec<&str> = WINDOW_METHODS.iter().map(|(m, _)| *m).collect();
            let mut d = Diagnostic::new("std.ui.2", pos, format!("`ui.Window` has no method `{name}`"));
            d = match suggest(name, &names) {
                Some(s) => d.help(format!("did you mean `{s}`?")),
                None => d.help(format!("a window's methods are {}", list_names(&names))),
            };
            self.diags.push(d);
            return Some(Ty::Err);
        };
        if mutable {
            self.require_mutable(recv, Some("trait.method.3"), "the receiver of a `mut self` method");
        }
        self.resolved.std_calls.insert(pos, StdCall::Method(m));
        let ty = |name: &str| match self.types.get(name) {
            Some(Def::Struct(s)) => Ty::Struct(*s, Vec::new()),
            Some(Def::Enum(e)) => Ty::Enum(*e, Vec::new()),
            _ => unreachable!("std/types.ail declares `{name}`"),
        };
        use Ty::{Int, Str, Unit};
        let (params, ret) = match m {
            "size" => (vec![], ty("ui.Size")),
            "modifiers" => (vec![], ty("ui.Modifiers")),
            "font_size" => (vec![], Int),
            "clear" | "set_font_size" => (vec![Int], Unit),
            "put" => (vec![Int, Int, Str, Int, Int], Unit),
            "cursor" => (vec![Int, Int], Unit),
            "set_title" => (vec![Str], Unit),
            "present" | "close" => (vec![], Unit),
            "next_event" => (vec![], ty("ui.Event")),
            "poll_event" => (vec![], Ty::Opt(Box::new(ty("ui.Event")))),
            _ => unreachable!("WINDOW_METHODS lists `{m}`"),
        };
        let sig = Sig::plain(Some(mutable), params.into_iter().map(|t| (false, t)).collect(), ret);
        Some(self.check_call(&format!("ui.Window.{m}"), &sig, Vec::new(), args, Some(recv), pos, expect))
    }
}
