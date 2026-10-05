//! Built-in methods (`check/builtins.rs` has their signatures), and the
//! insertion-ordered table behind `Map` and `Set` (type.map.1).

use std::cmp::Ordering;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;

use super::{Env, Flow, Interp, Operand, Trap, Value, element, float_text, peek, slot_mut, trap};
use crate::ast::Arg;
use crate::diag::Pos;

/// The hashable form of a key (type.map.2): only types whose equality is
/// structural and exact can be keys, so equal keys have equal forms.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Int(i64),
    Bool(bool),
    Str(Rc<str>),
    Char(char),
    None,
    Seq(Vec<Key>),
    Struct(usize, Vec<Key>),
    Variant(usize, usize, Vec<Key>),
}

impl Key {
    pub(super) fn of(v: &Value) -> Key {
        let all = |xs: &[Value]| xs.iter().map(Key::of).collect();
        match v {
            Value::Int(n) => Key::Int(*n),
            Value::Bool(b) => Key::Bool(*b),
            Value::Str(s) => Key::Str(s.clone()),
            Value::Char(c) => Key::Char(*c),
            Value::None => Key::None,
            Value::List(xs) => Key::Seq(all(xs)),
            Value::Struct(s, fields) => Key::Struct(*s, all(fields)),
            Value::Enum(e, i, fields) => Key::Variant(*e, *i, all(fields)),
            other => unreachable!("checked: {other:?} isn't `Hash`"),
        }
    }
}

/// Keys in insertion order, with their values for a map (type.map.1).
#[derive(Clone, Debug, Default)]
pub(super) struct Table {
    pub keys: Vec<Value>,
    /// Parallel to `keys` for a map; empty for a set.
    pub vals: Vec<Value>,
    index: HashMap<Key, usize>,
}

impl Table {
    pub fn get(&self, k: &Value) -> Option<usize> {
        self.index.get(&Key::of(k)).copied()
    }

    /// Insert or replace; a replaced key keeps its position. Returns its index.
    pub fn insert(&mut self, k: Value, v: Option<Value>) -> usize {
        if let Some(i) = self.get(&k) {
            if let Some(v) = v {
                self.vals[i] = v;
            }
            return i;
        }
        self.index.insert(Key::of(&k), self.keys.len());
        self.keys.push(k);
        if let Some(v) = v {
            self.vals.push(v);
        }
        self.keys.len() - 1
    }

    /// Remove a key, returning its value (for a map) if it was there.
    pub fn remove(&mut self, k: &Value) -> Option<Option<Value>> {
        let i = self.index.remove(&Key::of(k))?;
        self.keys.remove(i);
        let v = if self.vals.is_empty() { None } else { Some(self.vals.remove(i)) };
        for (j, key) in self.keys.iter().enumerate().skip(i) {
            self.index.insert(Key::of(key), j);
        }
        Some(v)
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }
}

fn opt(v: Option<Value>) -> Value {
    v.unwrap_or(Value::None)
}

fn s(text: impl AsRef<str>) -> Value {
    Value::Str(Rc::from(text.as_ref()))
}

/// `s.to_float()` (type.num.8): an optional `-`, digits, an optional
/// fraction, and an optional exponent.
fn parse_float(text: &str) -> Option<f64> {
    // The text forms of the special values read back (type.num.8).
    match text {
        "inf" => return Some(f64::INFINITY),
        "-inf" => return Some(f64::NEG_INFINITY),
        "nan" => return Some(f64::NAN),
        _ => {}
    }
    let b = text.as_bytes();
    let mut i = usize::from(b.first() == Some(&b'-'));
    let digits = |i: &mut usize| {
        let start = *i;
        while *i < b.len() && b[*i].is_ascii_digit() {
            *i += 1;
        }
        *i > start
    };
    if !digits(&mut i) {
        return None;
    }
    if i < b.len() && b[i] == b'.' {
        i += 1;
        if !digits(&mut i) {
            return None;
        }
    }
    if i < b.len() && (b[i] == b'e' || b[i] == b'E') {
        i += 1;
        if i < b.len() && (b[i] == b'+' || b[i] == b'-') {
            i += 1;
        }
        if !digits(&mut i) {
            return None;
        }
    }
    if i != b.len() {
        return None;
    }
    text.parse().ok()
}

