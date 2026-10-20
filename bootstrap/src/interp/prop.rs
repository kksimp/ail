//! Property tests: a `@test` with parameters runs on generated inputs, and a
//! failing input is shrunk (test.prop.1-5).

use super::*;
use crate::ast::TypeRef;

/// What a property run reports besides pass or fail (test.prop.4).
pub struct PropInfo {
    /// Cases run, including the failing one.
    pub cases: u64,
    pub seed: u64,
    /// The shrunk failing input as AIL literals, `a = 0, b = [1]`.
    pub input: Option<String>,
}

/// A generatable type (test.prop.2). Structs and enums are by index and read
/// from the program, so recursive enums need no infinite value here.
#[derive(Clone)]
enum G {
    Int,
    Float,
    Bool,
    Char,
    Str,
    Bytes,
    List(Box<G>),
    Set(Box<G>),
    Map(Box<G>, Box<G>),
    Opt(Box<G>),
    Struct(usize),
    Enum(usize),
}

/// splitmix64: small, fast, and the same on every platform (test.prop.3).
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        if n == 0 { 0 } else { self.next() % n }
    }

    fn range(&mut self, lo: i64, hi: i64) -> i64 {
        lo + self.below((hi - lo) as u64 + 1) as i64
    }
}

/// FNV-1a: the default seed of a property is a hash of its name, so a plain
/// `ail test` run is reproducible (test.prop.3).
pub fn default_seed(name: &str) -> u64 {
    name.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3))
}

/// How deep generated values nest before they stop growing.
const MAX_DEPTH: u32 = 3;
/// Candidate inputs tried while shrinking one failure.
const SHRINK_BUDGET: u32 = 2000;

impl<'a, W: Write> Interp<'a, W> {
    fn gen_of(&self, t: &TypeRef) -> G {
        match &t.kind {
            TypeKind::Optional(inner) => G::Opt(Box::new(self.gen_of(inner))),
            TypeKind::Named { name, args } => match name.as_str() {
                "int" | "i64" => G::Int,
                "float" | "f64" => G::Float,
                "bool" => G::Bool,
                "char" => G::Char,
                "str" => G::Str,
                "bytes" => G::Bytes,
                "List" => G::List(Box::new(self.gen_of(&args[0]))),
                "Set" => G::Set(Box::new(self.gen_of(&args[0]))),
                "Map" => G::Map(Box::new(self.gen_of(&args[0])), Box::new(self.gen_of(&args[1]))),
                _ => match self.structs.get(name.as_str()) {
                    Some(&i) => G::Struct(i),
                    None => G::Enum(self.enums[name.as_str()]),
                },
            },
            _ => unreachable!("checked: parameter types are generatable (test.prop.2)"),
        }
    }

    fn struct_gens(&self, i: usize) -> Vec<G> {
        self.prog.structs[i].fields.iter().map(|f| self.gen_of(&f.ty)).collect()
    }

    fn variant_gens(&self, e: usize, v: usize) -> Vec<G> {
        self.prog.enums[e].variants[v].fields.iter().map(|f| self.gen_of(&f.ty)).collect()
    }

    /// The variant with the fewest fields: the simplest one to build.
    fn simplest_variant(&self, e: usize) -> usize {
        let vs = &self.prog.enums[e].variants;
        (0..vs.len()).min_by_key(|&i| vs[i].fields.len()).unwrap_or(0)
    }

