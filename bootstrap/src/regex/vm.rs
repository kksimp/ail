//! The matcher of `std.regex` (std.regex.4): a pattern tree compiled to a
//! small program, run by a Pike VM. It keeps one thread per program counter
//! and steps all of them across the text once, so a match costs
//! O(text x program) time and no pattern backtracks (ADR 0438).

use super::parse::{Assert, Class, Item, Node, Parsed};

/// The largest compiled program, in instructions (std.regex.3).
const MAX_PROG: usize = 50_000;

const NONE: usize = usize::MAX;

#[derive(Debug, Clone)]
enum Inst {
    Char(char),
    CharFold(char),
    Any,
    AnyNoNl,
    Class(usize, bool),
    Split(usize, usize),
    Jmp(usize),
    Save(usize),
    Assert(Assert),
    Match,
}

#[derive(Debug)]
pub struct Prog {
    insts: Vec<Inst>,
    classes: Vec<Class>,
    /// Groups, counting the whole match as group 0.
    pub groups: usize,
    pub names: Vec<(String, usize)>,
    /// The program can only match at the start of the text.
    anchored: bool,
}

pub struct TooLarge;

struct Compiler {
    insts: Vec<Inst>,
    classes: Vec<Class>,
}

impl Compiler {
    fn push(&mut self, i: Inst) -> Result<usize, TooLarge> {
        if self.insts.len() >= MAX_PROG {
            return Err(TooLarge);
        }
        self.insts.push(i);
        Ok(self.insts.len() - 1)
    }

