//! `std.ui` at run time (spec 17-std, std.ui.1-8): `ui.open` and the
//! methods of `ui.Window`, over the window host in `crate::ui`.

use std::io::Write;
use std::rc::Rc;

use super::stdlib::fail;
use super::{Env, Flow, Interp, Value};
use crate::ast::{Arg, Expr};
use crate::diag::Pos;
use crate::ui::{self, Event};

impl<'a, W: Write> Interp<'a, W> {
    /// std.ui.1: a `ui.Window`, or a `ui.UiError` failure.
    pub(super) fn ui_open(&mut self, title: &str, cols: i64, rows: i64, pos: Pos) -> Value {
        match ui::open(title, cols, rows) {
            Ok(handle) => Value::Struct(self.structs["ui.Window"], Rc::new(vec![Value::Int(handle)])),
            Err((kind, message)) => {
                let fields = vec![Value::Str(kind.into()), Value::Str(message.into())];
                fail(Value::Struct(self.structs["ui.UiError"], Rc::new(fields)), pos)
            }
        }
    }

    /// `w.m(args)` on a `ui.Window` (std.ui.2-8). The window's state lives
    /// in `crate::ui`, so a `mut self` receiver is only read here.
    pub(super) fn ui_method(&mut self, m: &str, recv: &'a Expr, args: &'a [Arg], _pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let Value::Struct(_, fields) = self.eval(recv, env)? else { unreachable!("checked: a ui.Window") };
        let id = fields[0].int();
        let mut values = Vec::with_capacity(args.len());
        for a in args {
            values.push(self.eval(&a.value, env)?);
        }
        let int = |i: usize| values[i].int();
        Ok(match m {
            "size" => {
                let (cols, rows) = ui::size(id);
                Value::Struct(self.structs["ui.Size"], Rc::new(vec![Value::Int(cols), Value::Int(rows)]))
            }
            "clear" => {
                ui::clear(id, int(0));
                Value::Unit
            }
            "put" => {
                let Value::Str(text) = &values[2] else { unreachable!("checked: str") };
                ui::put(id, int(0), int(1), text, int(3), int(4));
                Value::Unit
            }
            "cursor" => {
                ui::cursor(id, int(0), int(1));
                Value::Unit
            }
            "present" => {
                ui::present(id);
                Value::Unit
            }
            "close" => {
                ui::close(id);
                Value::Unit
            }
            "next_event" | "poll_event" => match ui::next_event(id, m == "next_event") {
                Some(e) => self.ui_event(e),
                None => Value::None,
            },
            _ => unreachable!("checked: `ui.Window.{m}`"),
        })
    }

    /// A `ui.Event` value; its variants are `Key`, `Resize`, `Close` in order.
    fn ui_event(&self, e: Event) -> Value {
        let enum_ = self.enums["ui.Event"];
        let (v, fields) = match e {
            Event::Key { key, ctrl, alt, shift } => {
                (0, vec![Value::Str(key.into()), Value::Bool(ctrl), Value::Bool(alt), Value::Bool(shift)])
            }
            Event::Resize { cols, rows } => (1, vec![Value::Int(cols as i64), Value::Int(rows as i64)]),
            Event::Close => (2, Vec::new()),
        };
        Value::Enum(enum_, v, Rc::new(fields))
    }
}
