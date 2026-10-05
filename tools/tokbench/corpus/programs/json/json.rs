use std::collections::HashMap;
use std::fmt;

const WHITESPACE: &str = " \t\n\r";
const KINDS: [&str; 6] = ["object", "array", "string", "number", "boolean", "null"];

#[derive(PartialEq)]
enum Json {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<Json>),
    Object(Vec<(String, Json)>),
}

#[derive(Debug)]
struct ParseError {
    line: usize,
    col: usize,
    msg: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "parse error at {}:{}: {}", self.line, self.col, self.msg)
    }
}

impl std::error::Error for ParseError {}

type Result<T> = std::result::Result<T, ParseError>;

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(text: &str) -> Parser {
        Parser {
            chars: text.chars().collect(),
            pos: 0,
        }
    }

    fn fail_at(&self, at: usize, msg: impl Into<String>) -> ParseError {
        let (mut line, mut col) = (1, 1);
        for &c in &self.chars[..at] {
            if c == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }
        ParseError {
            line,
            col,
            msg: msg.into(),
        }
    }

    fn fail(&self, msg: impl Into<String>) -> ParseError {
        self.fail_at(self.pos, msg)
    }

    fn at_end(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn unexpected(&self) -> ParseError {
        match self.peek() {
            Some(c) => self.fail(format!("unexpected character '{c}'")),
            None => self.fail("unexpected end of input"),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(|c| WHITESPACE.contains(c)) {
            self.pos += 1;
        }
    }

    fn parse_document(&mut self) -> Result<Json> {
        let value = self.parse_value()?;
        self.skip_whitespace();
        if !self.at_end() {
            return Err(self.fail("extra data after value"));
        }
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<Json> {
        self.skip_whitespace();
        match self.peek() {
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(Json::Str),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            _ if self.take_word("true") => Ok(Json::Bool(true)),
            _ if self.take_word("false") => Ok(Json::Bool(false)),
            _ if self.take_word("null") => Ok(Json::Null),
            _ => Err(self.unexpected()),
        }
    }

    fn take_word(&mut self, word: &str) -> bool {
        let letters: Vec<char> = word.chars().collect();
        if !self.chars[self.pos..].starts_with(&letters) {
            return false;
        }
        self.pos += letters.len();
        true
    }

    fn parse_object(&mut self) -> Result<Json> {
        self.pos += 1;
        let mut members = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(Json::Object(members));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return Err(self.unexpected());
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.peek() != Some(':') {
                if self.at_end() {
                    return Err(self.unexpected());
                }
                return Err(self.fail("expected ':' after object key"));
            }
            self.pos += 1;
            members.push((key, self.parse_value()?));
            if self.end_of_container('}')? {
                return Ok(Json::Object(members));
            }
        }
    }

    fn parse_array(&mut self) -> Result<Json> {
        self.pos += 1;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(Json::Array(items));
        }
        loop {
            items.push(self.parse_value()?);
            if self.end_of_container(']')? {
                return Ok(Json::Array(items));
            }
        }
    }

    fn end_of_container(&mut self, close: char) -> Result<bool> {
        self.skip_whitespace();
        let c = self.peek();
        if c == Some(close) {
            self.pos += 1;
            return Ok(true);
        }
        if c != Some(',') {
            return Err(self.unexpected());
        }
        let comma = self.pos;
        self.pos += 1;
        self.skip_whitespace();
        if matches!(self.peek(), Some(']' | '}')) {
            return Err(self.fail_at(comma, "trailing comma"));
        }
        Ok(false)
    }

    fn parse_string(&mut self) -> Result<String> {
        let start = self.pos;
        self.pos += 1;
        let mut text = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => return Err(self.fail_at(start, "unterminated string")),
                Some('"') => {
                    self.pos += 1;
                    return Ok(text);
                }
                Some('\\') => text.push(self.parse_escape(start)?),
                Some(c) if c < ' ' => return Err(self.fail("control character in string")),
                Some(c) => {
                    text.push(c);
                    self.pos += 1;
                }
            }
        }
    }

    fn parse_escape(&mut self, start: usize) -> Result<char> {
        let escape = self.pos;
        self.pos += 1;
        let c = self
            .peek()
            .ok_or_else(|| self.fail_at(start, "unterminated string"))?;
        self.pos += 1;
        match c {
            '"' | '\\' | '/' => Ok(c),
            'b' => Ok('\u{8}'),
            'f' => Ok('\u{c}'),
            'n' => Ok('\n'),
            'r' => Ok('\r'),
            't' => Ok('\t'),
            'u' => self.parse_unicode(escape),
            _ => Err(self.fail_at(escape, format!("invalid escape '\\{c}'"))),
        }
    }

    fn parse_unicode(&mut self, escape: usize) -> Result<char> {
        let end = self.pos + 4;
        let code = self
            .chars
            .get(self.pos..end)
            .and_then(|digits| {
                digits
                    .iter()
                    .try_fold(0, |code, d| Some(code * 16 + d.to_digit(16)?))
            })
            .ok_or_else(|| self.fail_at(escape, "invalid unicode escape"))?;
        let ch = char::from_u32(code)
            .ok_or_else(|| self.fail_at(escape, "surrogate escapes are not supported"))?;
        self.pos = end;
        Ok(ch)
    }

    fn skip_digits(&mut self) -> usize {
        let start = self.pos;
        while self.peek().is_some_and(|c| c.is_ascii_digit()) {
            self.pos += 1;
        }
        self.pos - start
    }

    fn parse_number(&mut self) -> Result<Json> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        if self.peek() == Some('0') {
            self.pos += 1;
        } else if self.skip_digits() == 0 {
            return Err(self.fail("invalid number"));
        }
        let mut is_float = false;
        if self.peek() == Some('.') {
            self.pos += 1;
            if self.skip_digits() == 0 {
                return Err(self.fail("invalid number"));
            }
            is_float = true;
        }
        if matches!(self.peek(), Some('e' | 'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.pos += 1;
            }
            if self.skip_digits() == 0 {
                return Err(self.fail("invalid number"));
            }
            is_float = true;
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        if is_float {
            text.parse()
                .map(Json::Float)
                .map_err(|_| self.fail_at(start, "invalid number"))
        } else {
            text.parse()
                .map(Json::Int)
                .map_err(|_| self.fail_at(start, "number out of range"))
        }
    }
}

