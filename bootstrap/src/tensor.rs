//! `std.tensor`'s backend (spec 17-std, std.tensor.1-9, ADRs 0328-0330):
//! contiguous `f64` and `i64` buffers and their kernels. `sum`, `dot`,
//! `add` and `mul` on `float` run on AVX2 (x86-64-v3, ADR 0005) or NEON
//! (ARMv8.2-A, ADR 0006), picked once at run time, with a scalar fallback
//! that does the same operations in the same order (lockstep, ADR 0008),
//! so all three give bit-identical results (std.tensor.5). `matmul` of
//! at least 64x64x64 splits its rows across the thread pool (conc.par.9).
//! `int` kernels are scalar and checked: an overflow traps (std.tensor.6).

use std::sync::OnceLock;

/// The elements of a tensor, row-major.
#[derive(Clone, Debug, PartialEq)]
pub enum Data {
    F(Vec<f64>),
    I(Vec<i64>),
}

/// A tensor value (std.tensor.1): a shape and its elements.
#[derive(Clone, Debug, PartialEq)]
pub struct Tensor {
    pub shape: Vec<usize>,
    pub data: Data,
}

/// A kernel's failure, which the interpreter reports as a trap.
pub struct Fault {
    pub rule: &'static str,
    pub message: String,
}

fn fault<T>(rule: &'static str, message: String) -> Result<T, Fault> {
    Err(Fault { rule, message })
}

/// The partial sums `sum` and `dot` keep (std.tensor.5): element `i` goes
/// to accumulator `i % LANES`; four AVX2 registers or eight NEON registers,
/// enough independent additions to hide the adder's latency.
const LANES: usize = 16;

/// The smallest `m * k * n` that `matmul` spreads over threads (64^3).
const PAR_WORK: usize = 64 * 64 * 64;

/// The vector unit in use (std.tensor.5), detected once.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Isa {
    Scalar,
    #[cfg_attr(not(target_arch = "x86_64"), allow(dead_code))]
    Avx2,
    #[cfg_attr(not(target_arch = "aarch64"), allow(dead_code))]
    Neon,
}

pub fn isa() -> Isa {
    static ISA: OnceLock<Isa> = OnceLock::new();
    *ISA.get_or_init(|| {
        if std::env::var_os("AIL_TENSOR_SCALAR").is_some() {
            return Isa::Scalar;
        }
        #[cfg(target_arch = "x86_64")]
        if std::arch::is_x86_feature_detected!("avx2") {
            return Isa::Avx2;
        }
        #[cfg(target_arch = "aarch64")]
        if std::arch::is_aarch64_feature_detected!("neon") {
            return Isa::Neon;
        }
        Isa::Scalar
    })
}

impl Tensor {
    /// std.tensor.2: `shape` must have only non-negative sizes, and `len`
    /// elements must fill it.
    pub fn shape_of(shape: &[i64], len: Option<usize>) -> Result<Vec<usize>, Fault> {
        let mut dims = Vec::with_capacity(shape.len());
        let mut count: usize = 1;
        for &d in shape {
            let Ok(d) = usize::try_from(d) else {
                return fault("std.tensor.2", format!("a tensor's sizes can't be negative, and the shape is {shape:?}"));
            };
            count = match count.checked_mul(d) {
                Some(c) if c <= 1 << 40 => c,
                _ => return fault("std.tensor.2", format!("the shape {shape:?} has too many elements")),
            };
            dims.push(d);
        }
        if let Some(n) = len
            && n != count
        {
            return fault("std.tensor.2", format!("{n} elements don't fill the shape {shape:?}, which holds {count}"));
        }
        Ok(dims)
    }

    pub fn len(&self) -> usize {
        match &self.data {
            Data::F(v) => v.len(),
            Data::I(v) => v.len(),
        }
    }

    /// std.tensor.4: `add`, `mul` and `dot` need equal shapes.
    fn same_shape(&self, other: &Tensor, what: &str) -> Result<(), Fault> {
        if self.shape == other.shape {
            return Ok(());
        }
        fault("std.tensor.4", format!("`{what}` needs equal shapes, but they are {:?} and {:?}", self.shape, other.shape))
    }

    /// std.tensor.3: the sum of the elements.
    pub fn sum(&self) -> Result<Sum, Fault> {
        Ok(match &self.data {
            Data::F(v) => Sum::F(sum_f64(v)),
            Data::I(v) => Sum::I(sum_i64(v.iter().copied())?),
        })
    }

    /// std.tensor.4: the sum of the elementwise products.
    pub fn dot(&self, other: &Tensor) -> Result<Sum, Fault> {
        self.same_shape(other, "dot")?;
        Ok(match (&self.data, &other.data) {
            (Data::F(a), Data::F(b)) => Sum::F(dot_f64(a, b)),
            (Data::I(a), Data::I(b)) => {
                let mut prods = Vec::with_capacity(a.len());
                for (x, y) in a.iter().zip(b) {
                    prods.push(x.checked_mul(*y).ok_or_else(overflow)?);
                }
                Sum::I(sum_i64(prods.into_iter())?)
            }
            _ => unreachable!("checked: one element type"),
        })
    }

    /// std.tensor.4, std.tensor.10: elementwise `op`, with NumPy-style
    /// broadcasting (std.tensor.4): shapes line up from the right, and an
    /// axis of size 1, or a missing axis, stretches to the other size.
    pub fn zip(&self, other: &Tensor, op: Op) -> Result<Tensor, Fault> {
        if self.shape == other.shape {
            return Ok(Tensor { shape: self.shape.clone(), data: zip_data(&self.data, &other.data, op)? });
        }
        let Some(shape) = broadcast_shape(&self.shape, &other.shape) else {
            return fault(
                "std.tensor.4",
                format!("`{}` can't broadcast shapes {:?} and {:?}", op.name(), self.shape, other.shape),
            );
        };
        let data = zip_data(&self.expanded(&shape), &other.expanded(&shape), op)?;
        Ok(Tensor { shape, data })
    }

