//! A small JSON reader for the harness (fuzz.prop.4): validity, and a
//! normal form without positions, so two `ail api` outputs compare by content.

pub struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

/// Parse `text` as one JSON value and write it back without `line`, `col`,
/// `end_line`, `end_col` and `file` keys. `Err` carries the byte offset.
pub fn normalise(text: &str) -> Result<String, String> {
    let mut p = Parser { s: text.as_bytes(), i: 0 };
    let mut out = String::new();
    p.value(&mut out)?;
    p.ws();
    if p.i != p.s.len() {
        return Err(format!("trailing text at byte {}", p.i));
    }
    Ok(out)
}

const POSITIONS: [&str; 5] = ["line", "col", "end_line", "end_col", "file"];

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> Result<(), String> {
        self.ws();
        if self.s.get(self.i) == Some(&c) {
            self.i += 1;
            Ok(())
        } else {
            Err(format!("expected `{}` at byte {}", c as char, self.i))
        }
    }

    fn string(&mut self, out: &mut String) -> Result<(), String> {
        self.eat(b'"')?;
        let start = self.i - 1;
        while let Some(&c) = self.s.get(self.i) {
            match c {
                b'"' => {
                    self.i += 1;
                    out.push_str(std::str::from_utf8(&self.s[start..self.i]).map_err(|e| e.to_string())?);
                    return Ok(());
                }
                b'\\' => self.i += 2,
                c if c < 0x20 => return Err(format!("raw control character at byte {}", self.i)),
                _ => self.i += 1,
            }
        }
        Err("unterminated string".into())
    }

    fn value(&mut self, out: &mut String) -> Result<(), String> {
        self.ws();
        match self.s.get(self.i) {
            Some(b'{') => {
                self.i += 1;
                out.push('{');
                let mut first = true;
                loop {
                    self.ws();
                    if self.s.get(self.i) == Some(&b'}') {
                        self.i += 1;
                        break;
                    }
                    if !first {
                        self.eat(b',')?;
                    }
                    let mut key = String::new();
                    self.string(&mut key)?;
                    self.eat(b':')?;
                    let mut val = String::new();
                    self.value(&mut val)?;
                    if !POSITIONS.contains(&key.trim_matches('"')) {
                        if !first && !out.ends_with('{') {
                            out.push(',');
                        }
                        out.push_str(&key);
                        out.push(':');
                        out.push_str(&val);
                    }
                    first = false;
                }
                out.push('}');
                Ok(())
            }
            Some(b'[') => {
                self.i += 1;
                out.push('[');
                let mut first = true;
                loop {
                    self.ws();
                    if self.s.get(self.i) == Some(&b']') {
                        self.i += 1;
                        break;
                    }
                    if !first {
                        self.eat(b',')?;
                        out.push(',');
                    }
                    self.value(out)?;
                    first = false;
                }
                out.push(']');
                Ok(())
            }
            Some(b'"') => self.string(out),
            Some(_) => {
                let start = self.i;
                while self.i < self.s.len()
                    && !matches!(self.s[self.i], b',' | b'}' | b']')
                    && !self.s[self.i].is_ascii_whitespace()
                {
                    self.i += 1;
                }
                let word = std::str::from_utf8(&self.s[start..self.i]).map_err(|e| e.to_string())?;
                if matches!(word, "true" | "false" | "null") || word.parse::<f64>().is_ok() {
                    out.push_str(word);
                    Ok(())
                } else {
                    Err(format!("bad value `{word}` at byte {start}"))
                }
            }
            None => Err("unexpected end".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::normalise;

    #[test]
    fn drops_positions_and_validates() {
        assert_eq!(
            normalise(r#"{"a":1,"line":3,"b":[1, {"col":2,"x":"y"}]}"#).unwrap(),
            r#"{"a":1,"b":[1,{"x":"y"}]}"#
        );
        assert!(normalise("{\"a\":").is_err());
        assert!(normalise("{} x").is_err());
    }
}
