//! Effect inference (eff.infer.1-6), `@pure` (eff.pure.1) and capability
//! denial (eff.cap.1-3), ADRs 0163-0166.
//!
//! Runs after type checking and only reads the program and the checker's
//! `Resolved` facts. Every function gets the least set of effects that is
//! closed over its calls: a fixed point over the call graph.

use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use super::{Call, Resolved};
use crate::ast::{BinOp, Block, Cond, Else, Expr, FnDecl, InterpPart, Program, Stmt, TypeKind, Within, is_prelude};
use crate::diag::{Diagnostic, Pos};

/// The effects (eff.infer.1). An `unsafe` block gives `unsafe`; `spawn` gives `spawn`.
const EFFECT_NAMES: &[&str] = &["io", "fail", "mutates", "unsafe", "spawn"];

/// Capabilities: the ambient resources a function may reach (eff.cap.1).
/// `std.net` gives `net` (std.net.1).
pub const CAPABILITIES: &[&str] = &["fs", "env", "proc", "clock", "random", "net", "net.serve", "ui"];

/// What `ail run --deterministic` denies (std.random.5, eff.det.2, ADR 0207): the
/// capabilities whose results the program's inputs don't fix.
pub const DETERMINISTIC_DENY: [&str; 4] = ["clock", "net", "proc", "random"];

/// Effects that a `@pure` function may not have (eff.pure.1): everything but
/// `fail` and `mutates`, which its signature already shows.
fn impure(e: &str) -> bool {
    !matches!(e, "fail" | "mutates")
}

/// Effects that pass from a callee to its caller (eff.infer.3). `fail` and
/// `mutates` are read from a function's own signature: a caller handles a
/// failure or propagates it (and is then fallible itself), and a `mut`
/// argument is the caller's own place.
fn propagates(e: &str) -> bool {
    impure(e)
}

/// The effects of a standard-module function `module.name` (eff.infer.4),
/// keyed by the qualified names of the module table in
/// `docs/design/ai-first-plan.md` section 4.
pub fn std_effects(module: &str, name: &str) -> Option<&'static [&'static str]> {
    let (names, effects): (&[&str], &'static [&'static str]) = match module {
        "fs" => (
            &[
                "read_text",
                "write_text",
                "append_text",
                "read_bytes",
                "write_bytes",
                "append_bytes",
                "exists",
                "list_dir",
                "remove",
                "make_dir",
                "walk",
                "glob",
                "metadata",
                "rename",
                "copy",
            ],
            &["fs", "io"],
        ),
        // std.hash.1: values only. std.compress.1: the byte functions are
        // values only; the file and directory ones read and write files.
        "hash" => (&["crc32", "hash64", "hmac_sha256", "sha256", "sha384", "sha512"], &[]),
        "compress" if matches!(name, "gzip_file" | "gunzip_file" | "zip_dir" | "unzip") => {
            (&["gzip_file", "gunzip_file", "zip_dir", "unzip"], &["fs", "io"])
        }
        "compress" => (&["deflate", "gunzip", "gzip", "inflate", "unzlib", "zlib"], &[]),
        "env" => (&["args", "get", "cwd", "home", "program", "vars"], &["env"]),
        "proc" => (&["run", "run_bytes", "which", "start", "wait", "kill"], &["proc", "io"]),
        "io" => (&["read_line", "read_all", "read_bytes", "eprint", "exit"], &["io"]),
        // std.net.1: every request reaches the network.
        "net" if matches!(name, "serve" | "listen") => {
            // std.net.14: a server needs the narrower capability as well.
            return Some(&["net", "net.serve", "io"]);
        }
        "net" => {
            (&["get", "post", "request", "get_bytes", "post_bytes", "request_bytes", "options", "connect", "connect_tls"], &["net", "io"])
        }
        // std.path.1: text only, so no effects.
        "path" => (&["base", "dir", "ext", "join"], &[]),
        "random" => (&["int", "float", "seed", "shuffle"], &["random"]),
        // std.regex.1: text in, text out, so no effects.
        "regex" => (
            &["compile", "is_match", "find", "find_all", "group_all", "captures", "replace", "replace_first", "split", "escape"],
            &[],
        ),
        "time" => (&["now_ms", "sleep_ms"], &["clock"]),
        // std.tz.1-2: reads the tz database files, and `TZ`.
        "tz" => (&["offset", "system"], &["fs", "env"]),
        // std.tensor.1: values only, so no effects.
        "tensor" => (&["from", "zeros"], &[]),
        // conc.cancel.2: reads the running task's flag; no capability.
        "task" => (&["cancelled", "check", "select"], &[]),
        // std.ui.9: opening a window and every `ui.Window` method.
        "ui" => (&["open", "clipboard_get", "clipboard_set", "Window"], &["ui", "io"]),
        _ => return None,
    };
    names.contains(&name).then_some(effects)
}

