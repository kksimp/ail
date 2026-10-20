//! Key functions: `xs.sorted(by: |x| key)` and friends (expr.key.1-4, ADR 0250).

use std::cmp::Ordering;
use std::io::Write;
use std::rc::Rc;

use super::{Env, Flow, Interp, Operand, Value, peek, slot_mut};
use crate::ast::Expr;
use crate::diag::Pos;

impl<'a, W: Write> Interp<'a, W> {
    /// `sorted`, `sort`, `min` or `max` with `by: |var| key`. Each key is
    /// computed once, in element order, before any comparison (expr.key.3);
    /// sorting is stable and `min`/`max` keep the first of equal keys (expr.key.4).
    pub(super) fn keyed(
        &mut self,
        recv: Operand,
        name: &str,
        var: &'a str,
        key: &'a Expr,
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        let items = match &recv {
            Operand::Place(p) => peek(env, p)?.clone(),
            Operand::Value(v) => v.clone(),
        };
        let items: Vec<Value> = match items {
            Value::List(xs) => Rc::unwrap_or_clone(xs),
            Value::Set(t) => t.key_list(),
            other => unreachable!("checked: a list or set, found {other:?}"),
        };
        let mut keys = Vec::with_capacity(items.len());
        for x in &items {
            env.push((var, x.clone()));
            let k = self.eval(key, env);
            env.pop();
            keys.push(k?);
        }
        if name == "min" || name == "max" {
            let mut best: Option<usize> = None;
            for i in 0..items.len() {
                let better = match best {
                    None => true,
                    Some(b) => {
                        let o = self.compare(&keys[i], &keys[b], pos)?;
                        if name == "min" { o == Ordering::Less } else { o == Ordering::Greater }
                    }
                };
                if better {
                    best = Some(i);
                }
            }
            return Ok(best.map_or(Value::None, |i| items[i].clone()));
        }
        let pairs: Vec<Value> =
            keys.into_iter().zip(items).map(|(k, x)| Value::List(Rc::new(vec![k, x]))).collect();
        let order = self.sorted_by_first(pairs, pos)?;
        let out: Vec<Value> = order
            .into_iter()
            .map(|p| match p {
                Value::List(kx) => kx[1].clone(),
                _ => unreachable!("a key and an element"),
            })
            .collect();
        if name == "sort" {
            let Operand::Place(p) = recv else { unreachable!("checked: `sort` needs a mutable place") };
            *slot_mut(env, &p, false)? = Value::List(Rc::new(out));
            return Ok(Value::Unit);
        }
        Ok(Value::List(Rc::new(out)))
    }

    /// A stable merge sort of `[key, element]` pairs by key.
    fn sorted_by_first(&mut self, mut xs: Vec<Value>, pos: Pos) -> Result<Vec<Value>, super::Trap> {
        if xs.len() <= 1 {
            return Ok(xs);
        }
        let right = xs.split_off(xs.len() / 2);
        let left = self.sorted_by_first(xs, pos)?;
        let right = self.sorted_by_first(right, pos)?;
        let key = |v: &Value| match v {
            Value::List(kx) => kx[0].clone(),
            _ => unreachable!("a key and an element"),
        };
        let mut out = Vec::with_capacity(left.len() + right.len());
        let (mut l, mut r) = (left.into_iter().peekable(), right.into_iter().peekable());
        while let (Some(x), Some(y)) = (l.peek(), r.peek()) {
            if self.compare(&key(y), &key(x), pos)? == Ordering::Less {
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
