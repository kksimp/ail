//! `std.net`'s handle types (spec 17-std, std.net.9-12): the types of
//! `srv.accept()`, `ex.respond(r)`, `conn.read(n)` and the rest on a
//! `net.Server`, `net.Exchange`, `net.Listener` or `net.Conn`. The functions
//! that make them are ordinary standard functions (check/stdlib.rs).

use super::{Checker, Def, Errs, Sig, Ty, list_names, suggest};
use crate::ast::{Arg, Expr};
use crate::diag::{Diagnostic, Pos};
use crate::stdlib::{NET_METHODS, StdCall};

impl<'p> Checker<'p> {
    /// `recv.name(args)` where `recv` is a `std.net` handle; `None` for any
    /// other receiver, so the caller goes on.
    pub(super) fn net_method(
        &mut self,
        recv: &'p Expr,
        rt: &Ty,
        name: &str,
        pos: Pos,
        args: &'p [Arg],
        expect: Option<&Ty>,
    ) -> Option<Ty> {
        let Ty::Struct(s, _) = rt else { return None };
        let full = &self.prog.structs[*s].name;
        let ty_name = full.strip_prefix("net.")?;
        if !NET_METHODS.iter().any(|(t, _, _)| *t == ty_name) {
            return None;
        }
        let Some(&(_, m, key)) = NET_METHODS.iter().find(|(t, m, _)| *t == ty_name && *m == name) else {
            self.args_only(args);
            let names: Vec<&str> = NET_METHODS.iter().filter(|(t, _, _)| *t == ty_name).map(|(_, m, _)| *m).collect();
            let mut d = Diagnostic::new("std.net.9", pos, format!("`net.{ty_name}` has no method `{name}`"));
            d = match suggest(name, &names) {
                Some(s) => d.help(format!("did you mean `{s}`?")),
                None => d.help(format!("its methods are {}", list_names(&names))),
            };
            self.diags.push(d);
            return Some(Ty::Err);
        };
        self.resolved.std_calls.insert(pos, StdCall::Method(key));
        let ty = |name: &str| match self.types.get(name) {
            Some(Def::Struct(s)) => Ty::Struct(*s, Vec::new()),
            _ => unreachable!("std/types.ail declares `{name}`"),
        };
        let net = |t: Ty| Ty::Res(Box::new(t), Errs::Set(vec![ty("net.NetError")]));
        use Ty::{Int, Str, Unit};
        let (params, ret) = match (ty_name, m) {
            ("Server", "accept") => (vec![], net(ty("net.Exchange"))),
            ("Listener", "accept") => (vec![], net(ty("net.Conn"))),
            ("Server" | "Listener", "addr") | ("Conn", "peer") => (vec![], Str),
            ("Server" | "Listener" | "Conn" | "Body", "close") | ("Server" | "Request", "shutdown" | "stop") => {
                (vec![], Unit)
            }
            ("Body", "read_bytes") => (vec![Int], net(Ty::Bytes)),
            ("Exchange", "respond") => (vec![ty("net.Response")], net(Unit)),
            ("Exchange", "respond_bytes") => (vec![Int, Ty::Map(Box::new(Str), Box::new(Str)), Ty::Bytes], net(Unit)),
            ("Conn", "read") => (vec![Int], net(Str)),
            ("Conn", "read_line") => (vec![], net(Ty::Opt(Box::new(Str)))),
            ("Conn", "write") => (vec![Str], net(Unit)),
            ("Conn", "read_bytes") => (vec![Int], net(Ty::Bytes)),
            ("Conn", "write_bytes") => (vec![Ty::Bytes], net(Unit)),
            ("Conn", "set_timeout") => (vec![Int], Unit),
            _ => unreachable!("NET_METHODS lists `{ty_name}.{m}`"),
        };
        let sig = Sig::plain(Some(false), params.into_iter().map(|t| (false, t)).collect(), ret);
        Some(self.check_call(&format!("net.{ty_name}.{m}"), &sig, Vec::new(), args, Some(recv), pos, expect))
    }
}
