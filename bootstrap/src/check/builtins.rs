//! Methods of the built-in types: `str` (type.str.4-11), `char`
//! (type.char.1-4), `int` and `float` (type.num.6-9), `List` (type.coll.5-7,
//! type.coll.9-11), `Map` and `Set` (type.map.1-6).

use super::{Checker, Sig, Ty};
use crate::ast::{Arg, Expr};
use crate::diag::Pos;
use crate::num::Num;

/// What a built-in method's element type must support.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum Need {
    Nothing,
    Eq,
    Ord,
    /// `Hash` (the set operations, type.coll.16).
    Hash,
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
    "slice",
    "to_lower",
    "to_upper",
    "find",
    "replace",
    "char_at",
    "starts_with",
    "ends_with",
    "strip_prefix",
    "strip_suffix",
    "contains",
    "repeat",
    "pad_left",
    "pad_right",
    "center",
    "words",
    "trim_start",
    "trim_end",
    "count",
    "rfind",
    "lines",
    "len",
    "byte_len",
    "to_bytes",
];
/// type.bytes.7-12
pub(super) const BYTES_METHODS: &[&str] =
    &["len", "to_str", "text", "to_list", "concat", "find", "starts_with", "ends_with", "contains", "hex", "base64"];
pub(super) const CHAR_METHODS: &[&str] = &[
    "is_digit", "is_alpha", "is_alnum", "is_space", "is_upper", "is_lower", "to_upper", "to_lower", "code", "to_digit",
];
pub(super) const INT_METHODS: &[&str] = &[
    "bit_and",
    "bit_or",
    "bit_xor",
    "bit_not",
    "shl",
    "shr",
    "checked_add",
    "checked_sub",
    "checked_mul",
    "checked_div",
    "checked_neg",
    "checked_rem",
    "wrapping_neg",
    "wrapping_add",
    "wrapping_sub",
    "wrapping_mul",
    "saturating_add",
    "saturating_sub",
    "saturating_mul",
    "abs",
    "to_float",
    "to_hex",
    "to_i8",
    "to_i16",
    "to_i32",
    "to_u8",
    "to_u16",
    "to_u32",
    "to_u64",
    "to_f32",
];
pub(super) const FLOAT_METHODS: &[&str] = &[
    "abs", "floor", "ceil", "round", "sqrt", "sin", "cos", "tan", "atan", "atan2", "exp", "ln", "log10", "pow", "hypot",
    "cbrt", "to_int", "to_f32",
];
pub(super) const SIZED_INT_METHODS: &[&str] = &[
    "checked_add", "checked_sub", "checked_mul", "checked_div", "checked_rem", "checked_neg", "wrapping_add",
    "wrapping_sub", "wrapping_mul", "wrapping_neg", "saturating_add", "saturating_sub", "saturating_mul", "abs",
    "to_int", "to_float", "to_hex", "to_i8", "to_i16", "to_i32", "to_u8", "to_u16", "to_u32", "to_u64", "to_f32",
];
pub(super) const F32_METHODS: &[&str] = &["abs", "floor", "ceil", "round", "sqrt", "to_int", "to_float"];
pub(super) const LIST_METHODS: &[&str] =
    &[
        "len", "push", "swap", "pop", "last", "first", "insert", "remove_at", "extend", "clear", "reverse", "contains",
        "index_of", "join", "sum", "min", "max", "sorted", "sort", "take", "union", "intersect", "difference", "chunks",
        "windows", "counts", "flatten",
    ];
pub(super) const MAP_METHODS: &[&str] = &["len", "get", "contains", "remove", "keys", "values", "top"];
pub(super) const SET_METHODS: &[&str] = &["len", "add", "remove", "contains", "sorted", "is_subset", "union", "intersect", "difference"];

