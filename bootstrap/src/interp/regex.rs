//! `std.regex` at run time (spec std-regex.md): the engine is `crate::regex`;
//! this builds the values the checker typed (`regex.Match` and so on).

use std::io::Write;
use std::rc::Rc;

use super::builtins::Table;
use super::stdlib::fail;
use super::{Exit, Flow, Interp, Trap, Value};
use crate::diag::Pos;
use crate::regex::{self, Part, Prog, UNSET};

fn text(s: impl Into<Rc<str>>) -> Value {
    Value::Str(s.into())
}

fn stop<T>(rule: &'static str, pos: Pos, message: String) -> Flow<T> {
    Err(Exit::Trap(Box::new(Trap { rule, pos, message })))
}

impl<W: Write> Interp<'_, W> {
    /// The compiled program of a `regex.Regex` value (std.regex.2).
    fn regex_prog(&self, re: &Value, pos: Pos) -> Flow<Rc<Prog>> {
        let Value::Struct(_, fields) = re else { unreachable!("checked: a regex.Regex") };
        let Value::Str(pat) = &fields[0] else { unreachable!("regex.Regex has a pattern") };
        match regex::compile(pat) {
            Ok(p) => Ok(p),
            Err(e) => stop(
                "std.regex.2",
                pos,
                format!("`regex.Regex(pat: ...)` holds a pattern that doesn't compile ({}): make a regex with `regex.compile`", e.message),
            ),
        }
    }

    fn match_value(&self, chars: &[char], m: &[usize]) -> Value {
        let s = self.structs["regex.Match"];
        Value::Struct(s, Rc::new(vec![
            Value::Int(m[0] as i64),
            Value::Int(m[1] as i64),
            text(regex::slice(chars, m[0], m[1])),
        ]))
    }

    pub(super) fn regex_call(&mut self, f: &str, values: &[Value], pos: Pos) -> Flow<Value> {
        let str_arg = |i: usize| match &values[i] {
            Value::Str(s) => s.clone(),
            other => unreachable!("checked: str, found {other:?}"),
        };
        if f == "compile" {
            let pat = str_arg(0);
            return Ok(match regex::compile(&pat) {
                Ok(_) => Value::Struct(self.structs["regex.Regex"], Rc::new(vec![Value::Str(pat)])),
                Err(e) => {
                    // std.regex.23: a `RegexError` has pattern, at, kind, rule and message.
                    let s = self.structs["regex.RegexError"];
                    fail(
                        Value::Struct(s, Rc::new(vec![
                            Value::Str(pat),
                            Value::Int(e.at as i64),
                            text(e.kind),
                            text(regex::kind_rule(e.kind)),
                            text(e.message),
                        ])),
                        pos,
                    )
                }
            });
        }
        if f == "escape" {
            return Ok(text(regex::escape(&str_arg(0))));
        }
        let prog = self.regex_prog(&values[0], pos)?;
        let chars: Vec<char> = str_arg(1).chars().collect();
        Ok(match f {
            "is_match" => Value::Bool(prog.is_match(&chars)),
            "find" => match prog.exec(&chars, 0, 2, false) {
                Some(m) => self.match_value(&chars, &m),
                None => Value::None,
            },
            "find_all" => {
                let ms = prog.find_all(&chars, 2, usize::MAX);
                Value::List(Rc::new(ms.iter().map(|m| self.match_value(&chars, m)).collect()))
            }
            "group_all" => {
                let Value::Int(g) = values[2] else { unreachable!("checked: an int group") };
                if g < 0 || g as usize >= prog.groups {
                    return stop(
                        "std.regex.24",
                        pos,
                        format!("the pattern has groups 0 to {}, not group {g}", prog.groups - 1),
                    );
                }
                let g = g as usize;
                let ms = prog.find_all(&chars, 2 * prog.groups, usize::MAX);
                Value::List(Rc::new(
                    ms.iter()
                        .map(|m| if m[2 * g] == UNSET { text("") } else { text(regex::slice(&chars, m[2 * g], m[2 * g + 1])) })
                        .collect(),
                ))
            }
            "captures" => match prog.exec(&chars, 0, 2 * prog.groups, false) {
                None => Value::None,
                Some(m) => {
                    let group = |g: usize| (m[2 * g] != UNSET).then(|| regex::slice(&chars, m[2 * g], m[2 * g + 1]));
                    let groups = (0..prog.groups).map(|g| group(g).map_or(Value::None, text)).collect();
                    let mut named = Table::default();
                    for (name, g) in &prog.names {
                        if let Some(t) = group(*g) {
                            named.insert(text(name.as_str()), Some(text(t)));
                        }
                    }
                    let s = self.structs["regex.Captures"];
                    Value::Struct(s, Rc::new(vec![
                        Value::Int(m[0] as i64),
                        Value::Int(m[1] as i64),
                        Value::List(Rc::new(groups)),
                        Value::Map(Rc::new(named)),
                    ]))
                }
            },
            "replace" | "replace_first" => {
                let parts: Vec<Part> = match regex::template(&str_arg(2), &prog) {
                    Ok(p) => p,
                    Err(why) => {
                        return stop("std.regex.7", pos, format!("the replacement refers to {why}"));
                    }
                };
                let limit = if f == "replace" { usize::MAX } else { 1 };
                text(regex::replace(&prog, &chars, &parts, limit))
            }
            "split" => Value::List(Rc::new(regex::split(&prog, &chars).into_iter().map(text).collect())),
            _ => unreachable!("stdlib::MODULES lists `regex.{f}`"),
        })
    }
}
