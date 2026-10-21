//! `parse_json`, `Json.to_text` and `json_quote` run natively (json.parse.1,
//! json.print.1, ADR 0658). They are declared in `prelude.ail` in AIL, and the
//! checker sees that source; `install` then marks them `extern` with the
//! library `ail:json`, so a call reaches `call_extern` and from there
//! `json_native`, which gives the same values, errors and offsets as the AIL
//! text did, about fifty times faster on large documents.

use std::io::Write;
use std::rc::Rc;

use super::stdlib::fail;
use super::{Interp, MAX_DEPTH, Trap, Value, float_text, trap};
use crate::ast::{Extern, FnDecl, Program, is_prelude};
use crate::diag::Pos;

/// The `lib` that marks a prelude function as native.
pub(super) const LIB: &str = "ail:json";

/// Mark the prelude's JSON functions native. Call after the program is checked.
pub fn install(prog: &mut Program) {
    for f in &mut prog.fns {
        let owner = f.owner.as_ref().map(|o| o.name.as_str());
        let native = is_prelude(f.pos)
            && matches!((owner, f.name.as_str()), (None, "parse_json" | "json_quote") | (Some("Json"), "to_text"));
        if native {
            f.ext = Some(Extern { lib: Some(LIB.into()) });
        }
    }
}

/// A parse failure: byte offset into the text, and the message.
struct Bad {
    at: usize,
    message: String,
    /// The AIL parser would have run out of call depth here.
    deep: bool,
}

type R<T> = Result<T, Bad>;

fn bad<T>(at: usize, message: &str) -> R<T> {
    Err(Bad { at, message: message.to_string(), deep: false })
}

/// The Json enum and its helpers as the program numbers them.
struct Mk {
    json: usize,
    member: usize,
    no_fields: Rc<Vec<Value>>,
}

impl Mk {
    fn node(&self, variant: usize, fields: Vec<Value>) -> Value {
        Value::Enum(self.json, variant, Rc::new(fields))
    }
}

const NULL: usize = 0;
const BOOL: usize = 1;
const NUM: usize = 2;
const STR: usize = 3;
const ARR: usize = 4;
const OBJ: usize = 5;