    /// A random value; `size` grows from 1 to 100 over the run (test.prop.3).
    fn generate(&self, g: &G, rng: &mut Rng, size: i64, depth: u32) -> Value {
        let bound = 10 + size * 10;
        let items = |rng: &mut Rng| if depth >= MAX_DEPTH { 0 } else { rng.below((size / 10).min(10) as u64 + 2) };
        match g {
            G::Int => Value::Int(match rng.below(20) {
                0 => [0, 1, -1][rng.below(3) as usize],
                1 => rng.range(-(1 << 31), 1 << 31),
                _ => rng.range(-bound, bound),
            }),
            G::Float => Value::Float(if rng.below(20) == 0 { 0.0 } else { rng.range(-bound * 100, bound * 100) as f64 / 100.0 }),
            G::Bool => Value::Bool(rng.below(2) == 0),
            G::Char => Value::Char(self.gen_char(rng)),
            G::Str => {
                let n = rng.below((size / 4) as u64 + 2);
                Value::Str((0..n).map(|_| self.gen_char(rng)).collect::<String>().into())
            }
            G::Bytes => {
                let n = rng.below((size / 4) as u64 + 2);
                // Mostly any byte, sometimes the edges, so UTF-8 and text paths both get exercised.
                super::bytes::bytes_value((0..n).map(|_| match rng.below(8) {
                    0 => 0,
                    1 => 255,
                    2 => b'a' + rng.below(26) as u8,
                    _ => rng.below(256) as u8,
                }).collect())
            }
            G::List(inner) => {
                let n = items(rng);
                Value::List(Rc::new((0..n).map(|_| self.generate(inner, rng, size, depth + 1)).collect()))
            }
            G::Set(inner) => {
                let n = items(rng);
                let mut t = Table::default();
                for _ in 0..n {
                    t.insert(self.generate(inner, rng, size, depth + 1), None);
                }
                Value::Set(Rc::new(t))
            }
            G::Map(k, v) => {
                let n = items(rng);
                let mut t = Table::default();
                for _ in 0..n {
                    let key = self.generate(k, rng, size, depth + 1);
                    t.insert(key, Some(self.generate(v, rng, size, depth + 1)));
                }
                Value::Map(Rc::new(t))
            }
            G::Opt(inner) => {
                if depth >= MAX_DEPTH || rng.below(4) == 0 { Value::None } else { self.generate(inner, rng, size, depth + 1) }
            }
            G::Struct(i) => {
                Value::Struct(*i, Rc::new(self.struct_gens(*i).iter().map(|g| self.generate(g, rng, size, depth + 1)).collect()))
            }
            G::Enum(e) => {
                let n = self.prog.enums[*e].variants.len();
                let v = if depth >= MAX_DEPTH { self.simplest_variant(*e) } else { rng.below(n as u64) as usize };
                let fields = self.variant_gens(*e, v).iter().map(|g| self.generate(g, rng, size, depth + 1)).collect();
                Value::Enum(*e, v, Rc::new(fields))
            }
        }
    }

    fn gen_char(&self, rng: &mut Rng) -> char {
        const ODD: &[char] = &['\n', '\t', 'é', 'λ', '日', '"', '\\'];
        if rng.below(12) == 0 {
            ODD[rng.below(ODD.len() as u64) as usize]
        } else {
            char::from(rng.range(0x20, 0x7e) as u8)
        }
    }

    /// The simplest value of a type, the end of every shrink.
    fn minimal(&self, g: &G) -> Value {
        match g {
            G::Int => Value::Int(0),
            G::Float => Value::Float(0.0),
            G::Bool => Value::Bool(false),
            G::Char => Value::Char('a'),
            G::Str => Value::Str("".into()),
            G::Bytes => super::bytes::bytes_value(Vec::new()),
            G::List(_) => Value::List(Rc::new(Vec::new())),
            G::Set(_) => Value::Set(Rc::new(Table::default())),
            G::Map(..) => Value::Map(Rc::new(Table::default())),
            G::Opt(_) => Value::None,
            G::Struct(i) => Value::Struct(*i, Rc::new(self.struct_gens(*i).iter().map(|g| self.minimal(g)).collect())),
            G::Enum(e) => self.minimal_variant(*e, self.simplest_variant(*e)),
        }
    }

    fn minimal_variant(&self, e: usize, v: usize) -> Value {
        Value::Enum(e, v, Rc::new(self.variant_gens(e, v).iter().map(|g| self.minimal(g)).collect()))
    }