    /// The elements of `self` broadcast to `shape` (std.tensor.4), copied
    /// row-major. A tensor already of `shape` is cloned as is.
    fn expanded(&self, shape: &[usize]) -> Data {
        if self.shape == shape {
            return self.data.clone();
        }
        match &self.data {
            Data::F(v) => Data::F(expand(v, &self.shape, shape)),
            Data::I(v) => Data::I(expand(v, &self.shape, shape)),
        }
    }

    /// std.tensor.10: `t op s` (`left`: `false`) or `s op t` (`true`),
    /// the scalar `s` taken with every element.
    pub fn with_scalar(&self, s: Scalar, op: Op, left: bool) -> Result<Tensor, Fault> {
        let data = match (&self.data, s) {
            (Data::F(a), Scalar::F(s)) => Data::F(scalar_f64(a, s, op, left)),
            (Data::I(a), Scalar::I(s)) => {
                let mut out = Vec::with_capacity(a.len());
                for &x in a {
                    let r = if left { op.i(s, x) } else { op.i(x, s) };
                    out.push(r.ok_or_else(overflow)?);
                }
                Data::I(out)
            }
            _ => unreachable!("checked: the scalar has the element type"),
        };
        Ok(Tensor { shape: self.shape.clone(), data })
    }

    /// std.tensor.15: row `i` of a tensor with at least one axis; the
    /// result drops that axis, so `t[i]` of a 1-D tensor has shape `[]`.
    pub fn row(&self, i: i64) -> Result<Tensor, Fault> {
        let n = self.rows_or_fault("index")?;
        let Some(i) = usize::try_from(i).ok().filter(|&i| i < n) else {
            return fault("std.tensor.15", format!("index {i} is out of bounds for a tensor of {n} rows"));
        };
        let mut t = self.rows_between(i, i + 1)?;
        t.shape.remove(0);
        Ok(t)
    }

    /// std.tensor.15: rows `lo..hi` (or `lo..=hi`, when `inclusive`) along
    /// axis 0, so the result keeps the axis.
    pub fn slice_rows(&self, lo: i64, hi: i64, inclusive: bool) -> Result<Tensor, Fault> {
        let n = self.rows_or_fault("slice")?;
        let end = if inclusive { hi.checked_add(1) } else { Some(hi) };
        match (usize::try_from(lo), end.and_then(|e| usize::try_from(e).ok())) {
            (Ok(a), Some(b)) if a <= b && b <= n => self.rows_between(a, b),
            _ => fault(
                "std.tensor.15",
                format!("rows {lo}..{hi} are out of bounds for a tensor of {n} rows"),
            ),
        }
    }

    /// std.tensor.15: the one element of a tensor of shape `[]` (`t[i].item()`).
    pub fn item(&self) -> Result<Scalar, Fault> {
        if self.len() != 1 {
            return fault("std.tensor.15", format!("`item` needs one element, but the shape is {:?}", self.shape));
        }
        Ok(match &self.data {
            Data::F(v) => Scalar::F(v[0]),
            Data::I(v) => Scalar::I(v[0]),
        })
    }

    fn rows_or_fault(&self, what: &str) -> Result<usize, Fault> {
        match self.shape.first() {
            Some(&n) => Ok(n),
            None => fault("std.tensor.15", format!("`{what}` of a tensor of shape [], which has no rows")),
        }
    }

    /// Rows `a..b` along axis 0, with every other axis kept.
    fn rows_between(&self, a: usize, b: usize) -> Result<Tensor, Fault> {
        let row: usize = self.shape[1..].iter().product();
        let (lo, hi) = (a * row, b * row);
        let data = match &self.data {
            Data::F(v) => Data::F(v[lo..hi].to_vec()),
            Data::I(v) => Data::I(v[lo..hi].to_vec()),
        };
        let mut shape = self.shape.clone();
        shape[0] = b - a;
        Ok(Tensor { shape, data })
    }

    /// std.tensor.16: the valid 2-D convolution of an `h x w` image with an
    /// `kh x kw` kernel, stride 1, no padding: the result is
    /// `(h - kh + 1) x (w - kw + 1)`. Each output adds its products in
    /// row-major order of the kernel, one at a time, so the order is fixed.
    pub fn conv2d(&self, kernel: &Tensor) -> Result<Tensor, Fault> {
        let (&[h, w], &[kh, kw]) = (self.shape.as_slice(), kernel.shape.as_slice()) else {
            return fault(
                "std.tensor.16",
                format!("`conv2d` needs two 2-D tensors, but the shapes are {:?} and {:?}", self.shape, kernel.shape),
            );
        };
        if kh > h || kw > w {
            return fault("std.tensor.16", format!("`conv2d`: a {kh}x{kw} kernel doesn't fit a {h}x{w} image"));
        }
        let (Data::F(img), Data::F(k)) = (&self.data, &kernel.data) else {
            return fault("std.tensor.16", "`conv2d` is on `float` tensors".into());
        };
        let (oh, ow) = (h - kh + 1, w - kw + 1);
        let mut out = vec![0.0; oh * ow];
        for oy in 0..oh {
            for ox in 0..ow {
                let mut s = 0.0;
                for ky in 0..kh {
                    for kx in 0..kw {
                        s += img[(oy + ky) * w + ox + kx] * k[ky * kw + kx];
                    }
                }
                out[oy * ow + ox] = s;
            }
        }
        Ok(Tensor { shape: vec![oh, ow], data: Data::F(out) })
    }

    /// std.tensor.17: the index of the largest (`max`) or smallest element
    /// along `axis`, or over every element when `axis` is `None` (a result
    /// of shape `[]`). The first of equal elements wins, and a NaN never
    /// replaces the running best, as with `max`.
    pub fn arg_reduce(&self, axis: Option<i64>, max: bool) -> Result<Tensor, Fault> {
        let (before, n, after, shape) = self.split(axis, if max { "argmax" } else { "argmin" })?;
        if n == 0 && before * after > 0 {
            return fault("std.tensor.17", format!("`argmax` of an empty axis of a tensor of shape {:?}", self.shape));
        }
        let data = match &self.data {
            Data::F(v) => Data::I(arg_axis(v, before, n, after, |x, best| if max { x > best } else { x < best })),
            Data::I(v) => Data::I(arg_axis(v, before, n, after, |x, best| if max { x > best } else { x < best })),
        };
        Ok(Tensor { shape, data })
    }

