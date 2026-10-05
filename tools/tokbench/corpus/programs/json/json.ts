import { isDeepStrictEqual } from "node:util";

const DIGITS = "0123456789";
const WHITESPACE = " \t\n\r";
const ESCAPES = new Map([
  ['"', '"'],
  ["\\", "\\"],
  ["/", "/"],
  ["b", "\b"],
  ["f", "\f"],
  ["n", "\n"],
  ["r", "\r"],
  ["t", "\t"],
]);
const KINDS = [
  "object",
  "array",
  "string",
  "number",
  "boolean",
  "null",
] as const;

type Kind = (typeof KINDS)[number];
type Scalar = null | boolean | bigint | number | string;
type Json = Scalar | Json[] | Map<string, Json>;
type Step = string | number;

class ParseError extends Error {
  constructor(line: number, col: number, msg: string) {
    super(`parse error at ${line}:${col}: ${msg}`);
  }
}

class Parser {
  private pos = 0;

  constructor(private readonly text: string) {}

  error(msg: string, at = this.pos): ParseError {
    let line = 1;
    let col = 1;
    for (const c of this.text.slice(0, at)) {
      if (c === "\n") {
        line += 1;
        col = 1;
      } else {
        col += 1;
      }
    }
    return new ParseError(line, col, msg);
  }

  atEnd(): boolean {
    return this.pos >= this.text.length;
  }

  peek(): string {
    return this.text.charAt(this.pos);
  }

  unexpected(): ParseError {
    if (this.atEnd()) return this.error("unexpected end of input");
    return this.error(`unexpected character '${this.peek()}'`);
  }

  skipWhitespace(): void {
    while (!this.atEnd() && WHITESPACE.includes(this.peek())) this.pos += 1;
  }

  parseDocument(): Json {
    const value = this.parseValue();
    this.skipWhitespace();
    if (!this.atEnd()) throw this.error("extra data after value");
    return value;
  }

  parseValue(): Json {
    this.skipWhitespace();
    const c = this.peek();
    if (c === "{") return this.parseObject();
    if (c === "[") return this.parseArray();
    if (c === '"') return this.parseString();
    if (c === "-" || (c !== "" && DIGITS.includes(c)))
      return this.parseNumber();
    for (const [word, value] of [
      ["true", true],
      ["false", false],
      ["null", null],
    ] as const) {
      if (this.text.startsWith(word, this.pos)) {
        this.pos += word.length;
        return value;
      }
    }
    throw this.unexpected();
  }

  parseObject(): Json {
    this.pos += 1;
    const members = new Map<string, Json>();
    this.skipWhitespace();
    if (this.peek() === "}") {
      this.pos += 1;
      return members;
    }
    while (true) {
      this.skipWhitespace();
      if (this.peek() !== '"') throw this.unexpected();
      const key = this.parseString();
      this.skipWhitespace();
      if (this.peek() !== ":") {
        if (this.atEnd()) throw this.unexpected();
        throw this.error("expected ':' after object key");
      }
      this.pos += 1;
      members.set(key, this.parseValue());
      if (this.endOfContainer("}")) return members;
    }
  }

  parseArray(): Json {
    this.pos += 1;
    const items: Json[] = [];
    this.skipWhitespace();
    if (this.peek() === "]") {
      this.pos += 1;
      return items;
    }
    while (true) {
      items.push(this.parseValue());
      if (this.endOfContainer("]")) return items;
    }
  }

  endOfContainer(close: string): boolean {
    this.skipWhitespace();
    const c = this.peek();
    if (c === close) {
      this.pos += 1;
      return true;
    }
    if (c !== ",") throw this.unexpected();
    const comma = this.pos;
    this.pos += 1;
    this.skipWhitespace();
    if (this.peek() === "]" || this.peek() === "}") {
      throw this.error("trailing comma", comma);
    }
    return false;
  }

