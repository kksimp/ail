//! Programs that favour what the bytecode tier compiles (fuzz.bc.*, ADR 0756):
//! counted and `while` loops with `break` and `continue`, int arithmetic at the
//! overflow edges, lists indexed in and out of range, `mut self` methods called
//! through `xs[i].m()`, copies of struct values, early `return` inside loops,
//! float arithmetic, and (outside `pure` programs) the hand-off constructs
//! (strings, maps, `else`, `print`) in the middle of a loop. Every program is a
//! pure function of its seed and always terminates: loops count up to a
//! small constant.

use super::progs::Rng;

const EDGES: [&str; 12] = [
    "0", "1", "2", "-1", "7", "255", "1000", "3037000500", "4611686018427387904", "9223372036854775807", "-9223372036854775807", "-4611686018427387904",
];
const FLOATS: [&str; 8] = ["0.0", "0.5", "1.5", "-2.25", "100.0", "1e10", "1e300", "-0.0"];

struct G<'a> {
    r: &'a mut Rng,
    /// No hand-off constructs: the tier should run the whole program itself.
    pure: bool,
    /// Where a function may `return`: it has the int result `t`.
    in_work: bool,
    /// Inside `Acc.step`: no `xs` and no `fl`.
    flat: bool,
    /// Heavy use of the overflow edges; otherwise values stay small and most runs finish.
    edgy: bool,
    /// The int the statements accumulate into.
    tgt: &'static str,
    /// How often an index is left unguarded, so it can trap.
    wild: usize,
    ints: Vec<String>,
    loops: usize,
    next_id: usize,
}