fn parse(text: &str) -> Result<Json> {
    Parser::new(text).parse_document()
}

fn quote(s: &str) -> String {
    let mut out = String::from('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn compact(value: &Json) -> String {
    match value {
        Json::Null => "null".to_string(),
        Json::Bool(b) => b.to_string(),
        Json::Int(n) => n.to_string(),
        Json::Float(x) => format!("{x:?}"),
        Json::Str(s) => quote(s),
        Json::Array(items) => {
            let parts: Vec<String> = items.iter().map(compact).collect();
            format!("[{}]", parts.join(","))
        }
        Json::Object(members) => {
            let parts: Vec<String> = members
                .iter()
                .map(|(k, v)| format!("{}:{}", quote(k), compact(v)))
                .collect();
            format!("{{{}}}", parts.join(","))
        }
    }
}

fn pretty(value: &Json, level: usize) -> String {
    let pad = "  ".repeat(level + 1);
    let close = "  ".repeat(level);
    match value {
        Json::Array(items) if items.is_empty() => "[]".to_string(),
        Json::Array(items) => {
            let lines: Vec<String> = items
                .iter()
                .map(|item| format!("{pad}{}", pretty(item, level + 1)))
                .collect();
            format!("[\n{}\n{close}]", lines.join(",\n"))
        }
        Json::Object(members) if members.is_empty() => "{}".to_string(),
        Json::Object(members) => {
            let lines: Vec<String> = members
                .iter()
                .map(|(k, v)| format!("{pad}{}: {}", quote(k), pretty(v, level + 1)))
                .collect();
            format!("{{\n{}\n{close}}}", lines.join(",\n"))
        }
        _ => compact(value),
    }
}

enum Step {
    Key(String),
    Index(i64),
}

fn parse_path(path: &str) -> Option<Vec<Step>> {
    let mut steps = Vec::new();
    for part in path.split('.') {
        let mut pieces = part.split('[');
        if let Some(name) = pieces.next().filter(|name| !name.is_empty()) {
            steps.push(Step::Key(name.to_string()));
        }
        for piece in pieces {
            let (index, rest) = piece.split_once(']')?;
            if !rest.is_empty() {
                return None;
            }
            steps.push(Step::Index(index.parse().ok()?));
        }
    }
    Some(steps)
}

fn query<'a>(doc: &'a Json, steps: &[Step]) -> Option<&'a Json> {
    let mut current = doc;
    for step in steps {
        current = match (step, current) {
            (Step::Index(i), Json::Array(items)) => items.get(usize::try_from(*i).ok()?)?,
            (Step::Key(name), Json::Object(members)) => {
                members.iter().find(|(k, _)| k == name).map(|(_, v)| v)?
            }
            _ => return None,
        };
    }
    Some(current)
}