/// The inferred effects of a whole program.
pub struct Effects<'p> {
    prog: &'p Program,
    /// Per function: each effect it has directly,
    /// with the source that gives it (`print`, `fs.write_text`, `T ! E`).
    direct: Vec<BTreeMap<&'static str, String>>,
    /// Per function: the functions it may call, in order.
    calls: Vec<Vec<usize>>,
    /// Per function: the closed effect set (eff.infer.3).
    sets: Vec<BTreeSet<&'static str>>,
}

impl<'p> Effects<'p> {
    pub fn infer(prog: &'p Program, resolved: &Resolved) -> Self {
        let nodes = prog.fns.len();
        let mut w = Walker::new(prog, resolved);
        let mut direct = Vec::with_capacity(nodes);
        let mut calls = Vec::with_capacity(nodes);
        for f in &prog.fns {
            w.direct = signature_effects(f);
            w.calls = Vec::new();
            if let Some(body) = &f.body {
                w.block(body);
            }
            direct.push(std::mem::take(&mut w.direct));
            calls.push(std::mem::take(&mut w.calls));
        }
        // Constants add nothing: they are computed at compile time, and
        // `check_constants` keeps what they call free of effects (ct.const.2).
        // eff.infer.3: the least fixed point.
        let mut sets: Vec<BTreeSet<&'static str>> = direct.iter().map(|d| d.keys().copied().collect()).collect();
        loop {
            let mut changed = false;
            for i in 0..nodes {
                let add: Vec<&'static str> = calls[i]
                    .iter()
                    .flat_map(|&j| sets[j].iter().copied())
                    .filter(|e| propagates(e) && !sets[i].contains(e))
                    .collect();
                if !add.is_empty() {
                    sets[i].extend(add);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        Effects { prog, direct, calls, sets }
    }

    /// The effects that running `body` has, each with its source: the
    /// block's own operations (`print`), or a chain of calls such as
    /// `log -> print` (conc.par.2, ADR 0237).
    pub fn of_block(&self, resolved: &Resolved, body: &Block) -> BTreeMap<&'static str, String> {
        let mut w = Walker::new(self.prog, resolved);
        w.block(body);
        let mut out = w.direct;
        for &j in &w.calls {
            for &e in self.sets[j].iter().filter(|e| propagates(e)) {
                out.entry(e).or_insert_with(|| self.chain(j, e));
            }
        }
        out
    }

    /// The effects of expression `e`, like `of_block` (conc.par.12).
    pub fn of_expr(&self, resolved: &Resolved, e: &Expr) -> BTreeMap<&'static str, String> {
        let mut w = Walker::new(self.prog, resolved);
        w.expr(e);
        let mut out = w.direct;
        for &j in &w.calls {
            for &e in self.sets[j].iter().filter(|e| propagates(e)) {
                out.entry(e).or_insert_with(|| self.chain(j, e));
            }
        }
        out
    }

    /// The functions that function `i` may call (eff.infer.3).
    pub fn calls_of(&self, i: usize) -> &[usize] {
        &self.calls[i]
    }

    /// The effect set of function `i`, sorted (eff.infer.6).
    pub fn of(&self, i: usize) -> &BTreeSet<&'static str> {
        &self.sets[i]
    }

    fn node_name(&self, i: usize) -> String {
        display_name(self.prog, &self.prog.fns[i])
    }

    /// The shortest chain of calls from function `from` to the source of effect
    /// `e`, e.g. `main -> save -> fs.write_text` (eff.pure.1, eff.cap.1).
    pub fn chain(&self, from: usize, e: &str) -> String {
        let mut prev: Vec<Option<usize>> = vec![None; self.sets.len()];
        let mut seen = vec![false; self.sets.len()];
        let mut queue = VecDeque::from([from]);
        seen[from] = true;
        while let Some(i) = queue.pop_front() {
            if let Some(source) = self.direct[i].get(e) {
                let mut path = vec![source.clone()];
                let mut at = Some(i);
                while let Some(k) = at {
                    path.push(self.node_name(k));
                    at = prev[k];
                }
                path.reverse();
                return path.join(" -> ");
            }
            for &j in &self.calls[i] {
                if !seen[j] && self.sets[j].contains(e) {
                    seen[j] = true;
                    prev[j] = Some(i);
                    queue.push_back(j);
                }
            }
        }
        self.node_name(from)
    }
}

