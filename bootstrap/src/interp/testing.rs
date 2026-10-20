//! `assert`, `assert_eq` and the test runner (test.assert.1-2, test.run.1-4).

use super::*;

/// How one test ended.
pub struct Outcome {
    pub name: String,
    /// Where the test is declared, or where it failed.
    pub pos: Pos,
    /// `None` when it passed.
    pub failure: Option<String>,
    /// What the test printed.
    pub output: String,
}

impl<'a, W: Write> Interp<'a, W> {
    /// `assert(cond)`, `assert_eq(a, b)` (test.assert.1).
    pub(super) fn assert(&mut self, name: &str, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let message = if name == "assert" {
            match self.eval(&args[0].value, env)? {
                Value::Bool(true) => return Ok(Value::Unit),
                _ => "assertion failed".to_string(),
            }
        } else {
            let a = self.eval(&args[0].value, env)?;
            let b = self.eval(&args[1].value, env)?;
            if self.equal(&a, &b, pos)? {
                return Ok(Value::Unit);
            }
            format!("assertion failed: left is {}, right is {}", self.describe(&a, pos)?, self.describe(&b, pos)?)
        };
        Err(Exit::Trap(Trap { rule: "test.assert.1", pos, message }))
    }
}

/// Run every `@test` function in source order, each on its own (test.run.1-2).
pub fn run_tests(prog: &Program, resolved: &Resolved) -> Vec<Outcome> {
    let mut it = build(prog, resolved, Vec::<u8>::new());
    let mut outcomes = Vec::new();
    let mut const_failure = None;
    for c in &prog.consts {
        if let Err(Exit::Trap(t)) = it.constant(&c.name, c.pos) {
            const_failure = Some(t);
            break;
        }
    }
    for f in prog.fns.iter().filter(|f| f.top_level() && f.owner.is_none() && f.is_test()) {
        it.depth = 0;
        let (pos, failure) = match &const_failure {
            Some(t) => (t.pos, Some(format!("trap[{}]: {}", t.rule, t.message))),
            None => match it.call(f, None, Vec::new(), f.pos) {
                Ok((Value::Fail(fail), _)) => {
                    let text = it.text(&fail.error, f.pos).unwrap_or_else(|t| t.message);
                    (fail.trace.first().copied().unwrap_or(f.pos), Some(format!("error: {text}")))
                }
                Ok(_) => (f.pos, None),
                Err(t) => (t.pos, Some(format!("trap[{}]: {}", t.rule, t.message))),
            },
        };
        let _ = it.out.flush();
        let output = String::from_utf8_lossy(it.out.get_ref()).into_owned();
        it.out.get_mut().clear();
        outcomes.push(Outcome { name: f.name.clone(), pos, failure, output });
    }
    outcomes
}