impl G<'_> {
    fn id(&mut self) -> usize {
        self.next_id += 1;
        self.next_id
    }

    fn atom(&mut self) -> String {
        if self.edgy && self.r.chance(1, 3) {
            return self.r.pick(&EDGES).to_string();
        }
        if !self.edgy && self.r.chance(1, 4) {
            return self.r.pick(&EDGES[..7]).to_string();
        }
        if !self.flat && self.r.chance(1, 8) {
            return format!("xs[{}].a", self.index());
        }
        self.r.pick(&self.ints.clone()).clone()
    }

    fn expr(&mut self, depth: usize) -> String {
        if depth == 0 || self.r.chance(1, 4) {
            return self.atom();
        }
        let (a, b) = (self.expr(depth - 1), self.expr(depth - 1));
        match self.r.below(8) {
            0 | 1 => format!("({a} + {b})"),
            2 => format!("({a} - {b})"),
            3 if !self.edgy => format!("({a} % 1000 * ({b} % 1000))"),
            3 => format!("({a} * {b})"),
            4 => format!("({a} / {})", if !self.edgy || self.r.chance(7, 8) { format!("({b} % 7 + 8)") } else { b }),
            5 => format!("({a} % {})", if !self.edgy || self.r.chance(7, 8) { "5".to_string() } else { b }),
            6 => format!("({a} % 1000 * {})", self.r.pick(&["3", "17", "1000"])),
            _ => format!("(if {} then {a} else {b})", self.cond(1)),
        }
    }

    fn cond(&mut self, depth: usize) -> String {
        if depth > 0 && self.r.chance(1, 4) {
            let (a, b) = (self.cond(depth - 1), self.cond(depth - 1));
            return if self.r.chance(1, 2) { format!("({a} and {b})") } else { format!("({a} or {b})") };
        }
        let (a, b) = (self.expr(1), self.expr(1));
        format!("{a} {} {b}", self.r.pick(&["<", "<=", ">", ">=", "==", "!="]))
    }

    /// An index into `xs`: in range, or (rarely) not.
    fn index(&mut self) -> String {
        if self.r.chance(self.wild, 100) {
            return self.r.pick(&["(0 - 1)", "3", "7", "i0"]).to_string().replace("i0", "(acc % 5)");
        }
        let v = self.ints.iter().find(|n| n.starts_with('i')).cloned();
        match v {
            Some(i) if self.r.chance(2, 3) => format!("{i} % xs.len()"),
            _ => format!("{} % 3", self.r.below(3)),
        }
    }

    fn block(&mut self, depth: usize, ind: usize, out: &mut String, lines: usize) {
        for _ in 0..lines {
            self.stmt(depth, ind, out);
        }
    }

    fn push(out: &mut String, ind: usize, line: &str) {
        out.push_str(&"    ".repeat(ind));
        out.push_str(line);
        out.push('\n');
    }

    /// Calls that return a value through `mut self`, and a struct holding a list.
    fn extra(&mut self, ind: usize, out: &mut String) {
        let tgt = self.target();
        match self.r.below(6) {
            0 => {
                let (i, e) = (self.index(), self.expr(1));
                
                Self::push(out, ind, &format!("{tgt} += xs[{i}].bump({e})"));
            }
            1 => {
                let e = self.expr(1);
                Self::push(out, ind, &format!("bag.add({e})"));
            }
            2 => Self::push(out, ind, &format!("{tgt} += bag.sum() + bag.n")),
            3 => {
                let n = self.id();
                Self::push(out, ind, &format!("mut b{n} := bag"));
                Self::push(out, ind, &format!("b{n}.add({})", self.r.below(50)));
                Self::push(out, ind, &format!("{tgt} += b{n}.n - bag.n"));
            }
            5 => {
                // Comprehensions (expr.comp.1-3): lists, sets and maps, with filters, `if` elements and two clauses.
                let (n, m) = (self.id(), self.id());
                let (q, q2) = (format!("q{n}"), format!("q{m}"));
                let (lo, hi) = (self.r.below(3), self.r.below(7));
                let kind = self.r.below(9);
                self.ints.push(q.clone());
                if kind == 3 || kind >= 6 {
                    self.ints.push(q2.clone());
                }
                let (e, c, e2) = (self.expr(2), self.cond(1), self.expr(1));
                let line = match kind {
                    0 => format!("print([{e} for {q} in {lo}..{hi}])"),
                    1 => format!("{tgt} += [{e} for {q} in {lo}..{hi} if {c}].len()"),
                    2 => format!("print([if {c} then {e} else {e2} for {q} in {lo}..={hi}])"),
                    3 => format!("print([{e} for {q} in 0..{hi} for {q2} in 0..{q} % 4 if {c}])"),
                    4 => format!("print({{{e} % 7 for {q} in {lo}..{hi}}}.len())"),
                    5 => format!("print({{{q}: {e} for {q} in {lo}..{hi}}}.len())"),
                    // type.tuple.5: `for a, b in xs` clauses, alone, after a range clause, and in a map.
                    6 => format!("print([{e} for {q}, {q2} in [(1, {lo}), (2, {hi}), (3, 4)] if {c}])"),
                    7 => format!("print([{e} for {q2} in 0..{hi} for {q}, _ in [({q2}, 1), (2, 3)]])"),
                    _ => format!("print({{{q}: {q2} + {e} % 5 for {q}, {q2} in [(1, {lo}), (2, {hi})]}}.len())"),
                };
                if kind == 3 || kind >= 6 {
                    self.ints.pop();
                }
                self.ints.pop();
                Self::push(out, ind, &line);
            }
            _ => {
                let i = if self.wild > 0 && self.r.chance(1, 2) { "bag.n".to_string() } else { "0".to_string() };
                Self::push(out, ind, &format!("if bag.n > 0\n{}{tgt} += bag.items[{i}]", "    ".repeat(ind + 1)).replace("\n", &format!("\n")));
            }
        }
    }

    fn target(&self) -> &'static str {
        self.tgt
    }

    fn stmt(&mut self, depth: usize, ind: usize, out: &mut String) {
        let tgt = self.target();
        let kinds = if depth == 0 { 9 } else { 17 };
        if !self.flat && self.r.chance(1, 7) {
            self.extra(ind, out);
            return;
        }
        let mut pick = self.r.below(kinds);
        if self.flat && (3..=8).contains(&pick) {
            pick = 0;
        }
        match pick {
            0 | 1 => {
                let e = self.expr(2);
                Self::push(out, ind, &format!("{tgt} += {e}"));
            }
            2 => {
                let e = self.expr(2);
                Self::push(out, ind, &format!("{tgt} = {e}"));
            }
            3 => {
                // Known (found/bytecode/elem_call_arg_before_bounds.ail): an argument that traps beside
                // an index that traps is evaluated in the other order on the tier. Keep them apart.
                let (i, e) = (self.index(), self.expr(1));
                
                Self::push(out, ind, &format!("xs[{i}].step({e})"));
            }
            4 => {
                let i = self.index();
                Self::push(out, ind, &format!("{tgt} += xs[{i}].total()"));
            }
            5 => {
                // A copy of a struct value is independent of the list element.
                let (n, i, e) = (self.id(), self.index(), self.expr(1));
                Self::push(out, ind, &format!("mut c{n} := xs[{i}]"));
                Self::push(out, ind, &format!("c{n}.step({e})"));
                Self::push(out, ind, &format!("{tgt} += c{n}.a - xs[0].a"));
            }
            6 => {
                let e = self.expr(1);
                let f = if self.edgy { self.r.pick(&FLOATS) } else { self.r.pick(&FLOATS[..5]) };
                Self::push(out, ind, &format!("fl += ({e} % 100).to_float() * {f}"));
            }
            7 => Self::push(out, ind, &format!("{tgt} += fl.to_int() % 1000")),
            8 => {
                if self.pure {
                    let e = self.expr(1);
                    Self::push(out, ind, &format!("{tgt} -= {e}"));
                } else {
                    match self.r.below(5) {
                        0 => Self::push(out, ind, &format!("print({tgt})")),
                        1 => Self::push(out, ind, &format!("{tgt} += \"{}\".to_int() else 3", self.r.pick(&["12", "x", "-4", ""]))),
                        2 => {
                            let n = self.id();
                            Self::push(out, ind, &format!("s{n} := \"n{{{tgt}}}\""));
                            Self::push(out, ind, &format!("{tgt} += s{n}.len()"));
                        }
                        3 => {
                            let n = self.id();
                            Self::push(out, ind, &format!("m[\"k{{{tgt} % 4}}\"] = {tgt}"));
                            Self::push(out, ind, &format!("{tgt} += m.len() * {n}"));
                        }
                        _ => Self::push(out, ind, &format!("{tgt} += xs[0].a.to_str().len()")),
                    }
                }
            }
            9 | 10 => {
                // counted loop
                let n = self.id();
                let v = format!("i{}", self.loops);
                self.loops += 1;
                Self::push(out, ind, &format!("for {v} in 0..{}", self.r.below(7) + 1));
                self.ints.push(v);
                let k = self.r.below(3) + 1;
                self.block(depth - 1, ind + 1, out, k);
                self.ints.pop();
                self.loops -= 1;
                let _ = n;
            }
            11 => {
                // while loop with a counter bumped first, so `continue` is safe
                let n = self.id();
                Self::push(out, ind, &format!("mut w{n} := 0"));
                Self::push(out, ind, &format!("while w{n} < {}", self.r.below(6) + 1));
                Self::push(out, ind + 1, &format!("w{n} += 1"));
                self.ints.push(format!("w{n}"));
                let k = self.r.below(3) + 1;
                self.block(depth - 1, ind + 1, out, k);
                self.ints.pop();
            }
            12 | 13 => {
                let c = self.cond(1);
                Self::push(out, ind, &format!("if {c}"));
                let k = self.r.below(2) + 1;
                self.block(depth - 1, ind + 1, out, k);
                if self.r.chance(1, 2) {
                    Self::push(out, ind, "else");
                    self.block(depth - 1, ind + 1, out, 1);
                }
            }
            14 if self.loops > 0 => {
                let c = self.cond(1);
                Self::push(out, ind, &format!("if {c}"));
                Self::push(out, ind + 1, if self.r.chance(1, 2) { "break" } else { "continue" });
            }
            15 if self.in_work && self.loops > 0 => {
                let (c, e) = (self.cond(1), self.expr(1));
                Self::push(out, ind, &format!("if {c}"));
                Self::push(out, ind + 1, &format!("return {e}"));
            }
            _ => {
                let e = self.expr(2);
                Self::push(out, ind, &format!("{tgt} += {e}"));
            }
        }
    }
}

