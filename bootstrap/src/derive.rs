//! Derive generators (ct.derive.1-4, ct.gen.1-4, refl.info.1-3; ADRs 0227-0229,
//! 0306-0309).
//!
//! An `impl Trait for T` with no methods, for a trait declared `@derive(f)`,
//! is filled in before checking. The loader parses the program, adds one
//! wrapper function per such `impl` that calls `f` with `T`'s `TypeInfo`,
//! checks that program and runs the wrappers in the interpreter (ct.eval.2).
//! Each generator's text becomes a file `gen:impl:T.Trait` in the `impl`'s
//! module, whose tokens are spliced into the `impl`, and the program is
//! parsed again. The checker then checks the generated methods like
//! hand-written ones (ct.gen.1), and `remap` reports a problem in them at
//! the `impl` with a note in the generated text (ct.gen.3).

use crate::ast::*;
use crate::diag::{Diagnostic, Note, Pos};
use crate::lexer::{Part, Tok, Token, lex};

/// Whether the tokens may hold a deriving `impl` for a generator trait: an
/// `impl`, and `@derive` or a prelude generator trait's name.
pub fn wanted(tokens: &[Token]) -> bool {
    tokens.iter().any(|t| matches!(t.tok, Tok::Kw("impl")))
        && tokens.windows(2).any(|w| match (&w[0].tok, &w[1].tok) {
            (Tok::Sym("@"), Tok::Ident(n)) => n == "derive",
            (_, Tok::Ident(n)) => n == "ToJson" || n == "FromJson",
            _ => false,
        })
}

/// One deriving `impl` to fill in.
struct Job {
    imp: usize,
    generator: String,
    /// `Version.ToJson`
    label: String,
    /// The short name of the deriving type, for `fn T.name` items.
    type_name: String,
    /// The module of the trait, where the generator is called from.
    trait_module: String,
    /// The `TypeInfo(...)` expression for the type (refl.info.1).
    info: String,
}

type Build<'a> = &'a dyn Fn(Vec<Token>, Vec<SourceFile>) -> (Program, Vec<Diagnostic>);

/// Fill in every deriving `impl` of `prog`, parsed from `tokens` and `files`;
/// returns the program parsed again with the generated code, or the
/// diagnostics that stopped generation.
pub fn expand(
    prog: Program,
    diags: Vec<Diagnostic>,
    tokens: Vec<Token>,
    files: Vec<SourceFile>,
    build: Build,
) -> (Program, Vec<Diagnostic>) {
    let mut errors = Vec::new();
    let jobs = jobs(&prog, &mut errors);
    if jobs.is_empty() {
        let mut diags = diags;
        diags.extend(errors);
        return (prog, diags);
    }
    let texts = match generate(&prog, &jobs, &tokens, &files, build, &mut errors) {
        Ok(texts) => texts,
        Err(stopped) => return stopped,
    };
    // Splice each generated text into its `impl`, as a file of its own.
    let mut inserts: Vec<(usize, Vec<Token>)> = Vec::new();
    let mut gen_files = Vec::new();
    let mut generated = Vec::new();
    for (job, text) in jobs.iter().zip(texts) {
        let Some(text) = text else { continue };
        let imp = &prog.impls[job.imp];
        let text = crate::fmt::format(&text).unwrap_or(text);
        let index = files.len() + gen_files.len();
        let base = index * FILE_LINES;
        let (mut toks, lex_diags) = lex(&text);
        toks.pop(); // Eof
        if let Some(d) = lex_diags.into_iter().next() {
            errors.push(at_impl(imp.pos, job, d.pos, format!("the generated text doesn't lex: {}", d.message)));
            continue;
        }
        shift(&mut toks, base);
        let Some(split) = split(toks, job, imp.pos, &mut errors) else { continue };
        let Some(header) = tokens.iter().position(|t| t.tok == Tok::Kw("impl") && t.pos == imp.pos) else { continue };
        let Some(newline) = tokens[header..].iter().position(|t| t.tok == Tok::Newline) else { continue };
        let mut spliced = Vec::new();
        if let (Some(first), Some(last)) = (split.0.first(), split.0.last()) {
            spliced.push(Token { tok: Tok::Indent, pos: first.pos });
            let end = last.pos;
            spliced.extend(split.0);
            spliced.push(Token { tok: Tok::Dedent, pos: end });
        }
        spliced.extend(split.1);
        inserts.push((header + newline + 1, spliced));
        generated.push(Generated {
            file: index,
            impl_pos: imp.pos,
            label: job.label.clone(),
            generator: job.generator.clone(),
        });
        gen_files.push(SourceFile {
            path: format!("gen:impl:{}", job.label),
            src: text,
            module: prog.module_at(imp.pos).to_string(),
            base,
        });
    }
    let mut tokens = tokens;
    inserts.sort_by_key(|(at, _)| std::cmp::Reverse(*at));
    for (at, toks) in inserts {
        tokens.splice(at..at, toks);
    }
    let mut files = files;
    files.extend(gen_files);
    let (mut prog, mut diags) = build(tokens, files);
    prog.generated = generated;
    remap(&prog, &mut diags);
    diags.extend(errors);
    (prog, diags)
}

