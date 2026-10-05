const COLS = "ABCDEFGH";
const MAX_ROW = 20;
const INT_MAX = Number.MAX_SAFE_INTEGER;

class Pos {
  private static interned = new Map<number, Pos>();

  private constructor(
    readonly col: number,
    readonly row: number,
  ) {}

  static at(col: number, row: number): Pos {
    const key = col * (MAX_ROW + 1) + row;
    let pos = Pos.interned.get(key);
    if (pos === undefined) {
      pos = new Pos(col, row);
      Pos.interned.set(key, pos);
    }
    return pos;
  }

  toString(): string {
    return `${COLS[this.col]}${this.row}`;
  }
}

function comparePos(a: Pos, b: Pos): number {
  return a.col - b.col || a.row - b.row;
}

function parsePos(text: string): Pos | null {
  if (text.length < 2) return null;
  const col = COLS.indexOf(text[0]);
  const digits = text.slice(1);
  if (col < 0 || !/^\d+$/.test(digits)) return null;
  const row = Number(digits);
  if (row < 1 || row > MAX_ROW) return null;
  return Pos.at(col, row);
}

function cellsIn(first: Pos, last: Pos): Pos[] {
  const c0 = Math.min(first.col, last.col);
  const c1 = Math.max(first.col, last.col);
  const r0 = Math.min(first.row, last.row);
  const r1 = Math.max(first.row, last.row);
  const cells: Pos[] = [];
  for (let r = r0; r <= r1; r++) {
    for (let c = c0; c <= c1; c++) {
      cells.push(Pos.at(c, r));
    }
  }
  return cells;
}

type Expr =
  | { kind: "lit"; n: number }
  | { kind: "ref"; pos: Pos }
  | { kind: "neg"; operand: Expr }
  | { kind: "bin"; op: string; left: Expr; right: Expr }
  | { kind: "agg"; func: string; first: Pos; last: Pos }
  | {
      kind: "if";
      cmp: string;
      left: Expr;
      right: Expr;
      yes: Expr;
      no: Expr;
    };

const AGGREGATES = ["SUM", "MIN", "MAX", "COUNT", "AVG"];
const SYMBOLS = "+-*/(),:<>=";

class ParseError extends Error {}

interface Token {
  kind: "num" | "cell" | "name" | "sym" | "end";
  text: string;
}

function describe(tok: Token): string {
  if (tok.kind === "end") return "end of formula";
  if (tok.kind === "sym") return `'${tok.text}'`;
  return tok.text;
}

const isDigit = (c: string) => c >= "0" && c <= "9";
const isAlpha = (c: string) => /^[A-Za-z]$/.test(c);

function tokenize(src: string): Token[] {
  const tokens: Token[] = [];
  let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (c === " ") {
      i++;
    } else if (isDigit(c)) {
      const start = i;
      while (i < src.length && isDigit(src[i])) i++;
      tokens.push({ kind: "num", text: src.slice(start, i) });
    } else if (isAlpha(c)) {
      const start = i;
      while (i < src.length && (isAlpha(src[i]) || isDigit(src[i]))) i++;
      const word = src.slice(start, i);
      if (isDigit(word[word.length - 1])) {
        if (parsePos(word) === null) {
          throw new ParseError(`bad cell name ${word}`);
        }
        tokens.push({ kind: "cell", text: word });
      } else {
        tokens.push({ kind: "name", text: word });
      }
    } else if (SYMBOLS.includes(c)) {
      tokens.push({ kind: "sym", text: c });
      i++;
    } else {
      throw new ParseError(`unexpected character '${c}'`);
    }
  }
  tokens.push({ kind: "end", text: "" });
  return tokens;
}

class Parser {
  private at = 0;

  constructor(private readonly tokens: Token[]) {}

  peek(): Token {
    return this.tokens[this.at];
  }

  next(): Token {
    const tok = this.tokens[this.at];
    if (tok.kind !== "end") this.at++;
    return tok;
  }

  atSym(syms: string): boolean {
    const tok = this.peek();
    return tok.kind === "sym" && syms.includes(tok.text);
  }

  expect(sym: string): void {
    const tok = this.next();
    if (tok.kind !== "sym" || tok.text !== sym) {
      throw new ParseError(`expected '${sym}' but found ${describe(tok)}`);
    }
  }

  cell(): Pos {
    const tok = this.next();
    if (tok.kind !== "cell") {
      throw new ParseError(`expected a cell but found ${describe(tok)}`);
    }
    return parsePos(tok.text)!;
  }

