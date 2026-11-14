// Rust twin of vm.ail.
struct Code {
    code: Vec<i64>,
}

impl Code {
    fn emit(&mut self, op: i64, arg: i64) {
        self.code.push(op);
        self.code.push(arg);
    }

    fn here(&self) -> usize {
        self.code.len() / 2
    }
}

fn build(limit: i64) -> Vec<i64> {
    let mut c = Code { code: Vec::with_capacity(256) };
    c.emit(0, 2); c.emit(2, 0); c.emit(0, 0); c.emit(2, 1);
    let top = c.here();
    c.emit(1, 0); c.emit(0, limit); c.emit(6, 0);
    let jz_end = c.here(); c.emit(8, 0);
    c.emit(0, 2); c.emit(2, 2);
    let inner = c.here();
    c.emit(1, 2); c.emit(1, 2); c.emit(4, 0); c.emit(1, 0); c.emit(0, 1); c.emit(3, 0); c.emit(6, 0);
    let jz_prime = c.here(); c.emit(8, 0);
    c.emit(1, 0); c.emit(1, 2); c.emit(5, 0);
    let jz_next = c.here(); c.emit(8, 0);
    c.emit(1, 2); c.emit(0, 1); c.emit(3, 0); c.emit(2, 2); c.emit(7, inner as i64);
    let prime = c.here();
    c.emit(1, 1); c.emit(0, 1); c.emit(3, 0); c.emit(2, 1);
    let next = c.here();
    c.emit(1, 0); c.emit(0, 1); c.emit(3, 0); c.emit(2, 0); c.emit(7, top as i64);
    let end = c.here(); c.emit(9, 0);
    c.code[jz_end * 2 + 1] = end as i64;
    c.code[jz_prime * 2 + 1] = prime as i64;
    c.code[jz_next * 2 + 1] = next as i64;
    c.code
}

fn run(code: &[i64], vars: &mut [i64; 3]) -> i64 {
    let mut st = [0i64; 64];
    let mut sp = 0usize;
    let mut pc = 0usize;
    let mut steps = 0i64;
    loop {
        let op = code[pc * 2];
        let arg = code[pc * 2 + 1];
        pc += 1;
        steps += 1;
        match op {
            0 => { st[sp] = arg; sp += 1; }
            1 => { st[sp] = vars[arg as usize]; sp += 1; }
            2 => { sp -= 1; vars[arg as usize] = st[sp]; }
            3 => { sp -= 1; st[sp - 1] = st[sp - 1] + st[sp]; }
            4 => { sp -= 1; st[sp - 1] = st[sp - 1] * st[sp]; }
            5 => { sp -= 1; st[sp - 1] = st[sp - 1] % st[sp]; }
            6 => { sp -= 1; st[sp - 1] = if st[sp - 1] < st[sp] { 1 } else { 0 }; }
            7 => { pc = arg as usize; }
            8 => { sp -= 1; if st[sp] == 0 { pc = arg as usize; } }
            9 => return steps,
            _ => std::process::abort(),
        }
    }
}

fn main() {
    let limit: i64 = std::env::args().nth(1).map(|a| a.parse().unwrap_or(0)).unwrap_or(200000);
    let code = build(limit);
    let mut vars = [0i64; 3];
    let steps = run(&code, &mut vars);
    println!("primes {}\nsteps {}", vars[1], steps);
}
