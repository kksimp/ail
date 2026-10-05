import re
from collections import Counter
from dataclasses import dataclass, field
DIGITS = "0123456789"
HEX_DIGITS = "0123456789abcdefABCDEF"
WHITESPACE = " \t\n\r"
ESCAPES = {
    '"': '"',
    "\\": "\\",
    "/": "/",
    "b": "\b",
    "f": "\f",
    "n": "\n",
    "r": "\r",
    "t": "\t",
}
KINDS = ["object", "array", "string", "number", "boolean", "null"]
class ParseError(Exception):
    def __init__(self, line, col, msg):
        super().__init__(f"parse error at {line}:{col}: {msg}")
class Parser:
    def __init__(self, text):
        self.text = text
        self.pos = 0
    def error(self, msg, at=None):
        if at is None:
            at = self.pos
        line, col = 1, 1
        for c in self.text[:at]:
            if c == "\n":
                line += 1
                col = 1
            else:
                col += 1
        return ParseError(line, col, msg)
    def at_end(self):
        return self.pos >= len(self.text)
    def peek(self):
        return "" if self.at_end() else self.text[self.pos]
    def unexpected(self):
        if self.at_end():
            return self.error("unexpected end of input")
        return self.error(f"unexpected character '{self.peek()}'")
    def skip_whitespace(self):
        while not self.at_end() and self.peek() in WHITESPACE:
            self.pos += 1
    def parse_document(self):
        value = self.parse_value()
        self.skip_whitespace()
        if not self.at_end():
            raise self.error("extra data after value")
        return value
    def parse_value(self):
        self.skip_whitespace()
        c = self.peek()
        if c == "{":
            return self.parse_object()
        if c == "[":
            return self.parse_array()
        if c == '"':
            return self.parse_string()
        if c == "-" or (c != "" and c in DIGITS):
            return self.parse_number()
        for word, value in (("true", True), ("false", False), ("null", None)):
            if self.text.startswith(word, self.pos):
                self.pos += len(word)
                return value
        raise self.unexpected()
    def parse_object(self):
        self.pos += 1
        members = {}
        self.skip_whitespace()
        if self.peek() == "}":
            self.pos += 1
            return members
        while True:
            self.skip_whitespace()
            if self.peek() != '"':
                raise self.unexpected()
            key = self.parse_string()
            self.skip_whitespace()
            if self.peek() != ":":
                if self.at_end():
                    raise self.unexpected()
                raise self.error("expected ':' after object key")
            self.pos += 1
            members[key] = self.parse_value()
            if self.end_of_container("}"):
                return members
    def parse_array(self):
        self.pos += 1
        items = []
        self.skip_whitespace()
        if self.peek() == "]":
            self.pos += 1
            return items
        while True:
            items.append(self.parse_value())
            if self.end_of_container("]"):
                return items
    def end_of_container(self, close):
        self.skip_whitespace()
        c = self.peek()
        if c == close:
            self.pos += 1
            return True
        if c != ",":
            raise self.unexpected()
        comma = self.pos
        self.pos += 1
        self.skip_whitespace()
        if self.peek() in ("]", "}"):
            raise self.error("trailing comma", comma)
        return False
    def parse_string(self):
        start = self.pos
        self.pos += 1
        chars = []
        while True:
            if self.at_end() or self.peek() == "\n":
                raise self.error("unterminated string", start)
            c = self.peek()
            if c == '"':
                self.pos += 1
                return "".join(chars)
            if c == "\\":
                chars.append(self.parse_escape(start))
            elif ord(c) < 0x20:
                raise self.error("control character in string")
            else:
                chars.append(c)
                self.pos += 1
    def parse_escape(self, start):
        escape = self.pos
        self.pos += 1
        if self.at_end():
            raise self.error("unterminated string", start)
        c = self.peek()
        self.pos += 1
        if c in ESCAPES:
            return ESCAPES[c]
        if c != "u":
            raise self.error(f"invalid escape '\\{c}'", escape)
        digits = self.text[self.pos : self.pos + 4]
        if len(digits) < 4 or any(d not in HEX_DIGITS for d in digits):
            raise self.error("invalid unicode escape", escape)
        code = int(digits, 16)
        if 0xD800 <= code <= 0xDFFF:
            raise self.error("surrogate escapes are not supported", escape)
        self.pos += 4
        return chr(code)
    def skip_digits(self):
        start = self.pos
        while not self.at_end() and self.peek() in DIGITS:
            self.pos += 1
        return self.pos - start
    def parse_number(self):
        start = self.pos
        if self.peek() == "-":
            self.pos += 1
        if self.peek() == "0":
            self.pos += 1
        elif self.skip_digits() == 0:
            raise self.error("invalid number")
        is_float = False
        if self.peek() == ".":
            self.pos += 1
            if self.skip_digits() == 0:
                raise self.error("invalid number")
            is_float = True
        if self.peek() in ("e", "E"):
            self.pos += 1
            if self.peek() in ("+", "-"):
                self.pos += 1
            if self.skip_digits() == 0:
                raise self.error("invalid number")
            is_float = True
        text = self.text[start : self.pos]
        return float(text) if is_float else int(text)