  parseString(): string {
    const start = this.pos;
    this.pos += 1;
    const chars: string[] = [];
    while (true) {
      if (this.atEnd() || this.peek() === "\n") {
        throw this.error("unterminated string", start);
      }
      const c = this.peek();
      if (c === '"') {
        this.pos += 1;
        return chars.join("");
      }
      if (c === "\\") {
        chars.push(this.parseEscape(start));
      } else if (c.charCodeAt(0) < 0x20) {
        throw this.error("control character in string");
      } else {
        chars.push(c);
        this.pos += 1;
      }
    }
  }

  parseEscape(start: number): string {
    const escape = this.pos;
    this.pos += 1;
    if (this.atEnd()) throw this.error("unterminated string", start);
    const c = this.peek();
    this.pos += 1;
    const simple = ESCAPES.get(c);
    if (simple !== undefined) return simple;
    if (c !== "u") throw this.error(`invalid escape '\\${c}'`, escape);
    const digits = this.text.slice(this.pos, this.pos + 4);
    if (!/^[0-9a-f]{4}$/i.test(digits)) {
      throw this.error("invalid unicode escape", escape);
    }
    const code = parseInt(digits, 16);
    if (code >= 0xd800 && code <= 0xdfff) {
      throw this.error("surrogate escapes are not supported", escape);
    }
    this.pos += 4;
    return String.fromCharCode(code);
  }

  skipDigits(): number {
    const start = this.pos;
    while (!this.atEnd() && DIGITS.includes(this.peek())) this.pos += 1;
    return this.pos - start;
  }

  parseNumber(): bigint | number {
    const start = this.pos;
    if (this.peek() === "-") this.pos += 1;
    if (this.peek() === "0") {
      this.pos += 1;
    } else if (this.skipDigits() === 0) {
      throw this.error("invalid number");
    }
    let isFloat = false;
    if (this.peek() === ".") {
      this.pos += 1;
      if (this.skipDigits() === 0) throw this.error("invalid number");
      isFloat = true;
    }
    if (this.peek() === "e" || this.peek() === "E") {
      this.pos += 1;
      if (this.peek() === "+" || this.peek() === "-") this.pos += 1;
      if (this.skipDigits() === 0) throw this.error("invalid number");
      isFloat = true;
    }
    const text = this.text.slice(start, this.pos);
    return isFloat ? Number(text) : BigInt(text);
  }
}

function parse(text: string): Json {
  return new Parser(text).parseDocument();
}

const QUOTED = new Map([
  ['"', '\\"'],
  ["\\", "\\\\"],
  ["\n", "\\n"],
  ["\r", "\\r"],
  ["\t", "\\t"],
  ["\b", "\\b"],
  ["\f", "\\f"],
]);

function quote(s: string): string {
  const out = ['"'];
  for (const c of s) {
    const code = c.charCodeAt(0);
    const escaped = QUOTED.get(c);
    if (escaped !== undefined) {
      out.push(escaped);
    } else if (code < 0x20) {
      out.push(`\\u${code.toString(16).padStart(4, "0")}`);
    } else {
      out.push(c);
    }
  }
  out.push('"');
  return out.join("");
}

function scalar(value: Scalar): string {
  if (typeof value === "string") return quote(value);
  if (typeof value === "number" && Number.isInteger(value)) {
    return value.toFixed(1);
  }
  return String(value);
}

function compact(value: Json): string {
  if (value instanceof Map) {
    const members = [...value].map(([k, v]) => `${quote(k)}:${compact(v)}`);
    return `{${members.join(",")}}`;
  }
  if (Array.isArray(value))
    return `[${value.map((v) => compact(v)).join(",")}]`;
  return scalar(value);
}

function pretty(value: Json, level = 0): string {
  const pad = "  ".repeat(level + 1);
  const close = "  ".repeat(level);
  if (value instanceof Map) {
    if (value.size === 0) return "{}";
    const members = [...value].map(
      ([k, v]) => `${pad}${quote(k)}: ${pretty(v, level + 1)}`,
    );
    return `{\n${members.join(",\n")}\n${close}}`;
  }
  if (Array.isArray(value)) {
    if (value.length === 0) return "[]";
    const items = value.map((v) => pad + pretty(v, level + 1));
    return `[\n${items.join(",\n")}\n${close}]`;
  }
  return scalar(value);
}

