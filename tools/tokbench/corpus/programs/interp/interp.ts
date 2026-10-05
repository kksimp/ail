const MAX_DEPTH = 200;
const INT_MIN = -(2n ** 63n);
const INT_MAX = 2n ** 63n - 1n;
const KEYWORDS = new Set([
  "let",
  "if",
  "else",
  "while",
  "fn",
  "return",
  "true",
  "false",
  "and",
  "or",
  "not",
]);
const TWO_CHAR_OPS = new Set(["==", "!=", "<=", ">="]);
const ONE_CHAR_OPS = new Set("+-*/%<>=(){},");
const ESCAPES = new Map([
  ["n", "\n"],
  ['"', '"'],
  ["\\", "\\"],
]);

interface Pos {
  line: number;
  col: number;
}

class MiniError extends Error {
  constructor(
    readonly pos: Pos,
    message: string,
  ) {
    super(message);
  }
}

interface Token {
  kind: "int" | "str" | "ident" | "keyword" | "op" | "end";
  text: string;
  pos: Pos;
}

function describe(tok: Token): string {
  switch (tok.kind) {
    case "end":
      return "end of input";
    case "int":
      return `number ${tok.text}`;
    case "str":
      return "string";
    default:
      return `'${tok.text}'`;
  }
}

class Lexer {
  private i = 0;
  private line = 1;
  private col = 1;

  constructor(private readonly src: string) {}

  private peek(offset = 0): string {
    return this.src.charAt(this.i + offset);
  }

  private advance(): string {
    const c = this.src[this.i++];
    if (c === "\n") {
      this.line++;
      this.col = 1;
    } else {
      this.col++;
    }
    return c;
  }

  tokenize(): Token[] {
    const tokens: Token[] = [];
    while (true) {
      this.skipSpaceAndComments();
      const pos = { line: this.line, col: this.col };
      const c = this.peek();
      if (c === "") {
        tokens.push({ kind: "end", text: "", pos });
        return tokens;
      }
      if (/[0-9]/.test(c)) {
        tokens.push(this.number(pos));
      } else if (/[A-Za-z_]/.test(c)) {
        tokens.push(this.word(pos));
      } else if (c === '"') {
        tokens.push(this.string(pos));
      } else if (TWO_CHAR_OPS.has(c + this.peek(1))) {
        tokens.push({ kind: "op", text: this.advance() + this.advance(), pos });
      } else if (ONE_CHAR_OPS.has(c)) {
        tokens.push({ kind: "op", text: this.advance(), pos });
      } else {
        throw new MiniError(pos, `unexpected character '${c}'`);
      }
    }
  }

  private skipSpaceAndComments(): void {
    while (true) {
      const c = this.peek();
      if ([" ", "\n", "\t", "\r"].includes(c)) {
        this.advance();
      } else if (c === "/" && this.peek(1) === "/") {
        while (this.peek() !== "" && this.peek() !== "\n") {
          this.advance();
        }
      } else {
        return;
      }
    }
  }

  private number(pos: Pos): Token {
    const start = this.i;
    while (/[0-9]/.test(this.peek())) {
      this.advance();
    }
    return { kind: "int", text: this.src.slice(start, this.i), pos };
  }

  private word(pos: Pos): Token {
    const start = this.i;
    while (/[A-Za-z0-9_]/.test(this.peek())) {
      this.advance();
    }
    const text = this.src.slice(start, this.i);
    return { kind: KEYWORDS.has(text) ? "keyword" : "ident", text, pos };
  }

  private string(pos: Pos): Token {
    this.advance();
    let value = "";
    while (true) {
      const c = this.peek();
      if (c === "" || c === "\n") {
        throw new MiniError(pos, "unterminated string");
      }
      this.advance();
      if (c === '"') {
        return { kind: "str", text: value, pos };
      }
      if (c === "\\") {
        const escapePos = { line: this.line, col: this.col - 1 };
        const e = this.peek();
        if (e === "" || e === "\n") {
          continue;
        }
        const escaped = ESCAPES.get(e);
        if (escaped === undefined) {
          throw new MiniError(escapePos, `unknown escape '\\${e}'`);
        }
        this.advance();
        value += escaped;
      } else {
        value += c;
      }
    }
  }
}

