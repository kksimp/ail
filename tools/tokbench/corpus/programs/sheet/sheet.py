\
\
\
\
\
from __future__ import annotations
from dataclasses import dataclass
COLS = "ABCDEFGH"
MAX_ROW = 20
INT_MAX = 2**63 - 1
@dataclass(frozen=True, order=True)
class Pos:
    col: int
    row: int
    def __str__(self) -> str:
        return f"{COLS[self.col]}{self.row}"
def parse_pos(text: str) -> Pos | None:
    if len(text) < 2 or text[0] not in COLS:
        return None
    digits = text[1:]
    if not digits.isdigit():
        return None
    row = int(digits)
    if row < 1 or row > MAX_ROW:
        return None
    return Pos(COLS.index(text[0]), row)
def cells_in(first: Pos, last: Pos) -> list[Pos]:
    c0, c1 = min(first.col, last.col), max(first.col, last.col)
    r0, r1 = min(first.row, last.row), max(first.row, last.row)
    return [Pos(c, r) for r in range(r0, r1 + 1) for c in range(c0, c1 + 1)]
def names(cells: list[Pos]) -> str:
    return ", ".join(str(p) for p in cells)
@dataclass
class Lit:
    n: int
@dataclass
class Ref:
    pos: Pos
@dataclass
class Neg:
    operand: Expr
@dataclass
class Bin:
    op: str
    left: Expr
    right: Expr
@dataclass
class Agg:
    func: str
    first: Pos
    last: Pos
@dataclass
class If:
    cmp: str
    left: Expr
    right: Expr
    then: Expr
    other: Expr
Expr = Lit | Ref | Neg | Bin | Agg | If
AGGREGATES = ("SUM", "MIN", "MAX", "COUNT", "AVG")
SYMBOLS = "+-*/(),:<>="
class ParseError(Exception):
    pass
@dataclass
class Token:
    kind: str
    text: str
    def describe(self) -> str:
        if self.kind == "end":
            return "end of formula"
        if self.kind == "sym":
            return f"'{self.text}'"
        return self.text
def tokenize(src: str) -> list[Token]:
    tokens = []
    i = 0
    while i < len(src):
        c = src[i]
        if c == " ":
            i += 1
        elif c.isdigit():
            start = i
            while i < len(src) and src[i].isdigit():
                i += 1
            tokens.append(Token("num", src[start:i]))
        elif c.isalpha():
            start = i
            while i < len(src) and src[i].isalnum():
                i += 1
            word = src[start:i]
            if word[-1].isdigit():
                if parse_pos(word) is None:
                    raise ParseError(f"bad cell name {word}")
                tokens.append(Token("cell", word))
            else:
                tokens.append(Token("name", word))
        elif c in SYMBOLS:
            tokens.append(Token("sym", c))
            i += 1
        else:
            raise ParseError(f"unexpected character '{c}'")
    tokens.append(Token("end", ""))
    return tokens
class Parser:
    def __init__(self, tokens: list[Token]):
        self.tokens = tokens
        self.at = 0
    def peek(self) -> Token:
        return self.tokens[self.at]
    def next(self) -> Token:
        tok = self.tokens[self.at]
        if tok.kind != "end":
            self.at += 1
        return tok
    def at_sym(self, syms: str) -> bool:
        tok = self.peek()
        return tok.kind == "sym" and tok.text in syms
    def expect(self, sym: str) -> None:
        tok = self.next()
        if tok.kind != "sym" or tok.text != sym:
            raise ParseError(f"expected '{sym}' but found {tok.describe()}")
    def cell(self) -> Pos:
        tok = self.next()
        if tok.kind != "cell":
            raise ParseError(f"expected a cell but found {tok.describe()}")
        pos = parse_pos(tok.text)
        assert pos is not None
        return pos
    def formula(self) -> Expr:
        expr = self.sum()
        if self.peek().kind != "end":
            raise ParseError(f"unexpected {self.peek().describe()}")
        return expr
    def sum(self) -> Expr:
        left = self.product()
        while self.at_sym("+-"):
            op = self.next().text
            left = Bin(op, left, self.product())
        return left
    def product(self) -> Expr:
        left = self.unary()
        while self.at_sym("*/"):
            op = self.next().text
            left = Bin(op, left, self.unary())
        return left
    def unary(self) -> Expr:
        if self.at_sym("-"):
            self.next()
            return Neg(self.unary())
        return self.primary()
    def primary(self) -> Expr:
        tok = self.next()
        if tok.kind == "num":
            n = int(tok.text)
            if n > INT_MAX:
                raise ParseError(f"number too large: {tok.text}")
            return Lit(n)
        if tok.kind == "cell":
            pos = parse_pos(tok.text)
            assert pos is not None
            return Ref(pos)
        if tok.kind == "name":
            return self.call(tok.text)
        if tok.kind == "sym" and tok.text == "(":
            inner = self.sum()
            self.expect(")")
            return inner
        raise ParseError(f"unexpected {tok.describe()}")
    def call(self, name: str) -> Expr:
        if name in AGGREGATES:
            self.expect("(")
            first = self.cell()
            self.expect(":")
            last = self.cell()
            self.expect(")")
            return Agg(name, first, last)
        if name == "IF":
            self.expect("(")
            left = self.sum()
            tok = self.next()
            if tok.kind != "sym" or tok.text not in "<>=":
                raise ParseError(f"expected a comparison but found {tok.describe()}")
            right = self.sum()
            self.expect(",")
            then = self.sum()
            self.expect(",")
            other = self.sum()
            self.expect(")")
            return If(tok.text, left, right, then, other)
        raise ParseError(f"unknown function {name}")