/// The stable id of a function (lane 2's `ail api` ids): `fn:name` or
/// `method:Type.name`.
pub fn fn_id(prog: &Program, f: &FnDecl) -> String {
    if f.owner.is_none() && f.top_level() {
        format!("fn:{}", f.name)
    } else {
        format!("method:{}", display_name(prog, f))
    }
}

/// `name` or `Type.name`; a function of an `impl` block is named after the
/// implementing type, one of a `trait` block after the trait.
fn display_name(prog: &Program, f: &FnDecl) -> String {
    match f.within {
        Within::TopLevel => f.full_name(),
        Within::Trait(k) => format!("{}.{}", prog.traits[k].name, f.name),
        Within::Impl(k) => format!("{}.{}", prog.impls[k].target.name, f.name),
    }
}

/// Every program function's effect set by stable id (eff.infer.6). The
/// prelude's functions are left out. Functions that share an id (methods
/// of two traits with the same name on one type) are merged.
pub fn effects_of(prog: &Program, resolved: &Resolved) -> BTreeMap<String, BTreeSet<&'static str>> {
    let fx = Effects::infer(prog, resolved);
    let mut out: BTreeMap<String, BTreeSet<&'static str>> = BTreeMap::new();
    for (i, f) in prog.fns.iter().enumerate() {
        if !is_prelude(f.pos) {
            out.entry(fn_id(prog, f)).or_default().extend(fx.of(i).iter().copied());
        }
    }
    out
}

/// eff.pure.1: a `@pure` function has none of `io`, `unsafe`, `spawn` or a
/// capability. One diagnostic per function, naming each chain of calls.
pub fn check_pure(prog: &Program, resolved: &Resolved) -> Vec<Diagnostic> {
    if !prog.fns.iter().any(|f| f.has_attr("pure")) {
        return Vec::new();
    }
    let fx = Effects::infer(prog, resolved);
    let mut diags = Vec::new();
    for (i, f) in prog.fns.iter().enumerate() {
        let Some(attr) = f.attrs.iter().find(|a| a.name == "pure") else { continue };
        let bad: Vec<&str> = fx.of(i).iter().copied().filter(|e| impure(e)).collect();
        if bad.is_empty() {
            continue;
        }
        let name = display_name(prog, f);
        let msg = format!("`{name}` is `@pure` but {}", reasons(&fx, i, &bad));
        diags.push(
            Diagnostic::new("eff.pure.1", attr.pos, msg).help(format!(
                "remove `@pure`, or move the effect out of `{name}` and pass its result in as an argument"
            )),
        );
    }
    diags
}

/// contract.decl.3: a contract's condition has no effect but `fail`. It
/// must not print, reach a capability or change a `mut` argument, so that
/// turning checks off (contract.run.3) can't change what a program does.
pub fn check_contracts(prog: &Program, resolved: &Resolved) -> Vec<Diagnostic> {
    if prog.fns.iter().all(|f| f.contracts.is_empty()) && prog.structs.iter().all(|s| s.invariants.is_empty()) {
        return Vec::new();
    }
    let fx = Effects::infer(prog, resolved);
    let mut w = Walker::new(prog, resolved);
    let mut diags = Vec::new();
    let named = prog
        .fns
        .iter()
        .flat_map(|f| f.contracts.iter().map(move |c| (c, f.full_name(), if c.ensures { "ensures" } else { "requires" })))
        .chain(prog.structs.iter().flat_map(|s| s.invariants.iter().map(move |c| (c, s.name.clone(), "invariant"))));
    for (c, owner, word) in named {
        w.direct = BTreeMap::new();
        w.calls = Vec::new();
        w.expr(&c.cond);
        for (_, e) in &c.olds {
            w.expr(e);
        }
        let mut bad: BTreeMap<&str, String> = BTreeMap::new();
        for (e, source) in &w.direct {
            bad.insert(e, source.clone());
        }
        for &j in &w.calls {
            for &e in fx.of(j) {
                bad.entry(e).or_insert_with(|| fx.chain(j, e));
            }
        }
        bad.remove("fail");
        if bad.is_empty() {
            continue;
        }
        let what: Vec<String> = bad.iter().map(|(e, chain)| format!("`{e}` via {chain}")).collect();
        diags.push(
            Diagnostic::new(
                "contract.decl.3",
                c.pos,
                format!("the `{word}` condition of `{owner}` must be pure, but it has {}", what.join(", ")),
            )
            .help("compute the value in the body, or check it with `assert`; a condition may only read its inputs"),
        );
    }
    diags
}