type Expr =
  | { kind: "int"; pos: Pos; value: bigint }
  | { kind: "str"; pos: Pos; value: string }
  | { kind: "bool"; pos: Pos; value: boolean }
  | { kind: "var"; pos: Pos; name: string }
  | { kind: "unary"; pos: Pos; op: string; operand: Expr }
  | { kind: "binary"; pos: Pos; op: string; left: Expr; right: Expr }
  | { kind: "call"; pos: Pos; name: string; args: Expr[] };

interface FnDecl {
  kind: "fn";
  pos: Pos;
  name: string;
  params: string[];
  body: Stmt[];
}

type Stmt =
  | { kind: "let"; pos: Pos; name: string; value: Expr }
  | { kind: "assign"; pos: Pos; name: string; value: Expr }
  | { kind: "if"; pos: Pos; cond: Expr; then: Stmt[]; otherwise: Stmt[] | null }
  | { kind: "while"; pos: Pos; cond: Expr; body: Stmt[] }
  | FnDecl
  | { kind: "return"; pos: Pos; value: Expr | null }
  | { kind: "expr"; pos: Pos; expr: Expr };

const BINARY_LEVELS = [
  ["or"],
  ["and"],
  ["==", "!="],
  ["<", "<=", ">", ">="],
  ["+", "-"],
  ["*", "/", "%"],
];

class Parser {
  private i = 0;

  constructor(private readonly tokens: Token[]) {}

  private peek(offset = 0): Token {
    return this.tokens[Math.min(this.i + offset, this.tokens.length - 1)];
  }

  private advance(): Token {
    const tok = this.peek();
    if (tok.kind !== "end") {
      this.i++;
    }
    return tok;
  }

  private at(text: string): boolean {
    const tok = this.peek();
    return (tok.kind === "op" || tok.kind === "keyword") && tok.text === text;
  }

  private expect(text: string): Token {
    if (!this.at(text)) {
      throw new MiniError(
        this.peek().pos,
        `expected '${text}' but found ${describe(this.peek())}`,
      );
    }
    return this.advance();
  }

  private expectIdent(): string {
    const tok = this.peek();
    if (tok.kind !== "ident") {
      throw new MiniError(
        tok.pos,
        `expected identifier but found ${describe(tok)}`,
      );
    }
    return this.advance().text;
  }

  program(): Stmt[] {
    const stmts: Stmt[] = [];
    while (this.peek().kind !== "end") {
      stmts.push(this.statement());
    }
    return stmts;
  }

  private block(): Stmt[] {
    this.expect("{");
    const stmts: Stmt[] = [];
    while (!this.at("}")) {
      if (this.peek().kind === "end") {
        throw new MiniError(
          this.peek().pos,
          "expected '}' but found end of input",
        );
      }
      stmts.push(this.statement());
    }
    this.advance();
    return stmts;
  }

  private statement(): Stmt {
    const tok = this.peek();
    const pos = tok.pos;
    if (this.at("let")) {
      this.advance();
      const name = this.expectIdent();
      this.expect("=");
      return { kind: "let", pos, name, value: this.expression() };
    }
    if (this.at("if")) {
      return this.ifStatement();
    }
    if (this.at("while")) {
      this.advance();
      const cond = this.expression();
      return { kind: "while", pos, cond, body: this.block() };
    }
    if (this.at("fn")) {
      this.advance();
      const name = this.expectIdent();
      this.expect("(");
      const params: string[] = [];
      if (!this.at(")")) {
        params.push(this.expectIdent());
        while (this.at(",")) {
          this.advance();
          params.push(this.expectIdent());
        }
      }
      this.expect(")");
      return { kind: "fn", pos, name, params, body: this.block() };
    }
    if (this.at("return")) {
      this.advance();
      if (this.at("}")) {
        return { kind: "return", pos, value: null };
      }
      return { kind: "return", pos, value: this.expression() };
    }
    if (
      tok.kind === "ident" &&
      this.peek(1).kind === "op" &&
      this.peek(1).text === "="
    ) {
      this.advance();
      this.advance();
      return { kind: "assign", pos, name: tok.text, value: this.expression() };
    }
    return { kind: "expr", pos, expr: this.expression() };
  }