    /// std.tensor.18: each element clamped to `[lo, hi]`; a NaN stays NaN.
    pub fn clamp(&self, lo: f64, hi: f64) -> Result<Tensor, Fault> {
        if lo.is_nan() || hi.is_nan() || lo > hi {
            return fault("std.tensor.18", format!("`clamp({lo}, {hi})`: the bounds must satisfy lo <= hi"));
        }
        let Data::F(v) = &self.data else {
            return fault("std.tensor.18", "`clamp` is on `float` tensors".into());
        };
        let data = v.iter().map(|&x| if x.is_nan() { x } else { x.clamp(lo, hi) }).collect();
        Ok(Tensor { shape: self.shape.clone(), data: Data::F(data) })
    }

    /// std.tensor.19: each element rounded half away from zero to an `int`.
    /// A NaN, an infinity or a value outside `int` traps.
    pub fn round_int(&self) -> Result<Tensor, Fault> {
        let Data::F(v) = &self.data else {
            return fault("std.tensor.19", "`round` is on `float` tensors".into());
        };
        let mut out = Vec::with_capacity(v.len());
        for &x in v {
            let r = x.round();
            // `i64::MAX as f64` rounds up to 2^63, which is out of range.
            if !(-9.223_372_036_854_776e18..9.223_372_036_854_776e18).contains(&r) {
                return fault("std.tensor.19", format!("`round` of {x}, which isn't a finite `int` value"));
            }
            out.push(r as i64);
        }
        Ok(Tensor { shape: self.shape.clone(), data: Data::I(out) })
    }

    /// std.tensor.14: `-t`, elementwise. A `float` is negated exactly; an
    /// `int` that is `i64::MIN` traps (std.tensor.6).
    pub fn neg(&self) -> Result<Tensor, Fault> {
        let data = match &self.data {
            Data::F(v) => Data::F(v.iter().map(|&x| -x).collect()),
            Data::I(v) => {
                let mut out = Vec::with_capacity(v.len());
                for &x in v {
                    out.push(x.checked_neg().ok_or_else(overflow)?);
                }
                Data::I(out)
            }
        };
        Ok(Tensor { shape: self.shape.clone(), data })
    }

    /// std.tensor.13: an activation of each element of a `float` tensor.
    /// The scalar code below runs on every target, so the results agree
    /// across ISAs (std.tensor.5, lockstep ADR 0008).
    pub fn activate(&self, act: Activation) -> Result<Tensor, Fault> {
        let Data::F(v) = &self.data else {
            return fault("std.tensor.13", format!("`{}` is on `float` tensors", act.name()));
        };
        let data = v.iter().map(|&x| act.apply(x)).collect();
        Ok(Tensor { shape: self.shape.clone(), data: Data::F(data) })
    }

    /// std.tensor.12: the transpose of a 2-D tensor.
    pub fn transpose(&self) -> Result<Tensor, Fault> {
        let &[m, n] = self.shape.as_slice() else {
            return fault("std.tensor.12", format!("`transpose` needs a 2-D tensor, but the shape is {:?}", self.shape));
        };
        let data = match &self.data {
            Data::F(v) => Data::F(transpose(v, m, n)),
            Data::I(v) => Data::I(transpose(v, m, n)),
        };
        Ok(Tensor { shape: vec![n, m], data })
    }

    /// The sizes around `axis`: (before, along, after), and the shape
    /// without it (std.tensor.11).
    fn split(&self, axis: Option<i64>, what: &str) -> Result<(usize, usize, usize, Vec<usize>), Fault> {
        let Some(axis) = axis else {
            return Ok((1, self.len(), 1, Vec::new()));
        };
        let rank = self.shape.len();
        let Some(a) = usize::try_from(axis).ok().filter(|&a| a < rank) else {
            return fault(
                "std.tensor.11",
                format!("`{what}({axis})` on a tensor of shape {:?}: the axis must be from 0 to {}", self.shape, rank as i64 - 1),
            );
        };
        let before = self.shape[..a].iter().product();
        let after = self.shape[a + 1..].iter().product();
        let mut shape = self.shape.clone();
        shape.remove(a);
        Ok((before, self.shape[a], after, shape))
    }

    /// std.tensor.11: `sum`, `max` or `mean` over `axis`, or over every
    /// element when `axis` is `None` (a tensor of shape `[]`).
    pub fn reduce(&self, kind: Reduce, axis: Option<i64>) -> Result<Tensor, Fault> {
        let what = kind.name();
        let (before, n, after, shape) = self.split(axis, what)?;
        if n == 0 && kind != Reduce::Sum && before * after > 0 {
            return fault("std.tensor.11", format!("`{what}` of an empty axis of a tensor of shape {:?}", self.shape));
        }
        let data = match (&self.data, kind) {
            (Data::F(v), Reduce::Sum) => Data::F(sum_axis(v, before, n, after)),
            (Data::F(v), Reduce::Max) => Data::F(max_axis(v, before, n, after)),
            (Data::F(v), Reduce::Mean) => {
                let mut s = sum_axis(v, before, n, after);
                for x in &mut s {
                    *x /= n as f64;
                }
                Data::F(s)
            }
            (Data::I(v), Reduce::Sum) => Data::I(sum_axis_i64(v, before, n, after)?),
            (Data::I(v), Reduce::Max) => Data::I(max_axis_i64(v, before, n, after)),
            (Data::I(_), Reduce::Mean) => unreachable!("checked: `mean` is on `float` tensors"),
        };
        Ok(Tensor { shape, data })
    }

