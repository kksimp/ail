//! Built-in methods (`check/builtins.rs` has their signatures), and the
//! insertion-ordered table behind `Map` and `Set` (type.map.1).

use std::cmp::Ordering;
use crate::fasthash::FastMap as HashMap;
use std::io::Write;
use std::rc::Rc;

use super::{Env, Flow, Interp, Operand, Trap, Value, element, float_text, load, put, take, trap, view, with_slot};

/// type.str.15: the largest text `repeat` or `pad_*` will build, in bytes.
const MAX_TEXT_BYTES: usize = 1 << 30;
use crate::ast::Arg;
use crate::diag::Pos;
use crate::num::Num;

/// The hashable form of a key (type.map.2): only types whose equality is
/// structural and exact can be keys, so equal keys have equal forms.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(super) enum Key {
    Int(i64),
    Bool(bool),
    Str(Rc<str>),
    Bytes(Rc<Vec<u8>>),
    Char(char),
    None,
    Nested(u32),
    Seq(Vec<Key>),
    Struct(usize, Vec<Key>),
    Variant(usize, usize, Vec<Key>),
}

impl Key {
    pub(super) fn of(v: &Value) -> Key {
        let all = |xs: &[Value]| xs.iter().map(Key::of).collect();
        match v {
            Value::Int(n) | Value::Num(_, n) => Key::Int(*n),
            Value::Bool(b) => Key::Bool(*b),
            Value::Str(s) => Key::Str(s.clone()),
            Value::Bytes(b) => Key::Bytes(b.clone()),
            Value::Char(c) => Key::Char(*c),
            Value::None => Key::None,
            Value::Nested(n) => Key::Nested(*n),
            Value::List(xs) => Key::Seq(all(xs)),
            Value::Struct(s, fields) => Key::Struct(*s, all(fields)),
            Value::Enum(e, i, fields) => Key::Variant(*e, *i, all(fields)),
            other => unreachable!("checked: {other:?} isn't `Hash`"),
        }
    }
}

/// type.coll.16: `union`, `intersect` and `difference` of the receiver's
/// elements `xs` with the argument (a set or a list), as a new set. The
/// result keeps the receiver's order, then the argument's new elements.
fn set_op<'v>(name: &str, xs: impl Iterator<Item = &'v Value>, other: &Value) -> Value {
    let mut other_set = Table::default();
    let other_items: Vec<&Value> = match other {
        Value::Set(t) => t.keys().collect(),
        Value::List(l) => l.iter().collect(),
        v => unreachable!("checked: a set or list, found {v:?}"),
    };
    for x in &other_items {
        other_set.insert((*x).clone(), None);
    }
    let mut out = Table::default();
    for x in xs {
        let keep = match name {
            "union" => true,
            "intersect" => other_set.get(x).is_some(),
            _ => other_set.get(x).is_none(),
        };
        if keep {
            out.insert(x.clone(), None);
        }
    }
    if name == "union" {
        for x in other_items {
            out.insert(x.clone(), None);
        }
    }
    Value::Set(Rc::new(out))
}

/// Keys in insertion order, with their values for a map (type.map.1). A
/// removed key leaves a gap, so removal is O(1); gaps are compacted away
/// once they outnumber the keys.
#[derive(Clone, Debug, Default)]
pub(super) struct Table {
    keys: Vec<Option<Value>>,
    /// Parallel to `keys` for a map; empty for a set.
    vals: Vec<Value>,
    index: HashMap<Key, usize>,
}

impl Table {
    /// The slot of `k`, if it is a key.
    pub fn get(&self, k: &Value) -> Option<usize> {
        self.index.get(&Key::of(k)).copied()
    }

    pub fn val(&self, i: usize) -> &Value {
        &self.vals[i]
    }

    pub fn val_mut(&mut self, i: usize) -> &mut Value {
        &mut self.vals[i]
    }

    /// The keys, in insertion order.
    pub fn keys(&self) -> impl Iterator<Item = &Value> {
        self.keys.iter().flatten()
    }

    /// The keys with their values, in insertion order (maps only).
    pub fn entries(&self) -> impl Iterator<Item = (&Value, &Value)> {
        self.keys.iter().zip(&self.vals).filter_map(|(k, v)| k.as_ref().map(|k| (k, v)))
    }

    pub fn key_list(&self) -> Vec<Value> {
        self.keys().cloned().collect()
    }