  private ifStatement(): Stmt {
    const pos = this.expect("if").pos;
    const cond = this.expression();
    const then = this.block();
    let otherwise: Stmt[] | null = null;
    if (this.at("else")) {
      this.advance();
      otherwise = this.at("if") ? [this.ifStatement()] : this.block();
    }
    return { kind: "if", pos, cond, then, otherwise };
  }

  private expression(level = 0): Expr {
    if (level === BINARY_LEVELS.length) {
      return this.unary();
    }
    let left = this.expression(level + 1);
    while (
      (this.peek().kind === "op" || this.peek().kind === "keyword") &&
      BINARY_LEVELS[level].includes(this.peek().text)
    ) {
      const op = this.advance();
      const right = this.expression(level + 1);
      left = { kind: "binary", pos: op.pos, op: op.text, left, right };
    }
    return left;
  }

  private unary(): Expr {
    if (this.at("-") || this.at("not")) {
      const op = this.advance();
      return { kind: "unary", pos: op.pos, op: op.text, operand: this.unary() };
    }
    return this.primary();
  }

  private primary(): Expr {
    const tok = this.advance();
    switch (tok.kind) {
      case "int": {
        const value = BigInt(tok.text);
        if (value > INT_MAX) {
          throw new MiniError(tok.pos, "number too large");
        }
        return { kind: "int", pos: tok.pos, value };
      }
      case "str":
        return { kind: "str", pos: tok.pos, value: tok.text };
      case "ident": {
        if (!this.at("(")) {
          return { kind: "var", pos: tok.pos, name: tok.text };
        }
        this.advance();
        const args: Expr[] = [];
        if (!this.at(")")) {
          args.push(this.expression());
          while (this.at(",")) {
            this.advance();
            args.push(this.expression());
          }
        }
        this.expect(")");
        return { kind: "call", pos: tok.pos, name: tok.text, args };
      }
      case "keyword":
        if (tok.text === "true" || tok.text === "false") {
          return { kind: "bool", pos: tok.pos, value: tok.text === "true" };
        }
        break;
      case "op":
        if (tok.text === "(") {
          const inner = this.expression();
          this.expect(")");
          return inner;
        }
        break;
    }
    throw new MiniError(
      tok.pos,
      `expected expression but found ${describe(tok)}`,
    );
  }
}

type Value = bigint | string | boolean | null;

function typeName(v: Value): string {
  switch (typeof v) {
    case "bigint":
      return "int";
    case "string":
      return "str";
    case "boolean":
      return "bool";
    default:
      return "nil";
  }
}

function show(v: Value): string {
  return v === null ? "nil" : String(v);
}

function checked(pos: Pos, n: bigint): bigint {
  if (n < INT_MIN || n > INT_MAX) {
    throw new MiniError(pos, "integer overflow");
  }
  return n;
}

class ReturnSignal {
  constructor(readonly value: Value) {}
}

class Env {
  scopes: Map<string, Value>[] = [new Map()];
  base = 0;

  find(name: string): Map<string, Value> | undefined {
    for (let i = this.scopes.length - 1; i >= this.base; i--) {
      if (this.scopes[i].has(name)) {
        return this.scopes[i];
      }
    }
    return this.scopes[0].has(name) ? this.scopes[0] : undefined;
  }
}

class Interpreter {
  private env = new Env();
  private functions = new Map<string, FnDecl>();
  private depth = 0;

  run(program: Stmt[]): void {
    try {
      for (const stmt of program) {
        this.execute(stmt);
      }
    } catch (e) {
      if (!(e instanceof ReturnSignal)) {
        throw e;
      }
    }
  }

