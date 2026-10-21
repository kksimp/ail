//! `std.tensor` at run time (spec 17-std, std.tensor.1-9, ADRs 0328-0329):
//! building tensors, running their methods on `crate::tensor`'s kernels,
//! and their text form.

use std::io::Write;
use std::rc::Rc;

use super::{Env, Flow, Interp, Trap, Value, float_text};
use crate::ast::{Arg, Expr};
use crate::diag::Pos;
use crate::ast::BinOp;
use crate::fuse::{self, FOp, Input};
use crate::tensor::{Activation, Data, Fault, Op, Reduce, Scalar, Sum, Tensor};

fn trapped(f: Fault, pos: Pos) -> super::Exit {
    Trap { rule: f.rule, pos, message: f.message }.into()
}

fn ints(v: &Value) -> Vec<i64> {
    match v {
        Value::List(xs) => xs.iter().map(Value::int).collect(),
        other => unreachable!("checked: List[int], found {other:?}"),
    }
}

/// A `List[int]` or `List[float]` as a tensor's elements; an empty list
/// is `float` unless `int` was expected.
fn elements(v: &Value, int: bool) -> Data {
    let Value::List(xs) = v else { unreachable!("checked: a list, found {v:?}") };
    match xs.first() {
        Some(Value::Int(_)) => Data::I(xs.iter().map(Value::int).collect()),
        Some(_) => Data::F(xs.iter().map(Value::float).collect()),
        None if int => Data::I(Vec::new()),
        None => Data::F(Vec::new()),
    }
}

fn list_of(xs: impl Iterator<Item = Value>) -> Value {
    Value::List(Rc::new(xs.collect()))
}

fn sum_value(s: Sum) -> Value {
    match s {
        Sum::F(x) => Value::Float(x),
        Sum::I(n) => Value::Int(n),
    }
}

impl<'a, W: Write> Interp<'a, W> {
    /// `tensor.from(xs)`, `tensor.from(xs, shape)` and `tensor.zeros(shape)`
    /// (std.tensor.2).
    pub(super) fn tensor_call(&mut self, f: &str, args: &'a [Arg], pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let mut values = Vec::with_capacity(args.len());
        for a in args {
            values.push(self.eval(&a.value, env)?);
        }
        let int = self.int_tensor_at(pos);
        let t = match f {
            "zeros" => {
                let shape = Tensor::shape_of(&ints(&values[0]), None).map_err(|e| trapped(e, pos))?;
                let n = shape.iter().product();
                let data = if int { Data::I(vec![0; n]) } else { Data::F(vec![0.0; n]) };
                Tensor { shape, data }
            }
            _ => {
                let data = elements(&values[0], int);
                let n = match &data {
                    Data::F(v) => v.len(),
                    Data::I(v) => v.len(),
                };
                let shape = match values.get(1) {
                    Some(s) => Tensor::shape_of(&ints(s), Some(n)).map_err(|e| trapped(e, pos))?,
                    None => vec![n],
                };
                Tensor { shape, data }
            }
        };
        Ok(Value::Tensor(Rc::new(t)))
    }

    /// Whether the checker typed the tensor built at `pos` as
    /// `Tensor[int]` (only an empty or `zeros` tensor needs to know).
    fn int_tensor_at(&self, pos: Pos) -> bool {
        self.resolved.int_tensors.contains(&pos)
    }

    /// std.tensor.15: `t[i]` drops axis 0 and `t[lo..hi]` keeps it.
    pub(super) fn tensor_index(&mut self, t: &Tensor, index: &'a Expr, pos: Pos, env: &mut Env<'a>) -> Flow<Value> {
        let r = if let Expr::Range { lo, hi, inclusive, .. } = index {
            let lo = match lo {
                Some(e) => self.eval(e, env)?.int(),
                None => 0,
            };
            let hi = match hi {
                Some(e) => self.eval(e, env)?.int(),
                None => t.shape.first().map_or(0, |&n| n as i64),
            };
            t.slice_rows(lo, hi, *inclusive)
        } else {
            let i = self.eval(index, env)?.int();
            t.row(i)
        };
        Ok(Value::Tensor(Rc::new(r.map_err(|e| trapped(e, pos))?)))
    }

    /// std.tensor.20: a fused chain (`fuse::plan`) at `e`. The names are read
    /// first, in order; when they are `float`s and `float` tensors of one
    /// shape, one pass computes the result, otherwise each step runs as the
    /// unfused operation would.
    pub(super) fn fused(&mut self, e: &'a Expr, env: &mut Env<'a>) -> Flow<Value> {
        let plan = fuse::plan(e).expect("checked: a fusable chain");
        let mut vals = Vec::with_capacity(plan.names.len());
        for name in &plan.names {
            vals.push(self.eval(name, env)?);
        }
        self.fused_values(&plan.ops, &vals)
    }

