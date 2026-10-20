//! Seeded generator of pure-subset programs for the native differential
//! (fuzz.nat.1, ADR 0462): integers, floats, lists of integers, loops,
//! functions. Arithmetic may overflow or divide by zero on purpose: a trap
//! must read the same on every tier.

use super::progs::Rng;

struct G {
    r: Rng,
    ints: Vec<String>,
    muts: Vec<String>,
    floats: Vec<String>,
    fns: usize,
    next: usize,
    out: String,
    depth: usize,
}

impl G {
    fn fresh(&mut self, p: &str) -> String {
        self.next += 1;
        format!("{p}{}", self.next)
    }

    fn line(&mut self, s: &str) {
        for _ in 0..self.depth {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn int(&mut self, d: usize) -> String {
        let leaf = d == 0 || self.r.chance(2, 5);
        if leaf {
            if self.r.chance(1, 40) {
                return "9223372036854775807".to_string(); // overflow edge
            }
            return match self.r.below(4) {
                0 if !self.ints.is_empty() => self.r.pick(&self.ints.clone()).clone(),
                1 => self.r.pick(&["0", "1", "2", "7", "100", "255", "1000003"]).to_string(),
                2 if !self.ints.is_empty() => self.r.pick(&self.ints.clone()).clone(),
                _ => (self.r.below(60) as i64 - 10).to_string(),
            };
        }
        let a = self.int(d - 1);
        let b = self.int(d - 1);
        match self.r.below(9) {
            0 => format!("({a} + {b})"),
            1 => format!("({a} - {b})"),
            2 => format!("({a} * {})", self.r.pick(&["2", "3", "-1", "10"]).to_string()),
            3 => format!("({a} / {})", self.r.pick(&["2", "3", "7", "-4"]).to_string()),
            4 => format!("({a} % {})", self.r.pick(&["2", "5", "16", "97"]).to_string()),
            5 if self.r.chance(1, 20) => format!("({a} / {b})"), // may trap
            6 if self.fns > 0 => {
                let f = self.r.below(self.fns);
                format!("f{f}({a}, {b})")
            }
            7 => format!("(-{a})"),
            _ => format!("({a} + {b})"),
        }
    }

    fn float(&mut self, d: usize) -> String {
        if d == 0 || self.r.chance(2, 5) {
            return match self.r.below(3) {
                0 if !self.floats.is_empty() => self.r.pick(&self.floats.clone()).clone(),
                1 => format!("{}.{}", self.r.below(20), self.r.below(100)),
                _ => format!("{}.to_float()", self.int(1)),
            };
        }
        let a = self.float(d - 1);
        let b = self.float(d - 1);
        let op = *self.r.pick(&["+", "-", "*", "/"]);
        format!("({a} {op} {b})")
    }

    fn cond(&mut self) -> String {
        let a = self.int(2);
        let b = self.int(2);
        let op = *self.r.pick(&["<", "<=", ">", ">=", "==", "!="]);
        let c = format!("{a} {op} {b}");
        match self.r.below(5) {
            0 => {
                let d = self.int(1);
                format!("{c} and {d} > 0")
            }
            1 => {
                let d = self.int(1);
                format!("{c} or {d} < 0")
            }
            2 => format!("not ({c})"),
            _ => c,
        }
    }

    fn stmts(&mut self, n: usize, d: usize, list: bool) {
        for _ in 0..n {
            match self.r.below(if d == 0 { 5 } else { 9 }) {
                0 | 1 => {
                    let v = self.fresh("v");
                    let e = self.int(3);
                    self.line(&format!("mut {v} := {e}"));
                    self.ints.push(v.clone());
                    self.muts.push(v);
                }
                2 if !self.muts.is_empty() => {
                    let v = self.r.pick(&self.muts.clone()).clone();
                    let e = self.int(2);
                    let op = *self.r.pick(&["=", "+=", "-="]);
                    self.line(&format!("{v} {op} {e}"));
                }
                3 => {
                    let v = self.fresh("x");
                    let e = self.float(3);
                    self.line(&format!("mut {v} := {e}"));
                    self.floats.push(v);
                }
                4 => {
                    if self.r.chance(1, 3) && !self.floats.is_empty() {
                        let f = self.r.pick(&self.floats.clone()).clone();
                        self.line(&format!("print(\"{{{f}:.6}}\")"));
                    } else {
                        let e = self.int(3);
                        self.line(&format!("print({e})"));
                    }
                }
                5 => {
                    let c = self.cond();
                    self.line(&format!("if {c}"));
                    self.block(d);
                    if self.r.chance(1, 2) {
                        self.line("else");
                        self.block(d);
                    }
                }
                6 => {
                    let i = self.fresh("it");
                    let n = self.r.below(30);
                    self.line(&format!("for {i} in 0..{n}"));
                    self.depth += 1;
                    let (a, b, c) = (self.ints.len(), self.muts.len(), self.floats.len());
                    self.ints.push(i.clone());
                    let n = 1 + self.r.below(3);
                    self.stmts(n, d - 1, list);
                    self.line("print(0)");
                    self.ints.truncate(a);
                    self.muts.truncate(b);
                    self.floats.truncate(c);
                    self.depth -= 1;
                }
                7 => {
                    let w = self.fresh("w");
                    let n = 1 + self.r.below(40);
                    self.line(&format!("mut {w} := 0"));
                    self.line(&format!("while {w} < {n}"));
                    self.depth += 1;
                    let (a, b, c) = (self.ints.len(), self.muts.len(), self.floats.len());
                    self.ints.push(w.clone());
                    let n = 1 + self.r.below(2);
                    self.stmts(n, d - 1, list);
                    self.line(&format!("{w} += 1"));
                    self.line("print(0)");
                    self.ints.truncate(a);
                    self.muts.truncate(b);
                    self.floats.truncate(c);
                    self.depth -= 1;
                }
                _ if list => {
                    let e = self.int(2);
                    self.line(&format!("xs.push({e})"));
                    self.line("if xs.len() > 0");
                    let k = self.int(1);
                    self.line(&format!("    print(xs[({k}).abs() % xs.len()])"));
                }
                _ => {}
            }
        }
    }

    fn block(&mut self, d: usize) {
        self.depth += 1;
        let (ints, muts, floats) = (self.ints.len(), self.muts.len(), self.floats.len());
        let n = 1 + self.r.below(2);
                    self.stmts(n, d.saturating_sub(1), false);
        self.ints.truncate(ints);
        self.muts.truncate(muts);
        self.floats.truncate(floats);
        self.line("print(0)");
        self.depth -= 1;
    }
}

pub fn program(seed: u64) -> String {
    let mut g = G { r: Rng::new(seed ^ 0xA7A7), ints: vec![], muts: vec![], floats: vec![], fns: 0, next: 0, out: String::new(), depth: 0 };
    for k in 0..g.r.below(3) {
        g.out.push_str(&format!("fn f{k}(a int, b int) int\n"));
        g.depth = 1;
        g.ints = vec!["a".into(), "b".into()];
        g.muts.clear();
        g.floats.clear();
        let n = 1 + g.r.below(3);
        g.stmts(n, 1, false);
        let e = g.int(3);
        g.line(&e);
        g.depth = 0;
        g.out.push('\n');
        g.fns += 1;
    }
    g.out.push_str("fn main()\n");
    g.depth = 1;
    g.ints.clear();
    g.muts.clear();
    g.floats.clear();
    g.line("mut xs := [1, 2, 3]");
    let n = 4 + g.r.below(8);
    g.stmts(n, 2, true);
    g.line("print(xs.len())");
    g.out
}

// ---- strings and chars (native.str.1, ADR 0565-0567) ----

struct S {
    r: Rng,
    strs: Vec<String>,
    out: String,
}

const LITS: [&str; 12] = ["", "a", "Hello", "  pad ", "x,y,,z", "é-ü", "ABC abc", "日本", "a\\tb", "{", "tail\\n", "0123456789"];

impl S {
    fn lit(&mut self) -> String {
        let l = self.r.pick(&LITS);
        // `{` must be escaped inside a literal (lex.str.1).
        format!("\"{}\"", l.replace('{', "\\{"))
    }

    fn str_(&mut self, d: usize) -> String {
        if d == 0 || self.r.chance(2, 5) {
            if !self.strs.is_empty() && self.r.chance(1, 2) {
                return self.r.pick(&self.strs.clone()).clone();
            }
            return self.lit();
        }
        let a = self.str_(d - 1);
        match self.r.below(12) {
            0 => format!("{a}.trim()"),
            1 => format!("{a}.to_upper()"),
            2 => format!("{a}.to_lower()"),
            3 => format!("{a}.repeat({})", self.r.below(4)),
            4 => format!("{a}.pad_left({}, '{}')", self.r.below(12), self.r.pick(&['*', ' ', '0'])),
            5 => format!("{a}.pad_right({}, '{}')", self.r.below(12), self.r.pick(&['-', ' ', '.'])),
            6 => {
                let b = self.str_(d - 1);
                format!("{a}.replace({b}, {})", self.lit())
            }
            7 => {
                let b = self.str_(d - 1);
                format!("\"{{{a}}}{{{b}}}\"")
            }
            8 => format!("{a}.slice({}, {})", self.r.below(3), 3 + self.r.below(4)), // may trap
            9 => format!("\"[{{{a}:>{}}}]\"", self.r.below(10)),
            10 => format!("\"[{{{a}:*^{}}}]\"", self.r.below(12)),
            _ => format!("{a}.to_str()"),
        }
    }

    fn int(&mut self, d: usize) -> String {
        let a = self.str_(d);
        match self.r.below(4) {
            0 => format!("{a}.len()"),
            1 => format!("{a}.byte_len()"),
            2 => format!("{a}.chars().len()"),
            _ => format!("{a}.split(\",\").len()"),
        }
    }

    fn boolean(&mut self, d: usize) -> String {
        let a = self.str_(d);
        let b = self.str_(d);
        match self.r.below(7) {
            0 => format!("{a}.contains({b})"),
            1 => format!("{a}.starts_with({b})"),
            2 => format!("{a}.ends_with({b})"),
            3 => format!("{a} == {b}"),
            4 => format!("{a} < {b}"),
            5 => format!("{a} >= {b}"),
            _ => format!("{a} != {b}"),
        }
    }
}

pub fn strings(seed: u64) -> String {
    let mut s = S { r: Rng::new(seed ^ 0x57A1), strs: vec![], out: String::new() };
    s.out.push_str("fn main()\n");
    for i in 0..4 + s.r.below(8) {
        match s.r.below(6) {
            0 | 1 => {
                let e = s.str_(3);
                s.out.push_str(&format!("    s{i} := {e}\n    print(s{i})\n"));
                s.strs.push(format!("s{i}"));
            }
            2 => {
                let e = s.int(2);
                s.out.push_str(&format!("    print({e})\n"));
            }
            3 => {
                let e = s.boolean(2);
                s.out.push_str(&format!("    print({e})\n"));
            }
            4 => {
                let a = s.str_(2);
                let m = s.r.pick(&["code", "is_digit", "is_alpha", "is_alnum", "is_space", "is_upper", "is_lower"]).to_string();
                s.out.push_str(&format!("    for c in {a}.chars()\n        print(c.{m}())\n"));
            }
            _ => {
                let a = s.str_(2);
                let m = s.r.pick(&["to_upper", "to_lower"]).to_string();
                s.out.push_str(&format!("    for c in {a}.chars()\n        print(c.{m}())\n"));
            }
        }
    }
    s.out
}