/// The program for `seed`. `pure` programs hold no hand-off construct.
pub fn program(seed: u64, pure: bool) -> String {
    let mut r = Rng::new(seed ^ 0xB17E_C0DE);
    let edgy = r.chance(1, 4);
    let wild = if edgy && r.chance(1, 2) { 5 } else { 0 };
    let mut out = String::from("struct Acc\n    a int\n    b int\n    f float\n\n");
    // The `mut self` method.
    out.push_str("fn Acc.step(mut self, d int)\n");
    {
        let mut g = G { r: &mut r, edgy, pure: true, in_work: false, flat: true, tgt: "t", wild: 0, ints: vec!["self.a".into(), "self.b".into(), "d".into()], loops: 0, next_id: 0 };
        out.push_str("    mut t := 0\n");
        let mut body = String::new();
        let k = g.r.below(3) + 1;
        g.block(2, 1, &mut body, k);
        let (e1, e2) = (g.expr(2), g.expr(2));
        out.push_str(&body);
        out.push_str(&format!("    self.a += {e1}\n    self.b = {e2}\n    self.f += 0.25\n"));
    }
    out.push_str("\nfn Acc.total(self) int\n    self.a % 1000 + self.b % 1000\n\n");
    out.push_str("fn Acc.bump(mut self, d int) int\n    self.a += d % 100\n    self.b -= 1\n    self.a + self.b\n\n");
    out.push_str("struct Bag\n    items List[int]\n    n int\n\nfn Bag.add(mut self, v int)\n    self.items.push(v % 1000)\n    self.n += 1\n\nfn Bag.sum(self) int\n    mut s := 0\n    for x in self.items\n        s += x\n    s\n\n");
    // A helper with loops and early returns.
    out.push_str("fn work(n int, k int) int\n    mut t := 0\n    mut fl := 0.5\n    mut xs := [Acc(a: n, b: k, f: 1.0), Acc(a: 2, b: 3, f: 0.0), Acc(a: -5, b: 8, f: 2.5)]\n    mut acc := 0\n    mut bag := Bag(items: [], n: 0)\n");
    if !pure {
        out.push_str("    mut m := Map[str, int]()\n");
    }
    {
        let mut g = G { r: &mut r, edgy, pure, in_work: true, flat: false, tgt: "t", wild, ints: vec!["t".into(), "n".into(), "k".into(), "acc".into()], loops: 0, next_id: 100 };
        let mut body = String::new();
        let k = g.r.below(5) + 3;
        g.block(3, 1, &mut body, k);
        out.push_str(&body);
    }
    out.push_str("    t + acc + fl.to_int() % 100\n\n");
    out.push_str("fn main()\n    mut xs := [Acc(a: 1, b: 2, f: 0.5), Acc(a: 100, b: -3, f: 1.5), Acc(a: 7, b: 7, f: 2.0)]\n    mut acc := 0\n    mut fl := 1.5\n    mut bag := Bag(items: [], n: 0)\n");
    if !pure {
        out.push_str("    mut m := Map[str, int]()\n");
    }
    {
        let mut g = G { r: &mut r, edgy, pure, in_work: false, flat: false, tgt: "acc", wild, ints: vec!["acc".into(), "xs[0].b".into()], loops: 0, next_id: 200 };
        let mut body = String::new();
        let k = g.r.below(6) + 3;
        g.block(3, 1, &mut body, k);
        out.push_str(&body);
        let (a, b) = (g.expr(1), g.expr(1));
        out.push_str(&format!("    acc += work({a} % 50, {b} % 9)\n"));
    }
    out.push_str("    print(acc)\n    print(fl)\n    for x in xs\n        print(\"{x.a} {x.b} {x.f}\")\n");
    out
}