/// The deriving `impl`s: no methods, and a trait with a generator
/// (ct.derive.2). A problem with one is an error in `errors`.
fn jobs(prog: &Program, errors: &mut Vec<Diagnostic>) -> Vec<Job> {
    let mut jobs = Vec::new();
    for (i, imp) in prog.impls.iter().enumerate() {
        if !imp.methods.is_empty() {
            continue;
        }
        let trait_name = last(&imp.trait_name);
        let Some(tr) = prog.traits.iter().find(|t| t.name == trait_name && t.derive.is_some()) else { continue };
        let (generator, gen_pos) = tr.derive.clone().expect("found with a generator");
        let declared = prog.fns.iter().any(|f| f.name == generator && f.owner.is_none() && f.top_level());
        if !declared {
            let d = Diagnostic::new(
                "ct.derive.1",
                gen_pos,
                format!("the derive generator `{generator}` of `{}` isn't declared", tr.name),
            )
            .help(format!("declare `fn {generator}(t TypeInfo) str ! DeriveError`"));
            if !errors.iter().any(|e| e.pos == gen_pos) {
                errors.push(d);
            }
            continue;
        }
        let type_name = last(&imp.target.name).to_string();
        let Some(info) = type_info(prog, &type_name, prog.module_at(imp.pos)) else {
            errors.push(
                Diagnostic::new(
                    "ct.derive.2",
                    imp.pos,
                    format!(
                        "`{}` is derived by its generator `{generator}`, which needs a struct or enum; `{}` isn't one declared in this program",
                        tr.name, imp.target.name
                    ),
                )
                .help(format!("define the methods of `{}` in this `impl`", tr.name)),
            );
            continue;
        };
        jobs.push(Job {
            imp: i,
            generator,
            label: format!("{type_name}.{}", tr.name),
            type_name,
            trait_module: prog.module_at(tr.pos).to_string(),
            info,
        });
    }
    jobs
}

fn last(name: &str) -> &str {
    name.rsplit('.').next().unwrap_or(name)
}

/// The `TypeInfo(...)` expression describing the struct or enum `name`,
/// preferring one declared in `module` (refl.info.1, refl.info.3).
fn type_info(prog: &Program, name: &str, module: &str) -> Option<String> {
    let pick = |pos: Pos| prog.module_at(pos) == module;
    let s = prog.structs.iter().filter(|s| s.name == name).min_by_key(|s| !pick(s.pos));
    let e = prog.enums.iter().filter(|e| e.name == name).min_by_key(|e| !pick(e.pos));
    let field = |f: &Field| {
        format!(
            "FieldInfo(name: {}, type_text: {}, public: {})",
            quote(&f.name),
            quote(&crate::api::type_text(&f.ty)),
            prog.public.contains(&f.pos.line)
        )
    };
    let list = |items: Vec<String>, ty: &str| {
        if items.is_empty() { format!("List[{ty}]()") } else { format!("[{}]", items.join(", ")) }
    };
    let params = |g: &[GenericParam]| list(g.iter().map(|p| quote(&p.name)).collect(), "str");
    let (pos, kind, error, generics, fields, variants) = match (s, e) {
        (Some(s), _) => (s.pos, "Struct", s.is_error, params(&s.generics), s.fields.iter().map(field).collect(), Vec::new()),
        (None, Some(e)) => {
            let variants = e
                .variants
                .iter()
                .map(|v| {
                    let fields = list(v.fields.iter().map(field).collect(), "FieldInfo");
                    format!("VariantInfo(name: {}, fields: {fields})", quote(&v.name))
                })
                .collect();
            (e.pos, "Enum", e.is_error, params(&e.generics), Vec::new(), variants)
        }
        (None, None) => return None,
    };
    Some(format!(
        "TypeInfo(name: {}, id: {}, module: {}, kind: TypeKind.{kind}, is_error: {error}, params: {generics}, fields: {}, variants: {})",
        quote(name),
        quote(&format!("type:{name}")),
        quote(prog.module_at(pos)),
        list(fields, "FieldInfo"),
        list(variants, "VariantInfo"),
    ))
}