    /// Simpler values than `v`, simplest first (test.prop.5): ints halve,
    /// lists and strings shorten, enums move to an earlier variant.
    fn shrinks(&self, g: &G, v: &Value) -> Vec<Value> {
        let mut out = Vec::new();
        match (g, v) {
            (G::Int, Value::Int(n)) if *n != 0 => {
                out.push(Value::Int(0));
                out.push(Value::Int(n / 2));
                if *n < 0 {
                    out.push(Value::Int(n.saturating_neg()));
                }
                out.push(Value::Int(n - n.signum()));
            }
            (G::Float, Value::Float(x)) if *x != 0.0 => {
                out.push(Value::Float(0.0));
                out.push(Value::Float(x.trunc()));
                out.push(Value::Float(x / 2.0));
            }
            (G::Bool, Value::Bool(true)) => out.push(Value::Bool(false)),
            (G::Char, Value::Char(c)) if *c != 'a' => out.push(Value::Char('a')),
            (G::Str, Value::Str(s)) => {
                let cs: Vec<char> = s.chars().collect();
                let text = |xs: &[char]| Value::Str(xs.iter().collect::<String>().into());
                if !cs.is_empty() {
                    out.push(text(&[]));
                    if cs.len() > 1 {
                        out.push(text(&cs[..cs.len() / 2]));
                        out.push(text(&cs[cs.len() / 2..]));
                    }
                    for i in 0..cs.len().min(8) {
                        let mut d = cs.clone();
                        d.remove(i);
                        out.push(text(&d));
                    }
                    for (i, c) in cs.iter().enumerate().take(8) {
                        if *c != 'a' {
                            let mut d = cs.clone();
                            d[i] = 'a';
                            out.push(text(&d));
                        }
                    }
                }
            }
            (G::Bytes, Value::Bytes(b)) => {
                let make = |xs: &[u8]| super::bytes::bytes_value(xs.to_vec());
                if !b.is_empty() {
                    out.push(make(&[]));
                    if b.len() > 1 {
                        out.push(make(&b[..b.len() / 2]));
                        out.push(make(&b[b.len() / 2..]));
                    }
                    for i in 0..b.len().min(8) {
                        let mut d = (**b).clone();
                        d.remove(i);
                        out.push(super::bytes::bytes_value(d));
                    }
                    for (i, x) in b.iter().enumerate().take(8) {
                        if *x != 0 {
                            let mut d = (**b).clone();
                            d[i] = 0;
                            out.push(super::bytes::bytes_value(d));
                        }
                    }
                }
            }
            (G::List(inner), Value::List(xs)) => {
                for shorter in shorten(xs) {
                    out.push(Value::List(Rc::new(shorter)));
                }
                for (i, x) in xs.iter().enumerate().take(8) {
                    for s in self.shrinks(inner, x).into_iter().take(3) {
                        let mut d = (**xs).clone();
                        d[i] = s;
                        out.push(Value::List(Rc::new(d)));
                    }
                }
            }
            (G::Set(inner), Value::Set(t)) => {
                let keys = t.key_list();
                for shorter in shorten(&keys) {
                    out.push(Value::Set(Rc::new(table(shorter, None))));
                }
                for (i, k) in keys.iter().enumerate().take(8) {
                    for s in self.shrinks(inner, k).into_iter().take(3) {
                        let mut d = keys.clone();
                        d[i] = s;
                        out.push(Value::Set(Rc::new(table(d, None))));
                    }
                }
            }
            (G::Map(kg, vg), Value::Map(t)) => {
                let entries: Vec<(Value, Value)> = t.entries().map(|(k, v)| (k.clone(), v.clone())).collect();
                let build = |es: Vec<(Value, Value)>| {
                    let mut t = Table::default();
                    for (k, v) in es {
                        t.insert(k, Some(v));
                    }
                    Value::Map(Rc::new(t))
                };
                for shorter in shorten(&entries) {
                    out.push(build(shorter));
                }
                for (i, (k, v)) in entries.iter().enumerate().take(8) {
                    for s in self.shrinks(kg, k).into_iter().take(2) {
                        let mut d = entries.clone();
                        d[i].0 = s;
                        out.push(build(d));
                    }
                    for s in self.shrinks(vg, v).into_iter().take(3) {
                        let mut d = entries.clone();
                        d[i].1 = s;
                        out.push(build(d));
                    }
                }
            }
            (G::Opt(inner), x) if !matches!(x, Value::None) => {
                out.push(Value::None);
                out.extend(self.shrinks(inner, x));
            }
            (G::Struct(i), Value::Struct(_, fields)) => {
                let gs = self.struct_gens(*i);
                for (j, f) in fields.iter().enumerate() {
                    for s in self.shrinks(&gs[j], f) {
                        let mut d = (**fields).clone();
                        d[j] = s;
                        out.push(Value::Struct(*i, Rc::new(d)));
                    }
                }
            }
            (G::Enum(e), Value::Enum(_, v, fields)) => {
                for earlier in 0..*v {
                    out.push(self.minimal_variant(*e, earlier));
                }
                let gs = self.variant_gens(*e, *v);
                for (j, f) in fields.iter().enumerate() {
                    for s in self.shrinks(&gs[j], f) {
                        let mut d = (**fields).clone();
                        d[j] = s;
                        out.push(Value::Enum(*e, *v, Rc::new(d)));
                    }
                }
            }
            _ => {}
        }
        out
    }

}