def parse_formula(src: str) -> Expr:
    return Parser(tokenize(src)).formula()
def collect_refs(expr: Expr, out: set[Pos]) -> None:
    match expr:
        case Lit():
            pass
        case Ref(pos):
            out.add(pos)
        case Neg(operand):
            collect_refs(operand, out)
        case Bin(_, left, right):
            collect_refs(left, out)
            collect_refs(right, out)
        case Agg(_, first, last):
            out.update(cells_in(first, last))
        case If(_, left, right, then, other):
            for part in (left, right, then, other):
                collect_refs(part, out)
@dataclass
class Number:
    n: int
@dataclass
class Label:
    text: str
@dataclass
class Formula:
    src: str
    expr: Expr
@dataclass
class Broken:
    src: str
    error: str
Content = Number | Label | Formula | Broken
VALUE = "#VALUE!"
CYCLE = "#CYCLE!"
DIV0 = "#DIV/0!"
PARSE = "#PARSE!"
class CellError(Exception):
    def __init__(self, code: str):
        super().__init__(code)
        self.code = code
Value = int | str | CellError
def show(value: Value) -> str:
    if isinstance(value, CellError):
        return value.code
    return str(value)
def div(a: int, b: int) -> int:
    if b == 0:
        raise CellError(DIV0)
    q = abs(a) // abs(b)
    return q if (a < 0) == (b < 0) else -q
class LoadError(Exception):
    pass
def parse_content(text: str) -> Content | None:
    if text == "":
        return None
    if text.startswith("="):
        src = text[1:].strip()
        try:
            return Formula(src, parse_formula(src))
        except ParseError as err:
            return Broken(src, str(err))
    if len(text) >= 2 and text.startswith('"') and text.endswith('"'):
        return Label(text[1:-1])
    try:
        return Number(int(text))
    except ValueError:
        raise LoadError(f"bad content '{text}'") from None
def parse_line(line: str) -> tuple[Pos, Content | None]:
    name, sep, rest = line.partition("=")
    if not sep:
        raise LoadError("missing '='")
    pos = parse_pos(name.strip())
    if pos is None:
        raise LoadError(f"bad cell name '{name.strip()}'")
    return pos, parse_content(rest.strip())
class Sheet:
    def __init__(self) -> None:
        self.cells: dict[Pos, Content] = {}
        self.memo: dict[Pos, Value] = {}
        self.active: set[Pos] = set()
    def apply(self, lines: list[str]) -> None:
        for number, line in enumerate(lines, start=1):
            try:
                pos, content = parse_line(line)
            except LoadError as err:
                print(f"line {number}: {err}: {line}")
                continue
            if content is None:
                self.cells.pop(pos, None)
            else:
                self.cells[pos] = content
        self.memo = {}
    def value(self, pos: Pos) -> Value:
        if pos in self.memo:
            return self.memo[pos]
        match self.cells.get(pos):
            case None:
                return 0
            case Number(n):
                return n
            case Label(text):
                return text
            case Broken():
                return CellError(PARSE)
            case Formula(_, expr):
                if pos in self.active:
                    return CellError(CYCLE)
                self.active.add(pos)
                try:
                    result: Value = self.eval(expr)
                except CellError as err:
                    result = err
                self.active.remove(pos)
                self.memo[pos] = result
                return result
    def number(self, pos: Pos) -> int:
        value = self.value(pos)
        if isinstance(value, CellError):
            raise value
        if isinstance(value, str):
            raise CellError(VALUE)
        return value
    def numbers(self, first: Pos, last: Pos) -> list[int]:
        found = []
        for pos in cells_in(first, last):
            if pos not in self.cells:
                continue
            value = self.value(pos)
            if isinstance(value, CellError):
                raise value
            if isinstance(value, int):
                found.append(value)
        return found
    def eval(self, expr: Expr) -> int:
        match expr:
            case Lit(n):
                return n
            case Ref(pos):
                return self.number(pos)
            case Neg(operand):
                return -self.eval(operand)
            case Bin(op, left, right):
                a = self.eval(left)
                b = self.eval(right)
                if op == "+":
                    return a + b
                if op == "-":
                    return a - b
                if op == "*":
                    return a * b
                return div(a, b)
            case Agg(func, first, last):
                nums = self.numbers(first, last)
                if func == "SUM":
                    return sum(nums)
                if func == "COUNT":
                    return len(nums)
                if func == "AVG":
                    return div(sum(nums), len(nums))
                if not nums:
                    return 0
                return min(nums) if func == "MIN" else max(nums)
            case If(cmp, left, right, then, other):
                a = self.eval(left)
                b = self.eval(right)
                holds = a < b if cmp == "<" else a > b if cmp == ">" else a == b
                return self.eval(then if holds else other)
    def display(self, pos: Pos) -> str:
        if pos not in self.cells:
            return ""
        return show(self.value(pos))
    def snapshot(self) -> dict[Pos, str]:
        return {pos: self.display(pos) for pos in self.cells}
    def formulas(self) -> list[Pos]:
        return sorted(p for p, c in self.cells.items() if isinstance(c, (Formula, Broken)))
    def deps(self, pos: Pos) -> list[Pos]:
        content = self.cells.get(pos)
        if not isinstance(content, Formula):
            return []
        refs: set[Pos] = set()
        collect_refs(content.expr, refs)
        return sorted(refs)
    def eval_order(self) -> tuple[list[Pos], list[Pos]]:
\
        formulas = self.formulas()
        is_formula = set(formulas)
        waits = {p: {d for d in self.deps(p) if d in is_formula} for p in formulas}
        order: list[Pos] = []
        done: set[Pos] = set()
        while True:
            ready = [p for p in formulas if p not in done and waits[p] <= done]
            if not ready:
                break
            order.append(ready[0])
            done.add(ready[0])
        blocked = [p for p in formulas if p not in done]
        return order, blocked
    def table(self) -> list[str]:
        cols = sorted({p.col for p in self.cells})
        rows = sorted({p.row for p in self.cells})
        grid = [[""] + [COLS[c] for c in cols]]
        for r in rows:
            grid.append([str(r)] + [self.display(Pos(c, r)) for c in cols])
        widths = [max(len(line[i]) for line in grid) for i in range(len(grid[0]))]
        lines = []
        for k, line in enumerate(grid):
            lines.append(" | ".join(text.rjust(w) for text, w in zip(line, widths)))
            if k == 0:
                lines.append("-+-".join("-" * w for w in widths))
        return lines
    def print_table(self) -> None:
        for line in self.table():
            print(line)
    def print_deps(self) -> None:
        print("dependencies:")
        for pos in self.formulas():
            content = self.cells[pos]
            if isinstance(content, Broken):
                print(f"  {pos} <- {PARSE} {content.error}")
                continue
            deps = self.deps(pos)
            print(f"  {pos} <- {names(deps) if deps else '(none)'}")
    def print_parse_errors(self) -> None:
        for pos in self.formulas():
            content = self.cells[pos]
            if isinstance(content, Broken):
                print(f"parse error in {pos}: {content.error} in '{content.src}'")
    def print_order(self) -> None:
        order, blocked = self.eval_order()
        print(f"order: {names(order)}")
        if blocked:
            print(f"blocked by a cycle: {names(blocked)}")
def print_changes(before: dict[Pos, str], after: dict[Pos, str]) -> None:
    changed = []
    for pos in sorted(set(before) | set(after)):
        old = before.get(pos, "")
        new = after.get(pos, "")
        if old != new:
            changed.append(f"{pos} {old or '(empty)'} -> {new or '(empty)'}")
    if not changed:
        print("changed: nothing")
        return
    print("changed:")
    for line in changed:
        print(f"  {line}")
BUDGET = """\
A1 = "Item"
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
C6 = twelve"""
EDITS = [
    ("change a value", ["B2 = 20"]),
    ("introduce a cycle", ["D11 = =D12 + 50", "G15 = =G15 + 1"]),
    ("break the cycle", ["D11 = 250", "G15 = "]),
    ("divide by zero", ["C5 = 0", "B11 = =D11 / (B5 - 1)", "B12 = =AVG(F1:F3)"]),
    ("parse errors", [
        "D4 = =B4 * (C4 + 1",
        "B12 = =SUMM(B2:B5)",
        "B13 = =B2 $ 3",
        "C11 = =IF(B2, 1, 2)",
        "C12 = =Z9 + 1",
        "C13 = =3 4",
    ]),
    ("fix the formula, then reference a text cell", [
        "D4 = =B4 * (C4 + 1)",
        "B11 = =A2 * 2",
        "B12 = =SUM(A2:A5) + COUNT(A2:B5)",
        "B13 = =IF(A3 = 1, 1, 0)",
    ]),
]
def main() -> None:
    sheet = Sheet()
    print("== budget")
    sheet.apply(BUDGET.splitlines())
    sheet.print_table()
    sheet.print_deps()
    sheet.print_order()
    for number, (title, lines) in enumerate(EDITS, start=1):
        print()
        print(f"== edit {number}: {title}")
        for line in lines:
            print(f"  {line}")
        before = sheet.snapshot()
        sheet.apply(lines)
        sheet.print_table()
        print_changes(before, sheet.snapshot())
        sheet.print_parse_errors()
        sheet.print_order()
    print()
    print("== final")
    sheet.print_deps()
if __name__ == "__main__":
    main()