    fn emit(&mut self, n: &Node) -> Result<(), TooLarge> {
        match n {
            Node::Empty => {}
            Node::Char(c, fold) => {
                if *fold && (lower(*c) != *c || upper(*c) != *c) {
                    self.push(Inst::CharFold(lower(*c)))?;
                } else {
                    self.push(Inst::Char(*c))?;
                }
            }
            Node::Any(dotall) => {
                self.push(if *dotall { Inst::Any } else { Inst::AnyNoNl })?;
            }
            Node::Class(c, fold) => {
                self.classes.push(c.clone());
                let k = self.classes.len() - 1;
                self.push(Inst::Class(k, *fold))?;
            }
            Node::Assert(a) => {
                self.push(Inst::Assert(*a))?;
            }
            Node::Cat(items) => {
                for i in items {
                    self.emit(i)?;
                }
            }
            Node::Alt(alts) => {
                let mut jumps = Vec::new();
                for (k, a) in alts.iter().enumerate() {
                    if k + 1 < alts.len() {
                        let split = self.push(Inst::Split(0, 0))?;
                        self.emit(a)?;
                        jumps.push(self.push(Inst::Jmp(0))?);
                        let next = self.insts.len();
                        self.insts[split] = Inst::Split(split + 1, next);
                    } else {
                        self.emit(a)?;
                    }
                }
                let end = self.insts.len();
                for j in jumps {
                    self.insts[j] = Inst::Jmp(end);
                }
            }
            Node::Group(index, body) => match index {
                Some(i) => {
                    self.push(Inst::Save(2 * i))?;
                    self.emit(body)?;
                    self.push(Inst::Save(2 * i + 1))?;
                }
                None => self.emit(body)?,
            },
            Node::Repeat { node, min, max, greedy } => {
                for _ in 0..*min {
                    self.emit(node)?;
                }
                let split = |at: usize, exit: usize| {
                    if *greedy { Inst::Split(at + 1, exit) } else { Inst::Split(exit, at + 1) }
                };
                match max {
                    None => {
                        let l = self.push(Inst::Split(0, 0))?;
                        self.emit(node)?;
                        self.push(Inst::Jmp(l))?;
                        let end = self.insts.len();
                        self.insts[l] = split(l, end);
                    }
                    Some(max) => {
                        let mut splits = Vec::new();
                        for _ in *min..*max {
                            splits.push(self.push(Inst::Split(0, 0))?);
                            self.emit(node)?;
                        }
                        let end = self.insts.len();
                        for s in splits {
                            self.insts[s] = split(s, end);
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn compile(p: &Parsed) -> Result<Prog, TooLarge> {
    let mut c = Compiler { insts: Vec::new(), classes: Vec::new() };
    c.push(Inst::Save(0))?;
    c.emit(&p.node)?;
    c.push(Inst::Save(1))?;
    c.push(Inst::Match)?;
    let anchored = matches!(c.insts.get(1), Some(Inst::Assert(Assert::BeginText)));
    Ok(Prog { insts: c.insts, classes: c.classes, groups: p.groups, names: p.names.clone(), anchored })
}

/// The simple lowercase of `c`: itself when it has none or several.
fn lower(c: char) -> char {
    let mut it = c.to_lowercase();
    match (it.next(), it.next()) {
        (Some(l), None) => l,
        _ => c,
    }
}

fn upper(c: char) -> char {
    let mut it = c.to_uppercase();
    match (it.next(), it.next()) {
        (Some(u), None) => u,
        _ => c,
    }
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

impl Class {
    fn has(&self, c: char) -> bool {
        self.items.iter().any(|i| match i {
            Item::Range(lo, hi) => *lo <= c && c <= *hi,
            Item::Prop(p, neg) => p.test(c) != *neg,
        })
    }

    fn matches(&self, c: char, fold: bool) -> bool {
        let mut hit = self.has(c);
        if !hit && fold {
            hit = self.has(lower(c)) || self.has(upper(c));
        }
        hit != self.neg
    }
}

enum Frame {
    Pc(usize),
    Restore(usize, usize),
}

struct List {
    items: Vec<(usize, Vec<usize>)>,
    seen: Vec<u32>,
    stamp: u32,
}

impl List {
    fn new(n: usize) -> List {
        List { items: Vec::new(), seen: vec![0; n], stamp: 1 }
    }
    fn clear(&mut self) {
        self.items.clear();
        self.stamp += 1;
    }
}

impl Prog {
    fn holds(&self, a: Assert, text: &[char], pos: usize) -> bool {
        let before = pos.checked_sub(1).map(|i| text[i]);
        let after = text.get(pos).copied();
        match a {
            Assert::BeginText => pos == 0,
            Assert::EndText => pos == text.len(),
            Assert::BeginLine => pos == 0 || before == Some('\n'),
            Assert::EndLine => pos == text.len() || after == Some('\n'),
            Assert::WordBoundary => before.is_some_and(is_word) != after.is_some_and(is_word),
            Assert::NotWordBoundary => before.is_some_and(is_word) == after.is_some_and(is_word),
        }
    }

    /// Follow empty transitions from `pc` at `pos` in priority order, adding
    /// each reachable consuming instruction (and `Match`) to `list`.
    fn add(&self, list: &mut List, pc: usize, pos: usize, caps: &mut Vec<usize>, text: &[char]) {
        let mut stack = vec![Frame::Pc(pc)];
        while let Some(f) = stack.pop() {
            let pc = match f {
                Frame::Restore(slot, old) => {
                    caps[slot] = old;
                    continue;
                }
                Frame::Pc(pc) => pc,
            };
            if list.seen[pc] == list.stamp {
                continue;
            }
            list.seen[pc] = list.stamp;
            match &self.insts[pc] {
                Inst::Jmp(x) => stack.push(Frame::Pc(*x)),
                Inst::Split(x, y) => {
                    stack.push(Frame::Pc(*y));
                    stack.push(Frame::Pc(*x));
                }
                Inst::Save(n) => {
                    if *n < caps.len() {
                        stack.push(Frame::Restore(*n, caps[*n]));
                        caps[*n] = pos;
                    }
                    stack.push(Frame::Pc(pc + 1));
                }
                Inst::Assert(a) => {
                    if self.holds(*a, text, pos) {
                        stack.push(Frame::Pc(pc + 1));
                    }
                }
                _ => list.items.push((pc, caps.clone())),
            }
        }
    }

    /// The leftmost-first match at or after `start` (std.regex.4): the
    /// capture offsets, two per group, `usize::MAX` for a group that didn't
    /// take part. `slots` is how many to keep (0, 2 or `2 * groups`); with
    /// `earliest` the first match found is returned without resolving
    /// priorities, which is enough to say whether there is one.
    pub fn exec(&self, text: &[char], start: usize, slots: usize, earliest: bool) -> Option<Vec<usize>> {
        let n = self.insts.len();
        let (mut clist, mut nlist) = (List::new(n), List::new(n));
        let mut matched: Option<Vec<usize>> = None;
        let mut caps = vec![NONE; slots];
        let mut pos = start;
        if self.anchored && start > 0 {
            return None;
        }
        loop {
            if matched.is_none() && (!self.anchored || pos == start) {
                caps.fill(NONE);
                self.add(&mut clist, 0, pos, &mut caps, text);
            }
            if clist.items.is_empty() && (matched.is_some() || self.anchored || pos >= text.len()) {
                break;
            }
            let c = text.get(pos).copied();
            nlist.clear();
            let items = std::mem::take(&mut clist.items);
            for (pc, tcaps) in items {
                let ok = match &self.insts[pc] {
                    Inst::Match => {
                        if earliest {
                            return Some(tcaps);
                        }
                        matched = Some(tcaps);
                        break;
                    }
                    Inst::Char(x) => c == Some(*x),
                    Inst::CharFold(x) => c.is_some_and(|c| lower(c) == *x),
                    Inst::Any => c.is_some(),
                    Inst::AnyNoNl => c.is_some_and(|c| c != '\n'),
                    Inst::Class(k, fold) => c.is_some_and(|c| self.classes[*k].matches(c, *fold)),
                    _ => unreachable!("only consuming instructions and Match are in a thread list"),
                };
                if ok {
                    let mut w = tcaps;
                    self.add(&mut nlist, pc + 1, pos + 1, &mut w, text);
                }
            }
            std::mem::swap(&mut clist, &mut nlist);
            if pos >= text.len() {
                break;
            }
            pos += 1;
        }
        // Threads that survived to the end of the text may sit on Match.
        matched
    }
}
