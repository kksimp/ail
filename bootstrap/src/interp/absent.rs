//! `?` on `none` in a function whose errors are inferred fails with the
//! prelude's `Absent` (err.prop.5, ADR 0279).

use std::io::Write;
use std::rc::Rc;

use super::{Env, Exit, Failure, Flow, Interp, Value};
use crate::ast::Expr;
use crate::diag::Pos;

impl<'a, W: Write> Interp<'a, W> {
    /// The failure `Absent`, with the `?`'s context if it has one (err.ctx.2).
    pub(super) fn absent(&mut self, context: Option<&'a Expr>, pos: Pos, env: &mut Env<'a>) -> Flow<Exit> {
        let mut context_list = Vec::new();
        if let Some(c) = context {
            let Value::Str(msg) = self.eval(c, env)? else { unreachable!("checked: str") };
            context_list.push(msg.to_string());
        }
        let error = Value::Struct(self.structs["Absent"], Rc::new(Vec::new()));
        let failure = Failure { error, context: context_list, trace: vec![pos] };
        Ok(Exit::Return(Value::Fail(Rc::new(failure)), pos))
    }
}
