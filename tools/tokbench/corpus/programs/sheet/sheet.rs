use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::iter::Peekable;
use std::str::Chars;

const COLS: &str = "ABCDEFGH";
const MAX_ROW: usize = 20;

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Pos {
    col: usize,
    row: usize,
}

fn col_name(col: usize) -> char {
    COLS.as_bytes()[col] as char
}

impl fmt::Display for Pos {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", col_name(self.col), self.row)
    }
}

fn parse_pos(text: &str) -> Option<Pos> {
    let col = COLS.find(text.chars().next()?)?;
    let digits = &text[1..];
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let row = digits.parse().ok()?;
    if !(1..=MAX_ROW).contains(&row) {
        return None;
    }
    Some(Pos { col, row })
}

fn cells_in(first: Pos, last: Pos) -> Vec<Pos> {
    let cols = first.col.min(last.col)..=first.col.max(last.col);
    let rows = first.row.min(last.row)..=first.row.max(last.row);
    rows.flat_map(|row| cols.clone().map(move |col| Pos { col, row }))
        .collect()
}

fn names(cells: &[Pos]) -> String {
    cells
        .iter()
        .map(Pos::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Clone, Copy)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy)]
enum Cmp {
    Less,
    Greater,
    Equal,
}

#[derive(Clone, Copy)]
enum Func {
    Sum,
    Min,
    Max,
    Count,
    Avg,
}

impl Func {
    fn parse(name: &str) -> Option<Func> {
        match name {
            "SUM" => Some(Func::Sum),
            "MIN" => Some(Func::Min),
            "MAX" => Some(Func::Max),
            "COUNT" => Some(Func::Count),
            "AVG" => Some(Func::Avg),
            _ => None,
        }
    }
}

enum Expr {
    Lit(i64),
    Ref(Pos),
    Neg(Box<Expr>),
    Bin(Op, Box<Expr>, Box<Expr>),
    Agg(Func, Pos, Pos),
    If {
        cmp: Cmp,
        left: Box<Expr>,
        right: Box<Expr>,
        then: Box<Expr>,
        other: Box<Expr>,
    },
}

const SYMBOLS: &str = "+-*/(),:<>=";

#[derive(Clone, PartialEq)]
enum Token {
    Num(String),
    Cell(String, Pos),
    Name(String),
    Sym(char),
    End,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Num(text) | Token::Cell(text, _) | Token::Name(text) => write!(f, "{text}"),
            Token::Sym(c) => write!(f, "'{c}'"),
            Token::End => write!(f, "end of formula"),
        }
    }
}

enum ParseError {
    BadCell(String),
    BadChar(char),
    Expected(String, Token),
    Unexpected(Token),
    UnknownFunc(String),
    TooLarge(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::BadCell(text) => write!(f, "bad cell name {text}"),
            ParseError::BadChar(c) => write!(f, "unexpected character '{c}'"),
            ParseError::Expected(what, found) => write!(f, "expected {what} but found {found}"),
            ParseError::Unexpected(found) => write!(f, "unexpected {found}"),
            ParseError::UnknownFunc(name) => write!(f, "unknown function {name}"),
            ParseError::TooLarge(text) => write!(f, "number too large: {text}"),
        }
    }
}

fn take_while(chars: &mut Peekable<Chars>, pred: fn(&char) -> bool) -> String {
    let mut text = String::new();
    while let Some(c) = chars.next_if(pred) {
        text.push(c);
    }
    text
}

fn tokenize(src: &str) -> Result<Vec<Token>, ParseError> {
    let mut tokens = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c == ' ' {
            chars.next();
        } else if c.is_ascii_digit() {
            tokens.push(Token::Num(take_while(&mut chars, char::is_ascii_digit)));
        } else if c.is_ascii_alphabetic() {
            let word = take_while(&mut chars, char::is_ascii_alphanumeric);
            if word.ends_with(|c: char| c.is_ascii_digit()) {
                let pos = parse_pos(&word).ok_or_else(|| ParseError::BadCell(word.clone()))?;
                tokens.push(Token::Cell(word, pos));
            } else {
                tokens.push(Token::Name(word));
            }
        } else if SYMBOLS.contains(c) {
            tokens.push(Token::Sym(c));
            chars.next();
        } else {
            return Err(ParseError::BadChar(c));
        }
    }
    tokens.push(Token::End);
    Ok(tokens)
}

struct Parser {
    tokens: Vec<Token>,
    at: usize,
}

impl Parser {
    fn peek(&self) -> &Token {
        &self.tokens[self.at]
    }