    /// std.tensor.7: the matrix product of `m x k` and `k x n` tensors.
    /// Each result element adds its products in order of `k`, so it equals
    /// the plain triple loop exactly.
    pub fn matmul(&self, other: &Tensor) -> Result<Tensor, Fault> {
        let (&[m, k], &[k2, n]) = (self.shape.as_slice(), other.shape.as_slice()) else {
            return fault(
                "std.tensor.7",
                format!("`matmul` needs two 2-D tensors, but the shapes are {:?} and {:?}", self.shape, other.shape),
            );
        };
        if k != k2 {
            return fault(
                "std.tensor.7",
                format!("`matmul` of {m}x{k} and {k2}x{n}: the inner sizes {k} and {k2} differ"),
            );
        }
        let data = match (&self.data, &other.data) {
            (Data::F(a), Data::F(b)) => Data::F(matmul_f64(a, b, m, k, n)),
            (Data::I(a), Data::I(b)) => Data::I(matmul_i64(a, b, m, k, n)?),
            _ => unreachable!("checked: one element type"),
        };
        Ok(Tensor { shape: vec![m, n], data })
    }
}

/// An elementwise operator (std.tensor.4, std.tensor.10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn name(self) -> &'static str {
        match self {
            Op::Add => "add",
            Op::Sub => "-",
            Op::Mul => "mul",
            Op::Div => "/",
        }
    }

    fn f(self, x: f64, y: f64) -> f64 {
        match self {
            Op::Add => x + y,
            Op::Sub => x - y,
            Op::Mul => x * y,
            Op::Div => x / y,
        }
    }

    fn i(self, x: i64, y: i64) -> Option<i64> {
        match self {
            Op::Add => x.checked_add(y),
            Op::Sub => x.checked_sub(y),
            Op::Mul => x.checked_mul(y),
            // `/` is `float`-only (std.tensor.10): the checker rejects it on `int`.
            Op::Div => None,
        }
    }
}

/// A scalar operand of an operator (std.tensor.10).
#[derive(Clone, Copy, Debug)]
pub enum Scalar {
    F(f64),
    I(i64),
}

/// The result of `sum` or `dot`.
pub enum Sum {
    F(f64),
    I(i64),
}

/// An elementwise activation (std.tensor.13).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Activation {
    Relu,
    Sigmoid,
    Tanh,
}

impl Activation {
    pub fn name(self) -> &'static str {
        match self {
            Activation::Relu => "relu",
            Activation::Sigmoid => "sigmoid",
            Activation::Tanh => "tanh",
        }
    }

    /// `relu` keeps a positive element and makes the rest `0.0`, but a NaN
    /// stays NaN. `sigmoid` and `tanh` use the platform's `exp` and `tanh`.
    fn apply(self, x: f64) -> f64 {
        match self {
            Activation::Relu if x.is_nan() => x,
            Activation::Relu if x > 0.0 => x,
            Activation::Relu => 0.0,
            Activation::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            Activation::Tanh => x.tanh(),
        }
    }
}

/// `a op b` elementwise over two buffers of one length (std.tensor.5, 6).
fn zip_data(a: &Data, b: &Data, op: Op) -> Result<Data, Fault> {
    Ok(match (a, b) {
        (Data::F(a), Data::F(b)) => Data::F(zip_f64(a, b, op)),
        (Data::I(a), Data::I(b)) => {
            let mut out = Vec::with_capacity(a.len());
            for (x, y) in a.iter().zip(b) {
                out.push(op.i(*x, *y).ok_or_else(overflow)?);
            }
            Data::I(out)
        }
        _ => unreachable!("checked: one element type"),
    })
}

/// The shape two shapes broadcast to, or `None` when they don't:
/// per axis from the right, sizes are equal, or one of them is 1.
fn broadcast_shape(a: &[usize], b: &[usize]) -> Option<Vec<usize>> {
    let rank = a.len().max(b.len());
    (0..rank)
        .map(|k| {
            let da = if k + a.len() >= rank { a[k + a.len() - rank] } else { 1 };
            let db = if k + b.len() >= rank { b[k + b.len() - rank] } else { 1 };
            match (da, db) {
                _ if da == db => Some(da),
                (1, _) => Some(db),
                (_, 1) => Some(da),
                _ => None,
            }
        })
        .collect()
}

/// `v` (of `src` shape) copied to `out` shape: the source's size-1 axes
/// repeat, so their stride is 0. Walks `out` row-major with a counter.
fn expand<T: Copy>(v: &[T], src: &[usize], out: &[usize]) -> Vec<T> {
    let rank = out.len();
    let mut stride = vec![0usize; rank];
    let mut s = 1;
    for k in (0..rank).rev() {
        let d = if k + src.len() >= rank { src[k + src.len() - rank] } else { 1 };
        stride[k] = if d == 1 { 0 } else { s };
        s *= d;
    }
    let n: usize = out.iter().product();
    let mut res = Vec::with_capacity(n);
    let mut idx = vec![0usize; rank];
    let mut off = 0;
    for _ in 0..n {
        res.push(v[off]);
        for k in (0..rank).rev() {
            idx[k] += 1;
            off += stride[k];
            if idx[k] < out[k] {
                break;
            }
            off -= stride[k] * idx[k];
            idx[k] = 0;
        }
    }
    res
}

/// The index along the axis of each result (std.tensor.17): the first
/// element, then each later one that `better` prefers over the best so far.
fn arg_axis<T: Copy>(v: &[T], before: usize, n: usize, after: usize, better: impl Fn(T, T) -> bool) -> Vec<i64> {
    let mut out = Vec::with_capacity(before * after);
    for b in 0..before {
        for a in 0..after {
            let at = |k: usize| v[(b * n + k) * after + a];
            let mut best = 0;
            for k in 1..n {
                if better(at(k), at(best)) {
                    best = k;
                }
            }
            out.push(best as i64);
        }
    }
    out
}

fn overflow() -> Fault {
    Fault { rule: "std.tensor.6", message: "integer overflow in a tensor operation".into() }
}

fn sum_i64(xs: impl Iterator<Item = i64>) -> Result<i64, Fault> {
    let mut s: i64 = 0;
    for x in xs {
        s = s.checked_add(x).ok_or_else(overflow)?;
    }
    Ok(s)
}

