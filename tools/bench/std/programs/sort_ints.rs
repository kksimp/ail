use std::env;

fn main() {
    let n: usize = env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1000000);
    let mut seed: i64 = 42;
    let mut xs: Vec<i64> = Vec::with_capacity(n);
    for _ in 0..n {
        seed = (seed * 3877 + 29573) % 139968;
        let k = xs.len() as i64 % 13;
        xs.push(seed * 7 + k);
    }
    xs.sort();
    let mut chk: i64 = 0;
    for x in &xs {
        chk = (chk * 31 + x) % 1000000007;
    }
    let mut ss: Vec<String> = Vec::with_capacity(n / 4);
    for j in 0..n / 4 {
        seed = (seed * 3877 + 29573) % 139968;
        ss.push(format!("k{}-{}", seed, j % 1000));
    }
    ss.sort();
    println!("ints {}, check {}, first {}, last {}", xs.len(), chk, xs[0], xs[n - 1]);
    println!("strings {}, first {}, last {}", ss.len(), ss[0], ss[ss.len() - 1]);
}
