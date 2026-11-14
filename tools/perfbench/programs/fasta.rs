// FASTA-style DNA: the same LCG and weights as fasta.ail.
use std::io::Write;

const WIDTH: usize = 60;

fn main() {
    let n: i64 = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(100000);
    let letters = b"acgt";
    let mut seed: i64 = 42;
    let mut counts = [0i64; 4];
    let mut line = [0u8; WIDTH + 1];
    let mut used = 0;
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    for _ in 0..n {
        seed = (seed * 3877 + 29573) % 139968;
        let t = seed * 100 / 139968;
        let base = if t < 30 {
            0
        } else if t < 50 {
            1
        } else if t < 70 {
            2
        } else {
            3
        };
        counts[base] += 1;
        line[used] = letters[base];
        used += 1;
        if used == WIDTH {
            line[used] = b'\n';
            out.write_all(&line[..used + 1]).unwrap();
            used = 0;
        }
    }
    if used > 0 {
        line[used] = b'\n';
        out.write_all(&line[..used + 1]).unwrap();
    }
    writeln!(out, "a {} c {} g {} t {}", counts[0], counts[1], counts[2], counts[3]).unwrap();
}
