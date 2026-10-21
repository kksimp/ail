//! Methods on `regex.Regex` (spec std.regex.25, ADR 0749): `re.is_match(s)`
//! is `regex.is_match(re, s)`, and so for `find`, `find_all`, `captures`,
//! `replace`, `replace_first` and `split`. A method is the function with the
//! regex first, so the two forms always agree.

use super::{Checker, Sig, Ty, list_names, suggest};
use crate::ast::{Arg, Expr};
use crate::diag::{Diagnostic, Pos};
use crate::stdlib::StdCall;

/// The methods of `regex.Regex`, sorted.
pub const REGEX_METHODS: &[&str] = &["captures", "find", "find_all", "is_match", "replace", "replace_first", "split"];

impl<'p> Checker<'p> {
    /// `recv.name(args)` where `recv` is a `regex.Regex`; `None` for any
    /// other receiver.
    pub(super) fn regex_method(
        &mut self,
        recv: &'p Expr,
        rt: &Ty,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let Ty::Struct(s, _) = rt else { return None };
        if self.prog.structs[*s].name != "regex.Regex" {
            return None;
        }
        if !REGEX_METHODS.contains(&name) {
            self.args_only(args);
            let mut d = Diagnostic::new("std.regex.25", pos, format!("`regex.Regex` has no method `{name}`"));
            d = match suggest(name, REGEX_METHODS) {
                Some(s) => d.help(format!("did you mean `{s}`?")),
                None => d.help(format!("its methods are {}", list_names(REGEX_METHODS))),
            };
            self.diags.push(d);
            return Some(Ty::Err);
        }
        let key: &'static str = match name {
            "captures" => "regex.captures",
            "find" => "regex.find",
            "find_all" => "regex.find_all",
            "is_match" => "regex.is_match",
            "replace" => "regex.replace",
            "replace_first" => "regex.replace_first",
            _ => "regex.split",
        };
        self.resolved.std_calls.insert(pos, StdCall::Method(key));
        let ty = |name: &str| match self.types.get(name) {
            Some(super::Def::Struct(s)) => Ty::Struct(*s, Vec::new()),
            _ => unreachable!("std/types.ail declares `{name}`"),
        };
        use Ty::{Bool, Str};
        let opt = |t: Ty| Ty::Opt(Box::new(t));
        let list = |t: Ty| Ty::List(Box::new(t));
        let (params, ret) = match name {
            "is_match" => (vec![Str], Bool),
            "find" => (vec![Str], opt(ty("regex.Match"))),
            "find_all" => (vec![Str], list(ty("regex.Match"))),
            "captures" => (vec![Str], opt(ty("regex.Captures"))),
            "replace" | "replace_first" => (vec![Str, Str], Str),
            _ => (vec![Str], list(Str)),
        };
        let sig = Sig::plain(Some(false), params.into_iter().map(|t| (false, t)).collect(), ret);
        Some(self.check_call(&format!("regex.Regex.{name}"), &sig, Vec::new(), args, Some(recv), pos, expect))
    }
}
