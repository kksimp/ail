use std::collections::HashMap;
use std::env;

fn main() {
    let n: usize = env::args().nth(1).and_then(|a| a.parse().ok()).unwrap_or(1000000);
    let mut seed: i64 = 5;
    let mut a: HashMap<i64, i64> = HashMap::new();
    let mut b: HashMap<String, i64> = HashMap::new();
    for _ in 0..n {
        seed = (seed * 3877 + 29573) % 139968;
        *a.entry(seed % 50021).or_insert(0) += 1;
        *b.entry(format!("key{}", seed % 20011)).or_insert(0) += 1;
    }
    let sq: i64 = a.iter().map(|(k, v)| v * v + k % 7).sum();
    let sq2: i64 = b.iter().map(|(k, v)| v * v + k.len() as i64).sum();
    println!("int keys {}, check {}", a.len(), sq);
    println!("str keys {}, check {}", b.len(), sq2);
}