    fn next(&mut self) -> Token {
        let tok = self.tokens[self.at].clone();
        if tok != Token::End {
            self.at += 1;
        }
        tok
    }

    fn take(&mut self, syms: &str) -> Option<char> {
        match self.peek() {
            Token::Sym(c) if syms.contains(*c) => {
                let c = *c;
                self.at += 1;
                Some(c)
            }
            _ => None,
        }
    }

    fn expect(&mut self, sym: char) -> Result<(), ParseError> {
        let tok = self.next();
        if tok != Token::Sym(sym) {
            return Err(ParseError::Expected(format!("'{sym}'"), tok));
        }
        Ok(())
    }

    fn cell(&mut self) -> Result<Pos, ParseError> {
        match self.next() {
            Token::Cell(_, pos) => Ok(pos),
            tok => Err(ParseError::Expected("a cell".to_string(), tok)),
        }
    }

    fn formula(&mut self) -> Result<Expr, ParseError> {
        let expr = self.sum()?;
        match self.peek() {
            Token::End => Ok(expr),
            tok => Err(ParseError::Unexpected(tok.clone())),
        }
    }

    fn sum(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.product()?;
        while let Some(c) = self.take("+-") {
            let op = if c == '+' { Op::Add } else { Op::Sub };
            left = Expr::Bin(op, Box::new(left), Box::new(self.product()?));
        }
        Ok(left)
    }

    fn product(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.unary()?;
        while let Some(c) = self.take("*/") {
            let op = if c == '*' { Op::Mul } else { Op::Div };
            left = Expr::Bin(op, Box::new(left), Box::new(self.unary()?));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.take("-").is_some() {
            return Ok(Expr::Neg(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        match self.next() {
            Token::Num(text) => text
                .parse()
                .map(Expr::Lit)
                .map_err(|_| ParseError::TooLarge(text)),
            Token::Cell(_, pos) => Ok(Expr::Ref(pos)),
            Token::Name(name) => self.call(&name),
            Token::Sym('(') => {
                let inner = self.sum()?;
                self.expect(')')?;
                Ok(inner)
            }
            tok => Err(ParseError::Unexpected(tok)),
        }
    }

    fn call(&mut self, name: &str) -> Result<Expr, ParseError> {
        if let Some(func) = Func::parse(name) {
            self.expect('(')?;
            let first = self.cell()?;
            self.expect(':')?;
            let last = self.cell()?;
            self.expect(')')?;
            return Ok(Expr::Agg(func, first, last));
        }
        if name != "IF" {
            return Err(ParseError::UnknownFunc(name.to_string()));
        }
        self.expect('(')?;
        let left = Box::new(self.sum()?);
        let cmp = match self.next() {
            Token::Sym('<') => Cmp::Less,
            Token::Sym('>') => Cmp::Greater,
            Token::Sym('=') => Cmp::Equal,
            tok => return Err(ParseError::Expected("a comparison".to_string(), tok)),
        };
        let right = Box::new(self.sum()?);
        self.expect(',')?;
        let then = Box::new(self.sum()?);
        self.expect(',')?;
        let other = Box::new(self.sum()?);
        self.expect(')')?;
        Ok(Expr::If {
            cmp,
            left,
            right,
            then,
            other,
        })
    }
}

fn parse_formula(src: &str) -> Result<Expr, ParseError> {
    Parser {
        tokens: tokenize(src)?,
        at: 0,
    }
    .formula()
}

fn collect_refs(expr: &Expr, out: &mut BTreeSet<Pos>) {
    match expr {
        Expr::Lit(_) => {}
        Expr::Ref(pos) => {
            out.insert(*pos);
        }
        Expr::Neg(operand) => collect_refs(operand, out),
        Expr::Bin(_, left, right) => {
            collect_refs(left, out);
            collect_refs(right, out);
        }
        Expr::Agg(_, first, last) => out.extend(cells_in(*first, *last)),
        Expr::If {
            left,
            right,
            then,
            other,
            ..
        } => {
            for part in [left, right, then, other] {
                collect_refs(part, out);
            }
        }
    }
}

enum Content {
    Number(i64),
    Label(String),
    Formula(Expr),
    Broken(String, ParseError),
}

#[derive(Clone, Copy)]
enum CellError {
    Value,
    Cycle,
    DivZero,
    Parse,
}

impl fmt::Display for CellError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(match self {
            CellError::Value => "#VALUE!",
            CellError::Cycle => "#CYCLE!",
            CellError::DivZero => "#DIV/0!",
            CellError::Parse => "#PARSE!",
        })
    }
}

#[derive(Clone)]
enum Value {
    Num(i64),
    Text(String),
    Fail(CellError),
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Num(n) => write!(f, "{n}"),
            Value::Text(text) => write!(f, "{text}"),
            Value::Fail(err) => write!(f, "{err}"),
        }
    }
}

