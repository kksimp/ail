use std::collections::HashMap;
use std::fmt;

const MAX_DEPTH: usize = 200;
const KEYWORDS: &[&str] = &[
    "let", "if", "else", "while", "fn", "return", "true", "false", "and", "or", "not",
];
const ONE_CHAR_OPS: &str = "+-*/%<>=(){},";

#[derive(Clone, Copy)]
struct Pos {
    line: usize,
    col: usize,
}

struct MiniError {
    pos: Pos,
    message: String,
}

impl MiniError {
    fn new(pos: Pos, message: impl Into<String>) -> Self {
        MiniError {
            pos,
            message: message.into(),
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TokKind {
    Int,
    Str,
    Ident,
    Keyword,
    Op,
    End,
}

#[derive(Clone)]
struct Token {
    kind: TokKind,
    text: String,
    pos: Pos,
}

impl Token {
    fn describe(&self) -> String {
        match self.kind {
            TokKind::End => "end of input".to_string(),
            TokKind::Int => format!("number {}", self.text),
            TokKind::Str => "string".to_string(),
            _ => format!("'{}'", self.text),
        }
    }
}

struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: usize,
    col: usize,
}

impl Lexer {
    fn new(source: &str) -> Self {
        Lexer {
            chars: source.chars().collect(),
            i: 0,
            line: 1,
            col: 1,
        }
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.chars.get(self.i + offset).copied()
    }

    fn pos(&self) -> Pos {
        Pos {
            line: self.line,
            col: self.col,
        }
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.i];
        self.i += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        c
    }

    fn take_while(&mut self, keep: impl Fn(char) -> bool) -> String {
        let start = self.i;
        while self.peek(0).is_some_and(&keep) {
            self.advance();
        }
        self.chars[start..self.i].iter().collect()
    }

    fn tokenize(&mut self) -> Result<Vec<Token>, MiniError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_space_and_comments();
            let pos = self.pos();
            let Some(c) = self.peek(0) else {
                tokens.push(Token {
                    kind: TokKind::End,
                    text: String::new(),
                    pos,
                });
                return Ok(tokens);
            };
            let token = match c {
                '0'..='9' => self.number(pos),
                '"' => self.string(pos)?,
                _ if c.is_alphabetic() || c == '_' => self.word(pos),
                '=' | '!' | '<' | '>' if self.peek(1) == Some('=') => self.operator(2, pos),
                _ if ONE_CHAR_OPS.contains(c) => self.operator(1, pos),
                _ => return Err(MiniError::new(pos, format!("unexpected character '{c}'"))),
            };
            tokens.push(token);
        }
    }

    fn skip_space_and_comments(&mut self) {
        loop {
            match (self.peek(0), self.peek(1)) {
                (Some(' ' | '\n' | '\t' | '\r'), _) => {
                    self.advance();
                }
                (Some('/'), Some('/')) => {
                    self.take_while(|c| c != '\n');
                }
                _ => return,
            }
        }
    }

    fn number(&mut self, pos: Pos) -> Token {
        let text = self.take_while(|c| c.is_ascii_digit());
        Token {
            kind: TokKind::Int,
            text,
            pos,
        }
    }

    fn word(&mut self, pos: Pos) -> Token {
        let text = self.take_while(|c| c.is_alphanumeric() || c == '_');
        let kind = if KEYWORDS.contains(&text.as_str()) {
            TokKind::Keyword
        } else {
            TokKind::Ident
        };
        Token { kind, text, pos }
    }

    fn operator(&mut self, len: usize, pos: Pos) -> Token {
        let text = (0..len).map(|_| self.advance()).collect();
        Token {
            kind: TokKind::Op,
            text,
            pos,
        }
    }

    fn string(&mut self, pos: Pos) -> Result<Token, MiniError> {
        self.advance();
        let mut text = String::new();
        loop {
            let Some(c) = self.peek(0).filter(|&c| c != '\n') else {
                return Err(MiniError::new(pos, "unterminated string"));
            };
            self.advance();
            match c {
                '"' => {
                    return Ok(Token {
                        kind: TokKind::Str,
                        text,
                        pos,
                    })
                }
                '\\' => {
                    let escape_pos = Pos {
                        line: self.line,
                        col: self.col - 1,
                    };
                    let decoded = match self.peek(0) {
                        None | Some('\n') => continue,
                        Some('n') => '\n',
                        Some(e @ ('"' | '\\')) => e,
                        Some(e) => {
                            return Err(MiniError::new(
                                escape_pos,
                                format!("unknown escape '\\{e}'"),
                            ))
                        }
                    };
                    self.advance();
                    text.push(decoded);
                }
                _ => text.push(c),
            }
        }
    }
}