/// Wider grammar: maps, strings, function values, enums with `match`, and
/// nested structs, inside the same kind of loops. Values stay small so most
/// runs finish; `wild` sometimes leaves a list index or a map key unchecked.
pub fn wide(seed: u64, pure: bool) -> String {
    wide_for(seed, pure, false)
}

/// `nat`: keep to what `ail build` can be asked for next (no function values).
pub fn wide_for(seed: u64, pure: bool, nat: bool) -> String {
    let mut r = Rng::new(seed ^ 0x5EED_1DE5);
    let wild = if r.chance(1, 4) { 6 } else { 0 };
    let mut out = String::from(
        "struct In\n    n int\n    tag str\n\nstruct Out\n    inner In\n    xs List[int]\n\nenum Op\n    Add(k int)\n    Mul(k int)\n    Neg\n    Nop\n\n\
fn In.grow(mut self, d int) int\n    self.n += d % 100\n    self.n\n\n\
fn dbl(x int) int\n    x * 2 % 1000\n\nfn inc(x int) int\n    (x + 1) % 1000\n\nfn apply(f fn(int) int, x int) int\n    f(f(x))\n\n\
fn run(op Op, v int) int\n    match op\n        Add(k) => (v + k) % 1000\n        Mul(k) => v * k % 1000\n        Neg => 0 - v\n        Nop => v\n\n",
    );
    out.push_str("fn main()\n    mut acc := 0\n    mut o := Out(inner: In(n: 1, tag: \"t\"), xs: [3, 1, 4])\n    mut f := dbl\n    ops := [Op.Add(k: 5), Op.Mul(k: 3), Op.Neg, Op.Nop]\n    mut m := Map[str, int]()\n    mut s := \"\"\n");
    let mut body = String::new();
    let loops = r.below(3) + 1;
    let mut uid = 0;
    for l in 0..loops {
        let bound = r.below(8) + 1;
        let ind = if l % 2 == 0 { 1 } else { 1 };
        Wd::push(&mut body, ind, &format!("for i{l} in 0..{bound}"));
        for _ in 0..r.below(5) + 2 {
            let i = format!("i{l}");
            let line: Vec<String> = match r.below(18) {
                0 => vec![format!("acc += run(ops[{i} % 4], acc)")],
                1 => vec![format!("acc += run(ops[{}], {i})", if wild > 0 && r.chance(1, 3) { "5".to_string() } else { format!("{i} % 4") })],
                2 => vec![format!("acc += o.inner.grow({i} * 7)")],
                3 => vec![format!("o.inner.n += acc % 10")],
                4 => vec![format!("o.xs.push(acc % 100)"), format!("acc += o.xs.len()")],
                5 => vec![format!("acc += o.xs[{}]", if wild > 0 && r.chance(1, 2) { format!("{i} + 3") } else { format!("{i} % o.xs.len()") })],
                6 => vec![format!("o.xs[{i} % o.xs.len()] = acc % 50")],
                7 | 8 | 9 if nat => vec![format!("acc += {i} % 7")],
                7 => vec![format!("f = if {i} % 2 == 0 then dbl else inc")],
                8 => vec![format!("acc = apply(f, acc % 1000)")],
                9 => vec![format!("acc += f({i})")],
                10 => {
                    uid += 1;
                    vec![format!("mut c{uid} := o"), format!("c{uid}.inner.grow({i})"), format!("acc += c{uid}.inner.n - o.inner.n")]
                }
                11 if !pure => vec![format!("m[\"k{{{i} % 3}}\"] = acc % 100"), format!("acc += m.len()")],
                12 if !pure => vec![if wild > 0 && r.chance(1, 2) { format!("acc += m[\"k{{{i}}}\"]") } else { format!("acc += m.get(\"k{{{i} % 3}}\") else 7") }],
                13 if !pure => vec![format!("s = \"{{s}}{{acc % 10}}\""), format!("acc += s.len()")],
                14 if !pure => vec![format!("acc += s.slice(0, {}).len()", if wild > 0 && r.chance(1, 2) { "100".to_string() } else { "s.len() / 2".to_string() })],
                15 if !pure => vec![format!("print(\"{{o.inner.tag}} {{acc}} {{o.inner.n}}\")")],
                16 => vec![format!("acc += [\"b{{{i} % 3}}\", \"a\", \"c{{acc % 4}}\"].sorted()[0].len()")],
                17 => vec![format!("acc += o.xs.sorted()[0] % 9")],
                _ => vec![format!("acc += {i}")],
            };
            for l in line {
                Wd::push(&mut body, ind + 1, &l);
            }
        }
        if r.chance(1, 3) {
            Wd::push(&mut body, ind + 1, "if acc > 5000");
            Wd::push(&mut body, ind + 2, if r.chance(1, 2) { "break" } else { "continue" });
        }
    }
    out.push_str(&body);
    out.push_str("    print(acc)\n    print(o.inner.n)\n    print(o.xs.len())\n    for k in m.keys().sorted()\n        print(\"{k}={m[k]}\")\n    print(s)\n");
    if nat {
        out = out.replace("    mut f := dbl\n", "");
        out = out.replace("fn apply(f fn(int) int, x int) int\n    f(f(x))\n\n", "");
    }
    if nat && !pure {
        out = out.replace("    for k in m.keys().sorted()\n        print(\"{k}={m[k]}\")\n", "    print(m.len())\n");
    }
    if pure {
        out = out.replace("    for k in m.keys().sorted()\n        print(\"{k}={m[k]}\")\n    print(s)\n", "");
        out = out.replace("    mut m := Map[str, int]()\n    mut s := \"\"\n", "");
    }
    out
}

