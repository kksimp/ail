//! Operator fusion for `std.tensor` (spec std.tensor.20, ADR 0524). A chain
//! of elementwise tensor operations (`+ - * /`, unary `-`, `relu`, `sigmoid`,
//! `tanh`) over plain names and float literals runs in one pass over the
//! elements, with the same IEEE operations in the same order as the unfused
//! calls, so the results are identical. The elements are `float` or `f32`
//! (std.tensor.20, std.tensor.21); shapes that don't match, or values of any
//! other type, take the unfused path, which the interpreter runs one
//! operation at a time.

use crate::ast::{BinOp, Expr, UnOp};
use crate::diag::Pos;
use crate::tensor::{Activation, Data, Op, Tensor};

/// Elements per chunk of a fused run.
const CHUNK: usize = 1024;

/// One step of a chain in postfix order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FOp {
    /// Pushes the value of the `k`th name of the chain.
    Name(usize),
    /// Pushes a float literal.
    Const(f64),
    Bin(BinOp, Pos),
    Neg(Pos),
    Act(Activation, Pos),
}

/// A fusable chain: its names (in the order they are pushed) and its steps.
pub struct Plan<'a> {
    pub names: Vec<&'a Expr>,
    pub ops: Vec<FOp>,
}

impl Plan<'_> {
    /// Operations that a fusion saves: at least two are needed.
    fn operations(&self) -> usize {
        self.ops.iter().filter(|o| !matches!(o, FOp::Name(_) | FOp::Const(_))).count()
    }
}

/// The plan of `e` when it is a fusable chain of at least two operations.
pub fn plan(e: &Expr) -> Option<Plan<'_>> {
    let mut p = Plan { names: Vec::new(), ops: Vec::new() };
    build(e, &mut p)?;
    (p.operations() >= 2).then_some(p)
}

fn build<'a>(e: &'a Expr, p: &mut Plan<'a>) -> Option<()> {
    match e {
        Expr::Name(..) => {
            p.names.push(e);
            p.ops.push(FOp::Name(p.names.len() - 1));
        }
        Expr::Float(x, _) => p.ops.push(FOp::Const(*x)),
        Expr::Binary { op: op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div), lhs, rhs, pos } => {
            build(lhs, p)?;
            build(rhs, p)?;
            p.ops.push(FOp::Bin(*op, *pos));
        }
        Expr::Unary { op: UnOp::Neg, operand, pos } => {
            build(operand, p)?;
            p.ops.push(FOp::Neg(*pos));
        }
        Expr::Method { recv, name, args, .. } if args.is_empty() => {
            let pos = e.pos();
            let act = match name.as_str() {
                "relu" => Activation::Relu,
                "sigmoid" => Activation::Sigmoid,
                "tanh" => Activation::Tanh,
                _ => return None,
            };
            build(recv, p)?;
            p.ops.push(FOp::Act(act, pos));
        }
        _ => return None,
    }
    Some(())
}

/// The kernel operator of an arithmetic operator (std.tensor.10).
pub fn kernel_op(op: BinOp) -> Op {
    match op {
        BinOp::Add => Op::Add,
        BinOp::Sub => Op::Sub,
        BinOp::Mul => Op::Mul,
        BinOp::Div => Op::Div,
        _ => unreachable!("checked: a fusable operator"),
    }
}

/// A chain's value at run time: a `float` tensor, or a `float`.
pub enum Input<'a> {
    T(&'a Tensor),
    F(f64),
}

/// An element type a fused run computes in: `f64`, or `f32` for `Tensor[f32]`
/// (std.tensor.21: the same IEEE operation at `f32`, an activation computed
/// in `f64` and rounded once, a `float` scalar rounded to `f32`).
trait Elem: Copy + Default + std::ops::Add<Output = Self> + std::ops::Sub<Output = Self> + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self> + std::ops::Neg<Output = Self>
{
    fn lit(x: f64) -> Self;
    fn act(a: Activation, x: Self) -> Self;
    fn slice(d: &Data) -> Option<&[Self]>;
    fn data(v: Vec<Self>) -> Data;
}

impl Elem for f64 {
    fn lit(x: f64) -> f64 {
        x
    }
    fn act(a: Activation, x: f64) -> f64 {
        a.apply(x)
    }
    fn slice(d: &Data) -> Option<&[f64]> {
        if let Data::F(v) = d { Some(v) } else { None }
    }
    fn data(v: Vec<f64>) -> Data {
        Data::F(v)
    }
}

