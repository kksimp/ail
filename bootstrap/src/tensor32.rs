//! `Tensor[f32]`'s kernels (spec std.tensor.21, ADR 0796): the `float`
//! kernels of `tensor.rs` on 32-bit elements. Every operation is one IEEE
//! `f32` operation, rounded once, so the scalar, AVX2 (8 lanes) and NEON
//! (4 lanes) paths give bit-identical results, with the same accumulator
//! order as the `f64` kernels (std.tensor.5): element `i` of a sum goes to
//! accumulator `i % LANES`, and the accumulators halve down to one.

use crate::tensor::{Isa, LANES, Op, isa};

/// The halving combine of `combine` in `tensor.rs`, in `f32`.
fn combine(mut acc: [f32; LANES]) -> f32 {
    let mut half = LANES / 2;
    while half > 0 {
        for i in 0..half {
            acc[i] += acc[i + half];
        }
        half /= 2;
    }
    acc[0]
}

pub fn sum(v: &[f32]) -> f32 {
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

pub fn dot(a: &[f32], b: &[f32]) -> f32 {
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

pub fn zip(a: &[f32], b: &[f32], op: Op) -> Vec<f32> {
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
pub fn with_scalar(a: &[f32], s: f32, op: Op, left: bool) -> Vec<f32> {
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

/// `m[i] = b[i]` when `b[i] > m[i]` (IEEE `>`, std.tensor.11).
pub fn max_into(m: &mut [f32], b: &[f32]) {
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

/// `c += x * b`, elementwise, with separate rounding of the product (as in
/// the `f64` kernels; there is no fused multiply-add).
pub fn axpy(c: &mut [f32], x: f32, b: &[f32]) {
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

mod scalar {
    use super::{LANES, Op, combine};

    pub fn sum(v: &[f32]) -> f32 {
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

    pub fn dot(a: &[f32], b: &[f32]) -> f32 {
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

    pub fn zip(a: &[f32], b: &[f32], out: &mut [f32], op: Op) {
        for i in 0..a.len() {
            out[i] = op.f32(a[i], b[i]);
        }
    }

    pub fn with_scalar(a: &[f32], s: f32, out: &mut [f32], op: Op, left: bool) {
        for i in 0..a.len() {
            out[i] = if left { op.f32(s, a[i]) } else { op.f32(a[i], s) };
        }
    }

    pub fn max_into(m: &mut [f32], b: &[f32]) {
        for (m, &b) in m.iter_mut().zip(b) {
            if b > *m {
                *m = b;
            }
        }
    }

    pub fn axpy(c: &mut [f32], x: f32, b: &[f32]) {
        for (c, b) in c.iter_mut().zip(b) {
            *c += x * b;
        }
    }
}

#[cfg(target_arch = "x86_64")]
mod x86 {
    use super::{LANES, Op, combine};
    use std::arch::x86_64::*;

    /// Eight `f32` lanes per AVX2 register: two registers hold 16 lanes.
    const REGS: usize = LANES / 8;

    #[target_feature(enable = "avx2")]
    pub unsafe fn sum(v: &[f32]) -> f32 {
        let body = v.len() / LANES * LANES;
        let p = v.as_ptr();
        let mut acc = [_mm256_setzero_ps(); REGS];
        let mut i = 0;
        while i < body {
            for (r, a) in acc.iter_mut().enumerate() {
                // SAFETY: `i + LANES <= body <= v.len()`.
                *a = _mm256_add_ps(*a, unsafe { _mm256_loadu_ps(p.add(i + 8 * r)) });
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
    pub unsafe fn dot(a: &[f32], b: &[f32]) -> f32 {
        let body = a.len() / LANES * LANES;
        let (pa, pb) = (a.as_ptr(), b.as_ptr());
        let mut acc = [_mm256_setzero_ps(); REGS];
        let mut i = 0;
        while i < body {
            for (r, s) in acc.iter_mut().enumerate() {
                // SAFETY: `i + LANES <= body`, and `a` and `b` have equal lengths.
                let (x, y) = unsafe { (_mm256_loadu_ps(pa.add(i + 8 * r)), _mm256_loadu_ps(pb.add(i + 8 * r))) };
                *s = _mm256_add_ps(*s, _mm256_mul_ps(x, y));
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
    fn apply(op: Op, x: __m256, y: __m256) -> __m256 {
        match op {
            Op::Add => _mm256_add_ps(x, y),
            Op::Sub => _mm256_sub_ps(x, y),
            Op::Mul => _mm256_mul_ps(x, y),
            Op::Div => _mm256_div_ps(x, y),
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn zip(a: &[f32], b: &[f32], out: &mut [f32], op: Op) {
        let body = a.len() / 8 * 8;
        let (pa, pb, po) = (a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 8 <= body`, and all three slices have equal lengths.
            unsafe {
                let (x, y) = (_mm256_loadu_ps(pa.add(i)), _mm256_loadu_ps(pb.add(i)));
                _mm256_storeu_ps(po.add(i), apply(op, x, y));
            }
            i += 8;
        }
        for i in body..a.len() {
            out[i] = op.f32(a[i], b[i]);
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn with_scalar(a: &[f32], s: f32, out: &mut [f32], op: Op, left: bool) {
        let body = a.len() / 8 * 8;
        let (pa, po) = (a.as_ptr(), out.as_mut_ptr());
        let ss = _mm256_set1_ps(s);
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 8 <= body`, and `out` is as long as `a`.
            unsafe {
                let x = _mm256_loadu_ps(pa.add(i));
                _mm256_storeu_ps(po.add(i), if left { apply(op, ss, x) } else { apply(op, x, ss) });
            }
            i += 8;
        }
        for i in body..a.len() {
            out[i] = if left { op.f32(s, a[i]) } else { op.f32(a[i], s) };
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn max_into(m: &mut [f32], b: &[f32]) {
        let body = m.len() / 8 * 8;
        let (pm, pb) = (m.as_mut_ptr(), b.as_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 8 <= body`, and `b` is as long as `m`.
            unsafe {
                let (x, y) = (_mm256_loadu_ps(pm.add(i)), _mm256_loadu_ps(pb.add(i)));
                let gt = _mm256_cmp_ps::<_CMP_GT_OQ>(y, x);
                _mm256_storeu_ps(pm.add(i), _mm256_blendv_ps(x, y, gt));
            }
            i += 8;
        }
        for i in body..m.len() {
            if b[i] > m[i] {
                m[i] = b[i];
            }
        }
    }

    #[target_feature(enable = "avx2")]
    pub unsafe fn axpy(c: &mut [f32], x: f32, b: &[f32]) {
        let body = c.len() / 8 * 8;
        let (pc, pb) = (c.as_mut_ptr(), b.as_ptr());
        let xs = _mm256_set1_ps(x);
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 8 <= body`, and `b` is as long as `c`.
            unsafe {
                let r = _mm256_add_ps(_mm256_loadu_ps(pc.add(i)), _mm256_mul_ps(xs, _mm256_loadu_ps(pb.add(i))));
                _mm256_storeu_ps(pc.add(i), r);
            }
            i += 8;
        }
        for i in body..c.len() {
            c[i] += x * b[i];
        }
    }

    /// Accumulator `r` holds lanes `8r` to `8r + 7`.
    #[target_feature(enable = "avx2")]
    fn lanes(acc: [__m256; REGS]) -> [f32; LANES] {
        let mut out = [0.0; LANES];
        for (r, a) in acc.iter().enumerate() {
            // SAFETY: `out` holds `LANES` `f32`s.
            unsafe { _mm256_storeu_ps(out.as_mut_ptr().add(8 * r), *a) };
        }
        out
    }
}

#[cfg(target_arch = "aarch64")]
mod arm {
    use super::{LANES, Op, combine};
    use std::arch::aarch64::*;

    /// Four `f32` lanes per NEON register: four registers hold 16 lanes.
    const REGS: usize = LANES / 4;

    #[target_feature(enable = "neon")]
    pub unsafe fn sum(v: &[f32]) -> f32 {
        let body = v.len() / LANES * LANES;
        let p = v.as_ptr();
        let mut acc = [vdupq_n_f32(0.0); REGS];
        let mut i = 0;
        while i < body {
            for (r, a) in acc.iter_mut().enumerate() {
                // SAFETY: `i + LANES <= body <= v.len()`.
                *a = vaddq_f32(*a, unsafe { vld1q_f32(p.add(i + 4 * r)) });
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
    pub unsafe fn dot(a: &[f32], b: &[f32]) -> f32 {
        let body = a.len() / LANES * LANES;
        let (pa, pb) = (a.as_ptr(), b.as_ptr());
        let mut acc = [vdupq_n_f32(0.0); REGS];
        let mut i = 0;
        while i < body {
            for (r, s) in acc.iter_mut().enumerate() {
                // SAFETY: `i + LANES <= body`, and `a` and `b` have equal lengths.
                let (x, y) = unsafe { (vld1q_f32(pa.add(i + 4 * r)), vld1q_f32(pb.add(i + 4 * r))) };
                *s = vaddq_f32(*s, vmulq_f32(x, y));
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
    fn apply(op: Op, x: float32x4_t, y: float32x4_t) -> float32x4_t {
        match op {
            Op::Add => vaddq_f32(x, y),
            Op::Sub => vsubq_f32(x, y),
            Op::Mul => vmulq_f32(x, y),
            Op::Div => vdivq_f32(x, y),
        }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn zip(a: &[f32], b: &[f32], out: &mut [f32], op: Op) {
        let body = a.len() / 4 * 4;
        let (pa, pb, po) = (a.as_ptr(), b.as_ptr(), out.as_mut_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and all three slices have equal lengths.
            unsafe {
                let (x, y) = (vld1q_f32(pa.add(i)), vld1q_f32(pb.add(i)));
                vst1q_f32(po.add(i), apply(op, x, y));
            }
            i += 4;
        }
        for i in body..a.len() {
            out[i] = op.f32(a[i], b[i]);
        }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn with_scalar(a: &[f32], s: f32, out: &mut [f32], op: Op, left: bool) {
        let body = a.len() / 4 * 4;
        let (pa, po) = (a.as_ptr(), out.as_mut_ptr());
        let ss = vdupq_n_f32(s);
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and `out` is as long as `a`.
            unsafe {
                let x = vld1q_f32(pa.add(i));
                vst1q_f32(po.add(i), if left { apply(op, ss, x) } else { apply(op, x, ss) });
            }
            i += 4;
        }
        for i in body..a.len() {
            out[i] = if left { op.f32(s, a[i]) } else { op.f32(a[i], s) };
        }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn max_into(m: &mut [f32], b: &[f32]) {
        let body = m.len() / 4 * 4;
        let (pm, pb) = (m.as_mut_ptr(), b.as_ptr());
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and `b` is as long as `m`.
            unsafe {
                let (x, y) = (vld1q_f32(pm.add(i)), vld1q_f32(pb.add(i)));
                vst1q_f32(pm.add(i), vbslq_f32(vcgtq_f32(y, x), y, x));
            }
            i += 4;
        }
        for i in body..m.len() {
            if b[i] > m[i] {
                m[i] = b[i];
            }
        }
    }

    #[target_feature(enable = "neon")]
    pub unsafe fn axpy(c: &mut [f32], x: f32, b: &[f32]) {
        let body = c.len() / 4 * 4;
        let (pc, pb) = (c.as_mut_ptr(), b.as_ptr());
        let xs = vdupq_n_f32(x);
        let mut i = 0;
        while i < body {
            // SAFETY: `i + 4 <= body`, and `b` is as long as `c`.
            unsafe {
                vst1q_f32(pc.add(i), vaddq_f32(vld1q_f32(pc.add(i)), vmulq_f32(xs, vld1q_f32(pb.add(i)))));
            }
            i += 4;
        }
        for i in body..c.len() {
            c[i] += x * b[i];
        }
    }

    /// Accumulator `r` holds lanes `4r` to `4r + 3`.
    #[target_feature(enable = "neon")]
    fn lanes(acc: [float32x4_t; REGS]) -> [f32; LANES] {
        let mut out = [0.0; LANES];
        for (r, a) in acc.iter().enumerate() {
            // SAFETY: `out` holds `LANES` `f32`s.
            unsafe { vst1q_f32(out.as_mut_ptr().add(4 * r), *a) };
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(n: usize) -> Vec<f32> {
        (0..n).map(|i| ((i * 7919) % 1000) as f32 * 0.001 + 1e-6 * i as f32).collect()
    }

    /// The dispatched kernels (AVX2 or NEON when present) equal the scalar
    /// reference bit for bit (std.tensor.5, ADR 0008).
    #[test]
    fn vector_kernels_match_scalar_bit_for_bit() {
        for n in [0, 1, 3, 7, 8, 9, 15, 16, 17, 100, 1001] {
            let (a, b) = (data(n), data(n + 3)[3..].to_vec());
            assert_eq!(sum(&a).to_bits(), scalar::sum(&a).to_bits(), "sum {n}");
            assert_eq!(dot(&a, &b).to_bits(), scalar::dot(&a, &b).to_bits(), "dot {n}");
            for op in [Op::Add, Op::Sub, Op::Mul, Op::Div] {
                let mut out = vec![0.0; n];
                scalar::zip(&a, &b, &mut out, op);
                assert_eq!(zip(&a, &b, op), out, "{op:?} {n}");
                for left in [false, true] {
                    scalar::with_scalar(&a, 0.37, &mut out, op, left);
                    assert_eq!(with_scalar(&a, 0.37, op, left), out, "{op:?} scalar {n}");
                }
            }
            let mut m = a.clone();
            max_into(&mut m, &b);
            let mut want = a.clone();
            scalar::max_into(&mut want, &b);
            assert_eq!(m, want, "max {n}");
            let mut c = a.clone();
            axpy(&mut c, 0.5, &b);
            let mut want = a.clone();
            scalar::axpy(&mut want, 0.5, &b);
            assert_eq!(c, want, "axpy {n}");
        }
    }
}