/// `to_i8` ... `to_f32`: the sized type a conversion method names (type.num.15).
fn convert(name: &str) -> Option<Num> {
    crate::num::CONVERSIONS.iter().find(|(n, _)| *n == name).map(|(_, k)| *k)
}

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
            "trim" | "trim_start" | "trim_end" | "to_lower" | "to_upper" => read(vec![], Str),
            "words" => read(vec![], list(Str)),
            "count" => read(vec![Str], Int),
            "rfind" => read(vec![Str], opt(Int)),
            "slice" => read(vec![Int, Int], Str),
            "find" => read(vec![Str], opt(Int)),
            "replace" => read(vec![Str, Str], Str),
            "char_at" => read(vec![Int], opt(Char)),
            "starts_with" | "ends_with" | "contains" => read(vec![Str], Bool),
            "strip_prefix" | "strip_suffix" => read(vec![Str], opt(Str)),
            "repeat" => read(vec![Int], Str),
            "pad_left" | "pad_right" | "center" => read(vec![Int, Char], Str),
            "lines" => read(vec![], list(Str)),
            "len" | "byte_len" => read(vec![], Int),
            "to_bytes" => read(vec![], Ty::Bytes),
            _ => None,
        },
        // type.bytes.7-12: `bytes` is immutable, so no method changes it.
        Ty::Bytes => match name {
            "len" => read(vec![], Int),
            "to_str" => read(vec![], opt(Str)),
            "text" | "hex" | "base64" => read(vec![], Str),
            "to_list" => read(vec![], list(Ty::Num(Num::U8))),
            "concat" => read(vec![Ty::Bytes], Ty::Bytes),
            "find" => read(vec![Ty::Bytes], opt(Int)),
            "starts_with" | "ends_with" | "contains" => read(vec![Ty::Bytes], Bool),
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
            "checked_add" | "checked_sub" | "checked_mul" | "checked_div" | "checked_rem" => read(vec![Int], opt(Int)),
            "checked_neg" => read(vec![], opt(Int)),
            "wrapping_neg" => read(vec![], Int),
            "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "saturating_add" | "saturating_sub"
            | "saturating_mul" => read(vec![Int], Int),
            "abs" => read(vec![], Int),
            // expr.bit.1-2: bitwise operations on the 64-bit two's complement value.
            "bit_and" | "bit_or" | "bit_xor" => read(vec![Int], Int),
            "bit_not" => read(vec![], Int),
            "shl" | "shr" => read(vec![Int], Int),
            "to_float" => read(vec![], Float),
            "to_hex" => read(vec![], Str),
            _ => convert(name).map(|k| Builtin { mutates: false, params: vec![], ret: Ty::Num(k), need: Need::Nothing }),
        },
        Ty::Float => match name {
            "abs" | "floor" | "ceil" | "round" | "sqrt" | "sin" | "cos" | "tan" | "atan" | "exp" | "ln" | "log10"
            | "cbrt" => read(vec![], Float),
            // type.num.16
            "atan2" | "pow" | "hypot" => read(vec![Float], Float),
            "to_int" => read(vec![], Int),
            "to_f32" => read(vec![], Ty::Num(Num::F32)),
            _ => None,
        },
        // type.num.14-15: the sized integers' methods, and `f32`'s.
        Ty::Num(k) if k.is_float() => {
            let f32 = Ty::Num(Num::F32);
            match name {
                "abs" | "floor" | "ceil" | "round" | "sqrt" => read(vec![], f32),
                "to_int" => read(vec![], Int),
                "to_float" => read(vec![], Float),
                _ => None,
            }
        }
        Ty::Num(k) => {
            let me = Ty::Num(*k);
            match name {
                "checked_add" | "checked_sub" | "checked_mul" | "checked_div" | "checked_rem" => {
                    read(vec![me.clone()], opt(me))
                }
                "checked_neg" if k.is_signed() => read(vec![], opt(me)),
                "wrapping_neg" | "abs" if k.is_signed() => read(vec![], me),
                "wrapping_add" | "wrapping_sub" | "wrapping_mul" | "saturating_add" | "saturating_sub"
                | "saturating_mul" => read(vec![me.clone()], me),
                "to_int" => read(vec![], Int),
                "to_float" => read(vec![], Float),
                "to_hex" => read(vec![], Str),
                _ => convert(name).filter(|c| c != k).map(|c| Builtin {
                    mutates: false,
                    params: vec![],
                    ret: Ty::Num(c),
                    need: Need::Nothing,
                }),
            }
        }
        Ty::List(e) => {
            let e = (**e).clone();
            match name {
                "len" => read(vec![], Int),
                "push" => make(true, vec![e], Unit, Need::Nothing),
                "swap" => make(true, vec![Int, Int], Unit, Need::Nothing),
                "pop" => make(true, vec![], opt(e), Need::Nothing),
                "last" | "first" => read(vec![], opt(e)),
                "take" => read(vec![Int], list(e)),
                "chunks" | "windows" => read(vec![Int], list(list(e))),
                "flatten" => match &e {
                    Ty::List(inner) => read(vec![], list((**inner).clone())),
                    _ => None,
                },
                "counts" => make(false, vec![], Ty::Map(Box::new(e), Box::new(Int)), Need::Hash),
                "union" | "intersect" | "difference" => make(false, vec![list(e.clone())], Ty::Set(Box::new(e)), Need::Hash),
                "insert" => make(true, vec![Int, e], Unit, Need::Nothing),
                "remove_at" => make(true, vec![Int], e, Need::Nothing),
                "extend" => make(true, vec![list(e)], Unit, Need::Nothing),
                "clear" | "reverse" => make(true, vec![], Unit, Need::Nothing),
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
                // std.count.2: the keys with the largest int values.
                "top" => match v {
                    Ty::Int => read(vec![Int], list(k)),
                    _ => None,
                },
                // type.map.8: `m.push(k, x)` for a list-valued map.
                "push" => match v {
                    Ty::List(e) => make(true, vec![k, *e], Unit, Need::Nothing),
                    _ => None,
                },
                _ => None,
            }
        }
        Ty::Set(e) => {
            let e = (**e).clone();
            match name {
                "len" => read(vec![], Int),
                "add" => make(true, vec![e], Bool, Need::Nothing),
                "remove" => make(true, vec![e], Bool, Need::Nothing),
                "contains" => read(vec![e], Bool),
                "sorted" => make(false, vec![], list(e), Need::Ord),
                "is_subset" => read(vec![Ty::Set(Box::new(e))], Bool),
                "union" | "intersect" | "difference" => {
                    make(false, vec![Ty::Set(Box::new(e.clone()))], Ty::Set(Box::new(e)), Need::Hash)
                }
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
        Ty::Bytes => BYTES_METHODS,
        Ty::Char => CHAR_METHODS,
        Ty::Int => INT_METHODS,
        Ty::Float => FLOAT_METHODS,
        Ty::Num(k) if k.is_float() => F32_METHODS,
        Ty::Num(_) => SIZED_INT_METHODS,
        Ty::List(_) => LIST_METHODS,
        Ty::Map(..) => MAP_METHODS,
        Ty::Set(_) => SET_METHODS,
        _ => &[],
    }
}

