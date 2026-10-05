\
\
\
\
\
\
from __future__ import annotations
import sys
from dataclasses import dataclass
sys.setrecursionlimit(20000)
MAX_DEPTH = 200
INT_MIN, INT_MAX = -(2**63), 2**63 - 1
KEYWORDS = {"let", "if", "else", "while", "fn", "return", "true", "false", "and", "or", "not"}
TWO_CHAR_OPS = {"==", "!=", "<=", ">="}
ONE_CHAR_OPS = set("+-*/%<>=(){},")
ESCAPES = {"n": "\n", '"': '"', "\\": "\\"}
@dataclass(frozen=True)
class Pos:
    line: int
    col: int
class MiniError(Exception):
    def __init__(self, pos: Pos, message: str):
        super().__init__(message)
        self.pos = pos
        self.message = message
@dataclass
class Token:
    kind: str
    text: str
    pos: Pos
def describe(tok: Token) -> str:
    match tok.kind:
        case "end":
            return "end of input"
        case "int":
            return f"number {tok.text}"
        case "str":
            return "string"
        case _:
            return f"'{tok.text}'"
class Lexer:
    def __init__(self, source: str):
        self.src = source
        self.i = 0
        self.line = 1
        self.col = 1
    def peek(self, offset: int = 0) -> str:
        j = self.i + offset
        return self.src[j] if j < len(self.src) else ""
    def advance(self) -> str:
        c = self.src[self.i]
        self.i += 1
        if c == "\n":
            self.line += 1
            self.col = 1
        else:
            self.col += 1
        return c
    def tokenize(self) -> list[Token]:
        tokens = []
        while True:
            self.skip_space_and_comments()
            pos = Pos(self.line, self.col)
            c = self.peek()
            if c == "":
                tokens.append(Token("end", "", pos))
                return tokens
            if c.isdigit():
                tokens.append(self.number(pos))
            elif c.isalpha() or c == "_":
                tokens.append(self.word(pos))
            elif c == '"':
                tokens.append(self.string(pos))
            elif c + self.peek(1) in TWO_CHAR_OPS:
                tokens.append(Token("op", self.advance() + self.advance(), pos))
            elif c in ONE_CHAR_OPS:
                tokens.append(Token("op", self.advance(), pos))
            else:
                raise MiniError(pos, f"unexpected character '{c}'")
    def skip_space_and_comments(self) -> None:
        while True:
            c = self.peek()
            if c in (" ", "\n", "\t", "\r"):
                self.advance()
            elif c == "/" and self.peek(1) == "/":
                while self.peek() not in ("", "\n"):
                    self.advance()
            else:
                return
    def number(self, pos: Pos) -> Token:
        start = self.i
        while self.peek().isdigit():
            self.advance()
        return Token("int", self.src[start : self.i], pos)
    def word(self, pos: Pos) -> Token:
        start = self.i
        while self.peek().isalnum() or self.peek() == "_":
            self.advance()
        text = self.src[start : self.i]
        return Token("keyword" if text in KEYWORDS else "ident", text, pos)
    def string(self, pos: Pos) -> Token:
        self.advance()
        parts = []
        while True:
            c = self.peek()
            if c in ("", "\n"):
                raise MiniError(pos, "unterminated string")
            self.advance()
            if c == '"':
                return Token("str", "".join(parts), pos)
            if c == "\\":
                escape_pos = Pos(self.line, self.col - 1)
                e = self.peek()
                if e in ("", "\n"):
                    continue
                if e not in ESCAPES:
                    raise MiniError(escape_pos, f"unknown escape '\\{e}'")
                self.advance()
                parts.append(ESCAPES[e])
            else:
                parts.append(c)
@dataclass
class IntLit:
    pos: Pos
    value: int
@dataclass
class StrLit:
    pos: Pos
    value: str
@dataclass
class BoolLit:
    pos: Pos
    value: bool
@dataclass
class Var:
    pos: Pos
    name: str
@dataclass
class Unary:
    pos: Pos
    op: str
    operand: Expr
@dataclass
class Binary:
    pos: Pos
    op: str
    left: Expr
    right: Expr
@dataclass
class Call:
    pos: Pos
    name: str
    args: list[Expr]
Expr = IntLit | StrLit | BoolLit | Var | Unary | Binary | Call
@dataclass
class Let:
    pos: Pos
    name: str
    value: Expr
@dataclass
class Assign:
    pos: Pos
    name: str
    value: Expr