  formula(): Expr {
    const expr = this.sum();
    if (this.peek().kind !== "end") {
      throw new ParseError(`unexpected ${describe(this.peek())}`);
    }
    return expr;
  }

  sum(): Expr {
    let left = this.product();
    while (this.atSym("+-")) {
      const op = this.next().text;
      left = { kind: "bin", op, left, right: this.product() };
    }
    return left;
  }

  product(): Expr {
    let left = this.unary();
    while (this.atSym("*/")) {
      const op = this.next().text;
      left = { kind: "bin", op, left, right: this.unary() };
    }
    return left;
  }

  unary(): Expr {
    if (this.atSym("-")) {
      this.next();
      return { kind: "neg", operand: this.unary() };
    }
    return this.primary();
  }

  primary(): Expr {
    const tok = this.next();
    if (tok.kind === "num") {
      const n = Number(tok.text);
      if (n > INT_MAX) {
        throw new ParseError(`number too large: ${tok.text}`);
      }
      return { kind: "lit", n };
    }
    if (tok.kind === "cell") {
      return { kind: "ref", pos: parsePos(tok.text)! };
    }
    if (tok.kind === "name") {
      return this.call(tok.text);
    }
    if (tok.kind === "sym" && tok.text === "(") {
      const inner = this.sum();
      this.expect(")");
      return inner;
    }
    throw new ParseError(`unexpected ${describe(tok)}`);
  }

  call(name: string): Expr {
    if (AGGREGATES.includes(name)) {
      this.expect("(");
      const first = this.cell();
      this.expect(":");
      const last = this.cell();
      this.expect(")");
      return { kind: "agg", func: name, first, last };
    }
    if (name === "IF") {
      this.expect("(");
      const left = this.sum();
      const tok = this.next();
      if (tok.kind !== "sym" || !"<>=".includes(tok.text)) {
        throw new ParseError(
          `expected a comparison but found ${describe(tok)}`,
        );
      }
      const right = this.sum();
      this.expect(",");
      const yes = this.sum();
      this.expect(",");
      const no = this.sum();
      this.expect(")");
      return { kind: "if", cmp: tok.text, left, right, yes, no };
    }
    throw new ParseError(`unknown function ${name}`);
  }
}

function parseFormula(src: string): Expr {
  return new Parser(tokenize(src)).formula();
}

function collectRefs(expr: Expr, out: Set<Pos>): void {
  switch (expr.kind) {
    case "lit":
      break;
    case "ref":
      out.add(expr.pos);
      break;
    case "neg":
      collectRefs(expr.operand, out);
      break;
    case "bin":
      collectRefs(expr.left, out);
      collectRefs(expr.right, out);
      break;
    case "agg":
      for (const pos of cellsIn(expr.first, expr.last)) out.add(pos);
      break;
    case "if":
      for (const part of [expr.left, expr.right, expr.yes, expr.no]) {
        collectRefs(part, out);
      }
      break;
  }
}

type Content =
  | { kind: "number"; n: number }
  | { kind: "label"; text: string }
  | { kind: "formula"; src: string; expr: Expr }
  | { kind: "broken"; src: string; error: string };

const VALUE = "#VALUE!";
const CYCLE = "#CYCLE!";
const DIV0 = "#DIV/0!";
const PARSE = "#PARSE!";

class CellError extends Error {
  constructor(readonly code: string) {
    super(code);
  }
}

type Value = number | string | CellError;

function show(value: Value): string {
  return value instanceof CellError ? value.code : String(value);
}

function div(a: number, b: number): number {
  if (b === 0) throw new CellError(DIV0);
  return Math.trunc(a / b);
}

function sum(nums: number[]): number {
  return nums.reduce((a, b) => a + b, 0);
}

class LoadError extends Error {}

function parseContent(text: string): Content | null {
  if (text === "") return null;
  if (text.startsWith("=")) {
    const src = text.slice(1).trim();
    try {
      return { kind: "formula", src, expr: parseFormula(src) };
    } catch (err) {
      if (err instanceof ParseError) {
        return { kind: "broken", src, error: err.message };
      }
      throw err;
    }
  }
  if (text.length >= 2 && text.startsWith('"') && text.endsWith('"')) {
    return { kind: "label", text: text.slice(1, -1) };
  }
  if (/^[+-]?\d+$/.test(text)) {
    return { kind: "number", n: Number(text) };
  }
  throw new LoadError(`bad content '${text}'`);
}