/// std.tensor.5: how the partial sums combine: halve repeatedly,
/// `acc[i] += acc[i + half]`, down to one. Every backend ends this way,
/// then adds the tail elements in order.
fn combine(mut acc: [f64; LANES]) -> f64 {
    let mut half = LANES / 2;
    while half > 0 {
        for i in 0..half {
            acc[i] += acc[i + half];
        }
        half /= 2;
    }
    acc[0]
}

pub fn sum_f64(v: &[f64]) -> f64 {
    match isa() {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: `isa` saw AVX2.
        Isa::Avx2 => unsafe { x86::sum(v) },
        #[cfg(target_arch = "aarch64")]
        // SAFETY: `isa` saw NEON.
        Isa::Neon => unsafe { arm::sum(v) },
        _ => scalar::sum(v),
    }
}

pub fn dot_f64(a: &[f64], b: &[f64]) -> f64 {
    match isa() {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: `isa` saw AVX2.
        Isa::Avx2 => unsafe { x86::dot(a, b) },
        #[cfg(target_arch = "aarch64")]
        // SAFETY: `isa` saw NEON.
        Isa::Neon => unsafe { arm::dot(a, b) },
        _ => scalar::dot(a, b),
    }
}

pub fn zip_f64(a: &[f64], b: &[f64], op: Op) -> Vec<f64> {
    let mut out = vec![0.0; a.len()];
    match isa() {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: `isa` saw AVX2.
        Isa::Avx2 => unsafe { x86::zip(a, b, &mut out, op) },
        #[cfg(target_arch = "aarch64")]
        // SAFETY: `isa` saw NEON.
        Isa::Neon => unsafe { arm::zip(a, b, &mut out, op) },
        _ => scalar::zip(a, b, &mut out, op),
    }
    out
}

/// `a op s`, or `s op a` when `left` (std.tensor.10).
pub fn scalar_f64(a: &[f64], s: f64, op: Op, left: bool) -> Vec<f64> {
    let mut out = vec![0.0; a.len()];
    match isa() {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: `isa` saw AVX2.
        Isa::Avx2 => unsafe { x86::with_scalar(a, s, &mut out, op, left) },
        #[cfg(target_arch = "aarch64")]
        // SAFETY: `isa` saw NEON.
        Isa::Neon => unsafe { arm::with_scalar(a, s, &mut out, op, left) },
        _ => scalar::with_scalar(a, s, &mut out, op, left),
    }
    out
}

/// `m[i] = if b[i] > m[i] then b[i] else m[i]`: IEEE `>`, so a NaN never
/// replaces the running maximum, and ties keep the earlier element
/// (std.tensor.11).
fn max_into(m: &mut [f64], b: &[f64]) {
    match isa() {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: `isa` saw AVX2.
        Isa::Avx2 => unsafe { x86::max_into(m, b) },
        #[cfg(target_arch = "aarch64")]
        // SAFETY: `isa` saw NEON.
        Isa::Neon => unsafe { arm::max_into(m, b) },
        _ => scalar::max_into(m, b),
    }
}

/// `c += b`, elementwise. `axpy` with 1.0 is exact: `1.0 * b` is `b`.
fn add_into(c: &mut [f64], b: &[f64]) {
    axpy(c, 1.0, b);
}

/// What `reduce` computes (std.tensor.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reduce {
    Sum,
    Max,
    Mean,
}

impl Reduce {
    fn name(self) -> &'static str {
        match self {
            Reduce::Sum => "sum",
            Reduce::Max => "max",
            Reduce::Mean => "mean",
        }
    }
}

/// std.tensor.11: each of the `before * after` results sums its `n`
/// elements, `after` apart, in std.tensor.5's order. When `after` is 1
/// they are contiguous and `sum_f64` adds them; otherwise the partial
/// sums are rows of `after` results, added a whole row at a time, which
/// is the same order element by element.
fn sum_axis(v: &[f64], before: usize, n: usize, after: usize) -> Vec<f64> {
    if after == 1 {
        return (0..before).map(|o| sum_f64(&v[o * n..(o + 1) * n])).collect();
    }
    let mut out = Vec::with_capacity(before * after);
    let body = n / LANES * LANES;
    let mut acc = vec![0.0; LANES * after];
    for o in 0..before {
        let row = |i: usize| &v[(o * n + i) * after..(o * n + i + 1) * after];
        acc.fill(0.0);
        for i in 0..body {
            let l = i % LANES;
            add_into(&mut acc[l * after..(l + 1) * after], row(i));
        }
        let mut half = LANES / 2;
        while half > 0 {
            let (lo, hi) = acc.split_at_mut(half * after);
            for l in 0..half {
                add_into(&mut lo[l * after..(l + 1) * after], &hi[l * after..(l + 1) * after]);
            }
            half /= 2;
        }
        let s = &mut acc[..after];
        for i in body..n {
            add_into(s, row(i));
        }
        out.extend_from_slice(s);
    }
    out
}

/// std.tensor.11: the maximum of each `n` elements, `after` apart, the
/// first element then each later one in order.
fn max_axis(v: &[f64], before: usize, n: usize, after: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(before * after);
    for o in 0..before {
        let row = |i: usize| &v[(o * n + i) * after..(o * n + i + 1) * after];
        let start = out.len();
        out.extend_from_slice(row(0));
        for i in 1..n {
            max_into(&mut out[start..], row(i));
        }
    }
    out
}

fn sum_axis_i64(v: &[i64], before: usize, n: usize, after: usize) -> Result<Vec<i64>, Fault> {
    let mut out = Vec::with_capacity(before * after);
    for o in 0..before {
        for j in 0..after {
            out.push(sum_i64((0..n).map(|i| v[(o * n + i) * after + j]))?);
        }
    }
    Ok(out)
}

fn max_axis_i64(v: &[i64], before: usize, n: usize, after: usize) -> Vec<i64> {
    let mut out = Vec::with_capacity(before * after);
    for o in 0..before {
        for j in 0..after {
            out.push((0..n).map(|i| v[(o * n + i) * after + j]).max().expect("checked: `n` > 0"));
        }
    }
    out
}