struct Parser<'s> {
    text: &'s str,
    b: &'s [u8],
    pos: usize,
    /// Containers open now, and how many the AIL parser could nest.
    open: usize,
    max_open: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        let c = *self.b.get(self.pos)?;
        if c < 0x80 { Some(c as char) } else { self.text[self.pos..].chars().next() }
    }

    fn peek_is(&self, c: u8) -> bool {
        self.b.get(self.pos) == Some(&c)
    }

    fn unexpected(&self) -> Bad {
        match self.peek() {
            Some(c) => Bad { at: self.pos, message: format!("unexpected character '{c}'"), deep: false },
            None => Bad { at: self.pos, message: "unexpected end of input".into(), deep: false },
        }
    }

    fn skip_space(&mut self) {
        while matches!(self.b.get(self.pos), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn document(&mut self, mk: &Mk) -> R<Value> {
        let value = self.value(mk)?;
        self.skip_space();
        if self.pos < self.b.len() {
            return bad(self.pos, "extra data after value");
        }
        Ok(value)
    }

    fn value(&mut self, mk: &Mk) -> R<Value> {
        self.skip_space();
        let Some(c) = self.peek() else { return Err(self.unexpected()) };
        if c == '-' || c.is_ascii_digit() {
            return self.number(mk);
        }
        match c {
            '{' => self.object(mk),
            '[' => self.array(mk),
            '"' => Ok(mk.node(STR, vec![Value::Str(self.string()?)])),
            _ => {
                for (word, variant, fields) in [
                    ("true", BOOL, Some(true)),
                    ("false", BOOL, Some(false)),
                    ("null", NULL, None),
                ] {
                    if self.b[self.pos..].starts_with(word.as_bytes()) {
                        self.pos += word.len();
                        return Ok(match fields {
                            Some(b) => mk.node(variant, vec![Value::Bool(b)]),
                            None => Value::Enum(mk.json, NULL, Rc::clone(&mk.no_fields)),
                        });
                    }
                }
                Err(self.unexpected())
            }
        }
    }

    fn enter(&mut self) -> R<()> {
        self.open += 1;
        if self.open > self.max_open {
            return Err(Bad { at: self.pos, message: String::new(), deep: true });
        }
        Ok(())
    }

    fn object(&mut self, mk: &Mk) -> R<Value> {
        self.enter()?;
        self.pos += 1;
        self.skip_space();
        if self.peek_is(b'}') {
            self.pos += 1;
            self.open -= 1;
            return Ok(mk.node(OBJ, vec![Value::List(Rc::new(Vec::new()))]));
        }
        let mut members = Vec::new();
        loop {
            self.skip_space();
            if !self.peek_is(b'"') {
                return Err(self.unexpected());
            }
            let key = self.string()?;
            self.skip_space();
            if !self.peek_is(b':') {
                if self.pos >= self.b.len() {
                    return Err(self.unexpected());
                }
                return bad(self.pos, "expected ':' after object key");
            }
            self.pos += 1;
            let value = self.value(mk)?;
            members.push(Value::Struct(mk.member, Rc::new(vec![Value::Str(key), value])));
            if self.container_end(b'}')? {
                self.open -= 1;
                return Ok(mk.node(OBJ, vec![Value::List(Rc::new(members))]));
            }
        }
    }

    fn array(&mut self, mk: &Mk) -> R<Value> {
        self.enter()?;
        self.pos += 1;
        self.skip_space();
        if self.peek_is(b']') {
            self.pos += 1;
            self.open -= 1;
            return Ok(mk.node(ARR, vec![Value::List(Rc::new(Vec::new()))]));
        }
        let mut items = Vec::new();
        loop {
            items.push(self.value(mk)?);
            if self.container_end(b']')? {
                self.open -= 1;
                return Ok(mk.node(ARR, vec![Value::List(Rc::new(items))]));
            }
        }
    }

    /// After an element: whether the container ends here, else past a comma.
    fn container_end(&mut self, close: u8) -> R<bool> {
        self.skip_space();
        if self.peek_is(close) {
            self.pos += 1;
            return Ok(true);
        }
        if !self.peek_is(b',') {
            return Err(self.unexpected());
        }
        let comma = self.pos;
        self.pos += 1;
        self.skip_space();
        if self.peek_is(b']') || self.peek_is(b'}') {
            return bad(comma, "trailing comma");
        }
        Ok(false)
    }

    fn string(&mut self) -> R<Rc<str>> {
        let start = self.pos;
        self.pos += 1;
        let mut out = String::new();
        let mut escaped = false;
        let mut run = self.pos;
        loop {
            let Some(&c) = self.b.get(self.pos) else { return bad(start, "unterminated string") };
            match c {
                b'"' => {
                    // No escape: the text is a slice of the input, made once.
                    let text: Rc<str> = if escaped {
                        out.push_str(&self.text[run..self.pos]);
                        Rc::from(out)
                    } else {
                        Rc::from(&self.text[run..self.pos])
                    };
                    self.pos += 1;
                    return Ok(text);
                }
                b'\n' => return bad(start, "unterminated string"),
                b'\\' => {
                    escaped = true;
                    out.push_str(&self.text[run..self.pos]);
                    out.push(self.escape(start)?);
                    run = self.pos;
                }
                c if c < 0x20 => return bad(self.pos, "control character in string"),
                _ => self.pos += 1,
            }
        }
    }

    fn escape(&mut self, start: usize) -> R<char> {
        let at = self.pos;
        self.pos += 1;
        let Some(c) = self.peek() else { return bad(start, "unterminated string") };
        self.pos += c.len_utf8();
        Ok(match c {
            '"' | '\\' | '/' => c,
            'b' => '\u{8}',
            'f' => '\u{c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            'u' => return self.unicode(at),
            _ => return bad(at, &format!("invalid escape '\\{c}'")),
        })
    }

    /// Four hex digits at the position, advancing past them.
    fn hex4(&mut self, at: usize) -> R<u32> {
        let mut code = 0;
        let mut used = 0;
        let mut chars = self.text[self.pos..].chars();
        for _ in 0..4 {
            let Some(d) = chars.next() else { return bad(at, "invalid unicode escape") };
            let Some(v) = d.to_digit(16) else { return bad(at, "invalid unicode escape") };
            code = code * 16 + v;
            used += d.len_utf8();
        }
        self.pos += used;
        Ok(code)
    }

    fn unicode(&mut self, at: usize) -> R<char> {
        let mut code = self.hex4(at)?;
        if (0xDC00..=0xDFFF).contains(&code) {
            return bad(at, "unpaired surrogate escape");
        }
        if (0xD800..=0xDBFF).contains(&code) {
            if !self.b[self.pos..].starts_with(b"\\u") {
                return bad(at, "unpaired surrogate escape");
            }
            let low_at = self.pos;
            self.pos += 2;
            let low = self.hex4(low_at)?;
            if !(0xDC00..=0xDFFF).contains(&low) {
                return bad(at, "unpaired surrogate escape");
            }
            code = 0x10000 + (code - 0xD800) * 0x400 + (low - 0xDC00);
        }
        match char::from_u32(code) {
            Some(c) => Ok(c),
            None => bad(at, "invalid unicode escape"),
        }
    }

    fn digits(&mut self) -> usize {
        let start = self.pos;
        while self.b.get(self.pos).is_some_and(u8::is_ascii_digit) {
            self.pos += 1;
        }
        self.pos - start
    }

    fn number(&mut self, mk: &Mk) -> R<Value> {
        let start = self.pos;
        if self.peek_is(b'-') {
            self.pos += 1;
        }
        if self.peek_is(b'0') {
            self.pos += 1;
        } else if self.digits() == 0 {
            return bad(self.pos, "invalid number");
        }
        if self.peek_is(b'.') {
            self.pos += 1;
            if self.digits() == 0 {
                return bad(self.pos, "invalid number");
            }
        }
        if self.peek_is(b'e') || self.peek_is(b'E') {
            self.pos += 1;
            if self.peek_is(b'+') || self.peek_is(b'-') {
                self.pos += 1;
            }
            if self.digits() == 0 {
                return bad(self.pos, "invalid number");
            }
        }
        let Ok(n) = self.text[start..self.pos].parse::<f64>() else { return bad(start, "invalid number") };
        if !n.is_finite() {
            return bad(start, "number out of range");
        }
        Ok(mk.node(NUM, vec![Value::Float(n)]))
    }
}

/// `json_quote`: the string in double quotes with JSON escapes (json.print.1).
fn quote(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// `Json.to_text` into `out`; `false` when the value nests deeper than the
/// AIL function could recurse.
fn print(v: &Value, out: &mut String, room: usize) -> bool {
    let Value::Enum(_, variant, fields) = v else { unreachable!("checked: a Json") };
    match *variant {
        NULL => out.push_str("null"),
        BOOL => {
            let Value::Bool(b) = fields[0] else { unreachable!("Bool holds a bool") };
            out.push_str(if b { "true" } else { "false" });
        }
        NUM => {
            let Value::Float(n) = fields[0] else { unreachable!("Num holds a float") };
            if !n.is_finite() {
                out.push_str("null");
            } else if n.abs() < 1e15 && n.fract() == 0.0 {
                // json.print.1: whole numbers below 1e15 print without a fraction (ADR 0631).
                out.push_str(&format!("{}", n as i64));
            } else {
                out.push_str(&float_text(n));
            }
        }
        STR => {
            let Value::Str(s) = &fields[0] else { unreachable!("Str holds a str") };
            quote(s, out);
        }
        ARR => {
            let Value::List(items) = &fields[0] else { unreachable!("Arr holds a list") };
            if room == 0 {
                return false;
            }
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if !print(item, out, room - 1) {
                    return false;
                }
            }
            out.push(']');
        }
        _ => {
            let Value::List(members) = &fields[0] else { unreachable!("Obj holds a list") };
            if room == 0 {
                return false;
            }
            out.push('{');
            for (i, m) in members.iter().enumerate() {
                let Value::Struct(_, kv) = m else { unreachable!("a JsonMember") };
                let Value::Str(key) = &kv[0] else { unreachable!("a member key") };
                if i > 0 {
                    out.push(',');
                }
                quote(key, out);
                out.push(':');
                if !print(&kv[1], out, room - 1) {
                    return false;
                }
            }
            out.push('}');
        }
    }
    true
}

impl<W: Write> Interp<'_, W> {
    /// Run the native form of prelude function `f` on the arguments in `env`.
    pub(super) fn json_native(&mut self, f: &FnDecl, env: &[(&str, Value)], pos: Pos) -> Result<Value, Trap> {
        match f.name.as_str() {
            "json_quote" => {
                let Value::Str(s) = &env[0].1 else { unreachable!("checked: a str") };
                let mut out = String::with_capacity(s.len() + 2);
                quote(s, &mut out);
                Ok(Value::Str(out.into()))
            }
            "to_text" => {
                let mut out = String::new();
                // One frame per level of the AIL function, plus the call itself.
                let room = MAX_DEPTH.saturating_sub(self.depth + 2);
                if print(&env[0].1, &mut out, room) {
                    Ok(Value::Str(out.into()))
                } else {
                    trap("err.trap.1", pos, format!("call depth exceeded {MAX_DEPTH} in `Json.to_text`"))
                }
            }
            _ => {
                let Value::Str(text) = &env[0].1 else { unreachable!("checked: a str") };
                let mk = Mk { json: self.enums["Json"], member: self.structs["JsonMember"], no_fields: Rc::clone(&self.no_fields) };
                // `parse_json`, `document` and `value` take three frames, then
                // two for each container.
                let max_open = MAX_DEPTH.saturating_sub(self.depth + 4) / 2;
                let mut p = Parser { text, b: text.as_bytes(), pos: 0, open: 0, max_open };
                match p.document(&mk) {
                    Ok(v) => Ok(v),
                    Err(e) if e.deep => {
                        trap("err.trap.1", pos, format!("call depth exceeded {MAX_DEPTH} in `JsonParser.value`"))
                    }
                    Err(e) => {
                        let at = text[..e.at].chars().count() as i64;
                        let err = Value::Struct(self.structs["JsonError"], Rc::new(vec![Value::Int(at), Value::Str(e.message.into())]));
                        Ok(fail(err, pos))
                    }
                }
            }
        }
    }
}
