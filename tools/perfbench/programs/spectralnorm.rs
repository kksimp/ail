// Spectral norm: the same power iteration as spectralnorm.ail.
fn eval_a(i: i64, j: i64) -> f64 {
    let d = (i + j) * (i + j + 1) / 2 + i + 1;
    1.0 / d as f64
}

fn mul_av(v: &[f64], out: &mut [f64], n: usize) {
    for i in 0..n {
        let mut sum = 0.0f64;
        for j in 0..n {
            sum += eval_a(i as i64, j as i64) * v[j];
        }
        out[i] = sum;
    }
}

fn mul_atv(v: &[f64], out: &mut [f64], n: usize) {
    for i in 0..n {
        let mut sum = 0.0f64;
        for j in 0..n {
            sum += eval_a(j as i64, i as i64) * v[j];
        }
        out[i] = sum;
    }
}

fn mul_ata(v: &[f64], out: &mut [f64], tmp: &mut [f64], n: usize) {
    mul_av(v, tmp, n);
    mul_atv(tmp, out, n);
}

fn main() {
    let n: usize = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(200);
    let mut u = vec![1.0f64; n];
    let mut v = vec![0.0f64; n];
    let mut tmp = vec![0.0f64; n];
    for _ in 0..10 {
        mul_ata(&u, &mut v, &mut tmp, n);
        mul_ata(&v, &mut u, &mut tmp, n);
    }
    let mut vbv = 0.0f64;
    let mut vv = 0.0f64;
    for i in 0..n {
        vbv += u[i] * v[i];
        vv += v[i] * v[i];
    }
    println!("{:.9}", (vbv / vv).sqrt());
}