fn div(a: i64, b: i64) -> Result<i64, CellError> {
    if b == 0 {
        return Err(CellError::DivZero);
    }
    Ok(a / b)
}

enum LoadError {
    MissingEquals,
    BadName(String),
    BadContent(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LoadError::MissingEquals => write!(f, "missing '='"),
            LoadError::BadName(text) => write!(f, "bad cell name '{text}'"),
            LoadError::BadContent(text) => write!(f, "bad content '{text}'"),
        }
    }
}

fn parse_content(text: &str) -> Result<Option<Content>, LoadError> {
    if text.is_empty() {
        return Ok(None);
    }
    if let Some(rest) = text.strip_prefix('=') {
        let src = rest.trim();
        return Ok(Some(match parse_formula(src) {
            Ok(expr) => Content::Formula(expr),
            Err(err) => Content::Broken(src.to_string(), err),
        }));
    }
    if let Some(label) = text.strip_prefix('"').and_then(|t| t.strip_suffix('"')) {
        return Ok(Some(Content::Label(label.to_string())));
    }
    match text.parse() {
        Ok(n) => Ok(Some(Content::Number(n))),
        Err(_) => Err(LoadError::BadContent(text.to_string())),
    }
}

fn parse_line(line: &str) -> Result<(Pos, Option<Content>), LoadError> {
    let (name, rest) = line.split_once('=').ok_or(LoadError::MissingEquals)?;
    let name = name.trim();
    let pos = parse_pos(name).ok_or_else(|| LoadError::BadName(name.to_string()))?;
    Ok((pos, parse_content(rest.trim())?))
}

#[derive(Default)]
struct Sheet {
    cells: BTreeMap<Pos, Content>,
    memo: RefCell<HashMap<Pos, Value>>,
    active: RefCell<HashSet<Pos>>,
}

impl Sheet {
    fn apply(&mut self, lines: &[&str]) {
        for (index, line) in lines.iter().enumerate() {
            match parse_line(line) {
                Ok((pos, Some(content))) => {
                    self.cells.insert(pos, content);
                }
                Ok((pos, None)) => {
                    self.cells.remove(&pos);
                }
                Err(err) => println!("line {}: {err}: {line}", index + 1),
            }
        }
        self.memo.borrow_mut().clear();
    }

    fn value(&self, pos: Pos) -> Value {
        if let Some(known) = self.memo.borrow().get(&pos) {
            return known.clone();
        }
        match self.cells.get(&pos) {
            None => Value::Num(0),
            Some(Content::Number(n)) => Value::Num(*n),
            Some(Content::Label(text)) => Value::Text(text.clone()),
            Some(Content::Broken(..)) => Value::Fail(CellError::Parse),
            Some(Content::Formula(expr)) => {
                if !self.active.borrow_mut().insert(pos) {
                    return Value::Fail(CellError::Cycle);
                }
                let result = self.eval(expr).map_or_else(Value::Fail, Value::Num);
                self.active.borrow_mut().remove(&pos);
                self.memo.borrow_mut().insert(pos, result.clone());
                result
            }
        }
    }

    fn number(&self, pos: Pos) -> Result<i64, CellError> {
        match self.value(pos) {
            Value::Num(n) => Ok(n),
            Value::Text(_) => Err(CellError::Value),
            Value::Fail(err) => Err(err),
        }
    }

    fn numbers(&self, first: Pos, last: Pos) -> Result<Vec<i64>, CellError> {
        let mut found = Vec::new();
        for pos in cells_in(first, last) {
            if !self.cells.contains_key(&pos) {
                continue;
            }
            match self.value(pos) {
                Value::Num(n) => found.push(n),
                Value::Text(_) => {}
                Value::Fail(err) => return Err(err),
            }
        }
        Ok(found)
    }

