//! Methods of the built-in types: `str` (type.str.4-11), `char`
//! (type.char.1-4), `int` and `float` (type.num.6-9), `List` (type.coll.5-7,
//! type.coll.9-11), `Map` and `Set` (type.map.1-6).

use super::{Checker, Sig, Ty};
use crate::ast::{Arg, Expr};
use crate::diag::Pos;

/// What a built-in method's element type must support.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Need {
    Nothing,
    Eq,
    Ord,
    /// `int` or `float` (`sum`).
    Number,
}

/// A built-in method's signature.
pub(super) struct Builtin {
    pub mutates: bool,
    pub params: Vec<Ty>,
    pub ret: Ty,
    pub need: Need,
}

pub(super) const STR_METHODS: &[&str] = &[
    "split",
    "to_int",
    "to_float",
    "chars",
    "trim",
    "starts_with",
    "ends_with",
    "strip_prefix",
    "strip_suffix",
    "contains",
    "repeat",
    "pad_left",
    "pad_right",
];
pub(super) const CHAR_METHODS: &[&str] = &[
    "is_digit", "is_alpha", "is_alnum", "is_space", "is_upper", "is_lower", "to_upper", "to_lower", "code", "to_digit",
];
pub(super) const INT_METHODS: &[&str] = &[
    "checked_add",
    "checked_sub",
    "checked_mul",
    "checked_div",
    "checked_neg",
    "wrapping_add",
    "wrapping_sub",
    "wrapping_mul",
    "saturating_add",
    "saturating_sub",
    "saturating_mul",
    "abs",
    "to_float",
    "to_hex",
];
pub(super) const FLOAT_METHODS: &[&str] = &["abs", "floor", "ceil", "round", "sqrt", "to_int"];
pub(super) const LIST_METHODS: &[&str] =
    &["len", "push", "swap", "pop", "contains", "index_of", "join", "sum", "min", "max", "sorted", "sort"];
pub(super) const MAP_METHODS: &[&str] = &["len", "get", "contains", "remove", "keys", "values"];
pub(super) const SET_METHODS: &[&str] = &["len", "add", "remove", "contains", "sorted", "is_subset"];

/// The built-in method `name` of type `t`, if there is one.
pub(super) fn builtin(t: &Ty, name: &str) -> Option<Builtin> {
    use Ty::{Bool, Char, Float, Int, Str, Unit};
    let opt = |t: Ty| Ty::Opt(Box::new(t));
    let list = |t: Ty| Ty::List(Box::new(t));
    let make = |mutates: bool, params: Vec<Ty>, ret: Ty, need: Need| Some(Builtin { mutates, params, ret, need });
    let read = |params: Vec<Ty>, ret: Ty| make(false, params, ret, Need::Nothing);
    match t {
        Ty::Str => match name {
            "split" => read(vec![Str], list(Str)),
            "to_int" => read(vec![], opt(Int)),
            "to_float" => read(vec![], opt(Float)),
            "chars" => read(vec![], list(Char)),
            "trim" => read(vec![], Str),
            "starts_with" | "ends_with" | "contains" => read(vec![Str], Bool),
            "strip_prefix" | "strip_suffix" => read(vec![Str], opt(Str)),
            "repeat" | "pad_left" | "pad_right" => read(vec![Int], Str),
            _ => None,
        },
        Ty::Char => match name {
            "is_digit" | "is_alpha" | "is_alnum" | "is_space" | "is_upper" | "is_lower" => read(vec![], Bool),
            "to_upper" | "to_lower" => read(vec![], Char),
            "code" => read(vec![], Int),
            "to_digit" => read(vec![Int], opt(Int)),
            _ => None,
        },
        Ty::Int => match name {
            "checked_add" | "checked_sub" | "checked_mul" | "checked_div" => read(vec![Int], opt(Int)),
            "checked_neg" => read(vec![], opt(Int)),
            "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "saturating_add" | "saturating_sub"
            | "saturating_mul" => read(vec![Int], Int),
            "abs" => read(vec![], Int),
            "to_float" => read(vec![], Float),
            "to_hex" => read(vec![], Str),
            _ => None,
        },
        Ty::Float => match name {
            "abs" | "floor" | "ceil" | "round" | "sqrt" => read(vec![], Float),
            "to_int" => read(vec![], Int),
            _ => None,
        },
        Ty::List(e) => {
            let e = (**e).clone();
            match name {
                "len" => read(vec![], Int),
                "push" => make(true, vec![e], Unit, Need::Nothing),
                "swap" => make(true, vec![Int, Int], Unit, Need::Nothing),
                "pop" => make(true, vec![], opt(e), Need::Nothing),
                "contains" => make(false, vec![e], Bool, Need::Eq),
                "index_of" => make(false, vec![e], opt(Int), Need::Eq),
                "join" => read(vec![Str], Str),
                "sum" => make(false, vec![], e, Need::Number),
                "min" | "max" => make(false, vec![], opt(e), Need::Ord),
                "sorted" => make(false, vec![], list(e), Need::Ord),
                "sort" => make(true, vec![], Unit, Need::Ord),
                _ => None,
            }
        }
        Ty::Map(k, v) => {
            let (k, v) = ((**k).clone(), (**v).clone());
            match name {
                "len" => read(vec![], Int),
                "get" => read(vec![k], opt(v)),
                "contains" => read(vec![k], Bool),
                "remove" => make(true, vec![k], opt(v), Need::Nothing),
                "keys" => read(vec![], list(k)),
                "values" => read(vec![], list(v)),
                _ => None,
            }
        }
        Ty::Set(e) => {
            let e = (**e).clone();
            match name {
                "len" => read(vec![], Int),
                "add" => make(true, vec![e], Unit, Need::Nothing),
                "remove" => make(true, vec![e], Bool, Need::Nothing),
                "contains" => read(vec![e], Bool),
                "sorted" => make(false, vec![], list(e), Need::Ord),
                "is_subset" => read(vec![Ty::Set(Box::new(e))], Bool),
                _ => None,
            }
        }
        _ => None,
    }
}

