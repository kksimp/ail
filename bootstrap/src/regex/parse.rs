//! The pattern parser of `std.regex` (std.regex.3, std.regex.9-10): pattern
//! text to a syntax tree. Every failure is a `ParseError` whose `kind` has
//! a spec rule (std.regex.11-21).

use super::props::{Prop, UNICODE_NAMES};

/// One item of a class: a range of characters, or a named set (negated or not).
#[derive(Debug, Clone)]
pub enum Item {
    Range(char, char),
    Prop(Prop, bool),
}

/// A character class: items, and whether the class is negated.
#[derive(Debug, Clone)]
pub struct Class {
    pub neg: bool,
    pub items: Vec<Item>,
}

/// A zero-width assertion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Assert {
    BeginText,
    EndText,
    BeginLine,
    EndLine,
    WordBoundary,
    NotWordBoundary,
}

#[derive(Debug, Clone)]
pub enum Node {
    Empty,
    Char(char, bool),
    Any(bool),
    Class(Class, bool),
    Assert(Assert),
    Cat(Vec<Node>),
    Alt(Vec<Node>),
    Group(Option<usize>, Box<Node>),
    Repeat { node: Box<Node>, min: u32, max: Option<u32>, greedy: bool },
}

#[derive(Debug, Clone)]
pub struct ParseError {
    /// The 0-based character offset of the problem in the pattern.
    pub at: usize,
    /// The short kind (std.regex.11-21).
    pub kind: &'static str,
    pub message: String,
}

pub struct Parsed {
    pub node: Node,
    /// The number of groups, counting the whole match as group 0.
    pub groups: usize,
    pub names: Vec<(String, usize)>,
}

/// The most a counted repetition may count (std.regex.3).
pub const MAX_REPEAT: u32 = 1000;

#[derive(Clone, Copy)]
struct Flags {
    i: bool,
    s: bool,
    m: bool,
}

struct P<'a> {
    s: &'a [char],
    pos: usize,
    groups: usize,
    names: Vec<(String, usize)>,
    depth: usize,
}

type R<T> = Result<T, ParseError>;

fn err<T>(at: usize, kind: &'static str, message: impl Into<String>) -> R<T> {
    Err(ParseError { at, kind, message: message.into() })
}

pub fn parse(pattern: &str) -> R<Parsed> {
    let chars: Vec<char> = pattern.chars().collect();
    let mut p = P { s: &chars, pos: 0, groups: 1, names: Vec::new(), depth: 0 };
    let node = p.alt(Flags { i: false, s: false, m: false })?;
    if p.pos < p.s.len() {
        // Only an unmatched `)` stops `alt` before the end.
        return err(p.pos, "unopened_group", "`)` has no matching `(`");
    }
    Ok(Parsed { node, groups: p.groups, names: p.names })
}

