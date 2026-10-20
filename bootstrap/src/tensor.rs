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
/// to accumulator `i % LANES`; two AVX2 registers or four NEON registers.
const LANES: usize = 8;

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

    /// std.tensor.4: elementwise `+` (`mul`: `false`) or `*` (`true`).
    pub fn zip(&self, other: &Tensor, mul: bool) -> Result<Tensor, Fault> {
        self.same_shape(other, if mul { "mul" } else { "add" })?;
        let data = match (&self.data, &other.data) {
            (Data::F(a), Data::F(b)) => Data::F(zip_f64(a, b, mul)),
            (Data::I(a), Data::I(b)) => {
                let mut out = Vec::with_capacity(a.len());
                for (x, y) in a.iter().zip(b) {
                    let r = if mul { x.checked_mul(*y) } else { x.checked_add(*y) };
                    out.push(r.ok_or_else(overflow)?);
                }
                Data::I(out)
            }
            _ => unreachable!("checked: one element type"),
        };
        Ok(Tensor { shape: self.shape.clone(), data })
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

/// The result of `sum` or `dot`.
pub enum Sum {
    F(f64),
    I(i64),
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

/// std.tensor.5: how the eight partial sums combine: lanewise into four
/// (`acc[i] + acc[i + 4]`), then `(t0 + t2) + (t1 + t3)`. Every backend
/// ends this way, then adds the tail elements in order.
fn combine(acc: [f64; LANES]) -> f64 {
    let t = [acc[0] + acc[4], acc[1] + acc[5], acc[2] + acc[6], acc[3] + acc[7]];
    (t[0] + t[2]) + (t[1] + t[3])
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

pub fn zip_f64(a: &[f64], b: &[f64], mul: bool) -> Vec<f64> {
    let mut out = vec![0.0; a.len()];
    match isa() {
        #[cfg(target_arch = "x86_64")]
        // SAFETY: `isa` saw AVX2.
        Isa::Avx2 => unsafe { x86::zip(a, b, &mut out, mul) },
        #[cfg(target_arch = "aarch64")]
        // SAFETY: `isa` saw NEON.
        Isa::Neon => unsafe { arm::zip(a, b, &mut out, mul) },
        _ => scalar::zip(a, b, &mut out, mul),
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
    use super::{LANES, combine};

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

    pub fn zip(a: &[f64], b: &[f64], out: &mut [f64], mul: bool) {
        for i in 0..a.len() {
            out[i] = if mul { a[i] * b[i] } else { a[i] + b[i] };
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
    use super::{LANES, combine};
    use std::arch::x86_64::*;

    #[target_feature(enable = "avx2")]
    pub unsafe fn sum(v: &[f64]) -> f64 {
        let body = v.len() / LANES * LANES;
        let p = v.as_ptr();
        let (mut lo, mut hi) = (_mm256_setzero_pd(), _mm256_setzero_pd());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 8 <= body <= v.len()`.
            unsafe {
                lo = _mm256_add_pd(lo, _mm256_loadu_pd(p.add(i)));
                hi = _mm256_add_pd(hi, _mm256_loadu_pd(p.add(i + 4)));
            }
            i += LANES;
        }
        let mut s = combine(lanes(lo, hi));
        for x in &v[body..] {
            s += x;
        }
        s
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn dot(a: &[f64], b: &[f64]) -> f64 {
        let body = a.len() / LANES * LANES;
        let (pa, pb) = (a.as_ptr(), b.as_ptr());
        let (mut lo, mut hi) = (_mm256_setzero_pd(), _mm256_setzero_pd());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 8 <= body`, and `a` and `b` have equal lengths.
            unsafe {
                lo = _mm256_add_pd(lo, _mm256_mul_pd(_mm256_loadu_pd(pa.add(i)), _mm256_loadu_pd(pb.add(i))));
                hi = _mm256_add_pd(hi, _mm256_mul_pd(_mm256_loadu_pd(pa.add(i + 4)), _mm256_loadu_pd(pb.add(i + 4))));
            }
            i += LANES;
        }
        let mut s = combine(lanes(lo, hi));
        for i in body..a.len() {
            s += a[i] * b[i];
        }
        s
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn zip(a: &[f64], b: &[f64], out: &mut [f64], mul: bool) {
        let body = a.len() / 4 * 4;
        let (pa, pb, po) = (a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and all three slices have equal lengths.
            unsafe {
                let (x, y) = (_mm256_loadu_pd(pa.add(i)), _mm256_loadu_pd(pb.add(i)));
                _mm256_storeu_pd(po.add(i), if mul { _mm256_mul_pd(x, y) } else { _mm256_add_pd(x, y) });
            }
            i += 4;
        }
        for i in body..a.len() {
            out[i] = if mul { a[i] * b[i] } else { a[i] + b[i] };
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

    #[target_feature(enable = "avx2")]
    fn lanes(lo: __m256d, hi: __m256d) -> [f64; LANES] {
        let mut out = [0.0; LANES];
        // SAFETY: `out` holds eight `f64`s.
        unsafe {
            _mm256_storeu_pd(out.as_mut_ptr(), lo);
            _mm256_storeu_pd(out.as_mut_ptr().add(4), hi);
        }
        out
    }
}

#[cfg(target_arch = "aarch64")]
mod arm {
    use super::{LANES, combine};
    use std::arch::aarch64::*;

    #[target_feature(enable = "neon")]
    pub unsafe fn sum(v: &[f64]) -> f64 {
        let body = v.len() / LANES * LANES;
        let p = v.as_ptr();
        let mut acc = [vdupq_n_f64(0.0); 4];
        let mut i = 0;
        while i < body {
            for (r, a) in acc.iter_mut().enumerate() {
                // SAFETY: `i + 8 <= body <= v.len()`.
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
        let mut acc = [vdupq_n_f64(0.0); 4];
        let mut i = 0;
        while i < body {
            for (r, s) in acc.iter_mut().enumerate() {
                // SAFETY: `i + 8 <= body`, and `a` and `b` have equal lengths.
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
    pub unsafe fn zip(a: &[f64], b: &[f64], out: &mut [f64], mul: bool) {
        let body = a.len() / 2 * 2;
        let (pa, pb, po) = (a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 2 <= body`, and all three slices have equal lengths.
            unsafe {
                let (x, y) = (vld1q_f64(pa.add(i)), vld1q_f64(pb.add(i)));
                vst1q_f64(po.add(i), if mul { vmulq_f64(x, y) } else { vaddq_f64(x, y) });
            }
            i += 2;
        }
        for i in body..a.len() {
            out[i] = if mul { a[i] * b[i] } else { a[i] + b[i] };
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
    fn lanes(acc: [float64x2_t; 4]) -> [f64; LANES] {
        let mut out = [0.0; LANES];
        for (r, a) in acc.iter().enumerate() {
            // SAFETY: `out` holds eight `f64`s.
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
            let mut out = vec![0.0; n];
            scalar::zip(&a, &b, &mut out, true);
            assert_eq!(zip_f64(&a, &b, true), out, "mul {n}");
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