const INDEX = /^-?[0-9]+$/;

function parsePath(path: string): Step[] | undefined {
  const steps: Step[] = [];
  for (const part of path.split(".")) {
    const [name, ...indexes] = part.split("[");
    if (name !== "") steps.push(name);
    for (const piece of indexes) {
      const inner = piece.split("]");
      if (inner.length !== 2 || inner[1] !== "" || !INDEX.test(inner[0])) {
        return undefined;
      }
      steps.push(Number(inner[0]));
    }
  }
  return steps;
}

function query(doc: Json, steps: Step[]): Json | undefined {
  let current: Json | undefined = doc;
  for (const step of steps) {
    if (typeof step === "number") {
      current = Array.isArray(current) ? current[step] : undefined;
    } else {
      current = current instanceof Map ? current.get(step) : undefined;
    }
    if (current === undefined) return undefined;
  }
  return current;
}

interface Stats {
  counts: Map<Kind, number>;
  maxDepth: number;
  total: number;
}

function kindOf(value: Json): Kind {
  if (value instanceof Map) return "object";
  if (Array.isArray(value)) return "array";
  if (typeof value === "string") return "string";
  if (typeof value === "boolean") return "boolean";
  if (value === null) return "null";
  return "number";
}

function walk(value: Json, depth: number, stats: Stats): void {
  const kind = kindOf(value);
  stats.counts.set(kind, (stats.counts.get(kind) ?? 0) + 1);
  if (typeof value === "number" || typeof value === "bigint") {
    stats.total += Number(value);
  }
  if (value instanceof Map || Array.isArray(value)) {
    stats.maxDepth = Math.max(stats.maxDepth, depth);
    const children = value instanceof Map ? value.values() : value;
    for (const child of children) walk(child, depth + 1, stats);
  }
}

const DOCUMENT = String.raw`
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
`;

const QUERIES = [
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

const BAD_INPUTS = [
  '{"name": "Ann", "age": 30,}',
  '{"name" "Ann"}',
  '[\n  "a",\n  "b\n]',
  String.raw`{"path": "C:\qdir"}`,
  "[1, 2] [3]",
  "[1, @]",
  '{"a": {"b": [1, 2',
  "[-]",
  '{\n  "x": 1.,\n}',
  String.raw`"bad \u12G4"`,
  "",
];

function main(): void {
  const doc = parse(DOCUMENT);
  console.log("== pretty ==");
  console.log(pretty(doc));

  console.log("== compact ==");
  const text = compact(doc);
  console.log(text);
  console.log(
    "round trip:",
    isDeepStrictEqual(parse(text), doc) ? "ok" : "MISMATCH",
  );

  console.log("== queries ==");
  for (const path of QUERIES) {
    const steps = parsePath(path);
    if (steps === undefined) {
      console.log(`bad path: ${path}`);
      continue;
    }
    const result = query(doc, steps);
    if (result === undefined) {
      console.log(`not found: ${path}`);
    } else {
      console.log(`${path} = ${compact(result)}`);
    }
  }

  console.log("== stats ==");
  const stats: Stats = { counts: new Map(), maxDepth: 0, total: 0 };
  walk(doc, 1, stats);
  for (const kind of KINDS) {
    console.log(`${kind}: ${stats.counts.get(kind) ?? 0}`);
  }
  console.log(`max depth: ${stats.maxDepth}`);
  console.log(`sum of numbers: ${scalar(stats.total)}`);

  console.log("== errors ==");
  BAD_INPUTS.forEach((text, i) => {
    try {
      const value = parse(text);
      console.log(`${i + 1}: ok ${compact(value)}`);
    } catch (err) {
      if (!(err instanceof ParseError)) throw err;
      console.log(`${i + 1}: ${err.message}`);
    }
  });
}

main();
