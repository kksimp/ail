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

    fn target(&self) -> &'static str {
        self.tgt
    }

    fn stmt(&mut self, depth: usize, ind: usize, out: &mut String) {
        let tgt = self.target();
        let kinds = if depth == 0 { 9 } else { 17 };
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
    // A helper with loops and early returns.
    out.push_str("fn work(n int, k int) int\n    mut t := 0\n    mut fl := 0.5\n    mut xs := [Acc(a: n, b: k, f: 1.0), Acc(a: 2, b: 3, f: 0.0), Acc(a: -5, b: 8, f: 2.5)]\n    mut acc := 0\n");
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
    out.push_str("fn main()\n    mut xs := [Acc(a: 1, b: 2, f: 0.5), Acc(a: 100, b: -3, f: 1.5), Acc(a: 7, b: 7, f: 2.0)]\n    mut acc := 0\n    mut fl := 1.5\n");
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