/// The names of `t`'s built-in methods, for suggestions.
pub(super) fn builtin_names(t: &Ty) -> &'static [&'static str] {
    match t {
        Ty::Str => STR_METHODS,
        Ty::Char => CHAR_METHODS,
        Ty::Int => INT_METHODS,
        Ty::Float => FLOAT_METHODS,
        Ty::List(_) => LIST_METHODS,
        Ty::Map(..) => MAP_METHODS,
        Ty::Set(_) => SET_METHODS,
        _ => &[],
    }
}

impl<'p> Checker<'p> {
    /// `x.name(args)` for a built-in method of `x`'s type.
    pub(super) fn builtin_call(&mut self, recv: &'p Expr, rt: &Ty, name: &str, pos: Pos, args: &'p [Arg]) -> Ty {
        let b = builtin(rt, name).expect("a built-in method");
        let elem = match rt {
            Ty::List(e) | Ty::Set(e) => (**e).clone(),
            _ => Ty::Err,
        };
        let ok = match b.need {
            Need::Nothing => true,
            Need::Eq => self.implements(&elem, self.std.eq),
            Need::Ord => self.implements(&elem, self.std.ord),
            Need::Number => matches!(elem, Ty::Int | Ty::Float | Ty::Err),
        };
        if !ok {
            let what = match b.need {
                Need::Eq => "elements that can be compared with `==` (`Eq`)",
                Need::Ord => "elements that can be ordered (`Ord`)",
                _ => "`int` or `float` elements",
            };
            let msg = format!("`{name}` needs {what}, and {} has {}", self.show(rt), self.show(&elem));
            let help = match b.need {
                Need::Ord => self.how_to_implement(&elem, self.std.ord),
                Need::Eq => self.how_to_implement(&elem, self.std.eq),
                _ => None,
            };
            match help {
                Some(h) => self.error_help("gen.bound.3", pos, msg, h),
                None => self.error("gen.bound.3", pos, msg),
            }
        }
        if b.mutates {
            self.require_mutable(recv, Some("trait.method.3"), &format!("the receiver of `{name}`"));
        }
        let sig = Sig::plain(Some(b.mutates), b.params.into_iter().map(|t| (false, t)).collect(), b.ret);
        self.check_call(name, &sig, Vec::new(), args, Some(recv), pos, None)
    }
}