  private executeBlock(body: Stmt[]): void {
    this.env.scopes.push(new Map());
    try {
      for (const stmt of body) {
        this.execute(stmt);
      }
    } finally {
      this.env.scopes.pop();
    }
  }

  private execute(stmt: Stmt): void {
    switch (stmt.kind) {
      case "let": {
        const { scopes } = this.env;
        scopes[scopes.length - 1].set(stmt.name, this.evaluate(stmt.value));
        break;
      }
      case "assign": {
        const scope = this.env.find(stmt.name);
        if (scope === undefined) {
          throw new MiniError(stmt.pos, `undefined variable '${stmt.name}'`);
        }
        scope.set(stmt.name, this.evaluate(stmt.value));
        break;
      }
      case "if":
        if (this.condition(stmt.cond)) {
          this.executeBlock(stmt.then);
        } else if (stmt.otherwise !== null) {
          this.executeBlock(stmt.otherwise);
        }
        break;
      case "while":
        while (this.condition(stmt.cond)) {
          this.executeBlock(stmt.body);
        }
        break;
      case "fn":
        this.functions.set(stmt.name, stmt);
        break;
      case "return":
        throw new ReturnSignal(
          stmt.value === null ? null : this.evaluate(stmt.value),
        );
      case "expr":
        this.evaluate(stmt.expr);
        break;
    }
  }

  private condition(expr: Expr): boolean {
    const v = this.evaluate(expr);
    if (typeof v !== "boolean") {
      throw new MiniError(
        expr.pos,
        `type mismatch: condition must be bool, got ${typeName(v)}`,
      );
    }
    return v;
  }

  private evaluate(expr: Expr): Value {
    switch (expr.kind) {
      case "int":
      case "str":
      case "bool":
        return expr.value;
      case "var": {
        const scope = this.env.find(expr.name);
        if (scope === undefined) {
          throw new MiniError(expr.pos, `undefined variable '${expr.name}'`);
        }
        return scope.get(expr.name)!;
      }
      case "unary": {
        const v = this.evaluate(expr.operand);
        if (expr.op === "-" && typeof v === "bigint") {
          return checked(expr.pos, -v);
        }
        if (expr.op === "not" && typeof v === "boolean") {
          return !v;
        }
        throw new MiniError(
          expr.pos,
          `type mismatch: cannot apply '${expr.op}' to ${typeName(v)}`,
        );
      }
      case "binary":
        if (expr.op === "and" || expr.op === "or") {
          return this.logical(expr.pos, expr.op, expr.left, expr.right);
        }
        return this.binary(
          expr.pos,
          expr.op,
          this.evaluate(expr.left),
          this.evaluate(expr.right),
        );
      case "call":
        return this.call(
          expr.pos,
          expr.name,
          expr.args.map((a) => this.evaluate(a)),
        );
    }
  }

  private logical(pos: Pos, op: string, left: Expr, right: Expr): boolean {
    const a = this.evaluate(left);
    if (typeof a !== "boolean") {
      throw new MiniError(
        pos,
        `type mismatch: '${op}' needs bool operands, got ${typeName(a)}`,
      );
    }
    if ((op === "and" && !a) || (op === "or" && a)) {
      return a;
    }
    const b = this.evaluate(right);
    if (typeof b !== "boolean") {
      throw new MiniError(
        pos,
        `type mismatch: '${op}' needs bool operands, got ${typeName(b)}`,
      );
    }
    return b;
  }

  private binary(pos: Pos, op: string, a: Value, b: Value): Value {
    if (op === "==") {
      return a === b;
    }
    if (op === "!=") {
      return a !== b;
    }
    if (op === "+" && typeof a === "string" && typeof b === "string") {
      return a + b;
    }
    if (typeof a !== "bigint" || typeof b !== "bigint") {
      throw new MiniError(
        pos,
        `type mismatch: cannot apply '${op}' to ${typeName(a)} and ${typeName(b)}`,
      );
    }
    switch (op) {
      case "+":
        return checked(pos, a + b);
      case "-":
        return checked(pos, a - b);
      case "*":
        return checked(pos, a * b);
      case "/":
      case "%":
        if (b === 0n) {
          throw new MiniError(pos, "division by zero");
        }
        return op === "/" ? checked(pos, a / b) : a % b;
      case "<":
        return a < b;
      case "<=":
        return a <= b;
      case ">":
        return a > b;
      case ">=":
        return a >= b;
    }
    throw new Error(`unknown operator ${op}`);
  }

