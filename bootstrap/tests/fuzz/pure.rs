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

// ---- maps and sets (native.map, ADR 0568-0570) ----

pub fn maps(seed: u64) -> String {
    let mut r = Rng::new(seed ^ 0x3A95);
    let mut o = String::from("fn main()\n");
    o.push_str("    mut mi := Map[int, int]()\n    mut ms := Map[str, int]()\n    mut mf := Map[str, float]()\n    mut si := Set[int]()\n    mut ss := Set[str]()\n");
    let words = ["a", "b", "tea", "milk", "é", "", "x y", "zz"];
    for _ in 0..8 + r.below(14) {
        let k = r.below(12) as i64 - 2;
        let w = *r.pick(&words);
        let v = r.below(100) as i64 - 20;
        let line = match r.below(22) {
            0 | 1 => format!("mi[{k}] = {v}"),
            2 => format!("ms[\"{w}\"] = {v}"),
            3 => format!("mi[{k}] = (mi.get({k}) else 0) + {v}"),
            4 => format!("ms[\"{w}\"] = (ms.get(\"{w}\") else 0) + {v}"),
            5 => format!("mf[\"{w}\"] = {v}.5"),
            6 => format!("mf[\"{w}\"] += 0.25"),
            7 => format!("print(mi.get({k}) else -1)"),
            8 => format!("print(ms.get(\"{w}\") else -1)"),
            9 => format!("print(mi.contains({k}))"),
            10 => format!("print(ms.contains(\"{w}\"))"),
            11 => format!("mi.remove({k})"),
            12 => format!("ms.remove(\"{w}\")"),
            13 => format!("print(si.add({k}))"),
            14 => format!("print(ss.add(\"{w}\"))"),
            15 => format!("print(si.contains({k}))"),
            16 => format!("print(si.remove({k}))"),
            17 => "print(mi.len() + ms.len() + si.len() + ss.len())".to_string(),
            18 => format!("print([{}, {}, {}].len())", k, v, k + v),
            19 => "for x in si\n        print(x * 2)".to_string(),
            20 => "print(mi.keys())\n    print(ms.values())".to_string(),
            _ if r.chance(1, 8) => format!("print(mi[{k}])"), // traps when missing
            _ => format!("mi[{k}] = {v}"),
        };
        o.push_str("    ");
        o.push_str(&line);
        o.push('\n');
    }
    o.push_str("    print(mi)\n    print(ms)\n    print(mf)\n    print(si)\n    print(ss)\n");
    // Growth and rehash: many keys, deletions, then a checksum over the table.
    let (n, m, step) = (50 + r.below(3000), 7 + r.below(2000), 1 + r.below(13));
    o.push_str(&format!("    mut big := Map[int, int]()\n    for i in 0..{n}\n        big[i * {step} % {m}] = i\n    for i in 0..{n} / 3\n        big.remove(i * {step} % {m})\n    mut sum := 0\n    for k in big.keys()\n        sum += k * 31 + (big.get(k) else 0)\n    print(big.len())\n    print(sum)\n"));
    o.push_str("    mut total := 0\n    for v in mi.values()\n        total += v\n    print(total)\n");
    o.push_str("    mut words := Map[str, int]()\n    for w in [\"the\", \"cat\", \"the\", \"hat\", \"the\", \"end\"]\n        words[w] += 1\n    print(words)\n");
    o
}

// ---- optionals and results (native.opt, ADR 0574-0576) ----

pub fn options(seed: u64) -> String {
    let mut r = Rng::new(seed ^ 0x0971);
    let (k, m, t) = (2 + r.below(5), 1 + r.below(9), 20 + r.below(200));
    let mut o = format!(
        "error struct Empty \"the list is empty\"\nerror struct Big \"the number is too big\"\n\n\
fn find(xs List[int], want int) int?\n    for i in 0..xs.len()\n        if xs[i] == want\n            return i\n    none\n\n\
fn pick(n int) int?\n    if n % {k} == 0\n        return none\n    n * {m}\n\n\
fn halve(x float) float?\n    if x < 0.0\n        return none\n    x / 2.0\n\n\
fn first(xs List[int]) int ! Empty\n    if xs.len() == 0\n        return Empty()\n    xs[0]\n\n\
fn small(n int) int ! Big\n    if n > {t}\n        return Big()\n    n\n\n\
fn both(a List[int], b List[int]) int ! Empty | Big\n    x := small(first(a)?)?\n    y := small(first(b)?)?\n    x + y\n\n\
fn check(n int) ! Big\n    if n > {t}\n        return Big()\n\n\
fn lookup(xs List[int], i int) int?\n    if i < 0 or i >= xs.len()\n        return none\n    xs[i]\n\n\
fn twice(xs List[int], i int) int ! Empty\n    v := first(xs)?\n    w := lookup(xs, i) else return Empty()\n    v + w\n\n"
    );
    o.push_str("fn main()\n");
    let list = |r: &mut Rng| -> String {
        let n = r.below(4);
        format!("[{}]", (0..n).map(|_| (r.below(300) as i64 - 20).to_string()).collect::<Vec<_>>().join(", "))
    };
    let xs = list(&mut r);
    o.push_str(&format!("    xs := {xs}\n"));
    for u in 0..6 + r.below(10) {
        let n = r.below(40) as i64 - 5;
        let l = list(&mut r);
        let l2 = list(&mut r);
        let b = match r.below(14) {
            0 => format!("print(find(xs, {n}) else -1)"),
            1 => format!("print(pick({n}) else -7)"),
            2 => format!("if v := pick({n})\n        print(v + 1)\n    else\n        print(\"none\")"),
            3 => format!("print(halve({n}.5) else 0.0)"),
            4 => format!("w{u} := small({n} * 11) else err\n        print(\"failed: {{err}}\")\n        0\n    print(w{u})"),
            5 => format!("print(both({l}, {l2}) else -9)"),
            6 => format!("print(twice({l}, {n}) else -3)"),
            7 => format!("check({n} * 9) else err\n        print(\"check: {{err}}\")"),
            8 => format!("mut o{u} := pick({n})\n    print(o{u} else 5)\n    o{u} = none\n    print(o{u} else 100)\n    o{u} = {n}\n    print(o{u} else 100)"),
            9 => format!("if pick({n}) == none\n        print(\"missing\")"),
            10 => format!("if pick({n}) != none\n        print(\"found\")"),
            11 => format!("mut count{u} := 0\n    for i in 0..{}\n        v := small(i * {}) else continue\n        count{u} += v\n    print(count{u})", 3 + r.below(10), 1 + r.below(60)),
            12 => format!("if p := find(xs, {n})\n        print(\"p {{p}}\")\n    elif q := lookup(xs, {n})\n        print(\"q {{q}}\")\n    else\n        print(\"neither\")"),
            _ => format!("m{u} := first({l}) else\n        print(\"fallback block\")\n        7\n    print(m{u})"),
        };
        o.push_str("    ");
        o.push_str(&b);
        o.push('\n');
    }
    o.push_str("    print(\"done\")\n");
    o
}

