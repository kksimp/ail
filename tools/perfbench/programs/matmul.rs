// Dense matrix multiply: the same entries and checks as matmul.ail.
fn main() {
    let n: usize = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(120);
    let mut a = vec![0i64; n * n];
    let mut b = vec![0i64; n * n];
    let mut c = vec![0i64; n * n];
    for i in 0..n {
        for j in 0..n {
            let (ii, jj) = (i as i64, j as i64);
            a[i * n + j] = (ii * jj + ii + jj) % 7;
            b[i * n + j] = (ii + 2 * jj) % 5;
        }
    }
    for i in 0..n {
        for j in 0..n {
            let mut s = 0i64;
            for k in 0..n {
                s += a[i * n + k] * b[k * n + j];
            }
            c[i * n + j] = s;
        }
    }
    let mut trace = 0i64;
    let mut total = 0i64;
    for i in 0..n {
        trace += c[i * n + i];
        for j in 0..n {
            total += c[i * n + j];
        }
    }
    println!("trace: {}", trace);
    println!("sum: {}", total);
}