impl<'p> Checker<'p> {
    /// `x.name(args)` for a built-in method of `x`'s type.
    pub(super) fn builtin_call(&mut self, recv: &'p Expr, rt: &Ty, name: &str, pos: Pos, args: &'p [Arg]) -> Ty {
        if let Some(t) = self.keyed_call(recv, rt, name, pos, args) {
            return t;
        }
        let b = builtin(rt, name).expect("a built-in method");
        // type.str.6: these take a `str` or a `char`.
        if *rt == Ty::Str && matches!(name, "contains" | "starts_with" | "ends_with" | "split") && args.len() == 1 {
            let a = &args[0];
            let t = self.expr(&a.value, None, true);
            if a.mutable {
                self.error_help("own.param.2", a.value.pos(), "this parameter isn't `mut`", "remove the `mut`");
            }
            if !matches!(t, Ty::Str | Ty::Char | Ty::Err | Ty::Never) && !self.reject_unhandled(&t, a.value.pos()) {
                let msg = format!("`{name}` takes a `str` or a `char`, not {}", self.show(&t));
                self.error("expr.call.2", a.value.pos(), msg);
            }
            return b.ret;
        }
        if name == "get" && args.len() == 2 && matches!(rt, Ty::Map(..)) {
            self.args_only(args);
            self.error_help(
                "expr.call.1",
                pos,
                "`m.get(k)` takes only the key, and gives a `V?`",
                "for a default, write `v := m.get(k) else default`",
            );
            return b.ret;
        }
        let elem = match rt {
            Ty::List(e) | Ty::Set(e) => (**e).clone(),
            _ => Ty::Err,
        };
        let ok = match b.need {
            Need::Nothing => true,
            Need::Eq => self.implements(&elem, self.std.eq),
            Need::Ord => self.implements(&elem, self.std.ord),
            Need::Hash => self.implements(&elem, self.std.hash),
            Need::Number => matches!(elem, Ty::Int | Ty::Float | Ty::Num(_) | Ty::Err),
        };
        if !ok {
            let what = match b.need {
                Need::Eq => "elements that can be compared with `==` (`Eq`)",
                Need::Ord => "elements that can be ordered (`Ord`)",
                Need::Hash => "elements that can be hashed (`Hash`)",
                _ => "`int` or `float` elements",
            };
            let msg = format!("`{name}` needs {what}, and {} has {}", self.show(rt), self.show(&elem));
            let help = match b.need {
                Need::Ord => self.how_to_implement(&elem, self.std.ord),
                Need::Eq => self.how_to_implement(&elem, self.std.eq),
                Need::Hash => self.how_to_implement(&elem, self.std.hash),
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
        let moves = self.unique_builtin(rt, name, pos, b.params.len());
        let mut sig = Sig::plain(Some(b.mutates), b.params.into_iter().map(|t| (false, t)).collect(), b.ret);
        sig.moves = moves;
        self.check_call(name, &sig, Vec::new(), args, Some(recv), pos, None)
    }
}

/// Help for a function name that other languages have (expr.name.2).
pub(super) fn function_help(name: &str) -> Option<&'static str> {
    Some(match name {
        "len" => "write `x.len()`",
        "min" | "max" => "write `a.min(b)` / `a.max(b)`, or `xs.min()` / `xs.max()` for a list (they give `T?`)",
        "ord" => "write `c.code()`",
        "chr" => "write `char.from_code(n)`, which gives a `char?`",
        "sorted" => "write `xs.sorted()`, or `xs.sort()` to sort in place",
        "sum" => "write `xs.sum()`",
        "abs" => "write `x.abs()`",
        "round" => "write `x.round()`: a float, rounded half away from zero; add `.to_int()` for an int",
        "floor" | "ceil" => "write `x.floor()` / `x.ceil()`: a float; add `.to_int()` for an int",
        "sqrt" => "write `x.sqrt()`",
        "str" | "String" | "string" => "write `\"{x}\"` for a value's text",
        "int" | "Int" | "parseInt" | "atoi" => "write `s.to_int()`, which gives an `int?`; for a float, `x.to_int()`",
        "float" | "parseFloat" => "write `s.to_float()`, or `n.to_float()` for an int",
        "loop" => "write `while true`",
        "range" => "write a range: `0..n`, or `1..=n` to include `n`",
        "println" | "printf" | "puts" | "console" => "write `print(value)`",
        "assert" | "panic" => "write `unreachable(\"message\")` to stop the program",
        _ => return None,
    })
}

/// Help for a method name that other languages give the built-in types (expr.name.2).
pub(super) fn method_help(t: &Ty, name: &str) -> Option<&'static str> {
    Some(match (t, name) {
        (Ty::Str, "length" | "size" | "count") => "write `s.len()` for characters, or `s.byte_len()` for UTF-8 bytes",
        (Ty::Str, "strip" | "lstrip" | "rstrip" | "trim_start" | "trim_end") => "write `s.trim()`",
        (Ty::Str, "startswith") => "write `s.starts_with(prefix)`",
        (Ty::Str, "endswith") => "write `s.ends_with(suffix)`",
        (Ty::Str, "removeprefix") => "write `s.strip_prefix(p)`, which gives a `str?`",
        (Ty::Str, "removesuffix") => "write `s.strip_suffix(p)`, which gives a `str?`",
        (Ty::Str, "rjust" | "pad_start" | "padStart") => "write `s.pad_left(width, ' ')`",
        (Ty::Str, "ljust" | "pad_end" | "padEnd") => "write `s.pad_right(width, ' ')`",
        (Ty::Str, "zfill") => "write `s.pad_left(width, '0')`",
        (Ty::Str, "join") => "write `xs.join(sep)` on the list",
        (Ty::Str, "splitlines") => "write `s.lines()`",
        (Ty::Str, "isdigit" | "isalpha" | "isalnum" | "isspace" | "is_digit" | "is_alpha") => {
            "character tests are on `char`: `c.is_digit()`; for a string, test each of `s.chars()`"
        }
        (Ty::Str, "format") => "build text with interpolation: `\"{a} and {b}\"`",
        (Ty::Char, "is_whitespace") => "write `c.is_space()`",
        (Ty::Char, "is_alphabetic") => "write `c.is_alpha()`",
        (Ty::Char, "is_alphanumeric") => "write `c.is_alnum()`",
        (Ty::Char, "is_numeric" | "is_ascii_digit" | "isdigit") => "write `c.is_digit()`",
        (Ty::Char, "to_int") => "write `c.code()` for its scalar value, or `c.to_digit(10)` for a digit's value",
        (Ty::Char, "to_uppercase") => "write `c.to_upper()`",
        (Ty::Char, "to_lowercase") => "write `c.to_lower()`",
        (Ty::Map(..), "push") => "`m.push(k, x)` needs a map whose values are lists, `Map[K, List[T]]`; for other values write `m[k] = v`",
        (Ty::List(_), "append" | "add") => "write `xs.push(x)`",
        (Ty::List(_), "length" | "size" | "count") => "write `xs.len()`",
        (Ty::List(_), "index" | "find" | "indexOf") => "write `xs.index_of(x)`, which gives `int?` (`none` if absent)",
        (Ty::Map(..), "items" | "entries") => "iterate the keys and index: `for k in m.keys()` then `m[k]`",
        (Ty::Map(..), "has" | "has_key" | "contains_key") => "write `m.contains(k)`",
        (Ty::Map(..), "put" | "set" | "insert") => "write `m[k] = v`",
        (Ty::Map(..), "delete" | "pop") => "write `m.remove(k)`",
        (Ty::Set(_), "insert" | "push") => "write `s.add(x)`",
        (Ty::Set(_), "has") => "write `s.contains(x)`",
        (Ty::Set(_), "discard" | "delete") => "write `s.remove(x)`",
        (_, "message" | "msg" | "description" | "getMessage" | "get_message") => {
            "an error's message is its text: interpolate `\"{e}\"` or write `e.to_str()`; the variants' messages are in the `error enum`"
        }
        (_, "to_string" | "toString" | "__str__") => "write `x.to_str()`, or interpolate: `\"{x}\"`",
        _ => return None,
    })
}