def parse(text):
    return Parser(text).parse_document()
def quote(s):
    out = ['"']
    for c in s:
        if c == '"':
            out.append('\\"')
        elif c == "\\":
            out.append("\\\\")
        elif c == "\n":
            out.append("\\n")
        elif c == "\r":
            out.append("\\r")
        elif c == "\t":
            out.append("\\t")
        elif c == "\b":
            out.append("\\b")
        elif c == "\f":
            out.append("\\f")
        elif ord(c) < 0x20:
            out.append(f"\\u{ord(c):04x}")
        else:
            out.append(c)
    out.append('"')
    return "".join(out)
def scalar(value):
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, str):
        return quote(value)
    return repr(value)
def compact(value):
    if isinstance(value, dict):
        members = [f"{quote(k)}:{compact(v)}" for k, v in value.items()]
        return "{" + ",".join(members) + "}"
    if isinstance(value, list):
        return "[" + ",".join(compact(v) for v in value) + "]"
    return scalar(value)
def pretty(value, level=0):
    pad = "  " * (level + 1)
    close = "  " * level
    if isinstance(value, dict):
        if not value:
            return "{}"
        members = [f"{pad}{quote(k)}: {pretty(v, level + 1)}" for k, v in value.items()]
        return "{\n" + ",\n".join(members) + "\n" + close + "}"
    if isinstance(value, list):
        if not value:
            return "[]"
        items = [pad + pretty(v, level + 1) for v in value]
        return "[\n" + ",\n".join(items) + "\n" + close + "]"
    return scalar(value)
MISSING = object()
INDEX = re.compile(r"-?[0-9]+")
def parse_path(path):
    steps = []
    for part in path.split("."):
        pieces = part.split("[")
        if pieces[0] != "":
            steps.append(pieces[0])
        for piece in pieces[1:]:
            inner = piece.split("]")
            if len(inner) != 2 or inner[1] != "" or not INDEX.fullmatch(inner[0]):
                return None
            steps.append(int(inner[0]))
    return steps
def query(doc, steps):
    current = doc
    for step in steps:
        if isinstance(step, int):
            if not isinstance(current, list) or not 0 <= step < len(current):
                return MISSING
            current = current[step]
        else:
            if not isinstance(current, dict) or step not in current:
                return MISSING
            current = current[step]
    return current
@dataclass
class Stats:
    counts: Counter = field(default_factory=Counter)
    max_depth: int = 0
    total: float = 0.0
def kind_of(value):
    if isinstance(value, dict):
        return "object"
    if isinstance(value, list):
        return "array"
    if isinstance(value, str):
        return "string"
    if isinstance(value, bool):
        return "boolean"
    if value is None:
        return "null"
    return "number"
def walk(value, depth, stats):
    kind = kind_of(value)
    stats.counts[kind] += 1
    if kind == "number":
        stats.total += value
    if kind in ("object", "array"):
        stats.max_depth = max(stats.max_depth, depth)
        children = value.values() if kind == "object" else value
        for child in children:
            walk(child, depth + 1, stats)
DOCUMENT = r"""
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
"""
QUERIES = [
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
]
BAD_INPUTS = [
    '{"name": "Ann", "age": 30,}',
    '{"name" "Ann"}',
    '[\n  "a",\n  "b\n]',
    r'{"path": "C:\qdir"}',
    "[1, 2] [3]",
    "[1, @]",
    '{"a": {"b": [1, 2',
    "[-]",
    '{\n  "x": 1.,\n}',
    r'"bad \u12G4"',
    "",
]
def main():
    doc = parse(DOCUMENT)
    print("== pretty ==")
    print(pretty(doc))
    print("== compact ==")
    text = compact(doc)
    print(text)
    print("round trip:", "ok" if parse(text) == doc else "MISMATCH")
    print("== queries ==")
    for path in QUERIES:
        steps = parse_path(path)
        if steps is None:
            print(f"bad path: {path}")
            continue
        result = query(doc, steps)
        if result is MISSING:
            print(f"not found: {path}")
        else:
            print(f"{path} = {compact(result)}")
    print("== stats ==")
    stats = Stats()
    walk(doc, 1, stats)
    for kind in KINDS:
        print(f"{kind}: {stats.counts[kind]}")
    print(f"max depth: {stats.max_depth}")
    print(f"sum of numbers: {stats.total!r}")
    print("== errors ==")
    for i, text in enumerate(BAD_INPUTS, 1):
        try:
            value = parse(text)
            print(f"{i}: ok {compact(value)}")
        except ParseError as err:
            print(f"{i}: {err}")
if __name__ == "__main__":
    main()