function parseLine(line: string): [Pos, Content | null] {
  const eq = line.indexOf("=");
  if (eq < 0) throw new LoadError("missing '='");
  const name = line.slice(0, eq).trim();
  const pos = parsePos(name);
  if (pos === null) throw new LoadError(`bad cell name '${name}'`);
  return [pos, parseContent(line.slice(eq + 1).trim())];
}

class Sheet {
  cells = new Map<Pos, Content>();
  memo = new Map<Pos, Value>();
  active = new Set<Pos>();

  apply(lines: string[]): void {
    for (const [i, line] of lines.entries()) {
      try {
        const [pos, content] = parseLine(line);
        if (content === null) {
          this.cells.delete(pos);
        } else {
          this.cells.set(pos, content);
        }
      } catch (err) {
        if (!(err instanceof LoadError)) throw err;
        console.log(`line ${i + 1}: ${err.message}: ${line}`);
      }
    }
    this.memo = new Map();
  }

  value(pos: Pos): Value {
    const memoized = this.memo.get(pos);
    if (memoized !== undefined) return memoized;
    const content = this.cells.get(pos);
    if (content === undefined) return 0;
    switch (content.kind) {
      case "number":
        return content.n;
      case "label":
        return content.text;
      case "broken":
        return new CellError(PARSE);
      case "formula": {
        if (this.active.has(pos)) return new CellError(CYCLE);
        this.active.add(pos);
        let result: Value;
        try {
          result = this.evaluate(content.expr);
        } catch (err) {
          if (!(err instanceof CellError)) throw err;
          result = err;
        }
        this.active.delete(pos);
        this.memo.set(pos, result);
        return result;
      }
    }
  }

  number(pos: Pos): number {
    const value = this.value(pos);
    if (value instanceof CellError) throw value;
    if (typeof value === "string") throw new CellError(VALUE);
    return value;
  }

  numbers(first: Pos, last: Pos): number[] {
    const found: number[] = [];
    for (const pos of cellsIn(first, last)) {
      if (!this.cells.has(pos)) continue;
      const value = this.value(pos);
      if (value instanceof CellError) throw value;
      if (typeof value === "number") found.push(value);
    }
    return found;
  }

  evaluate(expr: Expr): number {
    switch (expr.kind) {
      case "lit":
        return expr.n;
      case "ref":
        return this.number(expr.pos);
      case "neg":
        return -this.evaluate(expr.operand);
      case "bin": {
        const a = this.evaluate(expr.left);
        const b = this.evaluate(expr.right);
        switch (expr.op) {
          case "+":
            return a + b;
          case "-":
            return a - b;
          case "*":
            return a * b;
          default:
            return div(a, b);
        }
      }
      case "agg": {
        const nums = this.numbers(expr.first, expr.last);
        switch (expr.func) {
          case "SUM":
            return sum(nums);
          case "COUNT":
            return nums.length;
          case "AVG":
            return div(sum(nums), nums.length);
        }
        if (nums.length === 0) return 0;
        return expr.func === "MIN" ? Math.min(...nums) : Math.max(...nums);
      }
      case "if": {
        const a = this.evaluate(expr.left);
        const b = this.evaluate(expr.right);
        const holds =
          expr.cmp === "<" ? a < b : expr.cmp === ">" ? a > b : a === b;
        return this.evaluate(holds ? expr.yes : expr.no);
      }
    }
  }

  display(pos: Pos): string {
    if (!this.cells.has(pos)) return "";
    return show(this.value(pos));
  }

  snapshot(): Map<Pos, string> {
    const shown = new Map<Pos, string>();
    for (const pos of this.cells.keys()) shown.set(pos, this.display(pos));
    return shown;
  }

  formulas(): Pos[] {
    return [...this.cells]
      .filter(([, c]) => c.kind === "formula" || c.kind === "broken")
      .map(([pos]) => pos)
      .sort(comparePos);
  }

  deps(pos: Pos): Pos[] {
    const content = this.cells.get(pos);
    if (content?.kind !== "formula") return [];
    const refs = new Set<Pos>();
    collectRefs(content.expr, refs);
    return [...refs].sort(comparePos);
  }

  evalOrder(): [Pos[], Pos[]] {
    const formulas = this.formulas();
    const isFormula = new Set(formulas);
    const waits = new Map<Pos, Pos[]>(
      formulas.map((p) => [p, this.deps(p).filter((d) => isFormula.has(d))]),
    );
    const order: Pos[] = [];
    const done = new Set<Pos>();
    while (true) {
      const ready = formulas.find(
        (p) => !done.has(p) && waits.get(p)!.every((d) => done.has(d)),
      );
      if (ready === undefined) break;
      order.push(ready);
      done.add(ready);
    }
    const blocked = formulas.filter((p) => !done.has(p));
    return [order, blocked];
  }