struct Wd;
impl Wd {
    fn push(out: &mut String, ind: usize, line: &str) {
        out.push_str(&"    ".repeat(ind));
        out.push_str(line);
        out.push('\n');
    }
}

/// Tuples (type.tuple.*): literals, `a, b := t`, `t.0`, tuple patterns in `match`, tuple
/// returns and parameters, tuples in lists and as map keys, `==` and `<`, and printing.
/// Values stay small. With `break_it` one construct is made invalid, so the checker
/// has something to reject.
pub fn tuples(seed: u64, break_it: bool) -> String {
    let mut r = Rng::new(seed ^ 0x7091_E5);
    let mut out = String::from(
        "fn pair(a int, b int) (int, int)\n    (a + b, a - b)\n\nfn triple(a int) (int, str, bool)\n    (a, \"t{a}\", a % 2 == 0)\n\nfn swap(t (int, str)) (str, int)\n    (t.1, t.0)\n\n",
    );
    out.push_str("fn main()\n    mut acc := 0\n    mut t := (1, \"a\")\n    mut p := (2, 3)\n    mut xs := [(1, 2), (3, 4)]\n    mut m := Map[(int, int), int]()\n");
    let mut id = 0;
    let bound = r.below(6) + 1;
    out.push_str(&format!("    for i in 0..{bound}\n"));
    let mut lines: Vec<String> = Vec::new();
    for _ in 0..r.below(8) + 3 {
        id += 1;
        let n = id;
        let wild = r.chance(1, 10);
        let l: Vec<String> = match r.below(18) {
            0 => vec![format!("a{n}, b{n} := pair(acc % 100, i)"), format!("acc += a{n} + b{n}")],
            1 => vec![format!("p = (p.1, p.0 + i)")],
            2 => vec!["match p".into(), "    (0, _) => acc += 1".into(), "    (k, w) => acc += k % 7 + w % 3".into()],
            3 => vec![format!("t = (i, \"s{{i}}\")")],
            4 => vec![format!("x{n}, y{n} := xs[{}]", if wild { "i + 5".to_string() } else { "i % xs.len()".to_string() }), format!("acc += x{n} + y{n}")],
            5 => vec![format!("c{n}, mut d{n} := p"), format!("d{n} += 1"), format!("acc += c{n} + d{n}")],
            6 => vec![format!("m[(i % 3, {})] += i", r.below(2))],
            7 => vec![format!("s{n}, n{n} := swap(t)"), format!("acc += s{n}.len() + n{n}")],
            8 => vec![format!("xs.push((i, acc % 10))")],
            9 => vec!["if p == (1, 2) or p < (5, 5)".into(), "    acc += 1".into()],
            10 => vec![format!("_, s{n}, ok{n} := triple(i)"), format!("if ok{n}"), format!("    acc += s{n}.len()")],
            11 => vec![format!("for q{n}, v{n} in xs.enumerate()"), format!("    acc += q{n} * v{n}.0 % 50")],
            12 => vec![format!("for e{n} in xs"), format!("    acc += e{n}.1 % 9")],
            13 => vec![format!("acc += t.1.len() + t.0 % 5")],
            14 => vec![format!("xs[i % xs.len()] = (acc % 7, i)")],
            15 => vec![format!("print(t)"), format!("print(p)")],
            16 => vec![format!("acc += (xs[0].0 + xs[0].1) * (p.0 % 3)")],
            _ => vec![format!("if t == (i, \"s{{i}}\")"), format!("    acc += 2")],
        };
        lines.extend(l);
    }
    if break_it {
        // One invalid construct, picked from a few the checker must reject.
        let bad = match r.below(7) {
            0 => "a9, b9, z9 := pair(1, 2)",
            1 => "acc += p",
            2 => "acc += p.2",
            3 => "p = (1, \"x\")",
            4 => "t = (1, 2, 3)",
            5 => "match p\n    (0, _) => acc += 1",
            _ => "s9, n9 := acc",
        };
        let at = r.below(lines.len() + 1);
        for (k, l) in bad.lines().enumerate() {
            lines.insert(at + k, l.to_string());
        }
    }
    for l in lines {
        out.push_str("        ");
        out.push_str(&l);
        out.push('\n');
    }
    out.push_str("    print(acc)\n    print(p)\n    print(t)\n    print(xs)\n    print(m.len())\n");
    out
}