/// ct.const.2 (ADR 0225) and ct.eval.1 (ADR 0226): a constant is computed at compile time, so
/// everything its value may call must be compile-time-safe: no `io`,
/// `unsafe`, `spawn` or capability. One diagnostic per constant, naming each
/// chain of calls.
pub fn check_constants(prog: &Program, resolved: &Resolved) -> Vec<Diagnostic> {
    if prog.consts.is_empty() {
        return Vec::new();
    }
    let mut w = Walker::new(prog, resolved);
    let walked: Vec<_> = prog
        .consts
        .iter()
        .map(|c| {
            w.direct = BTreeMap::new();
            w.calls = Vec::new();
            w.expr(&c.value);
            (std::mem::take(&mut w.direct), std::mem::take(&mut w.calls))
        })
        .collect();
    // Most constants call nothing; infer effects only when one does.
    if walked.iter().all(|(direct, calls)| calls.is_empty() && !direct.keys().any(|e| impure(e))) {
        return Vec::new();
    }
    let fx = Effects::infer(prog, resolved);
    let mut diags = Vec::new();
    for (c, (direct, calls)) in prog.consts.iter().zip(&walked) {
        // Each forbidden effect with the chain that gives it, grouped by chain.
        let mut groups: Vec<(Vec<&str>, String)> = Vec::new();
        let mut note = |e: &'static str, chain: String| match groups.iter_mut().find(|(_, ch)| *ch == chain) {
            Some((es, _)) => {
                if !es.contains(&e) {
                    es.push(e)
                }
            }
            None => groups.push((vec![e], chain)),
        };
        for (&e, source) in direct.iter().filter(|(e, _)| impure(e)) {
            note(e, format!("{} -> {source}", c.name));
        }
        let mut bad: BTreeSet<&'static str> = BTreeSet::new();
        for &j in calls {
            bad.extend(fx.of(j).iter().copied().filter(|e| impure(e) && !direct.contains_key(e)));
        }
        for e in bad {
            let j = calls.iter().copied().find(|&j| fx.of(j).contains(e)).expect("an effect comes from a callee");
            note(e, format!("{} -> {}", c.name, fx.chain(j, e)));
        }
        if groups.is_empty() {
            continue;
        }
        let parts: Vec<String> = groups
            .iter()
            .map(|(es, chain)| {
                let es: Vec<String> = es.iter().map(|e| format!("`{e}`")).collect();
                let noun = if es.len() == 1 { "effect" } else { "effects" };
                format!("has {noun} {} via {chain}", es.join(" and "))
            })
            .collect();
        let msg = format!("`{}` is computed at compile time but {}", c.name, parts.join("; "));
        diags.push(
            Diagnostic::new("ct.const.2", c.pos, msg)
                .help("constants may call only functions without I/O or capabilities; compute this value in `main`"),
        );
    }
    diags
}

/// ct.derive.3 (ADR 0227): the effects that keep function `f` from running
/// at compile time, each with the chain of calls that gives it.
/// `fx` is inferred once for the program, so many functions share one walk.
pub fn impure_chains(fx: &Effects, f: usize) -> Vec<String> {
    fx.of(f).iter().filter(|e| impure(e)).map(|e| format!("effect `{e}` via {}", fx.chain(f, e))).collect()
}

/// eff.cap.1: `main` must not reach a denied capability. One diagnostic
/// per capability, at `main`.
pub fn check_denied(prog: &Program, resolved: &Resolved, denied: &[&str]) -> Vec<Diagnostic> {
    let Some(main) = prog.fns.iter().position(|f| f.name == "main" && f.owner.is_none() && f.top_level()) else {
        return Vec::new();
    };
    let fx = Effects::infer(prog, resolved);
    let pos: Pos = prog.fns[main].pos;
    fx.of(main)
        .iter()
        .filter(|e| denied.contains(e))
        .map(|e| {
            Diagnostic::new(
                "eff.cap.1",
                pos,
                format!("the program reaches the denied capability `{e}`: {}", fx.chain(main, e)),
            )
            .help(format!("remove the call, or run without denying `{e}`"))
        })
        .collect()
}

/// `has effects io and fs: f -> save -> fs.write_text`, grouping effects
/// that come from the same chain.
fn reasons(fx: &Effects, i: usize, bad: &[&str]) -> String {
    let mut groups: Vec<(Vec<&str>, String)> = Vec::new();
    for &e in bad {
        let chain = fx.chain(i, e);
        match groups.iter_mut().find(|(_, c)| *c == chain) {
            Some((es, _)) => es.push(e),
            None => groups.push((vec![e], chain)),
        }
    }
    let parts: Vec<String> = groups
        .iter()
        .map(|(es, chain)| {
            let es: Vec<String> = es.iter().map(|e| format!("`{e}`")).collect();
            let noun = if es.len() == 1 { "effect" } else { "effects" };
            format!("has {noun} {} via {chain}", es.join(" and "))
        })
        .collect();
    parts.join("; ")
}