/// The AIL method for another language's name for it, when the arguments
/// carry over unchanged (gram.foreign.1, ADR 0262). The fix renames the call.
pub(super) fn method_rename(t: &Ty, name: &str) -> Option<&'static str> {
    Some(match (t, name) {
        (Ty::Str | Ty::List(_) | Ty::Map(..) | Ty::Set(_), "length" | "size" | "count") => "len",
        (Ty::Str, "strip" | "lstrip" | "rstrip" | "trim_start" | "trim_end") => "trim",
        (Ty::Str, "encode") => "to_bytes",
        (Ty::Bytes, "startswith" | "startsWith") => "starts_with",
        (Ty::Bytes, "endswith" | "endsWith") => "ends_with",
        (Ty::Str, "startswith" | "startsWith") => "starts_with",
        (Ty::Str, "endswith" | "endsWith") => "ends_with",
        (Ty::Str, "splitlines") => "lines",
        (Ty::Str, "lower" | "lowercase" | "toLowerCase") => "to_lower",
        (Ty::Str, "upper" | "uppercase" | "toUpperCase") => "to_upper",
        (Ty::Str, "includes") => "contains",
        (Ty::Str, "indexOf") => "find",
        (Ty::Str, "substring" | "substr") => "slice",
        (Ty::Char, "is_alphabetic") => "is_alpha",
        (Ty::Char, "is_whitespace") => "is_space",
        (Ty::Char, "is_alphanumeric") => "is_alnum",
        (Ty::Char, "is_numeric" | "is_ascii_digit" | "isdigit") => "is_digit",
        (Ty::Char, "to_uppercase") => "to_upper",
        (Ty::Char, "to_lowercase") => "to_lower",
        (Ty::List(_), "append" | "add" | "push_back") => "push",
        (Ty::List(_), "includes" | "has") => "contains",
        (Ty::List(_), "indexOf") => "index_of",
        (Ty::List(_), "pop_back") => "pop",
        (Ty::Map(..), "has" | "has_key" | "contains_key") => "contains",
        (Ty::Map(..), "delete") => "remove",
        (Ty::Set(_), "insert" | "push") => "add",
        (Ty::Set(_), "has") => "contains",
        (Ty::Set(_), "discard" | "delete") => "remove",
        _ => return None,
    })
}