@dataclass
class If:
    pos: Pos
    cond: Expr
    then: list[Stmt]
    otherwise: list[Stmt] | None
@dataclass
class While:
    pos: Pos
    cond: Expr
    body: list[Stmt]
@dataclass
class FnDecl:
    pos: Pos
    name: str
    params: list[str]
    body: list[Stmt]
@dataclass
class Return:
    pos: Pos
    value: Expr | None
@dataclass
class ExprStmt:
    pos: Pos
    expr: Expr
Stmt = Let | Assign | If | While | FnDecl | Return | ExprStmt
BINARY_LEVELS = [
    ["or"],
    ["and"],
    ["==", "!="],
    ["<", "<=", ">", ">="],
    ["+", "-"],
    ["*", "/", "%"],
]
class Parser:
    def __init__(self, tokens: list[Token]):
        self.tokens = tokens
        self.i = 0
    def peek(self, offset: int = 0) -> Token:
        return self.tokens[min(self.i + offset, len(self.tokens) - 1)]
    def advance(self) -> Token:
        tok = self.peek()
        if tok.kind != "end":
            self.i += 1
        return tok
    def at(self, text: str) -> bool:
        tok = self.peek()
        return tok.kind in ("op", "keyword") and tok.text == text
    def expect(self, text: str) -> Token:
        if not self.at(text):
            raise MiniError(self.peek().pos, f"expected '{text}' but found {describe(self.peek())}")
        return self.advance()
    def expect_ident(self) -> str:
        tok = self.peek()
        if tok.kind != "ident":
            raise MiniError(tok.pos, f"expected identifier but found {describe(tok)}")
        return self.advance().text
    def program(self) -> list[Stmt]:
        stmts = []
        while self.peek().kind != "end":
            stmts.append(self.statement())
        return stmts
    def block(self) -> list[Stmt]:
        self.expect("{")
        stmts = []
        while not self.at("}"):
            if self.peek().kind == "end":
                raise MiniError(self.peek().pos, "expected '}' but found end of input")
            stmts.append(self.statement())
        self.advance()
        return stmts
    def statement(self) -> Stmt:
        tok = self.peek()
        pos = tok.pos
        if self.at("let"):
            self.advance()
            name = self.expect_ident()
            self.expect("=")
            return Let(pos, name, self.expression())
        if self.at("if"):
            return self.if_statement()
        if self.at("while"):
            self.advance()
            cond = self.expression()
            return While(pos, cond, self.block())
        if self.at("fn"):
            self.advance()
            name = self.expect_ident()
            self.expect("(")
            params = []
            if not self.at(")"):
                params.append(self.expect_ident())
                while self.at(","):
                    self.advance()
                    params.append(self.expect_ident())
            self.expect(")")
            return FnDecl(pos, name, params, self.block())
        if self.at("return"):
            self.advance()
            if self.at("}"):
                return Return(pos, None)
            return Return(pos, self.expression())
        if tok.kind == "ident" and self.peek(1).kind == "op" and self.peek(1).text == "=":
            self.advance()
            self.advance()
            return Assign(pos, tok.text, self.expression())
        return ExprStmt(pos, self.expression())
    def if_statement(self) -> If:
        pos = self.expect("if").pos
        cond = self.expression()
        then = self.block()
        otherwise = None
        if self.at("else"):
            self.advance()
            otherwise = [self.if_statement()] if self.at("if") else self.block()
        return If(pos, cond, then, otherwise)
    def expression(self, level: int = 0) -> Expr:
        if level == len(BINARY_LEVELS):
            return self.unary()
        left = self.expression(level + 1)
        while self.peek().kind in ("op", "keyword") and self.peek().text in BINARY_LEVELS[level]:
            op = self.advance()
            right = self.expression(level + 1)
            left = Binary(op.pos, op.text, left, right)
        return left
    def unary(self) -> Expr:
        if self.at("-") or self.at("not"):
            op = self.advance()
            return Unary(op.pos, op.text, self.unary())
        return self.primary()
    def primary(self) -> Expr:
        tok = self.advance()
        match tok.kind:
            case "int":
                value = int(tok.text)
                if value > INT_MAX:
                    raise MiniError(tok.pos, "number too large")
                return IntLit(tok.pos, value)
            case "str":
                return StrLit(tok.pos, tok.text)
            case "ident":
                if not self.at("("):
                    return Var(tok.pos, tok.text)
                self.advance()
                args = []
                if not self.at(")"):
                    args.append(self.expression())
                    while self.at(","):
                        self.advance()
                        args.append(self.expression())
                self.expect(")")
                return Call(tok.pos, tok.text, args)
            case "keyword" if tok.text in ("true", "false"):
                return BoolLit(tok.pos, tok.text == "true")
            case "op" if tok.text == "(":
                inner = self.expression()
                self.expect(")")
                return inner
        raise MiniError(tok.pos, f"expected expression but found {describe(tok)}")