    /// A fused chain on its names' values (see `fused`), in either tier.
    pub(super) fn fused_values(&mut self, ops: &[FOp], vals: &[Value]) -> Flow<Value> {
        let inputs: Option<Vec<Input>> = vals
            .iter()
            .map(|v| match v {
                Value::Tensor(t) => Some(Input::T(t)),
                Value::Float(x) => Some(Input::F(*x)),
                _ => None,
            })
            .collect();
        if let Some(inputs) = inputs
            && let Some(t) = fuse::run_fast(ops, &inputs)
        {
            return Ok(Value::Tensor(Rc::new(t)));
        }
        self.fused_steps(ops, vals)
    }

    /// The unfused steps of a fused chain, one operation at a time.
    pub(super) fn fused_steps(&mut self, ops: &[FOp], vals: &[Value]) -> Flow<Value> {
        let mut stack: Vec<Value> = Vec::with_capacity(vals.len() + 1);
        for step in ops {
            match *step {
                FOp::Name(k) => stack.push(vals[k].clone()),
                FOp::Const(x) => stack.push(Value::Float(x)),
                FOp::Bin(op, pos) => {
                    let b = stack.pop().expect("postfix");
                    let a = stack.pop().expect("postfix");
                    stack.push(self.binop(op, a, b, pos)?);
                }
                FOp::Neg(pos) => {
                    let a = stack.pop().expect("postfix");
                    stack.push(match a {
                        Value::Float(x) => Value::Float(-x),
                        Value::Tensor(t) => Value::Tensor(Rc::new(t.neg().map_err(|f| trapped(f, pos))?)),
                        other => unreachable!("checked: a tensor or float, found {other:?}"),
                    });
                }
                FOp::Act(act, pos) => {
                    let a = stack.pop().expect("postfix");
                    let Value::Tensor(t) = a else { unreachable!("checked: a tensor") };
                    stack.push(Value::Tensor(Rc::new(t.activate(act).map_err(|f| trapped(f, pos))?)));
                }
            }
        }
        Ok(stack.pop().expect("one result"))
    }

    /// `t.m(args)` (std.tensor.3-8).
    pub(super) fn tensor_method(
        &mut self,
        m: &str,
        recv: &'a Expr,
        args: &'a [Arg],
        pos: Pos,
        env: &mut Env<'a>,
    ) -> Flow<Value> {
        let Value::Tensor(t) = self.eval(recv, env)? else { unreachable!("checked: a tensor") };
        if m == "map" {
            return self.tensor_map(&t, &args[0], env);
        }
        let other = match args.first() {
            Some(a) => Some(self.eval(&a.value, env)?),
            None => None,
        };
        let tensor_arg = || match &other {
            Some(Value::Tensor(u)) => Rc::clone(u),
            v => unreachable!("checked: a tensor, found {v:?}"),
        };
        let wrap = |r: Result<Tensor, Fault>| r.map(|t| Value::Tensor(Rc::new(t))).map_err(|e| trapped(e, pos));
        Ok(match m {
            "shape" => list_of(t.shape.iter().map(|&d| Value::Int(d as i64))),
            "len" => Value::Int(t.len() as i64),
            "to_list" => match &t.data {
                Data::F(v) => list_of(v.iter().map(|&x| Value::Float(x))),
                Data::I(v) => list_of(v.iter().map(|&x| Value::Int(x))),
            },
            "sum" if other.is_none() => sum_value(t.sum().map_err(|e| trapped(e, pos))?),
            "sum" | "max" | "mean" => {
                let kind = match m {
                    "sum" => Reduce::Sum,
                    "max" => Reduce::Max,
                    _ => Reduce::Mean,
                };
                let r = t.reduce(kind, other.as_ref().map(Value::int)).map_err(|e| trapped(e, pos))?;
                match (other, r.data) {
                    (Some(_), data) => Value::Tensor(Rc::new(Tensor { shape: r.shape, data })),
                    (None, Data::F(v)) => Value::Float(v[0]),
                    (None, Data::I(v)) => Value::Int(v[0]),
                }
            }
            "transpose" => wrap(t.transpose())?,
            "round" => wrap(t.round_int())?,
            "argmax" | "argmin" => {
                let r = t.arg_reduce(other.as_ref().map(Value::int), m == "argmax").map_err(|e| trapped(e, pos))?;
                match (other, r.data) {
                    (Some(_), data) => Value::Tensor(Rc::new(Tensor { shape: r.shape, data })),
                    (None, Data::I(v)) => Value::Int(v[0]),
                    (None, Data::F(_)) => unreachable!("arg_reduce gives `int`"),
                }
            }
            "clamp" => {
                let hi = match self.eval(&args[1].value, env)? {
                    Value::Float(x) => x,
                    other => unreachable!("checked: a float bound, found {other:?}"),
                };
                let Some(Value::Float(lo)) = other else { unreachable!("checked: a float bound") };
                wrap(t.clamp(lo, hi))?
            }
            "dot" => sum_value(t.dot(&tensor_arg()).map_err(|e| trapped(e, pos))?),
            "add" => wrap(t.zip(&tensor_arg(), Op::Add))?,
            "mul" => wrap(t.zip(&tensor_arg(), Op::Mul))?,
            "matmul" => wrap(t.matmul(&tensor_arg()))?,
            "conv2d" => wrap(t.conv2d(&tensor_arg()))?,
            "item" => match t.item().map_err(|e| trapped(e, pos))? {
                Scalar::F(x) => Value::Float(x),
                Scalar::I(n) => Value::Int(n),
            },
            "relu" => wrap(t.activate(Activation::Relu))?,
            "sigmoid" => wrap(t.activate(Activation::Sigmoid))?,
            "tanh" => wrap(t.activate(Activation::Tanh))?,
            "reshape" => {
                let shape = ints(other.as_ref().expect("checked: a shape"));
                let shape = Tensor::shape_of(&shape, Some(t.len())).map_err(|e| trapped(e, pos))?;
                Value::Tensor(Rc::new(Tensor { shape, data: t.data.clone() }))
            }
            _ => unreachable!("TENSOR_METHODS lists `{m}`"),
        })
    }

