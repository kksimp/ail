// Collatz: total steps and the longest chain for starts 1 up to N (exclusive).
fn steps(mut n: i64) -> i64 {
    let mut k = 0;
    while n != 1 {
        if n % 2 == 0 {
            n /= 2;
        } else {
            n = 3 * n + 1;
        }
        k += 1;
    }
    k
}

fn main() {
    let limit: i64 = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(30000);
    let mut total = 0i64;
    let mut best = 1i64;
    let mut best_steps = 0i64;
    for i in 1..limit {
        let s = steps(i);
        total += s;
        if s > best_steps {
            best = i;
            best_steps = s;
        }
    }
    println!("total steps: {}", total);
    println!("longest: {} with {} steps", best, best_steps);
}