/// The `n x m` transpose of an `m x n` matrix, in 32 x 32 tiles so both
/// sides stay in cache (std.tensor.12). It moves values, so every target
/// gives the same result.
fn transpose<T: Copy + Default>(v: &[T], m: usize, n: usize) -> Vec<T> {
    const TILE: usize = 32;
    let mut out = vec![T::default(); m * n];
    for i0 in (0..m).step_by(TILE) {
        for j0 in (0..n).step_by(TILE) {
            for i in i0..(i0 + TILE).min(m) {
                for j in j0..(j0 + TILE).min(n) {
                    out[j * m + i] = v[i * n + j];
                }
            }
        }
    }
    out
}

/// `c += x * b`, elementwise: one row of the product (no fused
/// multiply-add, so every backend rounds the same way).
fn axpy(c: &mut [f64], x: f64, b: &[f64]) {
    match isa() {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: `isa` saw AVX2.
        Isa::Avx2 => unsafe { x86::axpy(c, x, b) },
        #[cfg(target_arch = "aarch64")]
        // SAFETY: `isa` saw NEON.
        Isa::Neon => unsafe { arm::axpy(c, x, b) },
        _ => scalar::axpy(c, x, b),
    }
}

/// Rows `rows` of `a * b` into `out` (`rows.len() * n` elements).
fn matmul_rows(a: &[f64], b: &[f64], k: usize, n: usize, rows: std::ops::Range<usize>, out: &mut [f64]) {
    for (r, i) in rows.enumerate() {
        let c = &mut out[r * n..(r + 1) * n];
        for p in 0..k {
            axpy(c, a[i * k + p], &b[p * n..(p + 1) * n]);
        }
    }
}

fn threads_for(m: usize, k: usize, n: usize) -> usize {
    if m.saturating_mul(k).saturating_mul(n) < PAR_WORK {
        1
    } else {
        crate::interp::parallel::threads().min(m).max(1)
    }
}

fn matmul_f64(a: &[f64], b: &[f64], m: usize, k: usize, n: usize) -> Vec<f64> {
    let mut out = vec![0.0; m * n];
    let t = threads_for(m, k, n);
    if t <= 1 || n == 0 {
        matmul_rows(a, b, k, n, 0..m, &mut out);
        return out;
    }
    let per = m.div_ceil(t);
    std::thread::scope(|s| {
        for (c, chunk) in out.chunks_mut(per * n).enumerate() {
            let start = c * per;
            let rows = start..start + chunk.len() / n;
            s.spawn(move || matmul_rows(a, b, k, n, rows, chunk));
        }
    });
    out
}

fn matmul_i64(a: &[i64], b: &[i64], m: usize, k: usize, n: usize) -> Result<Vec<i64>, Fault> {
    let mut out = vec![0i64; m * n];
    for i in 0..m {
        for p in 0..k {
            let x = a[i * k + p];
            for j in 0..n {
                let prod = x.checked_mul(b[p * n + j]).ok_or_else(overflow)?;
                let c = &mut out[i * n + j];
                *c = c.checked_add(prod).ok_or_else(overflow)?;
            }
        }
    }
    Ok(out)
}

/// The reference kernels: what the vector ones compute, operation for
/// operation (std.tensor.5).
mod scalar {
    use super::{LANES, Op, combine};

    pub fn sum(v: &[f64]) -> f64 {
        let mut acc = [0.0; LANES];
        let body = v.len() / LANES * LANES;
        for chunk in v[..body].chunks_exact(LANES) {
            for l in 0..LANES {
                acc[l] += chunk[l];
            }
        }
        let mut s = combine(acc);
        for x in &v[body..] {
            s += x;
        }
        s
    }

    pub fn dot(a: &[f64], b: &[f64]) -> f64 {
        let mut acc = [0.0; LANES];
        let body = a.len() / LANES * LANES;
        for (x, y) in a[..body].chunks_exact(LANES).zip(b[..body].chunks_exact(LANES)) {
            for l in 0..LANES {
                acc[l] += x[l] * y[l];
            }
        }
        let mut s = combine(acc);
        for i in body..a.len() {
            s += a[i] * b[i];
        }
        s
    }

    pub fn zip(a: &[f64], b: &[f64], out: &mut [f64], op: Op) {
        for i in 0..a.len() {
            out[i] = op.f(a[i], b[i]);
        }
    }

    pub fn with_scalar(a: &[f64], s: f64, out: &mut [f64], op: Op, left: bool) {
        for i in 0..a.len() {
            out[i] = if left { op.f(s, a[i]) } else { op.f(a[i], s) };
        }
    }

    pub fn max_into(m: &mut [f64], b: &[f64]) {
        for (m, &b) in m.iter_mut().zip(b) {
            if b > *m {
                *m = b;
            }
        }
    }

