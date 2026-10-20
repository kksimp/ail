//! `Atomic[int]` and `Atomic[bool]` (conc.atomic.3-6, ADR 0320). An atomic
//! is a real hardware atomic shared by every copy, so tasks on the
//! scheduler, pooled tasks and `par for` workers all update the same cell
//! without a `ref` or a lock. Every operation is sequentially consistent
//! (conc.atomic.1, ADR 0025).

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering::SeqCst};

use super::{Env, Flow, Interp, Trap, Value};
use crate::ast::Arg;
use crate::diag::Pos;

/// The cell behind an `Atomic[T]`: an `int`, or a `bool` kept as 0 or 1.
#[derive(Debug)]
pub(super) struct Cell {
    v: AtomicI64,
    is_bool: bool,
}

/// Two atomics are equal only when they are the same cell.
impl PartialEq for Cell {
    fn eq(&self, other: &Cell) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Cell {
    fn value(&self, raw: i64) -> Value {
        if self.is_bool { Value::Bool(raw != 0) } else { Value::Int(raw) }
    }

    /// The cell's current value, for printing and tracing.
    pub(super) fn text(&self) -> String {
        match self.value(self.v.load(SeqCst)) {
            Value::Bool(b) => format!("Atomic({b})"),
            v => format!("Atomic({})", v.int()),
        }
    }
}

fn raw(v: &Value) -> i64 {
    match v {
        Value::Bool(b) => i64::from(*b),
        v => v.int(),
    }
}

/// `Atomic(v)` (conc.atomic.3).
pub(super) fn new(v: Value) -> Value {
    let is_bool = matches!(v, Value::Bool(_));
    Value::Atomic(Arc::new(Cell { v: AtomicI64::new(raw(&v)), is_bool }))
}

impl<'a, W: Write> Interp<'a, W> {
    /// `a.load()`, `a.store(v)`, `a.add(n)`, `a.cas(old, new)` (conc.atomic.4-5).
    pub(super) fn atomic_method(
        &mut self,
        cell: Arc<Cell>,
        name: &str,
        args: &'a [Arg],
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        let mut a = Vec::with_capacity(args.len());
        for arg in args {
            a.push(self.eval(&arg.value, env)?);
        }
        // conc.sched.1: each atomic operation is a switch point.
        self.switch_point(pos)?;
        Ok(match name {
            "load" => cell.value(cell.v.load(SeqCst)),
            "store" => {
                cell.v.store(raw(&a[0]), SeqCst);
                Value::Unit
            }
            "add" => {
                let n = a[0].int();
                // conc.atomic.5: like `+`, an add that would leave `int` traps
                // and leaves the cell as it was.
                match cell.v.try_update(SeqCst, SeqCst, |x| x.checked_add(n)) {
                    Ok(old) => Value::Int(old),
                    Err(old) => {
                        let message = format!("atomic add overflows: {old} + {n} is outside int");
                        return Err(Trap { rule: "conc.atomic.5", pos, message }.into());
                    }
                }
            }
            "cas" => Value::Bool(cell.v.compare_exchange(raw(&a[0]), raw(&a[1]), SeqCst, SeqCst).is_ok()),
            _ => unreachable!("checked: a method of `Atomic`"),
        })
    }
}
