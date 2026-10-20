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
    pub(super) fn ui_method(&mut self, m: &str, recv: &'a Expr, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        // The methods of the `std.net` handles share `StdCall::Method` (std.net.9-12).
        if let Some(rest) = m.strip_prefix("net.") {
            return self.net_method(rest, recv, args, pos, env);
        }
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
            "modifiers" => {
                let m = ui::modifiers(id).map(Value::Bool).to_vec();
                Value::Struct(self.structs["ui.Modifiers"], Rc::new(m))
            }
            "font_size" => Value::Int(ui::font_size(id)),
            "set_font_size" => {
                ui::set_font_size(id, int(0));
                Value::Unit
            }
            "clear" => {
                ui::clear(id, int(0));
                Value::Unit
            }
            "put" => {
                let Value::Str(text) = &values[2] else { unreachable!("checked: str") };
                ui::put(id, int(0), int(1), text, int(3), int(4), 0);
                Value::Unit
            }
            "put_styled" => {
                let Value::Str(text) = &values[2] else { unreachable!("checked: str") };
                let Value::Struct(_, style) = &values[5] else { unreachable!("checked: ui.Style") };
                let on = |i: usize| matches!(style[i], Value::Bool(true));
                let bits = [ui::BOLD, ui::ITALIC, ui::UNDERLINE];
                let style = (0..3).filter(|&i| on(i)).fold(0, |b, i| b | bits[i]);
                ui::put(id, int(0), int(1), text, int(3), int(4), style);
                Value::Unit
            }
            "set_cursor" => {
                let Value::Enum(_, v, _) = &values[2] else { unreachable!("checked: ui.Cursor") };
                let shape = [ui::CaretShape::Bar, ui::CaretShape::Block, ui::CaretShape::Underline][*v];
                ui::set_cursor(id, int(0), int(1), shape);
                Value::Unit
            }
            "set_title" => {
                let Value::Str(t) = &values[0] else { unreachable!("checked: str") };
                ui::set_title(id, t);
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

    /// A `ui.Event` value; its variants are `Key`, `Resize`, `Close`, `Mouse`,
    /// `Scroll` in order.
    fn ui_event(&self, e: Event) -> Value {
        let enum_ = self.enums["ui.Event"];
        let (v, fields) = match e {
            Event::Key { key, ctrl, alt, shift } => {
                (0, vec![Value::Str(key.into()), Value::Bool(ctrl), Value::Bool(alt), Value::Bool(shift)])
            }
            Event::Resize { cols, rows } => (1, vec![Value::Int(cols as i64), Value::Int(rows as i64)]),
            Event::Close => (2, Vec::new()),
            Event::Mouse { col, row, button, kind } => {
                (3, vec![Value::Int(col), Value::Int(row), Value::Str(button.into()), Value::Str(kind.into())])
            }
            Event::Scroll { lines } => (4, vec![Value::Int(lines)]),
        };
        Value::Enum(enum_, v, Rc::new(fields))
    }
}