/// An AIL string literal holding `s` exactly.
fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' | '\\' | '{' => {
                out.push('\\');
                out.push(c);
            }
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// Run each job's generator (ct.derive.2-4, ct.eval.2): its text, or `None`
/// with an error in `errors`. `Err` when the generators can't be checked.
fn generate(
    prog: &Program,
    jobs: &[Job],
    tokens: &[Token],
    files: &[SourceFile],
    build: Build,
    errors: &mut Vec<Diagnostic>,
) -> Result<Vec<Option<String>>, (Program, Vec<Diagnostic>)> {
    let mut toks: Vec<Token> = tokens[..tokens.len() - 1].to_vec();
    let mut all_files = files.to_vec();
    let names: Vec<String> = (0..jobs.len()).map(|k| format!("derive_job_{k}")).collect();
    for (job, name) in jobs.iter().zip(&names) {
        let base = all_files.len() * FILE_LINES;
        let text = format!("fn {name}() str ! DeriveError\n    {}({})?\n", job.generator, job.info);
        let (mut t, _) = lex(&text);
        t.pop(); // Eof
        shift(&mut t, base);
        toks.extend(t);
        all_files.push(SourceFile { path: "gen:derive".into(), src: text, module: job.trait_module.clone(), base });
    }
    toks.push(Token { tok: Tok::Eof, pos: Pos { line: all_files.len() * FILE_LINES, col: 1 } });
    let wrappers = files.len();
    let in_wrapper = |pos: Pos| pos.line > wrappers * FILE_LINES && !is_prelude(pos);
    let (gen_prog, diags) = build(toks, all_files);
    if diags.iter().any(|d| !d.warning) {
        return Err((gen_prog, diags));
    }
    let (diags, resolved) = crate::check::check(&gen_prog);
    // The program's own code may call what the derives will add, so only a
    // problem in the call of a generator stops them here; one inside a
    // generator is reported when the whole program is checked.
    let mut texts = vec![None; jobs.len()];
    let mut runnable = Vec::new();
    for (k, job) in jobs.iter().enumerate() {
        let at = prog.impls[job.imp].pos;
        let start = (wrappers + k) * FILE_LINES;
        if let Some(d) = diags.iter().find(|d| !d.warning && d.pos.line > start && d.pos.line <= start + FILE_LINES) {
            errors.push(
                Diagnostic::new(
                    "ct.derive.1",
                    at,
                    format!("`{}` can't be called as a derive generator: {}", job.generator, d.message),
                )
                .help(format!("declare it `fn {}(t TypeInfo) str ! DeriveError`", job.generator)),
            );
            continue;
        }
        let Some(f) = gen_prog.fns.iter().position(|f| f.name == names[k] && f.owner.is_none() && in_wrapper(f.pos)) else {
            continue;
        };
        // ct.derive.3: a generator is compile-time-safe.
        let wrapper = format!("{} -> ", names[k]);
        let impure: Vec<String> = crate::check::effects::impure_chains(&gen_prog, &resolved, f)
            .into_iter()
            .map(|c| c.replace(&wrapper, ""))
            .collect();
        if !impure.is_empty() {
            errors.push(
                Diagnostic::new(
                    "ct.derive.3",
                    at,
                    format!("the derive generator `{}` runs at compile time but has {}", job.generator, impure.join("; ")),
                )
                .help("a generator may not do I/O or use a capability; compute the text from its `TypeInfo` only"),
            );
            continue;
        }
        runnable.push((k, names[k].clone()));
    }
    let wanted: Vec<String> = runnable.iter().map(|(_, n)| n.clone()).collect();
    let results = std::thread::scope(|s| {
        std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn_scoped(s, || crate::interp::generate(&gen_prog, &resolved, &wanted))
            .expect("spawn the generator thread")
            .join()
    });
    let Ok(results) = results else {
        // An ill-typed generator can stop the interpreter; the full check
        // below names its problem.
        if !diags.iter().any(|d| !d.warning) {
            for (k, _) in &runnable {
                let job = &jobs[*k];
                errors.push(Diagnostic::new(
                    "ct.derive.4",
                    prog.impls[job.imp].pos,
                    format!("the derive generator `{}` stopped unexpectedly", job.generator),
                ));
            }
        }
        return Ok(texts);
    };
    for ((k, _), result) in runnable.into_iter().zip(results) {
        let job = &jobs[k];
        match result {
            Ok(text) => texts[k] = Some(text),
            Err(message) => errors.push(
                Diagnostic::new(
                    "ct.derive.4",
                    prog.impls[job.imp].pos,
                    format!("`{}` can't be derived for `{}`: {message}", last(&prog.impls[job.imp].trait_name), job.type_name),
                )
                .help(format!("define the methods in this `impl`, or change `{}`", job.generator)),
            ),
        }
    }
    Ok(texts)
}