impl<'a, W: Write> Interp<'a, W> {
    /// A built-in method of a value of a built-in type.
    pub(super) fn builtin(
        &mut self,
        recv: Operand,
        name: &str,
        args: &'a [Arg],
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        // Arguments are read before the receiver changes, so they are copies (own.excl.4).
        let mut a = Vec::with_capacity(args.len());
        for arg in args {
            a.push(self.eval(&arg.value, env)?);
        }
        let current = match &recv {
            Operand::Place(p) => peek(env, p)?,
            Operand::Value(v) => v,
        };
        let mutating = match current {
            Value::List(_) => matches!(name, "push" | "swap" | "pop" | "sort"),
            Value::Map(_) => name == "remove",
            Value::Set(_) => matches!(name, "add" | "remove"),
            _ => false,
        };
        if !mutating {
            let v = current.clone();
            return Ok(self.read_only(v, name, a, pos)?);
        }
        let Operand::Place(p) = recv else { unreachable!("checked: `{name}` needs a mutable place") };
        if name == "sort" {
            // Sorting may call `cmp` implementations, so the list is moved out first.
            let Value::List(items) = std::mem::replace(slot_mut(env, &p, false)?, Value::Unit) else {
                unreachable!("checked: a list")
            };
            let sorted = self.sorted(Rc::unwrap_or_clone(items), pos);
            let restored = match &sorted {
                Ok(xs) => xs.clone(),
                Err(_) => Vec::new(),
            };
            *slot_mut(env, &p, false)? = Value::List(Rc::new(restored));
            sorted?;
            return Ok(Value::Unit);
        }
        let slot = slot_mut(env, &p, false)?;
        Ok(match (slot, name) {
            (Value::List(items), "push") => {
                Rc::make_mut(items).push(a.pop().expect("checked: one argument"));
                Value::Unit
            }
            (Value::List(items), "swap") => {
                let (i, j) = (a[0].int(), a[1].int());
                element(items, i, pos)?;
                element(items, j, pos)?;
                Rc::make_mut(items).swap(i as usize, j as usize);
                Value::Unit
            }
            (Value::List(items), "pop") => opt(Rc::make_mut(items).pop()),
            (Value::Map(t), "remove") => opt(Rc::make_mut(t).remove(&a[0]).flatten()),
            (Value::Set(t), "add") => {
                let x = a.pop().expect("checked: one argument");
                let new = t.get(&x).is_none();
                Rc::make_mut(t).insert(x, None);
                Value::Bool(new)
            }
            (Value::Set(t), "remove") => Value::Bool(Rc::make_mut(t).remove(&a[0]).is_some()),
            (other, _) => unreachable!("checked: `{name}` on {other:?}"),
        })
    }

    fn read_only(&mut self, v: Value, name: &str, mut a: Vec<Value>, pos: Pos) -> Result<Value, Trap> {
        Ok(match v {
            Value::Str(text) => self.str_method(&text, name, a, pos)?,
            Value::Char(c) => char_method(c, name, &a, pos)?,
            Value::Int(n) => int_method(n, name, &a, pos)?,
            Value::Float(x) => float_method(x, name, pos)?,
            Value::List(items) => match name {
                "len" => Value::Int(items.len() as i64),
                "last" => opt(items.last().cloned()),
                "contains" | "index_of" => {
                    let mut found = None;
                    for (i, x) in items.iter().enumerate() {
                        if self.equal(x, &a[0], pos)? {
                            found = Some(i);
                            break;
                        }
                    }
                    if name == "contains" {
                        Value::Bool(found.is_some())
                    } else {
                        opt(found.map(|i| Value::Int(i as i64)))
                    }
                }
                "join" => {
                    let Value::Str(sep) = &a[0] else { unreachable!("checked: str") };
                    let mut parts = Vec::with_capacity(items.len());
                    for x in items.iter() {
                        parts.push(self.text(x, pos)?);
                    }
                    s(parts.join(sep))
                }
                "sum" => match items.first() {
                    Some(Value::Float(_)) => Value::Float(items.iter().map(|x| x.float()).sum()),
                    _ => {
                        let mut total: i64 = 0;
                        for x in items.iter() {
                            total = match total.checked_add(x.int()) {
                                Some(t) => t,
                                None => return trap("expr.arith.1", pos, "`sum` overflows `int`".into()),
                            };
                        }
                        Value::Int(total)
                    }
                },
                "min" | "max" => {
                    let mut best: Option<Value> = None;
                    for x in items.iter() {
                        let better = match &best {
                            None => true,
                            Some(b) => {
                                let o = self.compare(x, b, pos)?;
                                if name == "min" { o == Ordering::Less } else { o == Ordering::Greater }
                            }
                        };
                        if better {
                            best = Some(x.clone());
                        }
                    }
                    opt(best)
                }
                "sorted" => Value::List(Rc::new(self.sorted(Rc::unwrap_or_clone(items), pos)?)),
                _ => unreachable!("checked: List.{name}"),
            },
            Value::Map(t) => match name {
                "len" => Value::Int(t.len() as i64),
                "get" => opt(t.get(&a[0]).map(|i| t.vals[i].clone())),
                "contains" => Value::Bool(t.get(&a[0]).is_some()),
                "keys" => Value::List(Rc::new(t.keys.clone())),
                "values" => Value::List(Rc::new(t.vals.clone())),
                _ => unreachable!("checked: Map.{name}"),
            },
            Value::Set(t) => match name {
                "len" => Value::Int(t.len() as i64),
                "contains" => Value::Bool(t.get(&a[0]).is_some()),
                "sorted" => Value::List(Rc::new(self.sorted(t.keys.clone(), pos)?)),
                "is_subset" => {
                    let Value::Set(other) = a.pop().expect("checked: one argument") else {
                        unreachable!("checked: set")
                    };
                    Value::Bool(t.keys.iter().all(|k| other.get(k).is_some()))
                }
                _ => unreachable!("checked: Set.{name}"),
            },
            other => unreachable!("checked: `{name}` on {other:?}"),
        })
    }

