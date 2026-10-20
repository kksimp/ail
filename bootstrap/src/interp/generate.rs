//! Running derive generators at compile time (ct.derive.2-4, ct.eval.2).

use super::*;

/// Call each named function, which takes nothing and returns `str !
/// DeriveError`: its text, or a message for its failure or trap (ct.derive.4).
pub fn generate(prog: &Program, resolved: &Resolved, names: &[String]) -> Vec<Result<String, String>> {
    let mut it = build(prog, resolved, Vec::<u8>::new());
    names
        .iter()
        .map(|name| {
            let f = it.fns.get(name.as_str()).copied().ok_or_else(|| format!("no function `{name}`"))?;
            it.depth = 0;
            it.reset_limits();
            match it.call(f, None, Vec::new(), f.pos) {
                Ok((Value::Str(s), _)) => Ok(s.to_string()),
                Ok((Value::Fail(fail), _)) => {
                    let text = it.text(&fail.error, f.pos).unwrap_or_else(|t| t.message);
                    Err(format!("the generator failed with {text}"))
                }
                Ok(_) => Err("the generator returned no text".into()),
                Err(t) => Err(format!("the generator trapped: trap[{}]: {}", t.rule, t.message)),
            }
        })
        .collect()
}