// ---- function values and key sorts (native.fn, ADR 0571-0573) ----

pub fn funcs(seed: u64) -> String {
    let mut r = Rng::new(seed ^ 0xF00C);
    let mut o = String::new();
    let nf = 2 + r.below(3);
    for i in 0..nf {
        let (a, b) = (1 + r.below(9), r.below(20));
        let body = match r.below(5) {
            0 => format!("n * {a} + {b}"),
            1 => format!("n % {} - {b}", 2 + a),
            2 => format!("(n - {b}) * (n - {b})"),
            3 => format!("if n > {b} then n - {a} else n + {a}"),
            _ => format!("n / {} + {a}", 1 + a),
        };
        o.push_str(&format!("fn f{i}(n int) int\n    {body}\n\n"));
    }
    o.push_str("fn apply(f fn(int) int, x int) int\n    f(x)\n\nfn twice(f fn(int) int, x int) int\n    f(f(x))\n\n");
    o.push_str(&format!("fn pick(which int) fn(int) int\n    if which % 2 == 0\n        f0\n    else\n        f{}\n\n", nf - 1));
    o.push_str("fn each(f fn(int), n int)\n    for i in 0..n\n        f(i)\n\nfn show(n int)\n    print(\"show {n}\")\n\nfn mix(a int, b float, c int) float\n    a.to_float() * b + c.to_float()\n\nfn many(f fn(int, float, int) float, a int) float\n    f(a, 1.5, a + 1)\n\n");
    o.push_str("fn main()\n");
    let list = |r: &mut Rng, lo: i64| -> String {
        let n = 1 + r.below(8);
        format!("[{}]", (0..n).map(|_| (r.below(40) as i64 + lo).to_string()).collect::<Vec<_>>().join(", "))
    };
    for u in 0..5 + r.below(8) {
        let n = r.below(50) as i64 - 10;
        let fi = r.below(nf);
        let l = list(&mut r, -10);
        let sh = r.below(10);
        let b = match r.below(16) {
            0 => format!("print(apply(f{fi}, {n}))"),
            1 => format!("print(twice(f{fi}, {n}))"),
            2 => format!("print(pick({n})({n}))"),
            3 => format!("g{u} := f{fi}\n    print(g{u}({n}))"),
            4 => format!("each(show, {})", r.below(4)),
            5 => format!("print(many(mix, {n}))"),
            6 => format!("mut total{u} := 0\n    for k in 0..{}\n        h := if k % 2 == 0 then f0 else f{fi}\n        total{u} += h(k)\n    print(total{u})", 1 + r.below(8)),
            7 => format!("print({l}.sorted())"),
            8 => format!("print({l}.sorted(by: |x| -x))"),
            9 => format!("shift{u} := {sh}\n    print({l}.sorted(by: |x| (x - shift{u}) * (x - shift{u})))"),
            10 => format!("print({l}.sorted(by: |x| x % {}))", 2 + r.below(5)), // ties: stability
            11 => format!("mut ys{u} := {l}\n    ys{u}.sort()\n    print(ys{u})\n    ys{u}.sort(by: |y| -y)\n    print(ys{u})"),
            12 => format!("print([2.5, -1.0, {n}.25, 9.0].sorted(by: |f| f.to_int() * -1))"),
            13 => "print([\"pear\", \"fig\", \"banana\", \"kiwi\", \"apple\", \"date\"].sorted(by: |w| w.len()).join(\",\"))".to_string(),
            14 => {
                let (l, key) = *r.pick(&[("[\"b\", \"a\", \"é\", \"\", \"ab\"]", "v.len()"), ("[3.5, -2.0, 0.0, 1.25]", "v.to_int()")]);
                format!("print({l}.sorted(by: |v| {key}))")
            }
            _ => format!("print(List[int]().sorted(by: |x| x))"),
        };
        o.push_str("    ");
        o.push_str(&b);
        o.push('\n');
    }
    o
}