  private call(pos: Pos, name: string, args: Value[]): Value {
    if (name === "print" || name === "str") {
      if (args.length !== 1) {
        throw new MiniError(
          pos,
          `wrong number of arguments: '${name}' takes 1 but got ${args.length}`,
        );
      }
      if (name === "str") {
        return show(args[0]);
      }
      console.log(show(args[0]));
      return null;
    }
    const fn = this.functions.get(name);
    if (fn === undefined) {
      throw new MiniError(pos, `undefined function '${name}'`);
    }
    if (args.length !== fn.params.length) {
      throw new MiniError(
        pos,
        `wrong number of arguments: '${name}' takes ${fn.params.length} but got ${args.length}`,
      );
    }
    if (this.depth === MAX_DEPTH) {
      throw new MiniError(pos, "stack overflow");
    }
    const savedBase = this.env.base;
    this.env.base = this.env.scopes.length;
    this.env.scopes.push(
      new Map(fn.params.map((p, i): [string, Value] => [p, args[i]])),
    );
    this.depth++;
    try {
      for (const stmt of fn.body) {
        this.execute(stmt);
      }
      return null;
    } catch (e) {
      if (e instanceof ReturnSignal) {
        return e.value;
      }
      throw e;
    } finally {
      this.depth--;
      this.env.scopes.splice(this.env.base);
      this.env.base = savedBase;
    }
  }
}

const SCRIPTS: [string, string][] = [
  [
    "fibonacci",
    `// The first sixteen Fibonacci numbers, on one line
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
`,
  ],
  [
    "factorial",
    `fn fact(n) {
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
`,
  ],
  [
    "fizzbuzz",
    `let n = 1
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
`,
  ],
  [
    "strings",
    `fn repeat(s, times) {
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
`,
  ],
  [
    "even and odd",
    `fn is_even(n) {
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
`,
  ],
  [
    "scopes",
    `let x = "global"
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
`,
  ],
  [
    "bad character",
    `let a = 1
let b = a # 2
`,
  ],
  [
    "unterminated string",
    `print("ok")
print("hello)
`,
  ],
  [
    "missing paren",
    `fn add(a, b {
    return a + b
}
`,
  ],
  [
    "undefined variable",
    `fn peek() {
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
`,
  ],
  [
    "type mismatch",
    `let total = 5
print("total: " + str(total))
let flag = true
print(total + flag)
`,
  ],
  [
    "division by zero",
    `fn average(sum, count) {
    return sum / count
}
print(average(10, 2))
print(average(7, 0))
`,
  ],
  [
    "undefined function",
    `fn square(n) { return n * n }
print(square(4))
print(sqaure(5))
`,
  ],
  [
    "wrong arguments",
    `fn add(a, b) { return a + b }
print(add(1, 2))
print(add(1, 2, 3))
`,
  ],
  [
    "stack overflow",
    `fn down(n) {
    return down(n + 1)
}
print("going down")
down(0)
`,
  ],
  [
    "bad condition",
    `let n = 3
while n {
    n = n - 1
}
`,
  ],
];

function runScript(source: string): void {
  try {
    const tokens = new Lexer(source).tokenize();
    const program = new Parser(tokens).program();
    new Interpreter().run(program);
  } catch (e) {
    if (!(e instanceof MiniError)) {
      throw e;
    }
    console.log(`error at ${e.pos.line}:${e.pos.col}: ${e.message}`);
  }
}

function main(): void {
  for (const [name, source] of SCRIPTS) {
    console.log(`== ${name} ==`);
    runScript(source);
  }
}

main();