impl Elem for f32 {
    fn lit(x: f64) -> f32 {
        x as f32
    }
    fn act(a: Activation, x: f32) -> f32 {
        a.apply(f64::from(x)) as f32
    }
    fn slice(d: &Data) -> Option<&[f32]> {
        if let Data::F32(v) = d { Some(v) } else { None }
    }
    fn data(v: Vec<f32>) -> Data {
        Data::F32(v)
    }
}

/// One pass over the elements when every input is a tensor of one shape and
/// one element type (`float` or `f32`) or a `float`, and at least one is a
/// tensor. `None` otherwise.
pub fn run_fast(prog: &[FOp], inputs: &[Input<'_>]) -> Option<Tensor> {
    match inputs.iter().find_map(|i| if let Input::T(t) = i { Some(&t.data) } else { None })? {
        Data::F(_) => run_typed::<f64>(prog, inputs),
        Data::F32(_) => run_typed::<f32>(prog, inputs),
        Data::I(_) => None,
    }
}

fn run_typed<T: Elem>(prog: &[FOp], inputs: &[Input<'_>]) -> Option<Tensor> {
    let mut shape: Option<&[usize]> = None;
    let mut slices: Vec<Option<&[T]>> = Vec::with_capacity(inputs.len());
    for input in inputs {
        match input {
            Input::T(t) => {
                if shape.is_some_and(|s| s != t.shape.as_slice()) {
                    return None;
                }
                shape = Some(&t.shape);
                slices.push(Some(T::slice(&t.data)?));
            }
            Input::F(_) => slices.push(None),
        }
    }
    let shape = shape?;
    let n: usize = shape.iter().product();
    let depth = prog
        .iter()
        .scan(0usize, |d, step| {
            *d = match step {
                FOp::Name(_) | FOp::Const(_) => *d + 1,
                FOp::Bin(..) => *d - 1,
                FOp::Neg(_) | FOp::Act(..) => *d,
            };
            Some(*d)
        })
        .max()
        .unwrap_or(0);
    // Each stack slot is one chunk of elements, so every operator runs as a
    // tight loop over the chunk and the temporaries stay in cache.
    let mut slots: Vec<Vec<T>> = vec![vec![T::default(); CHUNK]; depth.max(1)];
    let mut out = Vec::with_capacity(n);
    let mut start = 0;
    while start < n {
        let len = CHUNK.min(n - start);
        let mut sp = 0;
        for step in prog {
            match *step {
                FOp::Name(k) => {
                    match (&inputs[k], slices[k]) {
                        (Input::T(_), Some(v)) => slots[sp][..len].copy_from_slice(&v[start..start + len]),
                        (Input::F(x), _) => slots[sp][..len].fill(T::lit(*x)),
                        (Input::T(_), None) => unreachable!("a tensor input has elements"),
                    }
                    sp += 1;
                }
                FOp::Const(x) => {
                    slots[sp][..len].fill(T::lit(x));
                    sp += 1;
                }
                FOp::Bin(op, _) => {
                    sp -= 1;
                    let (lo, hi) = slots.split_at_mut(sp);
                    zip_in_place(&mut lo[sp - 1][..len], &hi[0][..len], kernel_op(op));
                }
                FOp::Neg(_) => slots[sp - 1][..len].iter_mut().for_each(|x| *x = -*x),
                FOp::Act(act, _) => slots[sp - 1][..len].iter_mut().for_each(|x| *x = T::act(act, *x)),
            }
        }
        out.extend_from_slice(&slots[0][..len]);
        start += len;
    }
    Some(Tensor { shape: shape.to_vec(), data: T::data(out) })
}

/// `a[i] = a[i] op b[i]`, with the operator chosen once per chunk.
fn zip_in_place<T: Elem>(a: &mut [T], b: &[T], op: Op) {
    match op {
        Op::Add => a.iter_mut().zip(b).for_each(|(x, y)| *x = *x + *y),
        Op::Sub => a.iter_mut().zip(b).for_each(|(x, y)| *x = *x - *y),
        Op::Mul => a.iter_mut().zip(b).for_each(|(x, y)| *x = *x * *y),
        Op::Div => a.iter_mut().zip(b).for_each(|(x, y)| *x = *x / *y),
    }
}