impl P<'_> {
    fn peek(&self) -> Option<char> {
        self.s.get(self.pos).copied()
    }
    fn peek_at(&self, k: usize) -> Option<char> {
        self.s.get(self.pos + k).copied()
    }
    fn eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn alt(&mut self, mut f: Flags) -> R<Node> {
        let mut alts = vec![self.cat(&mut f)?];
        while self.eat('|') {
            alts.push(self.cat(&mut f)?);
        }
        Ok(if alts.len() == 1 { alts.pop().unwrap() } else { Node::Alt(alts) })
    }

    fn cat(&mut self, f: &mut Flags) -> R<Node> {
        let mut items = Vec::new();
        while let Some(c) = self.peek() {
            if c == '|' || c == ')' {
                break;
            }
            let start = self.pos;
            let Some(atom) = self.atom(f)? else { continue };
            let atom = self.repeats(atom, start)?;
            items.push(atom);
        }
        Ok(match items.len() {
            0 => Node::Empty,
            1 => items.pop().unwrap(),
            _ => Node::Cat(items),
        })
    }

    /// Repetition operators after an atom; `a**` is an error, `a*?` is lazy.
    fn repeats(&mut self, atom: Node, start: usize) -> R<Node> {
        let mut node = atom;
        let mut repeated = false;
        loop {
            let at = self.pos;
            let (min, max) = match self.peek() {
                Some('*') => {
                    self.pos += 1;
                    (0, None)
                }
                Some('+') => {
                    self.pos += 1;
                    (1, None)
                }
                Some('?') => {
                    self.pos += 1;
                    (0, Some(1))
                }
                Some('{') => match self.counted()? {
                    Some(r) => r,
                    None => return Ok(node),
                },
                _ => return Ok(node),
            };
            if repeated {
                return err(
                    at,
                    "bad_repeat",
                    "a repetition operator can't follow another one; group it with `(?:...)`",
                );
            }
            let greedy = !self.eat('?');
            node = Node::Repeat { node: Box::new(node), min, max, greedy };
            repeated = true;
            let _ = start;
        }
    }

    /// `{n}`, `{n,}` or `{n,m}` at the cursor; `None` (nothing consumed) when
    /// the `{` is a literal.
    fn counted(&mut self) -> R<Option<(u32, Option<u32>)>> {
        let at = self.pos;
        let mut k = 1;
        let num = |this: &Self, k: &mut usize| -> Option<u64> {
            let from = *k;
            let mut v: u64 = 0;
            while let Some(d) = this.peek_at(*k).and_then(|c| c.to_digit(10)) {
                v = (v * 10 + u64::from(d)).min(1 << 40);
                *k += 1;
            }
            (*k > from).then_some(v)
        };
        let Some(lo) = num(self, &mut k) else { return Ok(None) };
        let hi = if self.peek_at(k) == Some(',') {
            k += 1;
            if self.peek_at(k) == Some('}') { Some(None) } else { num(self, &mut k).map(Some) }
        } else {
            Some(Some(lo))
        };
        let Some(hi) = hi else { return Ok(None) };
        if self.peek_at(k) != Some('}') {
            return Ok(None);
        }
        self.pos += k + 1;
        let top = u64::from(MAX_REPEAT);
        if lo > top || hi.is_some_and(|h| h > top) {
            return err(at, "bad_repeat", format!("a repetition count can be at most {MAX_REPEAT}"));
        }
        if hi.is_some_and(|h| h < lo) {
            return err(at, "bad_repeat", "the lower repetition count is greater than the upper one");
        }
        Ok(Some((lo as u32, hi.map(|h| h as u32))))
    }

    fn atom(&mut self, f: &mut Flags) -> R<Option<Node>> {
        let at = self.pos;
        let c = self.s[self.pos];
        self.pos += 1;
        Ok(Some(match c {
            '(' => return self.group(f, at),
            '[' => Node::Class(self.class(at)?, f.i),
            '.' => Node::Any(f.s),
            '^' => Node::Assert(if f.m { Assert::BeginLine } else { Assert::BeginText }),
            '$' => Node::Assert(if f.m { Assert::EndLine } else { Assert::EndText }),
            '*' | '+' | '?' => return err(at, "nothing_to_repeat", format!("`{c}` has nothing before it to repeat")),
            '\\' => self.escape(f, at)?,
            c => Node::Char(c, f.i),
        }))
    }

    fn group(&mut self, f: &mut Flags, at: usize) -> R<Option<Node>> {
        self.depth += 1;
        if self.depth > 200 {
            return err(at, "too_large", "groups are nested more than 200 deep");
        }
        let mut inner = *f;
        let mut index = None;
        if self.eat('?') {
            match self.peek() {
                Some(':') => self.pos += 1,
                Some('P') | Some('<') => {
                    if self.peek() == Some('<') && matches!(self.peek_at(1), Some('=' | '!')) {
                        return err(at, "unsupported", "lookbehind isn't supported: the engine runs in linear time");
                    }
                    if self.peek() == Some('P') {
                        if matches!(self.peek_at(1), Some('=' | '>')) {
                            return err(
                                at,
                                "unsupported",
                                "named backreferences aren't supported: the engine runs in linear time",
                            );
                        }
                        self.pos += 1;
                        if self.peek() != Some('<') {
                            return err(self.pos, "bad_flag", "`(?P` must be followed by `<name>`");
                        }
                    }
                    self.pos += 1;
                    let name_at = self.pos;
                    let mut name = String::new();
                    while let Some(c) = self.peek() {
                        if c == '>' {
                            break;
                        }
                        if !(c.is_ascii_alphanumeric() || c == '_') {
                            return err(self.pos, "bad_group_name", format!("`{c}` can't be in a group name"));
                        }
                        name.push(c);
                        self.pos += 1;
                    }
                    if !self.eat('>') {
                        return err(name_at, "bad_group_name", "a group name must end with `>`");
                    }
                    if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) {
                        return err(
                            name_at,
                            "bad_group_name",
                            "a group name is not empty and doesn't start with a digit",
                        );
                    }
                    if self.names.iter().any(|(n, _)| *n == name) {
                        return err(name_at, "bad_group_name", format!("two groups are named `{name}`"));
                    }
                    index = Some(self.groups);
                    self.groups += 1;
                    self.names.push((name, index.unwrap()));
                }
                Some('=' | '!') => {
                    return err(at, "unsupported", "lookahead isn't supported: the engine runs in linear time");
                }
                _ => {
                    // Flags: `(?i)`, `(?-s)`, `(?im:...)`.
                    let mut on = true;
                    loop {
                        let Some(c) = self.peek() else {
                            return err(at, "unclosed_group", "`(` is never closed");
                        };
                        self.pos += 1;
                        match c {
                            'i' => inner.i = on,
                            's' => inner.s = on,
                            'm' => inner.m = on,
                            '-' if on => on = false,
                            ')' => {
                                // `(?i)`: applies to the rest of the enclosing group.
                                *f = inner;
                                self.depth -= 1;
                                return Ok(None);
                            }
                            ':' => break,
                            _ => {
                                return err(
                                    self.pos - 1,
                                    "bad_flag",
                                    format!("`{c}` isn't a flag; the flags are i, s and m"),
                                );
                            }
                        }
                    }
                }
            }
        } else {
            index = Some(self.groups);
            self.groups += 1;
        }
        let body = self.alt(inner)?;
        if !self.eat(')') {
            return err(at, "unclosed_group", "`(` is never closed");
        }
        self.depth -= 1;
        Ok(Some(Node::Group(index, Box::new(body))))
    }

    fn hex(&mut self, at: usize) -> R<char> {
        let bad = || err(at, "bad_escape", "`\\x` needs two hex digits or `{hex}`");
        let code = if self.eat('{') {
            let mut v: u32 = 0;
            let mut n = 0;
            while let Some(d) = self.peek().and_then(|c| c.to_digit(16)) {
                v = v.saturating_mul(16).saturating_add(d);
                n += 1;
                self.pos += 1;
            }
            if n == 0 || !self.eat('}') {
                return bad();
            }
            v
        } else {
            let (Some(a), Some(b)) =
                (self.peek().and_then(|c| c.to_digit(16)), self.peek_at(1).and_then(|c| c.to_digit(16)))
            else {
                return bad();
            };
            self.pos += 2;
            a * 16 + b
        };
        match char::from_u32(code) {
            Some(c) => Ok(c),
            None => err(at, "bad_escape", format!("`{code:X}` isn't a Unicode scalar value")),
        }
    }

    fn prop_name(&mut self, at: usize, neg: bool) -> R<Item> {
        let name = if self.eat('{') {
            let mut n = String::new();
            loop {
                match self.peek() {
                    Some('}') => {
                        self.pos += 1;
                        break;
                    }
                    Some(c) => {
                        n.push(c);
                        self.pos += 1;
                    }
                    None => return err(at, "bad_class", "`\\p{` is never closed"),
                }
            }
            n
        } else {
            match self.peek() {
                Some(c) => {
                    self.pos += 1;
                    c.to_string()
                }
                None => return err(at, "bad_escape", "`\\p` needs a class name"),
            }
        };
        let (name, neg) = match name.strip_prefix('^') {
            Some(rest) => (rest.to_string(), !neg),
            None => (name, neg),
        };
        match Prop::unicode(&name) {
            Some(p) => Ok(Item::Prop(p, neg)),
            None => {
                err(at, "bad_class", format!("`{name}` isn't a supported Unicode class; supported: {UNICODE_NAMES}"))
            }
        }
    }

    /// The escape after a `\` outside a class.
    fn escape(&mut self, f: &Flags, at: usize) -> R<Node> {
        let Some(c) = self.peek() else {
            return err(at, "bad_escape", "a pattern can't end with `\\`");
        };
        self.pos += 1;
        let perl = |p: Prop, neg: bool| Node::Class(Class { neg: false, items: vec![Item::Prop(p, neg)] }, false);
        Ok(match c {
            'd' => perl(Prop::Digit, false),
            'D' => perl(Prop::Digit, true),
            'w' => perl(Prop::Word, false),
            'W' => perl(Prop::Word, true),
            's' => perl(Prop::Space, false),
            'S' => perl(Prop::Space, true),
            'p' | 'P' => {
                let item = self.prop_name(at, c == 'P')?;
                Node::Class(Class { neg: false, items: vec![item] }, f.i)
            }
            'b' => Node::Assert(Assert::WordBoundary),
            'B' => Node::Assert(Assert::NotWordBoundary),
            'A' => Node::Assert(Assert::BeginText),
            'z' => Node::Assert(Assert::EndText),
            '1'..='9' => {
                return err(at, "unsupported", "backreferences aren't supported: the engine runs in linear time");
            }
            _ => Node::Char(self.char_escape(c, at)?, f.i),
        })
    }

    /// The character an escape letter or punctuation stands for.
    fn char_escape(&mut self, c: char, at: usize) -> R<char> {
        Ok(match c {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            'f' => '\x0C',
            'v' => '\x0B',
            'a' => '\x07',
            'x' => self.hex(at)?,
            c if !c.is_ascii_alphanumeric() => c,
            c => return err(at, "bad_escape", format!("`\\{c}` isn't an escape")),
        })
    }

    fn class(&mut self, at: usize) -> R<Class> {
        let mut cls = Class { neg: self.eat('^'), items: Vec::new() };
        let mut first = true;
        loop {
            let Some(c) = self.peek() else {
                return err(at, "unclosed_class", "`[` is never closed");
            };
            if c == ']' && !first {
                self.pos += 1;
                return Ok(cls);
            }
            first = false;
            let item_at = self.pos;
            if c == '[' && self.peek_at(1) == Some(':') {
                if let Some(end) =
                    (self.pos + 2..self.s.len().saturating_sub(1)).find(|&k| self.s[k] == ':' && self.s[k + 1] == ']')
                {
                    let name: String = self.s[self.pos + 2..end].iter().collect();
                    let (name, neg) = match name.strip_prefix('^') {
                        Some(r) => (r.to_string(), true),
                        None => (name, false),
                    };
                    let Some(p) = Prop::posix(&name) else {
                        return err(item_at, "bad_class", format!("`[:{name}:]` isn't a POSIX class"));
                    };
                    cls.items.push(Item::Prop(p, neg));
                    self.pos = end + 2;
                    continue;
                }
            }
            // One character (or a set) on the left.
            let lo = match self.class_atom(item_at)? {
                Atom::Set(item) => {
                    cls.items.push(item);
                    continue;
                }
                Atom::Char(c) => c,
            };
            if self.peek() == Some('-') && self.peek_at(1).is_some_and(|c| c != ']') {
                self.pos += 1;
                let hi_at = self.pos;
                let Atom::Char(hi) = self.class_atom(hi_at)? else {
                    return err(hi_at, "bad_range", "a range ends at a single character, not a class");
                };
                if hi < lo {
                    return err(item_at, "bad_range", format!("the range `{lo}-{hi}` runs backwards"));
                }
                cls.items.push(Item::Range(lo, hi));
            } else {
                cls.items.push(Item::Range(lo, lo));
            }
        }
    }

    fn class_atom(&mut self, at: usize) -> R<Atom> {
        let c = self.s[self.pos];
        self.pos += 1;
        if c != '\\' {
            return Ok(Atom::Char(c));
        }
        let Some(e) = self.peek() else {
            return err(at, "unclosed_class", "`[` is never closed");
        };
        self.pos += 1;
        Ok(match e {
            'd' => Atom::Set(Item::Prop(Prop::Digit, false)),
            'D' => Atom::Set(Item::Prop(Prop::Digit, true)),
            'w' => Atom::Set(Item::Prop(Prop::Word, false)),
            'W' => Atom::Set(Item::Prop(Prop::Word, true)),
            's' => Atom::Set(Item::Prop(Prop::Space, false)),
            'S' => Atom::Set(Item::Prop(Prop::Space, true)),
            'p' | 'P' => Atom::Set(self.prop_name(at, e == 'P')?),
            e => Atom::Char(self.char_escape(e, at)?),
        })
    }
}

enum Atom {
    Char(char),
    Set(Item),
}