    pub fn axpy(c: &mut [f64], x: f64, b: &[f64]) {
        for (c, b) in c.iter_mut().zip(b) {
            *c += x * b;
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::{LANES, Op, combine};
    use std::arch::x86_64::*;

    const REGS: usize = LANES / 4;

    #[target_feature(enable = "avx2")]
    pub unsafe fn sum(v: &[f64]) -> f64 {
        let body = v.len() / LANES * LANES;
        let p = v.as_ptr();
        let mut acc = [_mm256_setzero_pd(); REGS];
        let mut i = 0;
        while i < body {
            for (r, a) in acc.iter_mut().enumerate() {
                // SAFETY: `i + LANES <= body <= v.len()`.
                *a = _mm256_add_pd(*a, unsafe { _mm256_loadu_pd(p.add(i + 4 * r)) });
            }
            i += LANES;
        }
        let mut s = combine(lanes(acc));
        for x in &v[body..] {
            s += x;
        }
        s
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn dot(a: &[f64], b: &[f64]) -> f64 {
        let body = a.len() / LANES * LANES;
        let (pa, pb) = (a.as_ptr(), b.as_ptr());
        let mut acc = [_mm256_setzero_pd(); REGS];
        let mut i = 0;
        while i < body {
            for (r, s) in acc.iter_mut().enumerate() {
                // SAFETY: `i + LANES <= body`, and `a` and `b` have equal lengths.
                let (x, y) = unsafe { (_mm256_loadu_pd(pa.add(i + 4 * r)), _mm256_loadu_pd(pb.add(i + 4 * r))) };
                *s = _mm256_add_pd(*s, _mm256_mul_pd(x, y));
            }
            i += LANES;
        }
        let mut s = combine(lanes(acc));
        for i in body..a.len() {
            s += a[i] * b[i];
        }
        s
    }

    #[target_feature(enable = "avx2")]
    fn apply(op: Op, x: __m256d, y: __m256d) -> __m256d {
        match op {
            Op::Add => _mm256_add_pd(x, y),
            Op::Sub => _mm256_sub_pd(x, y),
            Op::Mul => _mm256_mul_pd(x, y),
            Op::Div => _mm256_div_pd(x, y),
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn zip(a: &[f64], b: &[f64], out: &mut [f64], op: Op) {
        let body = a.len() / 4 * 4;
        let (pa, pb, po) = (a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and all three slices have equal lengths.
            unsafe {
                let (x, y) = (_mm256_loadu_pd(pa.add(i)), _mm256_loadu_pd(pb.add(i)));
                _mm256_storeu_pd(po.add(i), apply(op, x, y));
            }
            i += 4;
        }
        for i in body..a.len() {
            out[i] = op.f(a[i], b[i]);
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn with_scalar(a: &[f64], s: f64, out: &mut [f64], op: Op, left: bool) {
        let body = a.len() / 4 * 4;
        let (pa, po) = (a.as_ptr(), out.as_mut_ptr());
        let ss = _mm256_set1_pd(s);
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and `out` is as long as `a`.
            unsafe {
                let x = _mm256_loadu_pd(pa.add(i));
                _mm256_storeu_pd(po.add(i), if left { apply(op, ss, x) } else { apply(op, x, ss) });
            }
            i += 4;
        }
        for i in body..a.len() {
            out[i] = if left { op.f(s, a[i]) } else { op.f(a[i], s) };
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn max_into(m: &mut [f64], b: &[f64]) {
        let body = m.len() / 4 * 4;
        let (pm, pb) = (m.as_mut_ptr(), b.as_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and `b` is as long as `m`.
            unsafe {
                let (x, y) = (_mm256_loadu_pd(pm.add(i)), _mm256_loadu_pd(pb.add(i)));
                let gt = _mm256_cmp_pd::<_CMP_GT_OQ>(y, x);
                _mm256_storeu_pd(pm.add(i), _mm256_blendv_pd(x, y, gt));
            }
            i += 4;
        }
        for i in body..m.len() {
            if b[i] > m[i] {
                m[i] = b[i];
            }
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn axpy(c: &mut [f64], x: f64, b: &[f64]) {
        let body = c.len() / 4 * 4;
        let (pc, pb) = (c.as_mut_ptr(), b.as_ptr());
        let xs = _mm256_set1_pd(x);
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and `b` is as long as `c`.
            unsafe {
                let r = _mm256_add_pd(_mm256_loadu_pd(pc.add(i)), _mm256_mul_pd(xs, _mm256_loadu_pd(pb.add(i))));
                _mm256_storeu_pd(pc.add(i), r);
            }
            i += 4;
        }
        for i in body..c.len() {
            c[i] += x * b[i];
        }
    }

    /// Accumulator `r` holds lanes `4r` to `4r + 3`.
    #[target_feature(enable = "avx2")]
    fn lanes(acc: [__m256d; REGS]) -> [f64; LANES] {
        let mut out = [0.0; LANES];
        for (r, a) in acc.iter().enumerate() {
            // SAFETY: `out` holds `LANES` `f64`s.
            unsafe { _mm256_storeu_pd(out.as_mut_ptr().add(4 * r), *a) };
        }
        out
    }
}

#[cfg(target_arch = "aarch64")]
mod arm {
    use super::{LANES, Op, combine};
    use std::arch::aarch64::*;

    const REGS: usize = LANES / 2;

    #[target_feature(enable = "neon")]
    pub unsafe fn sum(v: &[f64]) -> f64 {
        let body = v.len() / LANES * LANES;
        let p = v.as_ptr();
        let mut acc = [vdupq_n_f64(0.0); REGS];
        let mut i = 0;
        while i < body {
            for (r, a) in acc.iter_mut().enumerate() {
                // SAFETY: `i + LANES <= body <= v.len()`.
                *a = vaddq_f64(*a, unsafe { vld1q_f64(p.add(i + 2 * r)) });
            }
            i += LANES;
        }
        let mut s = combine(lanes(acc));
        for x in &v[body..] {
            s += x;
        }
        s
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn dot(a: &[f64], b: &[f64]) -> f64 {
        let body = a.len() / LANES * LANES;
        let (pa, pb) = (a.as_ptr(), b.as_ptr());
        let mut acc = [vdupq_n_f64(0.0); REGS];
        let mut i = 0;
        while i < body {
            for (r, s) in acc.iter_mut().enumerate() {
                // SAFETY: `i + LANES <= body`, and `a` and `b` have equal lengths.
                let (x, y) = unsafe { (vld1q_f64(pa.add(i + 2 * r)), vld1q_f64(pb.add(i + 2 * r))) };
                *s = vaddq_f64(*s, vmulq_f64(x, y));
            }
            i += LANES;
        }
        let mut s = combine(lanes(acc));
        for i in body..a.len() {
            s += a[i] * b[i];
        }
        s
    }

    #[target_feature(enable = "neon")]
    fn apply(op: Op, x: float64x2_t, y: float64x2_t) -> float64x2_t {
        match op {
            Op::Add => vaddq_f64(x, y),
            Op::Sub => vsubq_f64(x, y),
            Op::Mul => vmulq_f64(x, y),
            Op::Div => vdivq_f64(x, y),
        }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn zip(a: &[f64], b: &[f64], out: &mut [f64], op: Op) {
        let body = a.len() / 2 * 2;
        let (pa, pb, po) = (a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 2 <= body`, and all three slices have equal lengths.
            unsafe {
                let (x, y) = (vld1q_f64(pa.add(i)), vld1q_f64(pb.add(i)));
                vst1q_f64(po.add(i), apply(op, x, y));
            }
            i += 2;
        }
        for i in body..a.len() {
            out[i] = op.f(a[i], b[i]);
        }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn with_scalar(a: &[f64], s: f64, out: &mut [f64], op: Op, left: bool) {
        let body = a.len() / 2 * 2;
        let (pa, po) = (a.as_ptr(), out.as_mut_ptr());
        let ss = vdupq_n_f64(s);
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 2 <= body`, and `out` is as long as `a`.
            unsafe {
                let x = vld1q_f64(pa.add(i));
                vst1q_f64(po.add(i), if left { apply(op, ss, x) } else { apply(op, x, ss) });
            }
            i += 2;
        }
        for i in body..a.len() {
            out[i] = if left { op.f(s, a[i]) } else { op.f(a[i], s) };
        }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn max_into(m: &mut [f64], b: &[f64]) {
        let body = m.len() / 2 * 2;
        let (pm, pb) = (m.as_mut_ptr(), b.as_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 2 <= body`, and `b` is as long as `m`.
            unsafe {
                let (x, y) = (vld1q_f64(pm.add(i)), vld1q_f64(pb.add(i)));
                vst1q_f64(pm.add(i), vbslq_f64(vcgtq_f64(y, x), y, x));
            }
            i += 2;
        }
        for i in body..m.len() {
            if b[i] > m[i] {
                m[i] = b[i];
            }
        }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn axpy(c: &mut [f64], x: f64, b: &[f64]) {
        let body = c.len() / 2 * 2;
        let (pc, pb) = (c.as_mut_ptr(), b.as_ptr());
        let xs = vdupq_n_f64(x);
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 2 <= body`, and `b` is as long as `c`.
            unsafe {
                vst1q_f64(pc.add(i), vaddq_f64(vld1q_f64(pc.add(i)), vmulq_f64(xs, vld1q_f64(pb.add(i)))));
            }
            i += 2;
        }
        for i in body..c.len() {
            c[i] += x * b[i];
        }
    }

    /// Accumulator `r` holds lanes `2r` and `2r + 1`.
    #[target_feature(enable = "neon")]
    fn lanes(acc: [float64x2_t; REGS]) -> [f64; LANES] {
        let mut out = [0.0; LANES];
        for (r, a) in acc.iter().enumerate() {
            // SAFETY: `out` holds `LANES` `f64`s.
            unsafe { vst1q_f64(out.as_mut_ptr().add(2 * r), *a) };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(n: usize) -> Vec<f64> {
        (0..n).map(|i| ((i * 7919) % 1000) as f64 * 0.001 + 1e-9 * i as f64).collect()
    }

    #[test]
    fn vector_kernels_match_scalar_bit_for_bit() {
        for n in [0, 1, 3, 7, 8, 9, 15, 16, 17, 100, 1001] {
            let (a, b) = (data(n), data(n + 3)[3..].to_vec());
            assert_eq!(sum_f64(&a).to_bits(), scalar::sum(&a).to_bits(), "sum {n}");
            assert_eq!(dot_f64(&a, &b).to_bits(), scalar::dot(&a, &b).to_bits(), "dot {n}");
            for op in [Op::Add, Op::Sub, Op::Mul] {
                let mut out = vec![0.0; n];
                scalar::zip(&a, &b, &mut out, op);
                assert_eq!(zip_f64(&a, &b, op), out, "{op:?} {n}");
                for left in [false, true] {
                    scalar::with_scalar(&a, 0.37, &mut out, op, left);
                    assert_eq!(scalar_f64(&a, 0.37, op, left), out, "{op:?} scalar {n}");
                }
            }
            let mut m = a.clone();
            max_into(&mut m, &b);
            let mut want = a.clone();
            scalar::max_into(&mut want, &b);
            assert_eq!(m, want, "max {n}");
        }
    }

    #[test]
    fn axis_reductions_use_the_one_order() {
        for shape in [[2, 37, 3], [1, 16, 5], [3, 5, 1], [4, 1, 2], [2, 100, 9]] {
            let [before, n, after] = shape;
            let v = data(before * n * after);
            let mut v2 = v.clone();
            v2[1] = f64::NAN;
            let (s, m) = (sum_axis(&v, before, n, after), max_axis(&v2, before, n, after));
            for o in 0..before {
                for j in 0..after {
                    let col = |w: &[f64]| (0..n).map(|i| w[(o * n + i) * after + j]).collect::<Vec<f64>>();
                    assert_eq!(s[o * after + j].to_bits(), scalar::sum(&col(&v)).to_bits(), "sum {shape:?}");
                    let c = col(&v2);
                    let mut want = c[0];
                    for &x in &c[1..] {
                        if x > want {
                            want = x;
                        }
                    }
                    assert_eq!(m[o * after + j].to_bits(), want.to_bits(), "max {shape:?}");
                }
            }
        }
    }

    #[test]
    fn transpose_moves_every_element() {
        for (m, n) in [(1, 1), (3, 70), (33, 65)] {
            let v: Vec<i64> = (0..(m * n) as i64).collect();
            let t = transpose(&v, m, n);
            for i in 0..m {
                for j in 0..n {
                    assert_eq!(t[j * m + i], v[i * n + j]);
                }
            }
        }
    }

    #[test]
    fn matmul_equals_the_triple_loop() {
        for (m, k, n) in [(1, 1, 1), (3, 5, 7), (64, 64, 64), (70, 65, 9)] {
            let (a, b) = (data(m * k), data(k * n));
            let got = matmul_f64(&a, &b, m, k, n);
            for i in 0..m {
                for j in 0..n {
                    let mut c = 0.0;
                    for p in 0..k {
                        c += a[i * k + p] * b[p * n + j];
                    }
                    assert_eq!(got[i * n + j].to_bits(), c.to_bits(), "{m}x{k}x{n} at {i},{j}");
                }
            }
        }
    }
}