Value = int | str | bool | None
def type_name(v: Value) -> str:
    if isinstance(v, bool):
        return "bool"
    if isinstance(v, int):
        return "int"
    if isinstance(v, str):
        return "str"
    return "nil"
def show(v: Value) -> str:
    if isinstance(v, bool):
        return "true" if v else "false"
    if v is None:
        return "nil"
    return str(v)
def same(a: Value, b: Value) -> bool:
    return type_name(a) == type_name(b) and a == b
def checked(pos: Pos, n: int) -> int:
    if not INT_MIN <= n <= INT_MAX:
        raise MiniError(pos, "integer overflow")
    return n
def div_trunc(a: int, b: int) -> int:
    q = abs(a) // abs(b)
    return q if (a < 0) == (b < 0) else -q
class ReturnSignal(Exception):
    def __init__(self, value: Value):
        self.value = value
class Env:
    def __init__(self):
        self.scopes: list[dict[str, Value]] = [{}]
        self.base = 0
    def find(self, name: str) -> dict[str, Value] | None:
        for scope in reversed(self.scopes[self.base :]):
            if name in scope:
                return scope
        if name in self.scopes[0]:
            return self.scopes[0]
        return None
class Interpreter:
    def __init__(self):
        self.env = Env()
        self.functions: dict[str, FnDecl] = {}
        self.depth = 0
    def run(self, program: list[Stmt]) -> None:
        try:
            for stmt in program:
                self.execute(stmt)
        except ReturnSignal:
            pass
    def execute_block(self, body: list[Stmt]) -> None:
        self.env.scopes.append({})
        try:
            for stmt in body:
                self.execute(stmt)
        finally:
            self.env.scopes.pop()
    def execute(self, stmt: Stmt) -> None:
        match stmt:
            case Let(_, name, value):
                self.env.scopes[-1][name] = self.evaluate(value)
            case Assign(pos, name, value):
                scope = self.env.find(name)
                if scope is None:
                    raise MiniError(pos, f"undefined variable '{name}'")
                scope[name] = self.evaluate(value)
            case If(_, cond, then, otherwise):
                if self.condition(cond):
                    self.execute_block(then)
                elif otherwise is not None:
                    self.execute_block(otherwise)
            case While(_, cond, body):
                while self.condition(cond):
                    self.execute_block(body)
            case FnDecl(_, name, _, _):
                self.functions[name] = stmt
            case Return(_, value):
                raise ReturnSignal(None if value is None else self.evaluate(value))
            case ExprStmt(_, expr):
                self.evaluate(expr)
    def condition(self, expr: Expr) -> bool:
        v = self.evaluate(expr)
        if not isinstance(v, bool):
            raise MiniError(expr.pos, f"type mismatch: condition must be bool, got {type_name(v)}")
        return v
    def evaluate(self, expr: Expr) -> Value:
        match expr:
            case IntLit(_, value) | StrLit(_, value) | BoolLit(_, value):
                return value
            case Var(pos, name):
                scope = self.env.find(name)
                if scope is None:
                    raise MiniError(pos, f"undefined variable '{name}'")
                return scope[name]
            case Unary(pos, op, operand):
                v = self.evaluate(operand)
                if op == "-" and type_name(v) == "int":
                    return checked(pos, -v)
                if op == "not" and type_name(v) == "bool":
                    return not v
                raise MiniError(pos, f"type mismatch: cannot apply '{op}' to {type_name(v)}")
            case Binary(pos, op, left, right) if op in ("and", "or"):
                return self.logical(pos, op, left, right)
            case Binary(pos, op, left, right):
                return self.binary(pos, op, self.evaluate(left), self.evaluate(right))
            case Call(pos, name, args):
                return self.call(pos, name, [self.evaluate(a) for a in args])
    def logical(self, pos: Pos, op: str, left: Expr, right: Expr) -> bool:
        a = self.evaluate(left)
        if type_name(a) != "bool":
            raise MiniError(pos, f"type mismatch: '{op}' needs bool operands, got {type_name(a)}")
        if (op == "and" and not a) or (op == "or" and a):
            return a
        b = self.evaluate(right)
        if type_name(b) != "bool":
            raise MiniError(pos, f"type mismatch: '{op}' needs bool operands, got {type_name(b)}")
        return b
    def binary(self, pos: Pos, op: str, a: Value, b: Value) -> Value:
        if op == "==":
            return same(a, b)
        if op == "!=":
            return not same(a, b)
        ta, tb = type_name(a), type_name(b)
        if op == "+" and ta == "str" and tb == "str":
            return a + b
        if ta != "int" or tb != "int":
            raise MiniError(pos, f"type mismatch: cannot apply '{op}' to {ta} and {tb}")
        match op:
            case "+":
                return checked(pos, a + b)
            case "-":
                return checked(pos, a - b)
            case "*":
                return checked(pos, a * b)
            case "/" | "%":
                if b == 0:
                    raise MiniError(pos, "division by zero")
                q = div_trunc(a, b)
                return checked(pos, q) if op == "/" else a - b * q
            case "<":
                return a < b
            case "<=":
                return a <= b
            case ">":
                return a > b
            case ">=":
                return a >= b
        raise AssertionError(f"unknown operator {op}")
    def call(self, pos: Pos, name: str, args: list[Value]) -> Value:
        if name in ("print", "str"):
            if len(args) != 1:
                raise MiniError(pos, f"wrong number of arguments: '{name}' takes 1 but got {len(args)}")
            if name == "str":
                return show(args[0])
            print(show(args[0]))
            return None
        fn = self.functions.get(name)
        if fn is None:
            raise MiniError(pos, f"undefined function '{name}'")
        if len(args) != len(fn.params):
            raise MiniError(
                pos,
                f"wrong number of arguments: '{name}' takes {len(fn.params)} but got {len(args)}",
            )
        if self.depth == MAX_DEPTH:
            raise MiniError(pos, "stack overflow")
        saved_base = self.env.base
        self.env.base = len(self.env.scopes)
        self.env.scopes.append(dict(zip(fn.params, args)))
        self.depth += 1
        try:
            for stmt in fn.body:
                self.execute(stmt)
            return None
        except ReturnSignal as ret:
            return ret.value
        finally:
            self.depth -= 1
            del self.env.scopes[self.env.base :]
            self.env.base = saved_base