/// `fail` from a `T ! E` or `!` return type, `mutates` from a `mut`
/// parameter or `mut self` (eff.infer.2).
fn signature_effects(f: &FnDecl) -> BTreeMap<&'static str, String> {
    let mut d = BTreeMap::new();
    if matches!(f.ret.as_ref().map(|t| &t.kind), Some(TypeKind::Fallible { .. })) {
        d.insert("fail", "its return type".to_string());
    }
    if f.receiver.is_some_and(|r| r.mutable) || f.params.iter().any(|p| p.mutable) {
        d.insert("mutates", "a `mut` parameter".to_string());
    }
    d
}

/// `x.name(...)` through trait `tr` on a value whose type has `impl` block
/// `k`: the call runs that block's method or the trait's default, so record
/// it as the call's target (eff.infer.5).
pub(super) fn record_impl_target(prog: &Program, resolved: &mut Resolved, pos: Pos, k: usize, tr: usize, name: &str) {
    let own = prog.impls[k].methods.iter().copied().find(|&m| prog.fns[m].name == name);
    let default =
        || prog.traits[tr].methods.iter().copied().find(|&m| prog.fns[m].name == name && prog.fns[m].body.is_some());
    if let Some(m) = own.or_else(default) {
        resolved.targets.insert(pos, m);
    }
}

/// The functions that `Trait.name` can run: each implementation's own, or
/// the trait's default (eff.infer.5, whole-program per ADR 0015).
fn implementations(prog: &Program, tr: usize, name: &str) -> Vec<usize> {
    let Some(decl) = prog.traits.get(tr) else { return Vec::new() };
    let default = decl.methods.iter().copied().find(|&m| prog.fns[m].name == name && prog.fns[m].body.is_some());
    let mut out: Vec<usize> = default.into_iter().collect();
    for imp in prog.impls.iter().filter(|imp| imp.trait_name == decl.name) {
        if let Some(m) = imp.methods.iter().copied().find(|&m| prog.fns[m].name == name) {
            out.push(m);
        }
    }
    out
}

struct Walker<'p, 'r> {
    prog: &'p Program,
    resolved: &'r Resolved,
    /// Free functions by name.
    fns: HashMap<&'p str, usize>,
    /// The implementations of the standard traits (eff.infer.5).
    std_impls: Vec<usize>,
    direct: BTreeMap<&'static str, String>,
    calls: Vec<usize>,
}

impl<'p, 'r> Walker<'p, 'r> {
    fn new(prog: &'p Program, resolved: &'r Resolved) -> Self {
        let mut w = Walker {
            prog,
            resolved,
            fns: HashMap::new(),
            std_impls: Vec::new(),
            direct: BTreeMap::new(),
            calls: Vec::new(),
        };
        for (i, f) in prog.fns.iter().enumerate() {
            if f.owner.is_none() && f.within == Within::TopLevel {
                w.fns.insert(f.name.as_str(), i);
            }
        }
        // eff.infer.5: operators, interpolation, `print` and the built-in
        // collection methods call the standard traits' implementations.
        let std = resolved.std;
        if !prog.traits.is_empty() {
            for (tr, name) in [(std.eq, "eq"), (std.ord, "cmp"), (std.to_str, "to_str"), (std.hash, "hash")] {
                w.std_impls.extend(implementations(prog, tr, name));
            }
            w.std_impls.sort_unstable();
            w.std_impls.dedup();
        }
        w
    }

    fn add(&mut self, e: &'static str, source: impl Into<String>) {
        self.direct.entry(e).or_insert_with(|| source.into());
    }

    fn call(&mut self, i: usize) {
        if !self.calls.contains(&i) {
            self.calls.push(i);
        }
    }

    fn std_traits(&mut self) {
        for k in 0..self.std_impls.len() {
            self.call(self.std_impls[k]);
        }
    }

    fn block(&mut self, b: &Block) {
        for s in b {
            self.stmt(s);
        }
    }

