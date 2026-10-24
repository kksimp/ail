//! Seeded generator of well-typed AIL programs and a token-level mutator
//! (fuzz.gen.1-3, ADR 0269). Everything here is a pure function of the seed,
//! so a failure reproduces from `AIL_FUZZ_SEED` alone.

/// splitmix64: tiny, fast, and the same on every platform.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03)
    }
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A number in `0..n` (`n > 0`).
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    pub fn chance(&mut self, num: usize, den: usize) -> bool {
        self.below(den) < num
    }
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Ty {
    Int,
    Bool,
    Str,
    ListInt,
    ListStr,
    MapSI,
    OptInt,
    Pt,
    Shape,
    Float,
}

const ALL: [Ty; 10] =
    [Ty::Int, Ty::Bool, Ty::Str, Ty::ListInt, Ty::ListStr, Ty::MapSI, Ty::OptInt, Ty::Pt, Ty::Shape, Ty::Float];
const MUTABLE: [Ty; 5] = [Ty::Int, Ty::Str, Ty::ListInt, Ty::MapSI, Ty::Pt];
const WORDS: [&str; 6] = ["a", "b", "ab", "x y", "", "Hi"];

struct Var {
    name: String,
    ty: Ty,
    mutable: bool,
}

/// Types every generated program declares: a struct with methods, an enum with
/// payloads, an error enum, a fallible function and a function returning an
/// optional. Fixed text keeps the programs well-typed by construction.
const PRELUDE: &str = "\
struct Pt
    a int
    b int

fn Pt.sum(self) int
    a + b

fn Pt.shift(mut self, d int)
    a += d

enum Shape
    Circle(r int)
    Rect(w int, h int)
    Dot

fn Shape.area(self) int
    match self
        Circle(r) => 3 * r * r
        Rect(w, h) => w * h
        Dot => 0

fn mul(a int, b int) int
    a * b

trait Weigh
    fn weight(self) int
    fn heavy(self) bool
        weight() > 10

impl Weigh for Pt
    fn weight(self) int
        a * 2 + b

fn largest[T Ord](items List[T]) T
    mut best := items[0]
    for x in items
        if x > best
            best = x
    best

fn guarded(x int) int
    requires x == x
    ensures result >= 0 or result < 0
    x

fn chain(x int) !int
    y := check_pos(x)? \"chain\"
    y + 1

error enum Fail
    Neg(n int)

fn check_pos(x int) int ! Fail
    if x < 0
        return Fail.Neg(n: x)
    x

fn pick(xs List[int], i int) int?
    if i >= 0 and i < xs.len()
        return xs[i]
    none

";

struct Gen {
    rng: Rng,
    out: String,
    vars: Vec<Var>,
    fresh: usize,
    /// Helper functions `f0 .. f{helpers}` taking two ints.
    helpers: usize,
    /// Helpers callable from the body being generated (no recursion).
    callable: usize,
    ind: usize,
    /// Loops around the statement being generated, and whether it is in a helper.
    loops: usize,
    in_helper: bool,
}

/// A whole program for `seed`.
pub fn program(seed: u64) -> String {
    let mut g = Gen { rng: Rng::new(seed), out: String::from(PRELUDE), vars: Vec::new(), fresh: 0, helpers: 0, callable: 0, ind: 0, loops: 0, in_helper: false };
    g.helpers = g.rng.below(4);
    for i in 0..g.helpers {
        g.callable = i;
        g.vars.clear();
        g.vars.push(Var { name: "p0".into(), ty: Ty::Int, mutable: false });
        g.vars.push(Var { name: "p1".into(), ty: Ty::Int, mutable: false });
        g.out.push_str(&format!("fn f{i}(p0 int, p1 int) int\n"));
        g.ind = 1;
        g.in_helper = true;
        let n = 1 + g.rng.below(5);
        g.block(n, 2);
        let e = g.expr(Ty::Int, 2);
        g.line(&e);
        g.out.push('\n');
    }
    g.callable = g.helpers;
    g.in_helper = false;
    g.vars.clear();
    g.out.push_str("fn main()\n");
    g.ind = 1;
    let n = 4 + g.rng.below(10);
    g.block(n, 3);
    g.out
}