    fn str_method(&mut self, text: &str, name: &str, a: Vec<Value>, pos: Pos) -> Result<Value, Trap> {
        // type.str.6: a `char` argument works like a one-character string.
        let arg_str = |i: usize| match &a[i] {
            Value::Str(t) => t.clone(),
            Value::Char(c) => Rc::from(c.to_string()),
            other => unreachable!("checked: str, found {other:?}"),
        };
        Ok(match name {
            "split" => {
                let sep = arg_str(0);
                if sep.is_empty() {
                    return trap("type.str.4", pos, "`split` needs a non-empty separator".into());
                }
                Value::List(Rc::new(text.split(&*sep).map(s).collect()))
            }
            "to_int" => {
                let digits = text.strip_prefix('-').unwrap_or(text);
                let valid = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit());
                match text.parse::<i64>() {
                    Ok(n) if valid => Value::Int(n),
                    _ => Value::None,
                }
            }
            "to_float" => opt(parse_float(text).map(Value::Float)),
            "chars" => Value::List(Rc::new(text.chars().map(Value::Char).collect())),
            "lines" => Value::List(Rc::new(text.lines().map(s).collect())),
            "trim" => s(text.trim()),
            "starts_with" => Value::Bool(text.starts_with(&*arg_str(0))),
            "ends_with" => Value::Bool(text.ends_with(&*arg_str(0))),
            "contains" => Value::Bool(text.contains(&*arg_str(0))),
            "strip_prefix" => opt(text.strip_prefix(&*arg_str(0)).map(s)),
            "strip_suffix" => opt(text.strip_suffix(&*arg_str(0)).map(s)),
            "repeat" => match usize::try_from(a[0].int()) {
                Ok(n) => s(text.repeat(n)),
                Err(_) => {
                    return trap("type.str.10", pos, format!("`repeat` needs a count of 0 or more, not {}", a[0].int()));
                }
            },
            "pad_left" | "pad_right" => {
                // type.str.9: up to `width` characters; never shorter.
                let width = usize::try_from(a[0].int()).unwrap_or(0);
                let fill = a[1].char().to_string().repeat(width.saturating_sub(text.chars().count()));
                if name == "pad_left" { s(format!("{fill}{text}")) } else { s(format!("{text}{fill}")) }
            }
            _ => unreachable!("checked: str.{name}"),
        })
    }

    /// A stable merge sort by `cmp` (type.coll.11), which may call user code.
    pub(super) fn sorted(&mut self, mut xs: Vec<Value>, pos: Pos) -> Result<Vec<Value>, Trap> {
        if xs.len() <= 1 {
            return Ok(xs);
        }
        let right = xs.split_off(xs.len() / 2);
        let left = self.sorted(xs, pos)?;
        let right = self.sorted(right, pos)?;
        let mut out = Vec::with_capacity(left.len() + right.len());
        let (mut l, mut r) = (left.into_iter().peekable(), right.into_iter().peekable());
        while let (Some(x), Some(y)) = (l.peek(), r.peek()) {
            if self.compare(y, x, pos)? == Ordering::Less {
                out.push(r.next().expect("peeked"));
            } else {
                out.push(l.next().expect("peeked"));
            }
        }
        out.extend(l);
        out.extend(r);
        Ok(out)
    }

}