    fn stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let { value, otherwise, .. } => {
                self.expr(value);
                if let Some(o) = otherwise {
                    self.otherwise(o);
                }
            }
            Stmt::Assign { target, value, .. } => {
                self.ref_write(target);
                self.expr(target);
                self.expr(value);
            }
            Stmt::While { cond, body, .. } => {
                self.expr(cond);
                self.block(body);
            }
            Stmt::For { iter, body, .. } | Stmt::With { mutex: iter, body, .. } => {
                self.expr(iter);
                self.block(body);
            }
            Stmt::Break(_) | Stmt::Continue(_) | Stmt::Return(None, _) => {}
            Stmt::Return(Some(e), _) | Stmt::Expr(e) => self.expr(e),
            Stmt::Handle { value, otherwise } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
            Stmt::Defer(s, _) => self.stmt(s),
            // unsafe.block.5, eff.infer.4: an `unsafe` block gives `unsafe`.
            Stmt::Unsafe { body, .. } => {
                self.direct.entry("unsafe").or_insert_with(|| "an `unsafe` block".to_string());
                self.block(body);
            }
        }
    }

    /// eff.infer.2: a write through a `ref` (own.ref.4) gives `mutates`; the
    /// checker records the root position of each such write.
    fn ref_write(&mut self, place: &Expr) {
        if self.resolved.ref_writes.contains(&place.pos()) {
            self.direct.entry("mutates").or_insert_with(|| "a write through a `ref`".to_string());
        }
    }

    fn otherwise(&mut self, o: &Else) {
        match o {
            Else::Value(e) => self.expr(e),
            Else::Leave(s) => self.stmt(s),
            Else::Block { body, .. } => self.block(body),
        }
    }

    fn expr(&mut self, e: &Expr) {
        match e {
            Expr::Str(..)
            | Expr::Bytes(..)
            | Expr::Int(..)
            | Expr::Float(..)
            | Expr::Char(..)
            | Expr::Bool(..)
            | Expr::None(_)
            | Expr::Hole(_)
            | Expr::TypeArgs { .. } => {}
            // type.fn.2: naming a function as a value counts as calling it,
            // since the value can be called wherever it ends up.
            Expr::Name(_, pos) => {
                if let Some(&i) = self.resolved.fn_values.get(pos) {
                    self.call(i);
                }
            }
            Expr::Interp(parts, _) => {
                self.std_traits();
                for p in parts {
                    if let InterpPart::Expr(x) | InterpPart::Fmt(x, _) = p {
                        self.expr(x);
                    }
                }
            }
            Expr::List(items, _) | Expr::Set(items, _) => items.iter().for_each(|x| self.expr(x)),
            Expr::Comp { elem, value, clauses, .. } => {
                for c in clauses {
                    self.expr(&c.iter);
                    if let Some(cond) = &c.cond {
                        self.expr(cond);
                    }
                }
                self.expr(elem);
                if let Some(v) = value {
                    self.expr(v);
                }
            }
            Expr::Map(pairs, _) => {
                for (k, v) in pairs {
                    self.expr(k);
                    self.expr(v);
                }
            }
            Expr::Call { callee, args, .. } => {
                let name = match &**callee {
                    Expr::Name(n, _) | Expr::TypeArgs { name: n, .. } => Some(n.as_str()),
                    _ => None,
                };
                match name {
                    // eff.infer.4: `print` writes to standard output.
                    Some("print") => {
                        self.add("io", "print");
                        self.std_traits();
                    }
                    Some(n) => {
                        if let Some(&i) = self.fns.get(n) {
                            self.call(i);
                        }
                    }
                    None => self.expr(callee),
                }
                for a in args.iter().filter(|a| a.mutable) {
                    self.ref_write(&a.value);
                }
                args.iter().for_each(|a| self.expr(&a.value));
            }
            Expr::Method { recv, name, name_pos, args } => {
                if let Some(&i) = self.resolved.targets.get(name_pos) {
                    self.call(i);
                } else if let Some(&(tr, _)) = self.resolved.assoc_calls.get(name_pos) {
                    // `T.zero()` may run any implementation (trait.assoc.3, eff.infer.3).
                    for i in implementations(self.prog, tr, name) {
                        self.call(i);
                    }
                } else if let Some(call) = self.resolved.calls.get(name_pos) {
                    let (Call::Trait(tr) | Call::Qualified(tr)) = *call;
                    for i in implementations(self.prog, tr, name) {
                        self.call(i);
                    }
                } else if let Some(crate::stdlib::StdCall::Fn(module, f)) = self.resolved.std_calls.get(name_pos)
                    && let Expr::Name(m, _) = &**recv
                    && let Some(effects) = std_effects(module, f)
                {
                    for &e in effects {
                        self.add(e, format!("{m}.{name}"));
                    }
                } else if let Some(crate::stdlib::StdCall::Method(m)) = self.resolved.std_calls.get(name_pos)
                    && m.starts_with("net.")
                {
                    // std.net.14: the methods of the handles `net.serve`, `net.listen` and `net.connect` make.
                    for e in ["net", "io"] {
                        self.add(e, (*m).to_string());
                    }
                } else if let Some(crate::stdlib::StdCall::Method(m)) = self.resolved.std_calls.get(name_pos)
                    && let Some(effects) = std_effects("ui", "Window")
                {
                    for &e in effects {
                        self.add(e, format!("ui.Window.{m}"));
                    }
                } else if let Expr::Name(m, _) = &**recv
                    && self.prog.enums.iter().any(|d| d.name == *m)
                {
                    // `Enum.Variant(...)` builds a value.
                } else {
                    // A built-in method (`xs.sort()`, `m.get(k)`) may compare,
                    // hash or show its elements.
                    self.std_traits();
                }
                self.ref_write(recv);
                for a in args.iter().filter(|a| a.mutable) {
                    self.ref_write(&a.value);
                }
                self.expr(recv);
                args.iter().for_each(|a| self.expr(&a.value));
            }
            Expr::Field { recv, .. } => self.expr(recv),
            Expr::Index { recv, index, .. } => {
                self.expr(recv);
                self.expr(index);
            }
            // eff.infer.1: `spawn` starts a task (conc.task.1).
            Expr::Unary { op: crate::ast::UnOp::Spawn, operand, .. } => {
                self.add("spawn", "spawn");
                self.expr(operand);
            }
            Expr::Unary { operand, .. } | Expr::Ref { value: operand, .. } => self.expr(operand),
            Expr::Binary { op, lhs, rhs, .. } => {
                if !op.is_arithmetic() && !matches!(op, BinOp::And | BinOp::Or) {
                    self.std_traits();
                }
                self.expr(lhs);
                self.expr(rhs);
            }
            Expr::Range { lo, hi, .. } => {
                for x in [lo, hi].into_iter().flatten() {
                    self.expr(x);
                }
            }
            Expr::If { branches, els, .. } => {
                for (cond, body) in branches {
                    match cond {
                        Cond::Expr(x) | Cond::Bind { value: x, .. } => self.expr(x),
                    }
                    self.block(body);
                }
                if let Some(b) = els {
                    self.block(b);
                }
            }
            Expr::Match { scrutinee, arms, .. } => {
                self.expr(scrutinee);
                for arm in arms {
                    if let Some(g) = &arm.guard {
                        self.expr(g);
                    }
                    self.block(&arm.body);
                }
            }
            Expr::Try { expr, context, .. } => {
                self.expr(expr);
                if let Some(c) = context {
                    self.expr(c);
                }
            }
            Expr::Else { value, otherwise, .. } => {
                self.expr(value);
                self.otherwise(otherwise);
            }
        }
    }
}