#[derive(Default)]
struct Stats {
    counts: HashMap<&'static str, usize>,
    max_depth: usize,
    total: f64,
}

fn kind_of(value: &Json) -> &'static str {
    match value {
        Json::Object(_) => "object",
        Json::Array(_) => "array",
        Json::Str(_) => "string",
        Json::Bool(_) => "boolean",
        Json::Null => "null",
        Json::Int(_) | Json::Float(_) => "number",
    }
}

fn walk(value: &Json, depth: usize, stats: &mut Stats) {
    *stats.counts.entry(kind_of(value)).or_insert(0) += 1;
    match value {
        Json::Int(n) => stats.total += *n as f64,
        Json::Float(x) => stats.total += x,
        Json::Array(items) => {
            stats.max_depth = stats.max_depth.max(depth);
            for item in items {
                walk(item, depth + 1, stats);
            }
        }
        Json::Object(members) => {
            stats.max_depth = stats.max_depth.max(depth);
            for (_, child) in members {
                walk(child, depth + 1, stats);
            }
        }
        _ => {}
    }
}

const DOCUMENT: &str = r#"
{
  "meta": {"version": 3, "count": 3, "generated": "2026-10-05", "archived": null},
  "users": [
    {
      "id": 1,
      "name": "Ann \"Annie\" Lee",
      "email": "ann@example.com",
      "active": true,
      "score": 3.5,
      "tags": ["admin", "dev", "ops"],
      "address": {"city": "Z\u00fcrich", "zip": "8001"}
    },
    {
      "id": 2,
      "name": "Jos\u00e9 Garc\u00eda",
      "active": false,
      "score": 1e3,
      "tags": [],
      "note": "line one\nline two\ttabbed \/ slash \\ backslash"
    },
    {
      "id": 3,
      "name": "\u674E\u96F7",
      "active": true,
      "score": -0.25,
      "tags": ["guest"],
      "balance": -42,
      "ratio": 2.5E-3,
      "prefs": {"theme": "dark", "langs": ["en", "zh"], "beta": {}},
      "flags": "\u0007\b\f\r"
    }
  ],
  "snowman": "\u2603"
}
"#;

const QUERIES: [&str; 10] = [
    "users[1].name",
    "meta.count",
    "users[0].tags[2]",
    "users[2].prefs",
    "meta.archived",
    "users[1].email",
    "users[3]",
    "meta[0]",
    "users.name",
    "users[x]",
];

const BAD_INPUTS: [&str; 11] = [
    r#"{"name": "Ann", "age": 30,}"#,
    r#"{"name" "Ann"}"#,
    "[\n  \"a\",\n  \"b\n]",
    r#"{"path": "C:\qdir"}"#,
    "[1, 2] [3]",
    "[1, @]",
    r#"{"a": {"b": [1, 2"#,
    "[-]",
    "{\n  \"x\": 1.,\n}",
    r#""bad \u12G4""#,
    "",
];

fn main() -> Result<()> {
    let doc = parse(DOCUMENT)?;
    println!("== pretty ==");
    println!("{}", pretty(&doc, 0));

    println!("== compact ==");
    let text = compact(&doc);
    println!("{text}");
    println!(
        "round trip: {}",
        if parse(&text)? == doc {
            "ok"
        } else {
            "MISMATCH"
        }
    );

    println!("== queries ==");
    for path in QUERIES {
        let Some(steps) = parse_path(path) else {
            println!("bad path: {path}");
            continue;
        };
        match query(&doc, &steps) {
            Some(result) => println!("{path} = {}", compact(result)),
            None => println!("not found: {path}"),
        }
    }

    println!("== stats ==");
    let mut stats = Stats::default();
    walk(&doc, 1, &mut stats);
    for kind in KINDS {
        println!("{kind}: {}", stats.counts.get(kind).unwrap_or(&0));
    }
    println!("max depth: {}", stats.max_depth);
    println!("sum of numbers: {:?}", stats.total);

    println!("== errors ==");
    for (i, text) in BAD_INPUTS.iter().enumerate() {
        match parse(text) {
            Ok(value) => println!("{}: ok {}", i + 1, compact(&value)),
            Err(err) => println!("{}: {err}", i + 1),
        }
    }
    Ok(())
}