/// What the native backend grew next (fuzz.nat.*): `match` on `char` (literals, alternatives,
/// ranges), `==`/`!=` between optionals and values, and error values: error enums and
/// structs with fields, `return E.V(..)`, `else err` with `err.field`, `?` on fallible calls,
/// and `else e` on a method.
pub fn native_errs(seed: u64) -> String {
    let mut r = Rng::new(seed ^ 0xE22_0A5);
    let mut out = String::from(
        "error enum E\n    A \"a\"\n    B(n int) \"b {n}\"\n    C(s str, k int) \"c {s} {k}\"\n\nerror struct S \"bad {path}\"\n    path str\n    code int\n\n\
struct Box\n    v int\n\n\
fn pick(n int) int?\n    if n % 3 == 0\n        return none\n    n % 5\n\n\
fn f(n int) int ! E\n    if n % 4 == 0\n        return E.A\n    if n % 4 == 1\n        return E.B(n: n)\n    if n % 4 == 2\n        return E.C(s: \"x{n}\", k: n % 7)\n    n\n\n\
fn g(n int) str ! S\n    if n % 3 == 0\n        return S(path: \"p{n}\", code: n)\n    \"ok{n}\"\n\n\
fn h(n int) int ! E | S\n    a := f(n)?\n    b := g(n)?\n    a + b.len()\n\n\
fn cls(c char) int\n    match c\n        'a' => 1\n        'b' | 'c' => 2\n        'd'..='f' => 3\n        'x'..'z' => 4\n        _ => 0\n\n\
fn Box.get(self, n int) int ! E\n    f(n + v)?\n\n\
fn main()\n    mut acc := 0\n    word := \"abcdexyz-\"\n",
    );
    let bound = r.below(7) + 2;
    out.push_str(&format!("    for i in 0..{bound}\n"));
    let mut id = 0;
    let mut lines: Vec<String> = Vec::new();
    for _ in 0..r.below(8) + 4 {
        id += 1;
        let n = id;
        let l: Vec<String> = match r.below(19) {
            5 | 12 if seed % 4 != 0 => vec![format!("acc += i")],
            0 => vec![format!("r{n} := f(i) else err"), "    match err".into(), "        A => 0 - 1".into(), "        B(k) => k % 5".into(), "        C(s, k) => s.len() + k".into(), format!("acc += r{n}")],
            1 => vec![format!("acc += f(i + {}) else 7", r.below(3))],
            2 => vec![format!("t{n} := g(i) else err"), "    \"{err.path}-{err.code}\"".into(), format!("acc += t{n}.len()")],
            3 => vec![format!("hv{n} := h(i) else err"), "    print(err)".into(), "    0 - 2".into(), format!("acc += hv{n}")],
            4 => vec![format!("z{n} := f(i) else continue"), format!("acc += z{n}")],
            5 => vec![format!("bx{n} := Box(v: i)"), format!("bv{n} := bx{n}.get(i) else e"), "    print(e)".into(), "    3".into(), format!("acc += bv{n}")],
            6 => vec![format!("for ch{n} in word.chars()"), format!("    acc += cls(ch{n})")],
            7 => vec![format!("acc += cls('{}')", r.pick(&["a", "b", "c", "e", "x", "y", "z", "q"]))],
            8 => vec![format!("o{n} := pick(i)"), format!("if o{n} == {}", r.below(6)), "    acc += 1".into()],
            9 => vec![format!("o{n} := pick(i)"), format!("if o{n} != none"), "    acc += 2".into()],
            10 => vec![format!("o{n} := pick(i)"), format!("p{n} := pick(i + {})", r.below(3)), format!("if o{n} == p{n}"), "    acc += 3".into()],
            11 => vec![format!("o{n} := pick(i)"), format!("if o{n} == none"), "    acc += 4".into(), "else".into(), "    acc += 5".into()],
            12 => vec![format!("if v{n} := f(i)"), format!("    acc += v{n}"), "else err".into(), "    print(err)".into()],
            13 => vec![format!("acc += pick(i) else 9")],
            14 => vec![format!("x{n} := f(i) else err"), "    err.to_str().len()".into(), format!("acc += x{n}")],
            15 => vec![format!("p{n} := pick(i)"), format!("if p{n} != pick(i + 1)"), "    acc += 6".into()],
            16 => vec![format!("print(pick(i) else 0)")],
            17 => vec![format!("m{n} := match word.chars()[i % 9]"), "    'a'..='c' => 1".into(), "    'd' | 'e' => 2".into(), "    _ => 3".into(), format!("acc += m{n}")],
            _ => vec![format!("acc += i")],
        };
        lines.extend(l);
    }
    for l in lines {
        out.push_str("        ");
        out.push_str(&l);
        out.push('\n');
    }
    out.push_str("    print(acc)\n");
    out
}