/// `fs,proc` or `all` (eff.cap.2): the denied capabilities, or an error
/// naming the known ones.
pub fn parse_deny(list: &str) -> Result<Vec<&'static str>, String> {
    let mut out = Vec::new();
    for name in list.split(',').map(str::trim) {
        if name == "all" {
            out.extend_from_slice(CAPABILITIES);
        } else if let Some(&cap) = CAPABILITIES.iter().find(|c| **c == name) {
            out.push(cap);
        } else {
            let rule = if EFFECT_NAMES.contains(&name) { "eff.cap.3" } else { "eff.cap.2" };
            return Err(format!(
                "`{name}` is not a capability; `--deny` takes a comma-separated list of {} or `all` ({rule})",
                CAPABILITIES.join(", ")
            ));
        }
    }
    out.sort_unstable();
    out.dedup();
    Ok(out)
}

/// `ail effects` output (eff.infer.6): one `id {e, ...}` line per function,
/// or one JSON object `{"ail_api":1,"effects":{"fn:main":["io"],...}}`.
pub fn render(map: &BTreeMap<String, BTreeSet<&'static str>>, json: bool) -> String {
    if json {
        let items: Vec<String> = map
            .iter()
            .map(|(id, set)| {
                let es: Vec<String> = set.iter().map(|e| format!("\"{e}\"")).collect();
                format!("\"{}\":[{}]", id.replace('\\', "\\\\").replace('"', "\\\""), es.join(","))
            })
            .collect();
        format!("{{\"ail_api\":1,\"effects\":{{{}}}}}\n", items.join(","))
    } else {
        map.iter()
            .map(|(id, set)| format!("{id} {{{}}}\n", set.iter().copied().collect::<Vec<_>>().join(", ")))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{check::check, lexer::lex, parser::parse_with_prelude};

    /// Parse and check `src`, keeping the checker's facts even when it
    /// reports errors: the standard modules aren't resolved yet (lane 4).
    fn front(src: &str) -> (Program, Resolved) {
        let (tokens, ld) = lex(src);
        assert!(ld.is_empty(), "{ld:?}");
        let (prog, pd) = parse_with_prelude(tokens);
        assert!(pd.is_empty(), "{pd:?}");
        let (_, resolved) = check(&prog);
        (prog, resolved)
    }

    fn set(prog: &Program, resolved: &Resolved, id: &str) -> Vec<&'static str> {
        effects_of(prog, resolved)[id].iter().copied().collect()
    }

    const SAVE: &str = "use std.fs\nfn main()\n    n := count()\n    save(n)\nfn count() int\n    3\nfn save(n int)\n    fs.write_text(\"out.txt\", \"{n}\")\n";

    #[test]
    fn std_module_calls_give_capabilities() {
        let (prog, resolved) = front(SAVE);
        assert_eq!(set(&prog, &resolved, "fn:save"), ["fs", "io"]);
        assert_eq!(set(&prog, &resolved, "fn:main"), ["fs", "io"]);
        assert!(set(&prog, &resolved, "fn:count").is_empty());
        let src = "use std.time\nuse std.env\nuse std.proc\nfn main()\n    t := time.now_ms()\n    e := env.cwd()\n    r := proc.run(\"ls\", [])\n";
        let (prog, resolved) = front(src);
        assert_eq!(set(&prog, &resolved, "fn:main"), ["clock", "env", "io", "proc"]);
    }

    #[test]
    fn tz_reads_files_and_env() {
        let src = "use std.tz\nfn main()\n    print(tz.offset(\"UTC\", 0))\n    print(tz.system())\n";
        let (prog, resolved) = front(src);
        assert_eq!(set(&prog, &resolved, "fn:main"), ["env", "fs", "io"]);
    }

    #[test]
    fn deny_names_the_chain() {
        let (prog, resolved) = front(SAVE);
        let d = check_denied(&prog, &resolved, &["fs", "proc"]);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].rule, "eff.cap.1");
        assert_eq!(d[0].message, "the program reaches the denied capability `fs`: main -> save -> fs.write_text");
        assert!(check_denied(&prog, &resolved, &["proc", "clock"]).is_empty());
    }

    #[test]
    fn pure_violation_names_each_chain() {
        let src = "use std.fs\nfn main()\n    print(f(1))\n@pure fn f(x int) int\n    log(x)\n    save(x)\n    x\nfn log(x int)\n    print(x)\nfn save(n int)\n    fs.write_text(\"out.txt\", \"{n}\")\n";
        let (prog, resolved) = front(src);
        let d = check_pure(&prog, &resolved);
        assert_eq!(d.len(), 1);
        assert_eq!(
            d[0].message,
            "`f` is `@pure` but has effect `fs` via f -> save -> fs.write_text; has effect `io` via f -> log -> print"
        );
    }

    #[test]
    fn recursion_reaches_a_fixed_point() {
        let src = "fn main()\n    print(even(4))\nfn even(n int) bool\n    if n == 0\n        return true\n    odd(n - 1)\nfn odd(n int) bool\n    if n == 0\n        print(\"odd\")\n        return false\n    even(n - 1)\n";
        let (prog, resolved) = front(src);
        assert_eq!(set(&prog, &resolved, "fn:even"), ["io"]);
        assert_eq!(set(&prog, &resolved, "fn:odd"), ["io"]);
    }

    #[test]
    fn parse_deny_lists() {
        assert_eq!(parse_deny("proc,fs").unwrap(), ["fs", "proc"]);
        assert_eq!(parse_deny("all").unwrap(), ["clock", "env", "fs", "net", "net.serve", "proc", "random", "ui"]);
        assert!(parse_deny("io").unwrap_err().contains("(eff.cap.3)"));
        assert!(parse_deny("disk").unwrap_err().contains("(eff.cap.2)"));
    }

    #[test]
    fn commands_example_matches_expectation() {
        let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/commands.ail")).unwrap();
        let (prog, resolved) = front(&src);
        let want = include_str!("../../tests/effects/commands.effects");
        assert_eq!(render(&effects_of(&prog, &resolved), false), want);
        assert_eq!(
            render(&effects_of(&prog, &resolved), true),
            "{\"ail_api\":1,\"effects\":{\"fn:apply\":[\"io\",\"mutates\"],\"fn:main\":[\"fail\",\"io\"],\"fn:parse\":[\"fail\"],\"fn:total_added\":[\"fail\"]}}\n"
        );
    }
}