fn char_method(c: char, name: &str, a: &[Value], pos: Pos) -> Result<Value, Trap> {
    // A case mapping that gives more than one character leaves `c` unchanged.
    let one = |mapped: String| {
        let mut it = mapped.chars();
        match (it.next(), it.next()) {
            (Some(m), None) => m,
            _ => c,
        }
    };
    Ok(match name {
        "is_digit" => Value::Bool(c.is_ascii_digit()),
        "is_alpha" => Value::Bool(c.is_alphabetic()),
        "is_alnum" => Value::Bool(c.is_alphabetic() || c.is_ascii_digit()),
        "is_space" => Value::Bool(c.is_whitespace()),
        "is_upper" => Value::Bool(c.is_uppercase()),
        "is_lower" => Value::Bool(c.is_lowercase()),
        "to_upper" => Value::Char(one(c.to_uppercase().collect())),
        "to_lower" => Value::Char(one(c.to_lowercase().collect())),
        "code" => Value::Int(i64::from(u32::from(c))),
        "to_digit" => {
            let radix = a[0].int();
            if !(2..=36).contains(&radix) {
                return trap("type.char.3", pos, format!("`to_digit` needs a radix from 2 to 36, not {radix}"));
            }
            opt(c.to_digit(radix as u32).map(|d| Value::Int(i64::from(d))))
        }
        _ => unreachable!("checked: char.{name}"),
    })
}

fn int_method(n: i64, name: &str, a: &[Value], pos: Pos) -> Result<Value, Trap> {
    let m = || a[0].int();
    let checked = |r: Option<i64>| opt(r.map(Value::Int));
    Ok(match name {
        "checked_add" => checked(n.checked_add(m())),
        "checked_sub" => checked(n.checked_sub(m())),
        "checked_mul" => checked(n.checked_mul(m())),
        "checked_div" => checked(n.checked_div(m())),
        "checked_neg" => checked(n.checked_neg()),
        "checked_rem" => checked(n.checked_rem(m())),
        "wrapping_neg" => Value::Int(n.wrapping_neg()),
        "wrapping_add" => Value::Int(n.wrapping_add(m())),
        "wrapping_sub" => Value::Int(n.wrapping_sub(m())),
        "wrapping_mul" => Value::Int(n.wrapping_mul(m())),
        "saturating_add" => Value::Int(n.saturating_add(m())),
        "saturating_sub" => Value::Int(n.saturating_sub(m())),
        "saturating_mul" => Value::Int(n.saturating_mul(m())),
        "abs" => match n.checked_abs() {
            Some(v) => Value::Int(v),
            None => return trap("expr.arith.1", pos, format!("`({n}).abs()` overflows `int`")),
        },
        "to_float" => Value::Float(n as f64),
        "to_hex" => s(if n < 0 { format!("-{:x}", n.unsigned_abs()) } else { format!("{n:x}") }),
        _ => unreachable!("checked: int.{name}"),
    })
}

fn float_method(x: f64, name: &str, pos: Pos) -> Result<Value, Trap> {
    Ok(match name {
        "abs" => Value::Float(x.abs()),
        "floor" => Value::Float(x.floor()),
        "ceil" => Value::Float(x.ceil()),
        "round" => Value::Float(x.round()),
        "sqrt" => Value::Float(x.sqrt()),
        "to_int" => {
            // type.num.9: truncates toward zero; NaN and out-of-range values trap.
            if x.is_nan() || !(-9.223_372_036_854_775_808e18..9.223_372_036_854_775_808e18).contains(&x) {
                return trap("type.num.9", pos, format!("`{}` doesn't fit in `int`", float_text(x)));
            }
            Value::Int(x.trunc() as i64)
        }
        _ => unreachable!("checked: float.{name}"),
    })
}

/// `m[k]` (type.map.3): the value, or a trap if `k` isn't a key.
pub(super) fn map_value<'v>(
    t: &'v Table,
    k: &Value,
    shown: impl FnOnce() -> String,
    pos: Pos,
) -> Result<&'v Value, Trap> {
    match t.get(k) {
        Some(i) => Ok(&t.vals[i]),
        None => trap("type.map.3", pos, format!("the map has no key {}", shown())),
    }
}
