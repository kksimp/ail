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
    /// Set for a property test (test.prop.4).
    pub prop: Option<PropInfo>,
    /// The trap's source line, values and frames, when a trap failed it (test.run.5).
    pub snapshot: Option<String>,
    /// The failing test's trace events, one per line (trace.test.1, test.run.6).
    pub trace: Vec<String>,
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
            // snap.update.2: under `ail test --update`, a literal expectation takes the actual value.
            if crate::snapshot::enabled()
                && let Some(text) = literal_of(&a, &args[1].value)
                && crate::snapshot::record(args[1].value.pos(), text)
            {
                return Ok(Value::Unit);
            }
            format!("assertion failed: left is {}, right is {}", self.describe(&a, pos)?, self.describe(&b, pos)?)
        };
        Err(Exit::Trap(Box::new(Trap { rule: "test.assert.1", pos, message })))
    }
}

/// The literal that writes `v` in the shape of the expectation `e`, when `e`
/// is a literal and `v` has a literal form (snap.update.2).
fn literal_of(v: &Value, e: &Expr) -> Option<String> {
    let scalar = |v: &Value| match v {
        Value::Int(i) => Some(i.to_string()),
        Value::Float(f) if f.is_finite() => Some(format!("{f:?}")),
        Value::Bool(b) => Some(b.to_string()),
        Value::Char(c) => Some(crate::snapshot::quote_char(*c)),
        Value::Str(s) => Some(crate::snapshot::quote(s)),
        _ => None,
    };
    match e {
        Expr::Str(..) | Expr::Int(..) | Expr::Float(..) | Expr::Char(..) | Expr::Bool(..) => scalar(v),
        Expr::Unary { op: UnOp::Neg, operand, .. } if matches!(**operand, Expr::Int(..) | Expr::Float(..)) => scalar(v),
        Expr::List(items, _) if items.iter().all(is_literal) => {
            let Value::List(xs) = v else { return None };
            let parts: Option<Vec<String>> = xs.iter().map(scalar).collect();
            Some(format!("[{}]", parts?.join(", ")))
        }
        _ => None,
    }
}

/// A scalar literal, possibly negated (snap.update.2).
fn is_literal(e: &Expr) -> bool {
    match e {
        Expr::Str(..) | Expr::Int(..) | Expr::Float(..) | Expr::Char(..) | Expr::Bool(..) => true,
        Expr::Unary { op: UnOp::Neg, operand, .. } => matches!(**operand, Expr::Int(..) | Expr::Float(..)),
        _ => false,
    }
}

/// Run every `@test` function in source order, each on its own (test.run.1-2).
/// A property runs `cases` generated inputs from `seed`, or from a seed derived
/// from its name (test.prop.3).
pub fn run_tests(prog: &Program, resolved: &Resolved, cases: u64, seed: Option<u64>) -> Vec<Outcome> {
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
        if !super::replay::wants(&f.name) {
            continue;
        }
        super::replay::begin_test(&f.name);
        it.depth = 0;
        // run.limit.1-3: each test gets the whole time, memory and step allowance.
        let _watch = limits::watch();
        it.reset_limits();
        it.snap = Some(Box::default());
        super::trace::capture_begin();
        if !f.params.is_empty() && const_failure.is_none() {
            let (pos, failure, info) = it.run_property(f, cases, seed.unwrap_or_else(|| super::prop::default_seed(&f.name)));
            let output = String::from_utf8_lossy(it.out.get_ref()).into_owned();
            it.out.get_mut().clear();
            super::trace::capture_end();
            super::replay::end_test(failure.is_some());
            outcomes.push(Outcome { name: f.name.clone(), pos, failure, output, prop: Some(info), snapshot: None, trace: Vec::new() });
            continue;
        }
        let mut trapped = None;
        let (pos, failure) = match &const_failure {
            Some(t) => (t.pos, Some(format!("trap[{}]: {}", t.rule, t.message))),
            None => match it.call(f, None, Vec::new(), f.pos) {
                Ok((Value::Fail(fail), _)) => {
                    let text = it.text(&fail.error, f.pos).unwrap_or_else(|t| t.message);
                    (fail.trace.first().copied().unwrap_or(f.pos), Some(format!("error: {text}")))
                }
                Ok(_) => (f.pos, None),
                Err(t) => {
                    let shown = (t.pos, Some(format!("trap[{}]: {}", t.rule, t.message)));
                    trapped = Some(t);
                    shown
                }
            },
        };
        // test.run.5: the trap's snapshot, without its first line (the failure repeats it).
        let mut snapshot = None;
        if let (Some(t), Some(snap)) = (&trapped, it.snap.take()) {
            super::trace::keep(prog, *snap, t);
            let text = super::render_trap(prog, t, super::take_snapshot(), false);
            snapshot = text.split_once('\n').map(|(_, rest)| rest.to_string());
        }
        // conc.task.4: a test's tasks end with it; copy what they printed.
        if it.conc.is_some() {
            it.task_print("");
            super::tasks::shutdown(it.conc.take());
        }
        let _ = it.out.flush();
        let output = String::from_utf8_lossy(it.out.get_ref()).into_owned();
        it.out.get_mut().clear();
        let events = super::trace::capture_end();
        let trace = if failure.is_some() { events } else { Vec::new() };
        super::replay::end_test(failure.is_some());
        outcomes.push(Outcome { name: f.name.clone(), pos, failure, output, prop: None, snapshot, trace });
    }
    outcomes
}