SCRIPTS = [
    ("fibonacci", """\
// The first sixteen Fibonacci numbers, on one line
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
"""),
    ("factorial", """\
fn fact(n) {
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
"""),
    ("fizzbuzz", """\
let n = 1
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
"""),
    ("strings", """\
fn repeat(s, times) {
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
print("she said \\"hi\\"\\nand left")
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
"""),
    ("even and odd", """\
fn is_even(n) {
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
"""),
    ("scopes", """\
let x = "global"
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
"""),
    ("bad character", """\
let a = 1
let b = a # 2
"""),
    ("unterminated string", """\
print("ok")
print("hello)
"""),
    ("missing paren", """\
fn add(a, b {
    return a + b
}
"""),
    ("undefined variable", """\
fn peek() {
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
"""),
    ("type mismatch", """\
let total = 5
print("total: " + str(total))
let flag = true
print(total + flag)
"""),
    ("division by zero", """\
fn average(sum, count) {
    return sum / count
}
print(average(10, 2))
print(average(7, 0))
"""),
    ("undefined function", """\
fn square(n) { return n * n }
print(square(4))
print(sqaure(5))
"""),
    ("wrong arguments", """\
fn add(a, b) { return a + b }
print(add(1, 2))
print(add(1, 2, 3))
"""),
    ("stack overflow", """\
fn down(n) {
    return down(n + 1)
}
print("going down")
down(0)
"""),
    ("bad condition", """\
let n = 3
while n {
    n = n - 1
}
"""),
]
def run_script(source: str) -> None:
    try:
        tokens = Lexer(source).tokenize()
        program = Parser(tokens).program()
        Interpreter().run(program)
    except MiniError as err:
        print(f"error at {err.pos.line}:{err.pos.col}: {err.message}")
def main() -> None:
    for name, source in SCRIPTS:
        print(f"== {name} ==")
        run_script(source)
if __name__ == "__main__":
    main()