    /// std.tensor.10: `a + b`, `a - b` and `a * b` with a tensor operand;
    /// `None` when neither operand is a tensor.
    pub(super) fn tensor_binop(op: BinOp, l: &Value, r: &Value, pos: Pos) -> Option<Result<Value, Trap>> {
        let op = match op {
            BinOp::Add => Op::Add,
            BinOp::Sub => Op::Sub,
            BinOp::Mul => Op::Mul,
            BinOp::Div => Op::Div,
            _ => return None,
        };
        let scalar = |v: &Value| match v {
            Value::Float(x) => Scalar::F(*x),
            Value::Int(n) => Scalar::I(*n),
            other => unreachable!("checked: a tensor's element, found {other:?}"),
        };
        let r = match (l, r) {
            (Value::Tensor(a), Value::Tensor(b)) => a.zip(b, op),
            (Value::Tensor(a), s) => a.with_scalar(scalar(s), op, false),
            (s, Value::Tensor(b)) => b.with_scalar(scalar(s), op, true),
            _ => return None,
        };
        Some(r.map(|t| Value::Tensor(Rc::new(t))).map_err(|f| Trap { rule: f.rule, pos, message: f.message }))
    }

    /// std.tensor.8: `t.map(|x| e)`, `e` evaluated once per element, in order.
    fn tensor_map(&mut self, t: &Tensor, f: &'a Arg, env: &mut Env<'a>) -> Flow<Value> {
        let (var, _) = f.param.as_ref().expect("checked: a function");
        let mut out = Vec::with_capacity(t.len());
        let n = t.len();
        for i in 0..n {
            let x = match &t.data {
                Data::F(v) => Value::Float(v[i]),
                Data::I(v) => Value::Int(v[i]),
            };
            env.push((var.as_str(), x));
            let r = self.eval(&f.value, env);
            env.pop();
            out.push(r?);
        }
        let data = match out.first() {
            Some(Value::Int(_)) => Data::I(out.iter().map(Value::int).collect()),
            Some(_) => Data::F(out.iter().map(Value::float).collect()),
            None => t.data.clone(),
        };
        Ok(Value::Tensor(Rc::new(Tensor { shape: t.shape.clone(), data })))
    }

    /// std.tensor.9: the call that rebuilds `t`: `tensor.from([1.0, 2.0])`,
    /// with the shape when it isn't 1-D.
    pub(super) fn tensor_text(&self, t: &Tensor, _pos: Pos) -> Result<String, Trap> {
        let items: Vec<String> = match &t.data {
            Data::F(v) => v.iter().map(|&x| float_text(x)).collect(),
            Data::I(v) => v.iter().map(i64::to_string).collect(),
        };
        let list = format!("[{}]", items.join(", "));
        Ok(if t.shape.len() == 1 {
            format!("tensor.from({list})")
        } else {
            let dims: Vec<String> = t.shape.iter().map(usize::to_string).collect();
            format!("tensor.from({list}, [{}])", dims.join(", "))
        })
    }
}
