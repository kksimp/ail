//! Reference interpreter (ADR 0032) for checked programs.

use std::collections::HashMap;
use std::fmt;
use std::io::{self, BufWriter, Write};
use std::rc::Rc;

use crate::ast::{Expr, FnDecl, Program, Stmt};

#[derive(Clone, Debug)]
enum Value {
    Unit,
    Str(Rc<str>),
    Int(i64),
}

impl fmt::Display for Value {
    /// Text form, as used by `print` (mod.main.3).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Unit => Ok(()),
            Value::Str(s) => f.write_str(s),
            Value::Int(v) => write!(f, "{v}"),
        }
    }
}

/// A trap ends the program when it reaches `main` (err.trap.3).
#[derive(Debug)]
pub struct Trap {
    pub rule: &'static str,
    pub message: String,
}

impl fmt::Display for Trap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "trap[{}]: {}", self.rule, self.message)
    }
}

/// Deep recursion traps instead of crashing the interpreter.
const MAX_DEPTH: usize = 10_000;

pub fn run(prog: &Program) -> Result<(), Trap> {
    let fns: HashMap<&str, &FnDecl> = prog.fns.iter().map(|f| (f.name.as_str(), f)).collect();
    let stdout = io::stdout();
    let mut it = Interp { fns, out: BufWriter::new(stdout.lock()), depth: 0, closed: false };
    let main = it.fns["main"];
    it.call(main, Vec::new())?;
    it.flush()
}

struct Interp<'a, W: Write> {
    fns: HashMap<&'a str, &'a FnDecl>,
    out: BufWriter<W>,
    depth: usize,
    /// Standard output was closed by the reader (e.g. `| head`); stop writing quietly.
    closed: bool,
}

impl<'a, W: Write> Interp<'a, W> {
    fn call(&mut self, f: &'a FnDecl, args: Vec<Value>) -> Result<Value, Trap> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(Trap {
                rule: "err.trap.1",
                message: format!("call depth exceeded {MAX_DEPTH} in `{}`", f.name),
            });
        }
        let mut locals: Vec<(&'a str, Value)> = f.params.iter().map(|p| p.name.as_str()).zip(args).collect();
        let mut last = Value::Unit;
        for stmt in &f.body {
            match stmt {
                Stmt::Let { name, value, .. } => {
                    let v = self.eval(value, &locals)?;
                    locals.push((name.as_str(), v));
                    last = Value::Unit;
                }
                Stmt::Expr(e) => last = self.eval(e, &locals)?,
            }
        }
        self.depth -= 1;
        Ok(last)
    }

    fn eval(&mut self, e: &'a Expr, locals: &[(&'a str, Value)]) -> Result<Value, Trap> {
        Ok(match e {
            Expr::Str(s, _) => Value::Str(Rc::from(s.as_str())),
            Expr::Int(v, _) => Value::Int(*v),
            Expr::Name(name, _) => locals
                .iter()
                .rev()
                .find(|(n, _)| n == name)
                .map(|(_, v)| v.clone())
                .expect("checker guarantees names resolve"),
            Expr::Call { callee, args, .. } => {
                let Expr::Name(name, _) = callee.as_ref() else {
                    unreachable!("checker rejects non-name callees");
                };
                let mut values = Vec::with_capacity(args.len());
                for a in args {
                    values.push(self.eval(a, locals)?);
                }
                if name == "print" {
                    self.print(&values[0])?;
                    Value::Unit
                } else {
                    let f = self.fns[name.as_str()];
                    self.call(f, values)?
                }
            }
        })
    }

    fn print(&mut self, v: &Value) -> Result<(), Trap> {
        if self.closed {
            return Ok(());
        }
        match writeln!(self.out, "{v}") {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::BrokenPipe => {
                self.closed = true;
                Ok(())
            }
            Err(e) => Err(Trap { rule: "mod.main.3", message: format!("writing to standard output failed: {e}") }),
        }
    }

    fn flush(&mut self) -> Result<(), Trap> {
        match self.out.flush() {
            Err(e) if e.kind() != io::ErrorKind::BrokenPipe => {
                Err(Trap { rule: "mod.main.3", message: format!("writing to standard output failed: {e}") })
            }
            _ => Ok(()),
        }
    }
}