/// Split generated tokens into the methods of the `impl` and the associated
/// functions `fn T.name` of its type (ct.gen.1). `None`, with an error, for
/// anything else at the top level.
fn split(
    toks: Vec<Token>,
    job: &Job,
    at: Pos,
    errors: &mut Vec<Diagnostic>,
) -> Option<(Vec<Token>, Vec<Token>)> {
    let mut starts = Vec::new();
    let mut depth = 0usize;
    for (i, t) in toks.iter().enumerate() {
        match t.tok {
            Tok::Indent => depth += 1,
            Tok::Dedent => depth = depth.saturating_sub(1),
            Tok::Newline => {}
            Tok::Kw("fn") if depth == 0 => starts.push(i),
            _ if depth == 0 && starts.is_empty() => {
                errors.push(at_impl(at, job, t.pos, "generated text must hold only method definitions".into()));
                return None;
            }
            _ => {}
        }
        // A top-level line that doesn't start with `fn`.
        if depth == 0
            && i > 0
            && matches!(toks[i - 1].tok, Tok::Newline | Tok::Dedent)
            && !matches!(t.tok, Tok::Kw("fn") | Tok::Newline | Tok::Dedent | Tok::Indent)
        {
            errors.push(at_impl(at, job, t.pos, "generated text must hold only method definitions".into()));
            return None;
        }
    }
    let (mut methods, mut assoc) = (Vec::new(), Vec::new());
    for (n, &s) in starts.iter().enumerate() {
        let end = starts.get(n + 1).copied().unwrap_or(toks.len());
        let chunk = &toks[s..end];
        let owned = matches!(chunk.get(2).map(|t| &t.tok), Some(Tok::Sym(".")));
        if !owned {
            methods.extend_from_slice(chunk);
            continue;
        }
        if !matches!(&chunk[1].tok, Tok::Ident(n) if *n == job.type_name) {
            let msg = format!("generated functions outside the `impl` must belong to `{}`", job.type_name);
            errors.push(at_impl(at, job, chunk[1].pos, msg));
            return None;
        }
        assoc.extend_from_slice(chunk);
    }
    Some((methods, assoc))
}

/// ct.gen.3: an error about generated text, at the `impl`, with a note there.
fn at_impl(at: Pos, job: &Job, gen_pos: Pos, message: String) -> Diagnostic {
    Diagnostic::new(
        "ct.gen.1",
        at,
        format!(
            "the code `{}` generated for this `impl` is invalid at gen:impl:{}:{}:{}: {message}",
            job.generator,
            job.label,
            gen_pos.line % FILE_LINES,
            gen_pos.col
        ),
    )
    .help(format!("make `{}` return only `fn` definitions for the `impl` and its type", job.generator))
}

fn shift(tokens: &mut [Token], base: usize) {
    for t in tokens {
        t.pos.line += base;
        if let Tok::Interp(parts) = &mut t.tok {
            for p in parts {
                if let Part::Code(code) | Part::Fmt(code, _) = p {
                    shift(code, base);
                }
            }
        }
    }
}

/// ct.gen.3: a diagnostic inside generated code is reported at the deriving
/// `impl`, with a note at its place in the generated text.
pub fn remap(prog: &Program, diags: &mut [Diagnostic]) {
    if prog.generated.is_empty() {
        return;
    }
    for d in diags {
        let Some(g) = prog.generated.iter().find(|g| {
            let base = prog.files[g.file].base;
            d.pos.line > base && d.pos.line <= base + FILE_LINES
        }) else {
            continue;
        };
        let base = prog.files[g.file].base;
        let message = format!(
            "in the code `{}` generated for this `impl`, at gen:impl:{}:{}:{}",
            g.generator,
            g.label,
            d.pos.line - base,
            d.pos.col
        );
        d.notes.insert(0, Note { pos: d.pos, message });
        json_field_problem(g, d);
        d.pos = g.impl_pos;
        d.end = None;
        d.fixes.clear();
    }
}

/// json.derive.1-2: a derived JSON conversion that calls `to_json` or
/// `from_json` on a field type with none says so in the user's terms, not
/// as a missing method of generated code.
fn json_field_problem(g: &Generated, d: &mut Diagnostic) {
    let (trait_name, method) = match g.generator.as_str() {
        "derive_to_json" => ("ToJson", "to_json"),
        "derive_from_json" => ("FromJson", "from_json"),
        _ => return,
    };
    let Some(ty) = d.message.strip_suffix(&format!(" has no method `{method}`")).map(str::to_string) else { return };
    if d.rule != "expr.name.1" {
        return;
    }
    let owner = g.label.split('.').next().unwrap_or(&g.label);
    d.rule = if method == "to_json" { "json.derive.1" } else { "json.derive.2" };
    d.message = format!("can't derive `{trait_name}` for `{owner}`: a field has type {ty}, which has no JSON form");
    d.help = Some(format!(
        "give {ty} its own `impl {trait_name}`, or change the field to `int`, `float`, `bool`, `str`, `Json`, a `List`, a `Map[str, _]`, an optional or a derived type"
    ));
}
