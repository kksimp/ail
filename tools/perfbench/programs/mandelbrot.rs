// Mandelbrot as text: the same shades and pixel mapping as mandelbrot.ail.
use std::io::Write;

const MAX_ITER: i64 = 50;
const SHADES: &[u8] = b" .:-=+*#%@";

fn escape(cr: f64, ci: f64) -> i64 {
    let mut zr = 0.0f64;
    let mut zi = 0.0f64;
    let mut k = 0;
    while k < MAX_ITER {
        if zr * zr + zi * zi > 4.0 {
            break;
        }
        let t = zr * zr - zi * zi + cr;
        zi = 2.0 * zr * zi + ci;
        zr = t;
        k += 1;
    }
    k
}

fn main() {
    let rows: i64 = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(160);
    let cols = rows * 2;
    let stdout = std::io::stdout();
    let mut out = std::io::BufWriter::new(stdout.lock());
    let mut line = vec![0u8; cols as usize + 1];
    for y in 0..rows {
        let ci = -1.2 + 2.4 * y as f64 / rows as f64;
        for x in 0..cols {
            let cr = -2.0 + 3.0 * x as f64 / cols as f64;
            let k = escape(cr, ci);
            line[x as usize] = SHADES[(k * 9 / MAX_ITER) as usize];
        }
        line[cols as usize] = b'\n';
        out.write_all(&line).unwrap();
    }
}