    fn eval(&self, expr: &Expr) -> Result<i64, CellError> {
        match expr {
            Expr::Lit(n) => Ok(*n),
            Expr::Ref(pos) => self.number(*pos),
            Expr::Neg(operand) => Ok(-self.eval(operand)?),
            Expr::Bin(op, left, right) => {
                let a = self.eval(left)?;
                let b = self.eval(right)?;
                match op {
                    Op::Add => Ok(a + b),
                    Op::Sub => Ok(a - b),
                    Op::Mul => Ok(a * b),
                    Op::Div => div(a, b),
                }
            }
            Expr::Agg(func, first, last) => {
                let nums = self.numbers(*first, *last)?;
                match func {
                    Func::Sum => Ok(nums.iter().sum()),
                    Func::Count => Ok(nums.len() as i64),
                    Func::Avg => div(nums.iter().sum(), nums.len() as i64),
                    Func::Min => Ok(nums.into_iter().min().unwrap_or(0)),
                    Func::Max => Ok(nums.into_iter().max().unwrap_or(0)),
                }
            }
            Expr::If {
                cmp,
                left,
                right,
                then,
                other,
            } => {
                let a = self.eval(left)?;
                let b = self.eval(right)?;
                let holds = match cmp {
                    Cmp::Less => a < b,
                    Cmp::Greater => a > b,
                    Cmp::Equal => a == b,
                };
                self.eval(if holds { then } else { other })
            }
        }
    }

    fn display(&self, pos: Pos) -> String {
        if !self.cells.contains_key(&pos) {
            return String::new();
        }
        self.value(pos).to_string()
    }

    fn snapshot(&self) -> BTreeMap<Pos, String> {
        self.cells
            .keys()
            .map(|&pos| (pos, self.display(pos)))
            .collect()
    }

    fn formulas(&self) -> Vec<Pos> {
        self.cells
            .iter()
            .filter(|(_, content)| matches!(content, Content::Formula(_) | Content::Broken(..)))
            .map(|(&pos, _)| pos)
            .collect()
    }

    fn deps(&self, pos: Pos) -> Vec<Pos> {
        let Some(Content::Formula(expr)) = self.cells.get(&pos) else {
            return Vec::new();
        };
        let mut refs = BTreeSet::new();
        collect_refs(expr, &mut refs);
        refs.into_iter().collect()
    }

    fn eval_order(&self) -> (Vec<Pos>, Vec<Pos>) {
        let formulas = self.formulas();
        let is_formula: HashSet<Pos> = formulas.iter().copied().collect();
        let waits: HashMap<Pos, HashSet<Pos>> = formulas
            .iter()
            .map(|&pos| {
                let need = self.deps(pos);
                (
                    pos,
                    need.into_iter()
                        .filter(|d| is_formula.contains(d))
                        .collect(),
                )
            })
            .collect();
        let mut order = Vec::new();
        let mut done = HashSet::new();
        while let Some(&ready) = formulas
            .iter()
            .find(|pos| !done.contains(*pos) && waits[*pos].is_subset(&done))
        {
            order.push(ready);
            done.insert(ready);
        }
        let blocked = formulas
            .iter()
            .filter(|pos| !done.contains(*pos))
            .copied()
            .collect();
        (order, blocked)
    }