impl<'a> Interp<'a, Vec<u8>> {
    /// Run `f` once on `args`; the position and message of a failure.
    fn try_case(&mut self, f: &'a FnDecl, args: &[Value]) -> Option<(Pos, String)> {
        self.depth = 0;
        let _ = self.out.flush();
        self.out.get_mut().clear();
        match self.call(f, None, args.to_vec(), f.pos) {
            Ok((Value::Fail(fail), _)) => {
                let text = self.text(&fail.error, f.pos).unwrap_or_else(|t| t.message);
                Some((fail.trace.first().copied().unwrap_or(f.pos), format!("error: {text}")))
            }
            Ok(_) => None,
            Err(t) => Some((t.pos, format!("trap[{}]: {}", t.rule, t.message))),
        }
    }

    /// Run a property (test.prop.3-5): `cases` generated inputs from `seed`;
    /// on a failure, shrink it and report the smallest failing input.
    pub(super) fn run_property(&mut self, f: &'a FnDecl, cases: u64, seed: u64) -> (Pos, Option<String>, PropInfo) {
        let gens: Vec<G> = f.params.iter().map(|p| self.gen_of(&p.ty)).collect();
        let mut rng = Rng(seed);
        for n in 1..=cases {
            let size = 1 + (n - 1) as i64 * 99 / (cases.max(2) - 1) as i64;
            let mut args: Vec<Value> = gens.iter().map(|g| self.generate(g, &mut rng, size, 0)).collect();
            let Some(mut fail) = self.try_case(f, &args) else { continue };
            let mut budget = SHRINK_BUDGET;
            'shrink: loop {
                for i in 0..args.len() {
                    for cand in self.shrinks(&gens[i], &args[i]) {
                        if budget == 0 {
                            break 'shrink;
                        }
                        budget -= 1;
                        let mut trial = args.clone();
                        trial[i] = cand;
                        if let Some(again) = self.try_case(f, &trial) {
                            args = trial;
                            fail = again;
                            continue 'shrink;
                        }
                    }
                }
                break;
            }
            // Leave the shrunk run's output for the report.
            let _ = self.try_case(f, &args);
            let shown: Vec<String> = f
                .params
                .iter()
                .zip(&args)
                .map(|(p, v)| format!("{} = {}", p.name, self.describe(v, f.pos).unwrap_or_default()))
                .collect();
            let info = PropInfo { cases: n, seed, input: Some(shown.join(", ")) };
            return (fail.0, Some(fail.1), info);
        }
        (f.pos, None, PropInfo { cases, seed, input: None })
    }
}

/// Shorter versions of a sequence: empty, each half, each one removed.
fn shorten<T: Clone>(xs: &[T]) -> Vec<Vec<T>> {
    let mut out = Vec::new();
    if xs.is_empty() {
        return out;
    }
    out.push(Vec::new());
    if xs.len() > 1 {
        out.push(xs[..xs.len() / 2].to_vec());
        out.push(xs[xs.len() / 2..].to_vec());
    }
    for i in 0..xs.len().min(8) {
        let mut d = xs.to_vec();
        d.remove(i);
        out.push(d);
    }
    out
}

fn table(keys: Vec<Value>, v: Option<Value>) -> Table {
    let mut t = Table::default();
    for k in keys {
        t.insert(k, v.clone());
    }
    t
}
