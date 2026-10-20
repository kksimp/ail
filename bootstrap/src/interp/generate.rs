//! Running derive generators at compile time (ct.derive.2-4, ct.eval.2).

use super::*;

/// Call each named function, which takes nothing and returns `str !
/// DeriveError`: its text, or a message for its failure or trap (ct.derive.4).
pub fn generate(prog: &Program, resolved: &Resolved, names: &[String]) -> Vec<Result<String, String>> {
    let mut it = build(prog, resolved, Vec::<u8>::new());
    // `--trace` is about the program's run, not its compilation.
    it.tracing = 0;
    names
        .iter()
        .map(|name| {
            let f = it.fns.get(name.as_str()).copied().ok_or_else(|| format!("no function `{name}`"))?;
            it.depth = 0;
            it.reset_limits();
            match it.call(f, None, Vec::new(), f.pos) {
                Ok((Value::Str(s), _)) => Ok(s.to_string()),
                Ok((Value::Fail(fail), _)) => match &fail.error {
                    // A `DeriveError` says why in its message.
                    Value::Struct(s, fields) if prog.structs[*s].name == "DeriveError" => match fields.first() {
                        Some(Value::Str(m)) => Err(m.to_string()),
                        _ => Err("the generator failed".into()),
                    },
                    other => {
                        let text = it.text(other, f.pos).unwrap_or_else(|t| t.message);
                        Err(format!("the generator failed with {text}"))
                    }
                },
                Ok(_) => Err("the generator returned no text".into()),
                Err(t) => Err(format!("the generator trapped: trap[{}]: {}", t.rule, t.message)),
            }
        })
        .collect()
}