  table(): string[] {
    const positions = [...this.cells.keys()];
    const cols = [...new Set(positions.map((p) => p.col))].sort(
      (a, b) => a - b,
    );
    const rows = [...new Set(positions.map((p) => p.row))].sort(
      (a, b) => a - b,
    );
    const grid = [["", ...cols.map((c) => COLS[c])]];
    for (const r of rows) {
      grid.push([String(r), ...cols.map((c) => this.display(Pos.at(c, r)))]);
    }
    const widths = grid[0].map((_, i) =>
      Math.max(...grid.map((line) => line[i].length)),
    );
    const lines: string[] = [];
    for (const [k, line] of grid.entries()) {
      lines.push(line.map((text, i) => text.padStart(widths[i])).join(" | "));
      if (k === 0) lines.push(widths.map((w) => "-".repeat(w)).join("-+-"));
    }
    return lines;
  }

  printTable(): void {
    for (const line of this.table()) console.log(line);
  }

  printDeps(): void {
    console.log("dependencies:");
    for (const pos of this.formulas()) {
      const content = this.cells.get(pos);
      if (content?.kind === "broken") {
        console.log(`  ${pos} <- ${PARSE} ${content.error}`);
        continue;
      }
      const deps = this.deps(pos);
      console.log(
        `  ${pos} <- ${deps.length > 0 ? deps.join(", ") : "(none)"}`,
      );
    }
  }

  printParseErrors(): void {
    for (const pos of this.formulas()) {
      const content = this.cells.get(pos);
      if (content?.kind === "broken") {
        console.log(
          `parse error in ${pos}: ${content.error} in '${content.src}'`,
        );
      }
    }
  }

  printOrder(): void {
    const [order, blocked] = this.evalOrder();
    console.log(`order: ${order.join(", ")}`);
    if (blocked.length > 0) {
      console.log(`blocked by a cycle: ${blocked.join(", ")}`);
    }
  }
}

function printChanges(before: Map<Pos, string>, after: Map<Pos, string>): void {
  const changed: string[] = [];
  const all = new Set([...before.keys(), ...after.keys()]);
  for (const pos of [...all].sort(comparePos)) {
    const old = before.get(pos) ?? "";
    const now = after.get(pos) ?? "";
    if (old !== now) {
      changed.push(`${pos} ${old || "(empty)"} -> ${now || "(empty)"}`);
    }
  }
  if (changed.length === 0) {
    console.log("changed: nothing");
    return;
  }
  console.log("changed:");
  for (const line of changed) console.log(`  ${line}`);
}

const BUDGET = `A1 = "Item"
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
C6 = twelve`;

const EDITS: [string, string[]][] = [
  ["change a value", ["B2 = 20"]],
  ["introduce a cycle", ["D11 = =D12 + 50", "G15 = =G15 + 1"]],
  ["break the cycle", ["D11 = 250", "G15 = "]],
  ["divide by zero", ["C5 = 0", "B11 = =D11 / (B5 - 1)", "B12 = =AVG(F1:F3)"]],
  [
    "parse errors",
    [
      "D4 = =B4 * (C4 + 1",
      "B12 = =SUMM(B2:B5)",
      "B13 = =B2 $ 3",
      "C11 = =IF(B2, 1, 2)",
      "C12 = =Z9 + 1",
      "C13 = =3 4",
    ],
  ],
  [
    "fix the formula, then reference a text cell",
    [
      "D4 = =B4 * (C4 + 1)",
      "B11 = =A2 * 2",
      "B12 = =SUM(A2:A5) + COUNT(A2:B5)",
      "B13 = =IF(A3 = 1, 1, 0)",
    ],
  ],
];

function main(): void {
  const sheet = new Sheet();
  console.log("== budget");
  sheet.apply(BUDGET.split("\n"));
  sheet.printTable();
  sheet.printDeps();
  sheet.printOrder();
  for (const [i, [title, lines]] of EDITS.entries()) {
    console.log();
    console.log(`== edit ${i + 1}: ${title}`);
    for (const line of lines) console.log(`  ${line}`);
    const before = sheet.snapshot();
    sheet.apply(lines);
    sheet.printTable();
    printChanges(before, sheet.snapshot());
    sheet.printParseErrors();
    sheet.printOrder();
  }
  console.log();
  console.log("== final");
  sheet.printDeps();
}

main();