impl Gen {
    fn line(&mut self, s: &str) {
        for _ in 0..self.ind {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn name(&mut self, p: &str) -> String {
        self.fresh += 1;
        format!("{p}{}", self.fresh)
    }

    fn var_of(&mut self, ty: Ty) -> Option<String> {
        let names: Vec<&str> = self.vars.iter().filter(|v| v.ty == ty).map(|v| v.name.as_str()).collect();
        if names.is_empty() { None } else { Some(self.rng.pick(&names).to_string()) }
    }

    fn mut_var_of(&mut self, ty: Ty) -> Option<String> {
        let names: Vec<&str> = self.vars.iter().filter(|v| v.ty == ty && v.mutable).map(|v| v.name.as_str()).collect();
        if names.is_empty() { None } else { Some(self.rng.pick(&names).to_string()) }
    }

    /// A block of up to `n` statements; names it declares go out of scope.
    fn block(&mut self, n: usize, depth: usize) {
        let scope = self.vars.len();
        for _ in 0..n {
            self.stmt(depth);
        }
        self.vars.truncate(scope);
    }

    fn indented(&mut self, n: usize, depth: usize) {
        self.ind += 1;
        let scope = self.vars.len();
        // A block is never empty: it ends with a print.
        for _ in 0..n {
            self.stmt(depth);
        }
        let ty = *self.rng.pick(&[Ty::Int, Ty::Str, Ty::Bool]);
        let e = self.expr(ty, 1);
        self.line(&format!("print({e})"));
        self.vars.truncate(scope);
        self.ind -= 1;
    }

    fn stmt(&mut self, depth: usize) {
        let pick = self.rng.below(100);
        match pick {
            0..=14 => {
                let ty = *self.rng.pick(&ALL);
                let (n, e) = (self.name("v"), self.expr(ty, 2));
                self.line(&format!("{n} := {e}"));
                self.vars.push(Var { name: n, ty, mutable: false });
            }
            15..=26 => {
                let ty = *self.rng.pick(&MUTABLE);
                let (n, e) = (self.name("m"), self.expr(ty, 2));
                self.line(&format!("mut {n} := {e}"));
                self.vars.push(Var { name: n, ty, mutable: true });
            }
            27..=40 => {
                let ty = *self.rng.pick(&ALL);
                let e = self.expr(ty, 3);
                self.line(&format!("print({e})"));
            }
            41..=52 => self.mutate(),
            53..=60 if depth > 0 => self.if_stmt(depth - 1),
            61..=68 if depth > 0 => self.for_stmt(depth - 1),
            69..=72 if depth > 0 => {
                let (c, k) = (self.name("w"), 1 + self.rng.below(4));
                self.line(&format!("mut {c} := 0"));
                self.line(&format!("while {c} < {k}"));
                // Not assignable: the loop must end.
                self.vars.push(Var { name: c.clone(), ty: Ty::Int, mutable: false });
                self.ind += 1;
                self.loops += 1;
                self.line(&format!("{c} += 1"));
                let n = 1 + self.rng.below(2);
                self.block(n, depth - 1);
                self.loops -= 1;
                self.ind -= 1;
            }
            73..=78 => self.match_stmt(),
            79..=83 if depth > 0 => self.option_stmt(depth - 1),
            84..=87 => self.task_stmt(),
            88..=90 => {
                let (ch, v, e1, e2) = (self.name("ch"), self.name("r"), self.expr(Ty::Int, 1), self.expr(Ty::Int, 1));
                self.line(&format!("{ch} := Channel[int](4)"));
                self.line(&format!("{ch}.send({e1})"));
                self.line(&format!("{ch}.send({e2})"));
                self.line(&format!("{ch}.close()"));
                self.line(&format!("{v} := {ch}.recv() else 0"));
                self.vars.push(Var { name: v, ty: Ty::Int, mutable: false });
            }
            91..=92 => {
                let (mu, x, e) = (self.name("mu"), self.name("x"), self.expr(Ty::Int, 1));
                self.line(&format!("{mu} := Mutex({e})"));
                self.line(&format!("{x} := {mu}.lock()"));
                self.line(&format!("{mu}.unlock({x} + 1)"));
                self.vars.push(Var { name: x, ty: Ty::Int, mutable: false });
            }
            93..=94 => {
                let (v, e) = (self.name("g"), self.expr(Ty::Int, 1));
                self.line(&format!("{v} := chain({e}) else 0"));
                self.vars.push(Var { name: v, ty: Ty::Int, mutable: false });
                let (e, f) = (self.expr(Ty::Int, 1), self.expr(Ty::Float, 1));
                self.line(&format!("print(\"{{guarded({e}):>6}}|{{({f}):.2}}|{{7:*^8}}\")"));
            }
            95..=97 if self.loops > 0 => {
                let c = self.expr(Ty::Bool, 2);
                let word = *self.rng.pick(&["break", "continue"]);
                self.line(&format!("if {c}"));
                self.line(&format!("    {word}"));
            }
            98..=99 if self.in_helper => {
                let (c, e) = (self.expr(Ty::Bool, 2), self.expr(Ty::Int, 2));
                self.line(&format!("if {c}"));
                self.line(&format!("    return {e}"));
            }
            _ => {
                let e = self.expr(Ty::Str, 2);
                self.line(&format!("print({e})"));
            }
        }
    }

    fn mutate(&mut self) {
        let ty = *self.rng.pick(&MUTABLE);
        let Some(v) = self.mut_var_of(ty) else {
            let e = self.expr(Ty::Int, 1);
            return self.line(&format!("print({e})"));
        };
        match ty {
            Ty::Int => {
                let op = *self.rng.pick(&["=", "+=", "-="]);
                let e = self.expr(Ty::Int, 2);
                self.line(&format!("{v} {op} {e}"))
            }
            Ty::Str => {
                let e = self.expr(Ty::Str, 2);
                self.line(&format!("{v} = {e}"))
            }
            Ty::ListInt => {
                let e = self.expr(Ty::Int, 2);
                match self.rng.below(5) {
                    0 | 1 => self.line(&format!("{v}.push({e})")),
                    2 => self.line(&format!("{v}.sort()")),
                    3 => self.line(&format!("{v}.reverse()")),
                    _ => self.line(&format!("{v}.insert(0, {e})")),
                }
            }
            Ty::MapSI => {
                let k = *self.rng.pick(&["a", "b", "c"]);
                let e = self.expr(Ty::Int, 2);
                self.line(&format!("{v}[\"{k}\"] = {e}"))
            }
            _ => {
                let e = self.expr(Ty::Int, 2);
                match self.rng.below(3) {
                    0 => self.line(&format!("{v}.a = {e}")),
                    1 => self.line(&format!("{v}.shift({e})")),
                    _ => self.line(&format!("{v}.b += {e}")),
                }
            }
        }
    }

    fn if_stmt(&mut self, depth: usize) {
        let c = self.expr(Ty::Bool, 2);
        self.line(&format!("if {c}"));
        let n = self.rng.below(3);
        self.indented(n, depth);
        if self.rng.chance(1, 3) {
            let c = self.expr(Ty::Bool, 1);
            self.line(&format!("elif {c}"));
            self.indented(0, depth);
        }
        if self.rng.chance(1, 2) {
            self.line("else");
            let n = self.rng.below(3);
            self.indented(n, depth);
        }
    }

    fn for_stmt(&mut self, depth: usize) {
        let x = self.name("k");
        match self.rng.below(4) {
            0 | 1 => {
                let (k, op) = (1 + self.rng.below(4), *self.rng.pick(&["..", "..="]));
                self.line(&format!("for {x} in 0{op}{k}"));
                self.vars.push(Var { name: x, ty: Ty::Int, mutable: false });
            }
            2 => {
                let xs = self.expr(Ty::ListInt, 1);
                self.line(&format!("for {x} in {xs}"));
                self.vars.push(Var { name: x, ty: Ty::Int, mutable: false });
            }
            _ => {
                let xs = self.expr(Ty::ListStr, 1);
                self.line(&format!("for {x} in {xs}"));
                self.vars.push(Var { name: x, ty: Ty::Str, mutable: false });
            }
        }
        let n = self.rng.below(3);
        let at = self.vars.len() - 1;
        self.loops += 1;
        self.indented(n, depth);
        self.loops -= 1;
        self.vars.truncate(at);
    }

    fn match_stmt(&mut self) {
        let sh = self.expr(Ty::Shape, 1);
        let (r, w, h) = (self.name("r"), self.name("w"), self.name("h"));
        if self.rng.chance(1, 2) {
            let n = self.name("v");
            self.line(&format!("{n} := match {sh}"));
            self.ind += 1;
            let (e1, e2, e3) = (self.expr(Ty::Int, 1), self.expr(Ty::Int, 1), self.expr(Ty::Int, 1));
            self.line(&format!("Circle({r}) => {r} + {e1}"));
            self.line(&format!("Rect({w}, _) => {w} * {e2}"));
            self.line(&format!("Dot => {e3}"));
            self.ind -= 1;
            self.vars.push(Var { name: n, ty: Ty::Int, mutable: false });
        } else {
            self.line(&format!("match {sh}"));
            self.ind += 1;
            let (e1, e2) = (self.expr(Ty::Str, 1), self.expr(Ty::Int, 1));
            self.line(&format!("Circle({r}) => print({r})"));
            self.line(&format!("Rect({w}, {h}) => print({w} + {h})"));
            self.line(&format!("Dot => print({e1}.len() + {e2})"));
            self.ind -= 1;
        }
    }

    fn option_stmt(&mut self, depth: usize) {
        if self.rng.chance(1, 2) {
            let (v, xs, e) = (self.name("o"), self.expr(Ty::ListInt, 1), self.expr(Ty::Int, 1));
            self.line(&format!("if {v} := pick({xs}, {e})"));
            self.vars.push(Var { name: v.clone(), ty: Ty::Int, mutable: false });
            self.indented(1, depth);
            self.vars.pop();
            self.line("else");
            self.indented(0, depth);
        } else {
            let (v, er, e) = (self.name("c"), self.name("err"), self.expr(Ty::Int, 1));
            self.line(&format!("if {v} := check_pos({e})"));
            self.vars.push(Var { name: v, ty: Ty::Int, mutable: false });
            self.indented(1, depth);
            self.vars.pop();
            self.line(&format!("else {er}"));
            self.ind += 1;
            self.line(&format!("print({er})"));
            self.ind -= 1;
        }
    }

    fn task_stmt(&mut self) {
        let (t, v) = (self.name("t"), self.name("w"));
        let (a, b) = (self.expr(Ty::Int, 1), self.expr(Ty::Int, 1));
        let call = if self.callable > 0 && self.rng.chance(2, 3) {
            format!("f{}({a}, {b})", self.rng.below(self.callable))
        } else if self.rng.chance(1, 2) {
            format!("check_pos({a})")
        } else {
            format!("mul({a}, {b})")
        };
        self.line(&format!("{t} := spawn {call}"));
        self.line(&format!("{v} := {t}.wait() else 0"));
        self.vars.push(Var { name: v, ty: Ty::Int, mutable: false });
    }

    fn lit(&mut self, ty: Ty) -> String {
        match ty {
            Ty::Int => format!("{}", self.rng.below(20)),
            Ty::Bool => (if self.rng.chance(1, 2) { "true" } else { "false" }).into(),
            Ty::Str => format!("\"{}\"", self.rng.pick(&WORDS)),
            Ty::ListInt => {
                let n = self.rng.below(5);
                let items: Vec<String> = (0..n).map(|_| format!("{}", self.rng.below(10))).collect();
                if n == 0 { "List[int]()".into() } else { format!("[{}]", items.join(", ")) }
            }
            Ty::ListStr => {
                let n = 1 + self.rng.below(3);
                let items: Vec<String> = (0..n).map(|_| format!("\"{}\"", self.rng.pick(&WORDS))).collect();
                format!("[{}]", items.join(", "))
            }
            Ty::MapSI => {
                let (a, b) = (self.rng.below(9), self.rng.below(9));
                format!("{{\"a\": {a}, \"b\": {b}}}")
            }
            Ty::OptInt => "pick([1, 2], 1)".into(),
            Ty::Pt => format!("Pt(a: {}, b: {})", self.rng.below(9), self.rng.below(9)),
            Ty::Shape => "Shape.Dot".into(),
            Ty::Float => format!("{}.{}", self.rng.below(9), self.rng.below(10)),
        }
    }

    /// An expression of type `ty`; `d` bounds the nesting.
    fn expr(&mut self, ty: Ty, d: usize) -> String {
        if d == 0 || self.rng.chance(1, 4) {
            if self.rng.chance(2, 3) {
                if let Some(v) = self.var_of(ty) {
                    return v;
                }
            }
            return self.lit(ty);
        }
        let d = d - 1;
        match ty {
            Ty::Int => self.int_expr(d),
            Ty::Bool => match self.rng.below(8) {
                0 | 1 => {
                    let (a, b, op) = (self.expr(Ty::Int, d), self.expr(Ty::Int, d), *self.rng.pick(&["<", "<=", "==", "!=", ">", ">="]));
                    format!("{a} {op} {b}")
                }
                2 => {
                    let (a, b) = (self.expr(Ty::Str, d), self.expr(Ty::Str, d));
                    format!("{a} == {b}")
                }
                3 => format!("not ({})", self.expr(Ty::Bool, d)),
                4 => format!("({}) {} ({})", self.expr(Ty::Bool, d), self.rng.pick(&["and", "or"]), self.expr(Ty::Bool, d)),
                5 => format!("{}.contains({})", self.expr(Ty::ListInt, d), self.expr(Ty::Int, d)),
                6 => format!("{}.contains(\"{}\")", self.expr(Ty::Str, d), self.rng.pick(&["a", "b", " "])),
                _ => format!("{}.contains(\"a\")", self.expr(Ty::MapSI, d)),
            },
            Ty::Str => match self.rng.below(8) {
                0 => {
                    let e = self.expr(Ty::Int, d);
                    format!("\"n={{({e})}}\"")
                }
                1 => {
                    let e = self.expr(Ty::Str, d);
                    format!("{e}.to_upper()")
                }
                2 => format!("{}.trim()", self.expr(Ty::Str, d)),
                3 => format!("{}.repeat({})", self.expr(Ty::Str, d), self.rng.below(3)),
                4 => format!("{}.join(\",\")", self.expr(Ty::ListInt, d)),
                5 => format!("{}.join(\"-\")", self.expr(Ty::ListStr, d)),
                6 => {
                    let (a, b) = (self.expr(Ty::Str, d), self.expr(Ty::Bool, d));
                    format!("\"{{({a})}}/{{({b})}}\"")
                }
                _ => {
                    let e = self.expr(Ty::Pt, d);
                    format!("\"{{({e})}}\"")
                }
            },
            Ty::ListInt => match self.rng.below(5) {
                0 => format!("{}.sorted()", self.expr(Ty::ListInt, d)),
                1 => {
                    let (xs, x, k) = (self.expr(Ty::ListInt, d), self.name("c"), self.rng.below(5));
                    format!("[{x} * 2 for {x} in {xs} if {x} > {k}]")
                }
                2 => {
                    let (a, b) = (self.expr(Ty::Int, d), self.expr(Ty::Int, d));
                    format!("[{a}, {b}, 1]")
                }
                3 => format!("{}.values()", self.expr(Ty::MapSI, d)),
                _ => {
                    let (a, j, k) = (self.expr(Ty::Int, d), self.name("j"), 1 + self.rng.below(4));
                    format!("[{a} + {j} for {j} in 0..{k}]")
                }
            },
            Ty::ListStr => match self.rng.below(3) {
                0 => format!("{}.split(\",\")", self.expr(Ty::Str, d)),
                1 => format!("{}.lines()", self.expr(Ty::Str, d)),
                _ => format!("{}.keys()", self.expr(Ty::MapSI, d)),
            },
            Ty::MapSI => {
                let (a, b) = (self.expr(Ty::Int, d), self.expr(Ty::Int, d));
                format!("{{\"a\": {a}, \"b\": {b}}}")
            }
            Ty::OptInt => match self.rng.below(5) {
                0 => format!("{}.first()", self.expr(Ty::ListInt, d)),
                1 => format!("{}.max()", self.expr(Ty::ListInt, d)),
                2 => format!("{}.to_int()", self.expr(Ty::Str, d)),
                3 => format!("{}.get(\"b\")", self.expr(Ty::MapSI, d)),
                _ => format!("pick({}, {})", self.expr(Ty::ListInt, d), self.expr(Ty::Int, d)),
            },
            Ty::Pt => format!("Pt(b: {}, a: {})", self.expr(Ty::Int, d), self.expr(Ty::Int, d)),
            Ty::Float => match self.rng.below(5) {
                0 => format!("({} + {})", self.expr(Ty::Float, d), self.expr(Ty::Float, d)),
                1 => format!("({} * {})", self.expr(Ty::Float, d), self.expr(Ty::Float, d)),
                2 => format!("({} - {})", self.expr(Ty::Float, d), self.expr(Ty::Float, d)),
                3 => format!("({} / {})", self.expr(Ty::Float, d), self.expr(Ty::Float, d)),
                _ => format!("({}).to_float()", self.expr(Ty::Int, d)),
            },
            Ty::Shape => match self.rng.below(3) {
                0 => format!("Shape.Circle(r: {})", self.expr(Ty::Int, d)),
                1 => format!("Shape.Rect(w: {}, h: {})", self.expr(Ty::Int, d), self.expr(Ty::Int, d)),
                _ => "Shape.Dot".into(),
            },
        }
    }

    fn int_expr(&mut self, d: usize) -> String {
        match self.rng.below(21) {
            0 => format!("({} + {})", self.expr(Ty::Int, d), self.expr(Ty::Int, d)),
            1 => format!("({} - {})", self.expr(Ty::Int, d), self.expr(Ty::Int, d)),
            2 => format!("({} * {})", self.expr(Ty::Int, d), self.expr(Ty::Int, d)),
            3 => format!("({} / {})", self.expr(Ty::Int, d), 1 + self.rng.below(5)),
            4 => format!("({} % {})", self.expr(Ty::Int, d), 1 + self.rng.below(5)),
            5 => format!("{}.abs()", self.expr(Ty::Int, d)),
            6 => format!("{}.min({})", self.expr(Ty::Int, d), self.expr(Ty::Int, d)),
            7 => format!("{}.len()", self.expr(Ty::ListInt, d)),
            8 => format!("{}.len()", self.expr(Ty::Str, d)),
            9 => format!("{}.sum()", self.expr(Ty::ListInt, d)),
            10 => format!("{}.sum()", self.expr(Ty::Pt, d)),
            11 => format!("{}.area()", self.expr(Ty::Shape, d)),
            12 => format!("({} else 0)", self.expr(Ty::OptInt, d)),
            13 => format!("(check_pos({}) else 0)", self.expr(Ty::Int, d)),
            14 if self.callable > 0 => {
                let f = self.rng.below(self.callable);
                format!("f{f}({}, {})", self.expr(Ty::Int, d), self.expr(Ty::Int, d))
            }
            15 => match self.var_of(Ty::Pt) {
                Some(p) => format!("{p}.a"),
                None => format!("{}.b", self.lit(Ty::Pt)),
            },
            16 => match self.var_of(Ty::MapSI) {
                Some(m) => format!("{m}[\"a\"]"),
                None => self.lit(Ty::Int),
            },
            17 => format!("({}).to_int()", self.expr(Ty::Float, d)),
            18 => format!("largest([{}, {}, 1])", self.expr(Ty::Int, d), self.expr(Ty::Int, d)),
            19 => format!("{}.weight()", self.expr(Ty::Pt, d)),
            20 => format!("({}.slice(0, 0).len() + ({}.find(\"a\") else 1))", self.expr(Ty::Str, d), self.expr(Ty::Str, d)),
            _ => self.lit(Ty::Int),
        }
    }
}

// ---------------------------------------------------------------------------
// Probes (fuzz.gen.4): small programs that push edge values (extremes, zero,
// negative, empty, non-ASCII) through the operations most likely to trap. A
// trap is the right answer; a panic of the interpreter is the finding.

const EDGE_INTS: [&str; 12] =
    ["0", "1", "-1", "2", "7", "100", "255", "65536", "9223372036854775807", "-9223372036854775807", "4294967296", "-100"];
const EDGE_STRS: [&str; 8] = ["\"\"", "\"a\"", "\"héllo\"", "\"日本語\"", "\"a,b,,c\"", "\" x \"", "\"line1\\nline2\"", "\"😀x\""];
const EDGE_LISTS: [&str; 5] = ["List[int]()", "[1]", "[3, 1, 2]", "[9223372036854775807, 1]", "[-5, 0, 5, 10]"];

const PROBE_DEFS: &str = "
struct Pt
    a int
    b int

fn Pt.sum(self) int
    a + b

fn deep(n int) int
    if n == 0
        return 0
    1 + deep(n - 1)
";

/// A probe program for `seed`: a few statements, each independent of the others.
pub fn probe(seed: u64) -> String {
    let mut rng = Rng::new(seed ^ 0x5052_4F42_45);
    let mut out = String::from("fn main()\n");
    for k in 0..1 + rng.below(3) {
        let (a, b, c) = (*rng.pick(&EDGE_INTS), *rng.pick(&EDGE_INTS), *rng.pick(&EDGE_INTS));
        let (s, t) = (*rng.pick(&EDGE_STRS), *rng.pick(&EDGE_STRS));
        let l = *rng.pick(&EDGE_LISTS);
        let body: String = match rng.below(66) {
            0 => format!("print({a} + {b})"),
            1 => format!("print({a} - {b})"),
            2 => format!("print({a} * {b})"),
            3 => format!("print({a} / {b})"),
            4 => format!("print({a} % {b})"),
            5 => format!("print(({a}).abs())"),
            6 => format!("print(-({a}))"),
            7 => format!("print(({a}).wrapping_add({b}))"),
            8 => format!("print(({a}).checked_add({b}))"),
            9 => format!("print(({a}).saturating_add({b}))"),
            10 => format!("print(({a}).to_hex())"),
            11 => format!("print({l}[{a}])"),
            12 => format!("print({l}[{a}..{b}])"),
            13 => format!("print({l}[{a}..])"),
            14 => format!("print({l}[..{a}])"),
            15 => format!("mut xs := {l}\n    xs.remove_at({a})\n    print(xs)"),
            16 => format!("mut xs := {l}\n    xs.insert({a}, {b})\n    print(xs)"),
            17 => format!("mut xs := {l}\n    xs.swap({a}, {b})\n    print(xs)"),
            18 => format!("print({l}.sum())"),
            19 => format!("print({l}.min())"),
            20 => format!("print({s}.slice({a}, {b}))"),
            21 => format!("print({s}.char_at({a}))"),
            22 => format!("print({s}.repeat({a}))"),
            23 => format!("print({s}.pad_left({a}, ' '))"),
            24 => format!("print({s}.split({t}))"),
            25 => format!("print({s}.replace({t}, {t}))"),
            26 => format!("print({s}.find({t}))"),
            27 => format!("print({s}.to_int())"),
            28 => format!("print({s}.chars())"),
            29 => format!("print({s}.strip_prefix({t}))"),
            30 => format!("print(\"{{{a}:>{b}}}\")"),
            31 => format!("print(\"{{{a}:*^{c}}}\")"),
            32 => format!("for i in {a}..{b}\n        print(i)\n        break"),
            33 => format!("m := {{\"a\": {a}}}\n    print(m[\"b\"])"),
            34 => format!("print(({a}).to_float() / ({b}).to_float())"),
            35 => format!("print(({a}).to_float().to_int())"),
            36 => format!("print(({a}).min({b}).max({c}))"),
            37 => format!("print({l}.index_of({a}))"),
            38 => format!("print([x * {a} for x in {l}])"),
            39 => format!("print({s}.len() + {l}.len() * {a})"),
            40 => format!("print(deep({a}))"),
            41 => format!("print(deep({}))", *rng.pick(&["1000", "10000", "100000", "1000000", "100000000"])),
            42 => format!("print(({a}).to_float() * 1e308 * 10.0)"),
            43 => format!("print((0.0 / 0.0).to_int())"),
            44 => format!("print(({a}).to_float().to_int() + ({b}).to_float().to_int())"),
            45 => format!("print({l}.sorted())"),
            46 => format!("mut st := Set[int]()\n    st.add({a})\n    print(st.contains({b}))"),
            47 => format!("ch := Channel[int]({a})\n    ch.send({b})\n    print(ch.recv() else 0)"),
            48 => format!("mu := Mutex({a})\n    mu.unlock({b})"),
            49 => format!("print(char.from_code({a}))"),
            50 => format!("print('x'.to_digit({a}))"),
            51 => format!("print({s}.chars()[{a}])"),
            52 => format!("print({s}.to_float())"),
            53 => format!("print([[{a}], [{b}]][{c}][{a}])"),
            54 => format!("print(({a}).to_u8() + ({b}).to_u8())"),
            55 => format!("print(({a}).to_i32() * ({b}).to_i32())"),
            56 => format!("print(({a}).to_i8() - ({b}).to_i8())"),
            57 => format!("mut xs := {l}\n    print(xs.pop())\n    print(xs.pop())\n    print(xs.first())"),
            58 => format!("mut xs := {l}\n    xs[{a}] = {b}\n    print(xs)"),
            59 => format!("mut xs := {l}\n    xs.extend(xs)\n    xs.extend(xs)\n    print(xs.len())"),
            60 => format!("mut m := {{\"a\": {a}}}\n    m[\"a\"] += {b}\n    m[\"z\"] += 1\n    print(m)"),
            61 => format!("print({l}.max() else {a})"),
            62 => format!("print({s}.lines().len() + {s}.trim().len())"),
            63 => format!("print({s}.to_upper() == {t}.to_lower())"),
            64 => format!("print(Pt(a: {a}, b: {b}) == Pt(a: {b}, b: {a}))"),
            _ => format!("print(Pt(a: {a}, b: {b}).sum() * {c})"),
        };
        out.push_str(&format!("    # probe {k}\n    {body}\n"));
    }
    out.push_str(PROBE_DEFS);
    out
}

// ---------------------------------------------------------------------------
// Mutation (fuzz.mut.1)

const POOL: [&str; 24] = [
    "(", ")", "[", "]", "{", "}", "+", "-", "*", "/", ":=", "=", "==", "..", ",", "\"", "fn", "mut", "match", "else", "?", "0", "9999999999999999999", "spawn",
];

/// Split into whitespace and non-whitespace pieces so they concatenate back.
fn pieces(src: &str) -> Vec<String> {
    let chars: Vec<char> = src.chars().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let mut j = i + 1;
        if c == '"' {
            while j < chars.len() && chars[j] != '"' && chars[j] != '\n' {
                j += if chars[j] == '\\' { 2 } else { 1 };
            }
            j = (j + 1).min(chars.len());
        } else if c.is_alphanumeric() || c == '_' {
            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                j += 1;
            }
        } else if c == ' ' || c == '\n' {
            while j < chars.len() && chars[j] == c {
                j += 1;
            }
        } else if j < chars.len() && matches!((c, chars[j]), (':', '=') | ('=', '=') | ('!', '=') | ('<', '=') | ('>', '=') | ('.', '.') | ('+', '=') | ('-', '=') | ('=', '>')) {
            j += 1;
        }
        out.push(chars[i..j.min(chars.len())].iter().collect());
        i = j;
    }
    out
}