enum Expr {
    IntLit(Pos, i64),
    StrLit(Pos, String),
    BoolLit(Pos, bool),
    Var(Pos, String),
    Unary(Pos, String, Box<Expr>),
    Binary(Pos, String, Box<Expr>, Box<Expr>),
    Call(Pos, String, Vec<Expr>),
}

impl Expr {
    fn pos(&self) -> Pos {
        match self {
            Expr::IntLit(pos, _)
            | Expr::StrLit(pos, _)
            | Expr::BoolLit(pos, _)
            | Expr::Var(pos, _)
            | Expr::Unary(pos, _, _)
            | Expr::Binary(pos, _, _, _)
            | Expr::Call(pos, _, _) => *pos,
        }
    }
}

struct FnDecl {
    name: String,
    params: Vec<String>,
    body: Vec<Stmt>,
}

enum Stmt {
    Let(String, Expr),
    Assign(Pos, String, Expr),
    If(Expr, Vec<Stmt>, Option<Vec<Stmt>>),
    While(Expr, Vec<Stmt>),
    FnDecl(FnDecl),
    Return(Option<Expr>),
    ExprStmt(Expr),
}

const BINARY_LEVELS: [&[&str]; 6] = [
    &["or"],
    &["and"],
    &["==", "!="],
    &["<", "<=", ">", ">="],
    &["+", "-"],
    &["*", "/", "%"],
];

struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, i: 0 }
    }

    fn peek(&self, offset: usize) -> &Token {
        &self.tokens[(self.i + offset).min(self.tokens.len() - 1)]
    }

    fn advance(&mut self) -> Token {
        let tok = self.peek(0).clone();
        if tok.kind != TokKind::End {
            self.i += 1;
        }
        tok
    }

    fn at_any(&self, texts: &[&str]) -> bool {
        let tok = self.peek(0);
        matches!(tok.kind, TokKind::Op | TokKind::Keyword) && texts.contains(&tok.text.as_str())
    }

    fn at(&self, text: &str) -> bool {
        self.at_any(&[text])
    }

    fn expected(&self, what: &str) -> MiniError {
        let tok = self.peek(0);
        MiniError::new(
            tok.pos,
            format!("expected {what} but found {}", tok.describe()),
        )
    }

    fn expect(&mut self, text: &str) -> Result<Token, MiniError> {
        if !self.at(text) {
            return Err(self.expected(&format!("'{text}'")));
        }
        Ok(self.advance())
    }

    fn expect_ident(&mut self) -> Result<String, MiniError> {
        if self.peek(0).kind != TokKind::Ident {
            return Err(self.expected("identifier"));
        }
        Ok(self.advance().text)
    }

    fn program(&mut self) -> Result<Vec<Stmt>, MiniError> {
        let mut stmts = Vec::new();
        while self.peek(0).kind != TokKind::End {
            stmts.push(self.statement()?);
        }
        Ok(stmts)
    }

    fn block(&mut self) -> Result<Vec<Stmt>, MiniError> {
        self.expect("{")?;
        let mut stmts = Vec::new();
        while !self.at("}") {
            if self.peek(0).kind == TokKind::End {
                return Err(self.expected("'}'"));
            }
            stmts.push(self.statement()?);
        }
        self.advance();
        Ok(stmts)
    }

    fn statement(&mut self) -> Result<Stmt, MiniError> {
        let tok = self.peek(0).clone();
        if self.at("let") {
            self.advance();
            let name = self.expect_ident()?;
            self.expect("=")?;
            return Ok(Stmt::Let(name, self.expression(0)?));
        }
        if self.at("if") {
            return self.if_statement();
        }
        if self.at("while") {
            self.advance();
            let cond = self.expression(0)?;
            return Ok(Stmt::While(cond, self.block()?));
        }
        if self.at("fn") {
            self.advance();
            let name = self.expect_ident()?;
            self.expect("(")?;
            let mut params = Vec::new();
            if !self.at(")") {
                params.push(self.expect_ident()?);
                while self.at(",") {
                    self.advance();
                    params.push(self.expect_ident()?);
                }
            }
            self.expect(")")?;
            let body = self.block()?;
            return Ok(Stmt::FnDecl(FnDecl { name, params, body }));
        }
        if self.at("return") {
            self.advance();
            if self.at("}") {
                return Ok(Stmt::Return(None));
            }
            return Ok(Stmt::Return(Some(self.expression(0)?)));
        }
        let next = self.peek(1);
        if tok.kind == TokKind::Ident && next.kind == TokKind::Op && next.text == "=" {
            self.advance();
            self.advance();
            return Ok(Stmt::Assign(tok.pos, tok.text, self.expression(0)?));
        }
        Ok(Stmt::ExprStmt(self.expression(0)?))
    }

    fn if_statement(&mut self) -> Result<Stmt, MiniError> {
        self.expect("if")?;
        let cond = self.expression(0)?;
        let then = self.block()?;
        let mut otherwise = None;
        if self.at("else") {
            self.advance();
            otherwise = Some(if self.at("if") {
                vec![self.if_statement()?]
            } else {
                self.block()?
            });
        }
        Ok(Stmt::If(cond, then, otherwise))
    }

    fn expression(&mut self, level: usize) -> Result<Expr, MiniError> {
        if level == BINARY_LEVELS.len() {
            return self.unary();
        }
        let mut left = self.expression(level + 1)?;
        while self.at_any(BINARY_LEVELS[level]) {
            let op = self.advance();
            let right = self.expression(level + 1)?;
            left = Expr::Binary(op.pos, op.text, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, MiniError> {
        if self.at_any(&["-", "not"]) {
            let op = self.advance();
            return Ok(Expr::Unary(op.pos, op.text, Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, MiniError> {
        let tok = self.advance();
        match (tok.kind, tok.text.as_str()) {
            (TokKind::Int, text) => {
                let value = text
                    .parse()
                    .map_err(|_| MiniError::new(tok.pos, "number too large"))?;
                Ok(Expr::IntLit(tok.pos, value))
            }
            (TokKind::Str, _) => Ok(Expr::StrLit(tok.pos, tok.text)),
            (TokKind::Ident, _) => {
                if !self.at("(") {
                    return Ok(Expr::Var(tok.pos, tok.text));
                }
                self.advance();
                let mut args = Vec::new();
                if !self.at(")") {
                    args.push(self.expression(0)?);
                    while self.at(",") {
                        self.advance();
                        args.push(self.expression(0)?);
                    }
                }
                self.expect(")")?;
                Ok(Expr::Call(tok.pos, tok.text, args))
            }
            (TokKind::Keyword, "true" | "false") => Ok(Expr::BoolLit(tok.pos, tok.text == "true")),
            (TokKind::Op, "(") => {
                let inner = self.expression(0)?;
                self.expect(")")?;
                Ok(inner)
            }
            _ => Err(MiniError::new(
                tok.pos,
                format!("expected expression but found {}", tok.describe()),
            )),
        }
    }
}

#[derive(Clone, PartialEq)]
enum Value {
    Int(i64),
    Str(String),
    Bool(bool),
    Nil,
}

impl Value {
    fn type_name(&self) -> &'static str {
        match self {
            Value::Int(_) => "int",
            Value::Str(_) => "str",
            Value::Bool(_) => "bool",
            Value::Nil => "nil",
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{n}"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Nil => write!(f, "nil"),
        }
    }
}

fn checked(pos: Pos, n: Option<i64>) -> Result<Value, MiniError> {
    n.map(Value::Int)
        .ok_or_else(|| MiniError::new(pos, "integer overflow"))
}

fn binary(pos: Pos, op: &str, a: Value, b: Value) -> Result<Value, MiniError> {
    match (op, &a, &b) {
        ("==", _, _) => Ok(Value::Bool(a == b)),
        ("!=", _, _) => Ok(Value::Bool(a != b)),
        ("+", Value::Str(x), Value::Str(y)) => Ok(Value::Str(format!("{x}{y}"))),
        (_, &Value::Int(x), &Value::Int(y)) => match op {
            "+" => checked(pos, x.checked_add(y)),
            "-" => checked(pos, x.checked_sub(y)),
            "*" => checked(pos, x.checked_mul(y)),
            "/" | "%" if y == 0 => Err(MiniError::new(pos, "division by zero")),
            "/" => checked(pos, x.checked_div(y)),
            "%" => Ok(Value::Int(x.wrapping_rem(y))),
            "<" => Ok(Value::Bool(x < y)),
            "<=" => Ok(Value::Bool(x <= y)),
            ">" => Ok(Value::Bool(x > y)),
            ">=" => Ok(Value::Bool(x >= y)),
            _ => unreachable!("unknown operator {op}"),
        },
        _ => Err(MiniError::new(
            pos,
            format!(
                "type mismatch: cannot apply '{op}' to {} and {}",
                a.type_name(),
                b.type_name()
            ),
        )),
    }
}

fn wrong_arguments(pos: Pos, name: &str, expected: usize, got: usize) -> MiniError {
    MiniError::new(
        pos,
        format!("wrong number of arguments: '{name}' takes {expected} but got {got}"),
    )
}

enum Flow {
    Next,
    Return(Value),
}

struct Env {
    scopes: Vec<HashMap<String, Value>>,
    base: usize,
}

impl Env {
    fn new() -> Self {
        Env {
            scopes: vec![HashMap::new()],
            base: 0,
        }
    }

    fn find(&self, name: &str) -> Option<usize> {
        (self.base..self.scopes.len())
            .rev()
            .chain([0])
            .find(|&i| self.scopes[i].contains_key(name))
    }
}

struct Interpreter<'a> {
    env: Env,
    functions: HashMap<&'a str, &'a FnDecl>,
    depth: usize,
}

impl<'a> Interpreter<'a> {
    fn new() -> Self {
        Interpreter {
            env: Env::new(),
            functions: HashMap::new(),
            depth: 0,
        }
    }

    fn run(&mut self, program: &'a [Stmt]) -> Result<(), MiniError> {
        self.execute_all(program)?;
        Ok(())
    }

    fn execute_all(&mut self, body: &'a [Stmt]) -> Result<Flow, MiniError> {
        for stmt in body {
            if let flow @ Flow::Return(_) = self.execute(stmt)? {
                return Ok(flow);
            }
        }
        Ok(Flow::Next)
    }

    fn execute_block(&mut self, body: &'a [Stmt]) -> Result<Flow, MiniError> {
        self.env.scopes.push(HashMap::new());
        let flow = self.execute_all(body);
        self.env.scopes.pop();
        flow
    }

    fn execute(&mut self, stmt: &'a Stmt) -> Result<Flow, MiniError> {
        match stmt {
            Stmt::Let(name, value) => {
                let value = self.evaluate(value)?;
                self.env
                    .scopes
                    .last_mut()
                    .unwrap()
                    .insert(name.clone(), value);
            }
            Stmt::Assign(pos, name, value) => {
                let scope = self.lookup(*pos, name)?;
                let value = self.evaluate(value)?;
                self.env.scopes[scope].insert(name.clone(), value);
            }
            Stmt::If(cond, then, otherwise) => {
                if self.condition(cond)? {
                    return self.execute_block(then);
                } else if let Some(otherwise) = otherwise {
                    return self.execute_block(otherwise);
                }
            }
            Stmt::While(cond, body) => {
                while self.condition(cond)? {
                    if let flow @ Flow::Return(_) = self.execute_block(body)? {
                        return Ok(flow);
                    }
                }
            }
            Stmt::FnDecl(decl) => {
                self.functions.insert(&decl.name, decl);
            }
            Stmt::Return(value) => {
                let value = match value {
                    Some(expr) => self.evaluate(expr)?,
                    None => Value::Nil,
                };
                return Ok(Flow::Return(value));
            }
            Stmt::ExprStmt(expr) => {
                self.evaluate(expr)?;
            }
        }
        Ok(Flow::Next)
    }

    fn lookup(&self, pos: Pos, name: &str) -> Result<usize, MiniError> {
        self.env
            .find(name)
            .ok_or_else(|| MiniError::new(pos, format!("undefined variable '{name}'")))
    }

    fn condition(&mut self, expr: &Expr) -> Result<bool, MiniError> {
        match self.evaluate(expr)? {
            Value::Bool(b) => Ok(b),
            v => Err(MiniError::new(
                expr.pos(),
                format!(
                    "type mismatch: condition must be bool, got {}",
                    v.type_name()
                ),
            )),
        }
    }

    fn evaluate(&mut self, expr: &Expr) -> Result<Value, MiniError> {
        match expr {
            Expr::IntLit(_, value) => Ok(Value::Int(*value)),
            Expr::StrLit(_, value) => Ok(Value::Str(value.clone())),
            Expr::BoolLit(_, value) => Ok(Value::Bool(*value)),
            Expr::Var(pos, name) => {
                let scope = self.lookup(*pos, name)?;
                Ok(self.env.scopes[scope][name.as_str()].clone())
            }
            Expr::Unary(pos, op, operand) => match (op.as_str(), self.evaluate(operand)?) {
                ("-", Value::Int(n)) => checked(*pos, n.checked_neg()),
                ("not", Value::Bool(b)) => Ok(Value::Bool(!b)),
                (_, v) => Err(MiniError::new(
                    *pos,
                    format!("type mismatch: cannot apply '{op}' to {}", v.type_name()),
                )),
            },
            Expr::Binary(pos, op, left, right) if matches!(op.as_str(), "and" | "or") => {
                self.logical(*pos, op, left, right)
            }
            Expr::Binary(pos, op, left, right) => {
                let a = self.evaluate(left)?;
                let b = self.evaluate(right)?;
                binary(*pos, op, a, b)
            }
            Expr::Call(pos, name, args) => {
                let args = args
                    .iter()
                    .map(|arg| self.evaluate(arg))
                    .collect::<Result<Vec<_>, _>>()?;
                self.call(*pos, name, args)
            }
        }
    }

    fn bool_operand(&mut self, pos: Pos, op: &str, expr: &Expr) -> Result<bool, MiniError> {
        match self.evaluate(expr)? {
            Value::Bool(b) => Ok(b),
            v => Err(MiniError::new(
                pos,
                format!(
                    "type mismatch: '{op}' needs bool operands, got {}",
                    v.type_name()
                ),
            )),
        }
    }

    fn logical(
        &mut self,
        pos: Pos,
        op: &str,
        left: &Expr,
        right: &Expr,
    ) -> Result<Value, MiniError> {
        let a = self.bool_operand(pos, op, left)?;
        if (op == "and" && !a) || (op == "or" && a) {
            return Ok(Value::Bool(a));
        }
        Ok(Value::Bool(self.bool_operand(pos, op, right)?))
    }

    fn call(&mut self, pos: Pos, name: &str, args: Vec<Value>) -> Result<Value, MiniError> {
        match (name, args.as_slice()) {
            ("print", [arg]) => {
                println!("{arg}");
                Ok(Value::Nil)
            }
            ("str", [arg]) => Ok(Value::Str(arg.to_string())),
            ("print" | "str", _) => Err(wrong_arguments(pos, name, 1, args.len())),
            _ => {
                let Some(&func) = self.functions.get(name) else {
                    return Err(MiniError::new(pos, format!("undefined function '{name}'")));
                };
                if args.len() != func.params.len() {
                    return Err(wrong_arguments(pos, name, func.params.len(), args.len()));
                }
                if self.depth == MAX_DEPTH {
                    return Err(MiniError::new(pos, "stack overflow"));
                }
                let saved_base = self.env.base;
                self.env.base = self.env.scopes.len();
                self.env
                    .scopes
                    .push(func.params.iter().cloned().zip(args).collect());
                self.depth += 1;
                let flow = self.execute_all(&func.body);
                self.depth -= 1;
                self.env.scopes.truncate(self.env.base);
                self.env.base = saved_base;
                match flow? {
                    Flow::Return(value) => Ok(value),
                    Flow::Next => Ok(Value::Nil),
                }
            }
        }
    }
}

const SCRIPTS: &[(&str, &str)] = &[
    (
        "fibonacci",
        r#"// The first sixteen Fibonacci numbers, on one line
fn fib(n) {
    if n < 2 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}
let line = ""
let i = 0
while i <= 15 {
    if i > 0 {
        line = line + " "
    }
    line = line + str(fib(i))
    i = i + 1
}
print(line)
"#,
    ),
    (
        "factorial",
        r#"fn fact(n) {
    if n <= 1 {
        return 1
    }
    return n * fact(n - 1)
}
fn fact_loop(n) {
    let result = 1
    while n > 1 {
        result = result * n
        n = n - 1
    }
    return result
}
print(fact(10))
print(fact(20))
print(fact(12) == fact_loop(12))
print(fact(21))
"#,
    ),
    (
        "fizzbuzz",
        r#"let n = 1
let out = ""
while n <= 15 {
    if n % 15 == 0 {
        out = out + "FizzBuzz"
    } else if n % 3 == 0 {
        out = out + "Fizz"
    } else {
        if n % 5 == 0 {
            out = out + "Buzz"
        } else {
            out = out + str(n)
        }
    }
    if n < 15 { out = out + "," }
    n = n + 1
}
print(out)
"#,
    ),
    (
        "strings",
        r#"fn repeat(s, times) {
    let out = ""
    while times > 0 {
        out = out + s
        times = times - 1
    }
    return out
}
let row = 1
while row <= 4 {
    print(repeat(" ", 4 - row) + repeat("*", 2 * row - 1))
    row = row + 1
}
print("she said \"hi\"\nand left")
print("ab" + "cd" == "abcd")
print(1 == "1")
print(true != 1)
print(-7 / 2)
print(-7 % 2)
print(not (3 > 2) or 2 >= 2)
print(false and missing)
print(true or missing())
fn nothing() { return }
print(nothing())
"#,
    ),
    (
        "even and odd",
        r#"fn is_even(n) {
    if n == 0 { return true }
    return is_odd(n - 1)
}
fn is_odd(n) {
    if n == 0 { return false }
    return is_even(n - 1)
}
print(is_even(10))
print(is_odd(7))
print(is_even(7))
print(is_odd(150))
"#,
    ),
    (
        "scopes",
        r#"let x = "global"
let count = 0
fn bump() {
    count = count + 1
    let x = "local"
    return x
}
if true {
    let x = "block"
    print(x)
    count = 10
}
print(x)
print(bump())
print(count)
"#,
    ),
    (
        "bad character",
        r#"let a = 1
let b = a # 2
"#,
    ),
    (
        "unterminated string",
        r#"print("ok")
print("hello)
"#,
    ),
    (
        "missing paren",
        r#"fn add(a, b {
    return a + b
}
"#,
    ),
    (
        "undefined variable",
        r#"fn peek() {
    return secret
}
let secret = 1
fn outer() {
    let hidden = 2
    return inner()
}
fn inner() { return hidden }
print(peek())
print(outer())
"#,
    ),
    (
        "type mismatch",
        r#"let total = 5
print("total: " + str(total))
let flag = true
print(total + flag)
"#,
    ),
    (
        "division by zero",
        r#"fn average(sum, count) {
    return sum / count
}
print(average(10, 2))
print(average(7, 0))
"#,
    ),
    (
        "undefined function",
        r#"fn square(n) { return n * n }
print(square(4))
print(sqaure(5))
"#,
    ),
    (
        "wrong arguments",
        r#"fn add(a, b) { return a + b }
print(add(1, 2))
print(add(1, 2, 3))
"#,
    ),
    (
        "stack overflow",
        r#"fn down(n) {
    return down(n + 1)
}
print("going down")
down(0)
"#,
    ),
    (
        "bad condition",
        r#"let n = 3
while n {
    n = n - 1
}
"#,
    ),
];

fn run_script(source: &str) -> Result<(), MiniError> {
    let tokens = Lexer::new(source).tokenize()?;
    let program = Parser::new(tokens).program()?;
    let mut interpreter = Interpreter::new();
    interpreter.run(&program)
}

fn main() {
    for (name, source) in SCRIPTS {
        println!("== {name} ==");
        if let Err(err) = run_script(source) {
            println!("error at {}:{}: {}", err.pos.line, err.pos.col, err.message);
        }
    }
}
