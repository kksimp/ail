// Word frequencies: the same generator and vocabulary as wordfreq.ail, counted in the
// standard library's HashMap from word to count.
use std::collections::HashMap;

const WORDS: [&str; 12] = ["the", "of", "and", "to", "in", "fox", "jumps", "lazy", "dog", "river", "stone", "moss"];

fn main() {
    let n: i64 = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(200000);
    let mut counts: HashMap<&str, i64> = HashMap::new();
    let mut seed: i64 = 7;
    for _ in 0..n {
        seed = (seed * 3877 + 29573) % 139968;
        let w = WORDS[(seed % WORDS.len() as i64) as usize];
        let c = counts.get(w).copied().unwrap_or(0);
        counts.insert(w, c + 1);
    }
    for w in WORDS.iter() {
        println!("{} {}", w, counts.get(w).copied().unwrap_or(0));
    }
    println!("distinct {}", counts.len());
}