/// One random edit of `src`. Returns a description of the edit for the report.
pub fn mutate(src: &str, seed: u64) -> (String, String) {
    let mut rng = Rng::new(seed ^ 0x6D75_7461_7465);
    let mut ps = pieces(src);
    let words: Vec<usize> = (0..ps.len()).filter(|&i| !ps[i].starts_with([' ', '\n'])).collect();
    if words.is_empty() {
        return (src.to_string(), "none".into());
    }
    let at = *rng.pick(&words);
    let what;
    match rng.below(10) {
        0 | 1 => {
            what = format!("delete `{}`", ps[at]);
            ps.remove(at);
        }
        2 => {
            what = format!("duplicate `{}`", ps[at]);
            let p = ps[at].clone();
            ps.insert(at, format!("{p} "));
        }
        3 => {
            if let Some(&next) = words.iter().find(|&&w| w > at) {
                what = format!("swap `{}` and `{}`", ps[at], ps[next]);
                ps.swap(at, next);
            } else {
                what = "none".into();
            }
        }
        4 | 5 => {
            let t = *rng.pick(&POOL);
            what = format!("replace `{}` with `{t}`", ps[at]);
            ps[at] = t.to_string();
        }
        6 => {
            let t = *rng.pick(&POOL);
            what = format!("insert `{t}` before `{}`", ps[at]);
            ps.insert(at, format!("{t} "));
        }
        7 => {
            // Delete the whole line holding the piece.
            let start = ps[..at].iter().rposition(|p| p.contains('\n')).map_or(0, |i| i + 1);
            let end = ps[at..].iter().position(|p| p.contains('\n')).map_or(ps.len(), |i| at + i + 1);
            what = format!("delete a line starting `{}`", ps[start].trim());
            ps.drain(start..end);
        }
        8 => {
            // Change the indentation of the line.
            let start = ps[..at].iter().rposition(|p| p.contains('\n')).map_or(0, |i| i + 1);
            let shift = *rng.pick(&[1usize, 2, 3, 4, 8]);
            what = format!("indent a line by {shift}");
            ps.insert(start, " ".repeat(shift));
        }
        _ => {
            // Truncate in the middle of the file.
            what = "truncate".into();
            ps.truncate(at + 1);
        }
    }
    (ps.concat(), what)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn programs_are_deterministic_and_vary() {
        assert_eq!(program(7), program(7));
        assert_ne!(program(7), program(8));
    }

    #[test]
    fn pieces_concatenate_back() {
        let src = program(3);
        assert_eq!(pieces(&src).concat(), src);
    }
}