    fn table(&self) -> Vec<String> {
        let cols: BTreeSet<usize> = self.cells.keys().map(|pos| pos.col).collect();
        let rows: BTreeSet<usize> = self.cells.keys().map(|pos| pos.row).collect();
        let mut header = vec![String::new()];
        header.extend(cols.iter().map(|&col| col_name(col).to_string()));
        let mut grid = vec![header];
        for &row in &rows {
            let mut line = vec![row.to_string()];
            line.extend(cols.iter().map(|&col| self.display(Pos { col, row })));
            grid.push(line);
        }
        let widths: Vec<usize> = (0..grid[0].len())
            .map(|i| {
                grid.iter()
                    .map(|line| line[i].chars().count())
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let mut lines = Vec::new();
        for (k, line) in grid.iter().enumerate() {
            let padded: Vec<String> = line
                .iter()
                .zip(&widths)
                .map(|(text, &w)| format!("{text:>w$}"))
                .collect();
            lines.push(padded.join(" | "));
            if k == 0 {
                let dashes: Vec<String> = widths.iter().map(|&w| "-".repeat(w)).collect();
                lines.push(dashes.join("-+-"));
            }
        }
        lines
    }

    fn print_table(&self) {
        for line in self.table() {
            println!("{line}");
        }
    }

    fn print_deps(&self) {
        println!("dependencies:");
        for pos in self.formulas() {
            if let Content::Broken(_, err) = &self.cells[&pos] {
                println!("  {pos} <- {} {err}", CellError::Parse);
                continue;
            }
            let deps = self.deps(pos);
            let listing = if deps.is_empty() {
                "(none)".to_string()
            } else {
                names(&deps)
            };
            println!("  {pos} <- {listing}");
        }
    }

    fn print_parse_errors(&self) {
        for (pos, content) in &self.cells {
            if let Content::Broken(src, err) = content {
                println!("parse error in {pos}: {err} in '{src}'");
            }
        }
    }

    fn print_order(&self) {
        let (order, blocked) = self.eval_order();
        println!("order: {}", names(&order));
        if !blocked.is_empty() {
            println!("blocked by a cycle: {}", names(&blocked));
        }
    }
}

fn or_empty(text: &str) -> &str {
    if text.is_empty() {
        "(empty)"
    } else {
        text
    }
}

fn print_changes(before: &BTreeMap<Pos, String>, after: &BTreeMap<Pos, String>) {
    let positions: BTreeSet<&Pos> = before.keys().chain(after.keys()).collect();
    let mut changed = Vec::new();
    for pos in positions {
        let old = before.get(pos).map_or("", String::as_str);
        let new = after.get(pos).map_or("", String::as_str);
        if old != new {
            changed.push(format!("{pos} {} -> {}", or_empty(old), or_empty(new)));
        }
    }
    if changed.is_empty() {
        println!("changed: nothing");
        return;
    }
    println!("changed:");
    for line in changed {
        println!("  {line}");
    }
}

const BUDGET: &str = r#"A1 = "Item"
B1 = "Qty"
C1 = "Price"
D1 = "Total"
E1 = "Share"
A2 = "Pens"
B2 = 10
C2 = 2
D2 = =B2 * C2
E2 = =D2 * 100 / D7
A3 = "Paper"
B3 = 5
C3 = 7
D3 = =B3 * C3
E3 = =D3 * 100 / D7
A4 = "Ink"
B4 = 2
C4 = 25
D4 = =B4*C4
E4 = =D4 * 100 / D7
A5 = "Stapler"
B5 = 1
C5 = 15
D5 = =B5 * C5
E5 = =D5 * 100 / D7
A7 = "Sum"
B7 = =SUM(B2:B5)
D7 = =SUM(D2:D5)
E7 = =SUM(E2:E5)
A8 = "Average"
C8 = =AVG(C2:C5)
D8 = =AVG(D2:D5)
A9 = "Range"
C9 = =MAX(C2:C5) - MIN(C2:C5)
D9 = =MAX(D5:D2) - MIN(D2:D5)
A10 = "Numbers"
B10 = =COUNT(A1:C5)
A11 = "Budget"
D11 = 200
A12 = "Left"
D12 = =D11 - D7
A13 = "Status"
D13 = =IF(D7 > D11, -1, IF(D7 = D11, 0, 1))
E13 = =IF(D12 < 100, -(2 * (D11 / 3) - D12) / 3, 0) + -B5
G15 = "end"
I3 = 5
B6 12
C6 = twelve"#;

const EDITS: &[(&str, &[&str])] = &[
    ("change a value", &["B2 = 20"]),
    ("introduce a cycle", &["D11 = =D12 + 50", "G15 = =G15 + 1"]),
    ("break the cycle", &["D11 = 250", "G15 = "]),
    (
        "divide by zero",
        &["C5 = 0", "B11 = =D11 / (B5 - 1)", "B12 = =AVG(F1:F3)"],
    ),
    (
        "parse errors",
        &[
            "D4 = =B4 * (C4 + 1",
            "B12 = =SUMM(B2:B5)",
            "B13 = =B2 $ 3",
            "C11 = =IF(B2, 1, 2)",
            "C12 = =Z9 + 1",
            "C13 = =3 4",
        ],
    ),
    (
        "fix the formula, then reference a text cell",
        &[
            "D4 = =B4 * (C4 + 1)",
            "B11 = =A2 * 2",
            "B12 = =SUM(A2:A5) + COUNT(A2:B5)",
            "B13 = =IF(A3 = 1, 1, 0)",
        ],
    ),
];

fn main() {
    let mut sheet = Sheet::default();
    println!("== budget");
    sheet.apply(&BUDGET.lines().collect::<Vec<_>>());
    sheet.print_table();
    sheet.print_deps();
    sheet.print_order();
    for (index, (title, lines)) in EDITS.iter().enumerate() {
        println!();
        println!("== edit {}: {title}", index + 1);
        for line in *lines {
            println!("  {line}");
        }
        let before = sheet.snapshot();
        sheet.apply(lines);
        sheet.print_table();
        print_changes(&before, &sheet.snapshot());
        sheet.print_parse_errors();
        sheet.print_order();
    }
    println!();
    println!("== final");
    sheet.print_deps();
}