    pub fn val_list(&self) -> Vec<Value> {
        self.entries().map(|(_, v)| v.clone()).collect()
    }

    /// Insert or replace; a replaced key keeps its position. Returns its slot.
    pub fn insert(&mut self, k: Value, v: Option<Value>) -> usize {
        if let Some(i) = self.get(&k) {
            if let Some(v) = v {
                self.vals[i] = v;
            }
            return i;
        }
        self.index.insert(Key::of(&k), self.keys.len());
        self.keys.push(Some(k));
        if let Some(v) = v {
            self.vals.push(v);
        }
        self.keys.len() - 1
    }

    /// Remove a key, returning its value (for a map) if it was there. The
    /// others keep their order (type.map.1).
    pub fn remove(&mut self, k: &Value) -> Option<Option<Value>> {
        let i = self.index.remove(&Key::of(k))?;
        self.keys[i] = None;
        let v = if self.vals.is_empty() { None } else { Some(std::mem::replace(&mut self.vals[i], Value::Unit)) };
        let gaps = self.keys.len() - self.index.len();
        if gaps > 16 && gaps > self.index.len() {
            self.compact();
        }
        Some(v)
    }

    fn compact(&mut self) {
        let map = !self.vals.is_empty();
        let (keys, vals) = (std::mem::take(&mut self.keys), std::mem::take(&mut self.vals));
        self.index.clear();
        let mut vals = vals.into_iter();
        for k in keys {
            let v = if map { vals.next() } else { None };
            if let Some(k) = k {
                self.insert(k, v);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }
}

fn opt(v: Option<Value>) -> Value {
    v.map_or(Value::None, Value::present)
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
        if let [Arg { param: Some((var, _)), value, .. }] = args {
            return self.keyed(recv, name, var, value, pos, env);
        }
        // Arguments are read before the receiver changes, so they are copies (own.excl.4).
        let mut a = self.arg_pool.pop().unwrap_or_default();
        for arg in args {
            a.push(self.eval(&arg.value, env)?);
        }
        self.builtin_with(recv, name, a, pos, env)
    }

    /// `builtin` with the arguments' values in `a` (taken from `arg_pool`,
    /// which gets it back); the bytecode tier evaluates them itself (ADR 0363).
    pub(super) fn builtin_with(
        &mut self,
        recv: Operand,
        name: &str,
        mut a: Vec<Value>,
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        let mutating = |v: &Value| match v {
            Value::List(_) => matches!(name, "push" | "swap" | "pop" | "sort" | "insert" | "remove_at" | "extend" | "clear" | "reverse"),
            Value::Map(_) => matches!(name, "remove" | "push"),
            Value::Set(_) => matches!(name, "add" | "remove"),
            _ => false,
        };
        let is_mutating = match &recv {
            Operand::Place(p) => view(env, p, mutating)?,
            Operand::Value(v) => mutating(v),
        };
        if !is_mutating {
            let v = match recv {
                Operand::Place(p) => load(env, &p)?,
                Operand::Value(v) => v,
            };
            let r = self.read_only(v, name, &mut a, pos)?;
            a.clear();
            self.arg_pool.push(a);
            return Ok(r);
        }
        let Operand::Place(p) = recv else { unreachable!("checked: `{name}` needs a mutable place") };
        if name == "sort" {
            // Sorting may call `cmp` implementations, so the list is moved out first.
            let Value::List(items) = take(env, &p)? else { unreachable!("checked: a list") };
            let sorted = self.sorted(Rc::unwrap_or_clone(items), pos);
            let restored = match &sorted {
                Ok(xs) => xs.clone(),
                Err(_) => Vec::new(),
            };
            put(env, &p, false, Value::List(Rc::new(restored)))?;
            sorted?;
            if self.tracing & super::trace::WATCH != 0
                && let Some(before) = self.watch_before(env, &p, name, &a)
            {
                self.watch_after(env, &p, name, before, pos);
            }
            return Ok(Value::Unit);
        }
        if name == "clear" && self.has_unique {
            // own.uniq.8: the removed elements are released.
            let old = with_slot(env, &p, false, |slot| std::mem::replace(slot, Value::List(Rc::new(Vec::new()))))?;
            a.clear();
            self.arg_pool.push(a);
            self.release(old, pos)?;
            return Ok(Value::Unit);
        }
        let watching = if self.tracing & super::trace::WATCH != 0 { self.watch_before(env, &p, name, &a) } else { None };
        let result = with_slot(env, &p, false, |slot| -> Result<Value, Trap> {
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
            // type.coll.14: `insert` allows the index `len`, `remove_at` doesn't.
            (Value::List(items), "insert") => {
                let i = a[0].int();
                if usize::try_from(i).map_or(true, |u| u > items.len()) {
                    trap::<()>(
                        "type.coll.4",
                        pos,
                        format!("insert index {i} is out of bounds for a list of length {}", items.len()),
                    )?;
                }
                let x = a.pop().expect("checked: two arguments");
                Rc::make_mut(items).insert(i as usize, x);
                Value::Unit
            }
            (Value::List(items), "remove_at") => {
                let i = a[0].int();
                element(items, i, pos)?;
                Rc::make_mut(items).remove(i as usize)
            }
            (Value::List(items), "extend") => {
                let Value::List(more) = &a[0] else { unreachable!("checked: a list") };
                Rc::make_mut(items).extend(more.iter().cloned());
                Value::Unit
            }
            (Value::List(items), "clear") => {
                Rc::make_mut(items).clear();
                Value::Unit
            }
            (Value::List(items), "reverse") => {
                Rc::make_mut(items).reverse();
                Value::Unit
            }
            // type.map.8: append to the list at `k`, starting an empty one.
            (Value::Map(t), "push") => {
                let x = a.pop().expect("checked: two arguments");
                let table = Rc::make_mut(t);
                let i = match table.get(&a[0]) {
                    Some(i) => i,
                    None => table.insert(a[0].clone(), Some(Value::List(Rc::new(Vec::new())))),
                };
                let Value::List(items) = table.val_mut(i) else { unreachable!("checked: a list value") };
                Rc::make_mut(items).push(x);
                Value::Unit
            }
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
        })??;
        if let Some(before) = watching {
            self.watch_after(env, &p, name, before, pos);
        }
        a.clear();
        self.arg_pool.push(a);
        Ok(result)
    }

    fn read_only(&mut self, v: Value, name: &str, a: &mut Vec<Value>, pos: Pos) -> Result<Value, Trap> {
        Ok(match v {
            Value::Str(text) => self.str_method(&text, name, a, pos)?,
            Value::Bytes(b) => super::bytes::method(&b, name, a),
            Value::Char(c) => char_method(c, name, a, pos)?,
            Value::Int(n) => int_method(n, name, a, pos)?,
            Value::Num(k, bits) if k.is_float() => f32_method(bits, name, pos)?,
            Value::Num(k, bits) => sized_method(k, bits, name, a, pos)?,
            Value::Float(x) => float_method(x, name, pos)?,
            Value::List(items) => match name {
                "len" => Value::Int(items.len() as i64),
                "last" => opt(items.last().cloned()),
                "first" => opt(items.first().cloned()),
                // type.coll.15: the first `n` elements, or all of them.
                "take" => {
                    let n = a[0].int();
                    if n < 0 {
                        return trap("type.coll.15", pos, format!("`take` needs a count of at least 0, not {n}"));
                    }
                    Value::List(Rc::new(items[..(n as usize).min(items.len())].to_vec()))
                }
                // type.coll.17: `chunks(n)` pieces of `n` (the last may be shorter),
                // `windows(n)` every run of `n` neighbours.
                "chunks" | "windows" => {
                    let n = a[0].int();
                    if n < 1 {
                        return trap("type.coll.17", pos, format!("`{name}` needs a size of at least 1, not {n}"));
                    }
                    let n = n as usize;
                    let parts: Vec<Value> = if name == "chunks" {
                        items.chunks(n).map(|c| Value::List(Rc::new(c.to_vec()))).collect()
                    } else {
                        items.windows(n).map(|c| Value::List(Rc::new(c.to_vec()))).collect()
                    };
                    Value::List(Rc::new(parts))
                }
                // type.coll.18: how many times each element occurs, in order of first appearance.
                "counts" => {
                    let mut out = Table::default();
                    for x in items.iter() {
                        match out.get(x) {
                            Some(i) => {
                                let n = out.val(i).int();
                                *out.val_mut(i) = Value::Int(n + 1);
                            }
                            None => {
                                out.insert(x.clone(), Some(Value::Int(1)));
                            }
                        }
                    }
                    Value::Map(Rc::new(out))
                }
                // type.coll.19: one list of all the elements of the inner lists, in order.
                "flatten" => {
                    let mut out = Vec::new();
                    for inner in items.iter() {
                        let Value::List(l) = inner else { unreachable!("checked: a list of lists") };
                        out.extend(l.iter().cloned());
                    }
                    Value::List(Rc::new(out))
                }
                "union" | "intersect" | "difference" => set_op(name, items.iter(), &a[0]),
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
                    Some(Value::Num(k, _)) if k.is_float() => {
                        let mut total = 0.0f32;
                        for x in items.iter() {
                            if let Value::Num(_, bits) = x {
                                total += Num::f32_of(*bits);
                            }
                        }
                        Value::Num(*k, Num::f32_bits(total))
                    }
                    Some(Value::Num(k, _)) => {
                        let mut total: i128 = 0;
                        for x in items.iter() {
                            if let Value::Num(_, bits) = x {
                                total += k.decode(*bits);
                            }
                            if !k.fits(total) {
                                return trap("expr.arith.1", pos, format!("`sum` overflows `{}`", k.name()));
                            }
                        }
                        Value::Num(*k, k.encode(total))
                    }
                    Some(Value::Float(_)) => {
                        let xs: Vec<f64> = items.iter().map(|x| x.float()).collect();
                        Value::Float(tree_sum(&xs))
                    }
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
                "get" => opt(t.get(&a[0]).map(|i| t.val(i).clone())),
                "contains" => Value::Bool(t.get(&a[0]).is_some()),
                "keys" => Value::List(Rc::new(t.key_list())),
                "values" => Value::List(Rc::new(t.val_list())),
                _ => unreachable!("checked: Map.{name}"),
            },
            Value::Set(t) => match name {
                "len" => Value::Int(t.len() as i64),
                "contains" => Value::Bool(t.get(&a[0]).is_some()),
                "sorted" => Value::List(Rc::new(self.sorted(t.key_list(), pos)?)),
                "union" | "intersect" | "difference" => set_op(name, t.keys(), &a[0]),
                "is_subset" => {
                    let Value::Set(other) = a.pop().expect("checked: one argument") else {
                        unreachable!("checked: set")
                    };
                    Value::Bool(t.keys().all(|k| other.get(k).is_some()))
                }
                _ => unreachable!("checked: Set.{name}"),
            },
            other => unreachable!("checked: `{name}` on {other:?}"),
        })
    }

    fn str_method(&mut self, text: &str, name: &str, a: &[Value], pos: Pos) -> Result<Value, Trap> {
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
            "len" => Value::Int(text.chars().count() as i64),
            "byte_len" => Value::Int(text.len() as i64),
            "to_bytes" => super::bytes::bytes_value(text.as_bytes().to_vec()),
            "lines" => Value::List(Rc::new(text.lines().map(s).collect())),
            "trim" => s(text.trim()),
            "trim_start" => s(text.trim_start()),
            "trim_end" => s(text.trim_end()),
            // type.str.17: the pieces between runs of whitespace; none for blank text.
            "words" => Value::List(Rc::new(text.split_whitespace().map(s).collect())),
            // type.str.18: non-overlapping occurrences of `sub`.
            "count" => {
                let sub = arg_str(0);
                if sub.is_empty() {
                    return trap("type.str.18", pos, "`count` needs a non-empty text to find".into());
                }
                Value::Int(text.matches(&*sub).count() as i64)
            }
            "rfind" => {
                let sub = arg_str(0);
                opt(text.rfind(&*sub).map(|b| Value::Int(text[..b].chars().count() as i64)))
            }
            "to_lower" => s(text.to_lowercase()),
            "to_upper" => s(text.to_uppercase()),
            // type.str.13: character positions; bounds are never clamped.
            "slice" => {
                let (lo, hi) = (a[0].int(), a[1].int());
                let n = text.chars().count();
                match (usize::try_from(lo), usize::try_from(hi)) {
                    (Ok(l), Ok(h)) if l <= h && h <= n => {
                        s(text.chars().skip(l).take(h - l).collect::<String>())
                    }
                    _ => {
                        return trap(
                            "type.str.13",
                            pos,
                            format!("slice({lo}, {hi}) is out of bounds for a str of length {n}"),
                        );
                    }
                }
            }
            "char_at" => opt(usize::try_from(a[0].int()).ok().and_then(|i| text.chars().nth(i)).map(Value::Char)),
            "find" => {
                let sub = arg_str(0);
                opt(text.find(&*sub).map(|b| Value::Int(text[..b].chars().count() as i64)))
            }
            "replace" => {
                let from = arg_str(0);
                if from.is_empty() {
                    return trap("type.str.14", pos, "`replace` needs a non-empty text to find".into());
                }
                s(text.replace(&*from, &arg_str(1)))
            }
            "starts_with" => Value::Bool(text.starts_with(&*arg_str(0))),
            "ends_with" => Value::Bool(text.ends_with(&*arg_str(0))),
            "contains" => Value::Bool(text.contains(&*arg_str(0))),
            "strip_prefix" => opt(text.strip_prefix(&*arg_str(0)).map(s)),
            "strip_suffix" => opt(text.strip_suffix(&*arg_str(0)).map(s)),
            "repeat" => match usize::try_from(a[0].int()) {
                Ok(n) if n.saturating_mul(text.len()) > MAX_TEXT_BYTES => {
                    return trap("type.str.15", pos, format!("`repeat` would build a text over {MAX_TEXT_BYTES} bytes"));
                }
                Ok(n) => s(text.repeat(n)),
                Err(_) => {
                    return trap(
                        "type.str.10",
                        pos,
                        format!("`repeat` needs a count of 0 or more, not {}", a[0].int()),
                    );
                }
            },
            "pad_left" | "pad_right" | "center" => {
                // type.str.9: up to `width` characters; never shorter.
                let width = usize::try_from(a[0].int()).unwrap_or(0);
                let fill_len = a[1].char().len_utf8().saturating_mul(width);
                if fill_len > MAX_TEXT_BYTES {
                    return trap("type.str.15", pos, format!("`{name}` would build a text over {MAX_TEXT_BYTES} bytes"));
                }
                let fill = a[1].char().to_string().repeat(width.saturating_sub(text.chars().count()));
                if name == "center" {
                    // type.str.16: the extra fill goes on the right.
                    let n = fill.chars().count();
                    let left: String = fill.chars().take(n / 2).collect();
                    let right: String = fill.chars().skip(n / 2).collect();
                    s(format!("{left}{text}{right}"))
                } else if name == "pad_left" {
                    s(format!("{fill}{text}"))
                } else {
                    s(format!("{text}{fill}"))
                }
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
        // expr.arith.5: `a % -1` is 0 for every `a`, so it never overflows.
        "checked_rem" => checked(if m() == -1 { Some(0) } else { n.checked_rem(m()) }),
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
        // expr.bit.1: `&`, `|`, `^` and `!` on the two's complement bits.
        "bit_and" => Value::Int(n & m()),
        "bit_or" => Value::Int(n | m()),
        "bit_xor" => Value::Int(n ^ m()),
        "bit_not" => Value::Int(!n),
        // expr.bit.1: bits shifted past the top are discarded; `shr` keeps the sign.
        // expr.bit.2: the count must be in 0..=63.
        "shl" | "shr" => {
            let k = m();
            if !(0..=63).contains(&k) {
                return trap("expr.bit.2", pos, format!("shift count `{k}` is outside `0..=63`"));
            }
            if name == "shl" { Value::Int(n.wrapping_shl(k as u32)) } else { Value::Int(n >> k) }
        }
        "to_float" => Value::Float(n as f64),
        "to_hex" => s(if n < 0 { format!("-{:x}", n.unsigned_abs()) } else { format!("{n:x}") }),
        _ => convert_int(i128::from(n), name, pos)?,
    })
}

/// `to_i8` ... `to_f32` on an integer (type.num.15): exact where the value
/// fits, a trap where it doesn't. `to_f32` rounds to nearest, as `to_float` does.
fn convert_int(v: i128, name: &str, pos: Pos) -> Result<Value, Trap> {
    let Some(&(_, k)) = crate::num::CONVERSIONS.iter().find(|(n, _)| *n == name) else {
        unreachable!("checked: integer method {name}")
    };
    if k.is_float() {
        return Ok(Value::Num(k, Num::f32_bits(v as f32)));
    }
    if !k.fits(v) {
        return trap("type.num.15", pos, format!("`{v}` doesn't fit in `{}`", k.name()));
    }
    Ok(Value::Num(k, k.encode(v)))
}

/// Methods of the sized integers (type.num.14-15), computed exactly in 128 bits.
fn sized_method(k: Num, bits: i64, name: &str, a: &[Value], pos: Pos) -> Result<Value, Trap> {
    let n = k.decode(bits);
    let m = || match &a[0] {
        Value::Num(_, b) => k.decode(*b),
        other => unreachable!("checked: {other:?} is not a {}", k.name()),
    };
    let (lo, hi) = k.range();
    let wrap = |v: i128| {
        let span = hi - lo + 1;
        lo + (v - lo).rem_euclid(span)
    };
    let value = |v: i128| Value::Num(k, k.encode(v));
    let checked = |r: Option<i128>| opt(r.filter(|v| k.fits(*v)).map(value));
    Ok(match name {
        "checked_add" => checked(Some(n + m())),
        "checked_sub" => checked(Some(n - m())),
        "checked_mul" => checked(n.checked_mul(m())),
        "checked_div" => checked(n.checked_div(m())),
        "checked_rem" => checked(n.checked_rem(m())),
        "checked_neg" => checked(Some(-n)),
        "wrapping_add" => value(wrap(n + m())),
        "wrapping_sub" => value(wrap(n - m())),
        "wrapping_mul" => value(wrap(n.wrapping_mul(m()))), // 2^128 is a multiple of every span
        "wrapping_neg" => value(wrap(-n)),
        "saturating_add" => value((n + m()).clamp(lo, hi)),
        "saturating_sub" => value((n - m()).clamp(lo, hi)),
        "saturating_mul" => value(n.checked_mul(m()).map_or(if (n < 0) == (m() < 0) { hi } else { lo }, |v| v.clamp(lo, hi))),
        "abs" if k.fits(n.abs()) => value(n.abs()),
        "abs" => return trap("type.num.14", pos, format!("`({n}).abs()` overflows `{}`", k.name())),
        "to_int" => match i64::try_from(n) {
            Ok(v) => Value::Int(v),
            Err(_) => return trap("type.num.15", pos, format!("`{n}` doesn't fit in `int`")),
        },
        "to_float" => Value::Float(n as f64),
        "to_hex" => s(if n < 0 { format!("-{:x}", n.unsigned_abs()) } else { format!("{n:x}") }),
        _ => convert_int(n, name, pos)?,
    })
}

/// Methods of `f32` (type.num.14-15).
fn f32_method(bits: i64, name: &str, pos: Pos) -> Result<Value, Trap> {
    let x = Num::f32_of(bits);
    let f = |v: f32| Value::Num(Num::F32, Num::f32_bits(v));
    Ok(match name {
        "abs" => f(x.abs()),
        "floor" => f(x.floor()),
        "ceil" => f(x.ceil()),
        "round" => f(x.round()),
        "sqrt" => f(x.sqrt()),
        "to_float" => Value::Float(f64::from(x)),
        "to_int" => return float_method(f64::from(x), "to_int", pos),
        _ => unreachable!("checked: f32.{name}"),
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
            if x.is_nan() || !(i64::MIN as f64..-(i64::MIN as f64)).contains(&x) {
                return trap("type.num.9", pos, format!("`{}` doesn't fit in `int`", float_text(x)));
            }
            Value::Int(x.trunc() as i64)
        }
        "to_f32" => Value::Num(Num::F32, Num::f32_bits(x as f32)),
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
        Some(i) => Ok(t.val(i)),
        None => trap("type.map.3", pos, format!("the map has no key {}", shown())),
    }
}

/// A float list's sum in the fixed tree of type.coll.12 (ADR 0239). A
/// leaf of up to 32 elements keeps four running sums, element `k` going to
/// sum `k % 4`, and gives `(p0 + p1) + (p2 + p3)`; a longer list splits
/// after `32 * ceil(n / 64)` elements and adds its halves' sums. The shape
/// depends only on the length, so a vector or parallel sum gives the same bits.
pub fn tree_sum(xs: &[f64]) -> f64 {
    if xs.len() <= 32 {
        let mut p = [0.0f64; 4];
        for (k, x) in xs.iter().enumerate() {
            p[k % 4] += x;
        }
        return (p[0] + p[1]) + (p[2] + p[3]);
    }
    let m = 32 * xs.len().div_ceil(64);
    tree_sum(&xs[..m]) + tree_sum(&xs[m..])
}
