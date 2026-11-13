#!/usr/bin/env python3
"""Check docs/spec/grammar.ebnf (gram.ebnf.1) against the .ail files of the tree.

A measurement-style tool like tokbench (ADR 0070): it lexes each .ail file the way
compiler/syntax/lex.ail does (INDENT, DEDENT and NEWLINE included), then recognises the
token stream with the grammar. A file that compiles must be accepted; a file that the
grammar rejects is reported with the line and the token where the match got furthest.

    python3 tools/check-grammar.py                 every .ail file under the default roots
    python3 tools/check-grammar.py FILE_OR_DIR...  only these
    python3 tools/check-grammar.py --self-test     the tool's own small cases
    python3 tools/check-grammar.py --verbose       list skipped files and counts per root

Files that are meant to be rejected (a sibling NAME.err, a NAME.rules with no NAME.out, anything under tests/mistakes,
tests/fix, the fuzz inputs and the tokbench syntax variants) are skipped: the grammar describes the programs that compile.
Exit status 0 when every checked file is accepted, 1 otherwise.
"""
import os
import re
import sys
import threading

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
GRAMMAR = os.path.join(ROOT, "docs", "spec", "grammar.ebnf")
LEXER = os.path.join(ROOT, "compiler", "syntax", "lex.ail")
DEFAULT_ROOTS = ["compiler", "std", "examples", "tests/cases", "tests/native", "tools", "corpus"]
SKIP_DIRS = ["tests/mistakes", "tests/fix", "tests/fuzz", "tests/cases/packages/syntax",
             "tests/cases/packages/task_fix", "tools/tokbench/corpus"]
SKIP_NAMES = {".git", "node_modules", "target"}

UNIT = 4

# ---------------------------------------------------------------------------------------
# Lexer: the token stream the parser sees (spec 02-lexical.md)
# ---------------------------------------------------------------------------------------


def read_keywords():
    text = open(LEXER, encoding="utf-8").read()
    m = re.search(r"KEYWORDS := \{(.*?)\n\}", text, re.S)
    words = set(re.findall(r'"([a-z?]+)"', m.group(1)))
    return words


KEYWORDS = read_keywords() | {"uhwhatnow?"}
TWO = {"->", "=>", ":=", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "&&", "||",
       "<<", ">>", "&=", "|=", "^="}
ONE = set(".:,()[]{}=<>+-*/%!?@|&^~")
BINARY_END = {"+", "-", "*", "/", "%", "==", "!=", "<", "<=", ">", ">=", "..", "..=", "&", "^",
              "<<", ">>"}


class LexError(Exception):
    pass


def is_name_start(c):
    return c == "_" or c.isalpha()


def is_name_char(c):
    return c == "_" or c.isalnum()


def word_at(s, i, w):
    return s.startswith(w, i) and (i + len(w) >= len(s) or not is_name_char(s[i + len(w)]))


def leading_operator(s, i):
    if word_at(s, i, "and") or word_at(s, i, "or"):
        return True
    two = s[i + 1] if i + 1 < len(s) else ""
    c = s[i]
    if c in "=!":
        return two == "="
    if c in "<>":
        return True
    if c in "+*/%":
        return two != "="
    return False


def skip_string(s, i):
    """i at an opening quote of a one-line string; the index after its closing quote."""
    j = i + 1
    while j < len(s):
        c = s[j]
        if c == "\\":
            j += 2
        elif c == "{":
            j = skip_braces(s, j)
        elif c == '"':
            return j + 1
        elif c == "\n":
            raise LexError("unterminated string")
        else:
            j += 1
    raise LexError("unterminated string")


def skip_braces(s, i):
    """i at `{` of an interpolation; the index after its matching `}`."""
    depth = 0
    j = i
    while j < len(s):
        c = s[j]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return j + 1
        elif c == '"':
            j = skip_string(s, j)
            continue
        elif c == "'":
            j = char_end(s, j)
            continue
        elif c == "\n":
            raise LexError("newline in interpolation")
        j += 1
    raise LexError("unclosed interpolation")


def char_end(s, i):
    j = i + 1
    if j < len(s) and s[j] == "\\":
        if s[j + 1:j + 3] == "u{":
            k = s.index("}", j)
            j = k + 1
        else:
            j += 2
    else:
        j += 1
    if j < len(s) and s[j] == "'":
        return j + 1
    raise LexError("bad character literal")


def triple_end(s, i):
    """i at the opening `\"\"\"`; the index after the closing one (the last three of a run)."""
    j = s.find('"""', i + 3)
    if j < 0:
        raise LexError("unterminated multi-line string")
    j += 3
    while j < len(s) and s[j] == '"':
        j += 1
    return j


def number_end(s, i, prev_dot):
    n = len(s)
    j = i
    if prev_dot:  # type.tuple.3: after a single `.`, digits are an element index
        while j < n and s[j].isdigit():
            j += 1
        return j, "int"
    if s[i] == "0" and i + 1 < n and s[i + 1] in "xbo":
        j = i + 2
        while j < n and s[j].isascii() and (s[j].isalnum() or s[j] == "_"):
            j += 1
        return j, "int"
    k = i
    while k < n and (s[k].isdigit() or s[k] == "_"):
        k += 1
    is_float = False
    if k + 1 < n and s[k] == "." and s[k + 1].isdigit():
        k += 1
        while k < n and (s[k].isdigit() or s[k] == "_"):
            k += 1
        is_float = True
    if k < n and s[k] in "eE":
        m = k + 1
        if m < n and s[m] in "+-":
            m += 1
        if m < n and s[m].isdigit():
            while m < n and (s[m].isdigit() or s[m] == "_"):
                m += 1
            return m, "float"
    if is_float:
        return k, "float"
    while k < n and s[k].isascii() and (s[k].isalnum() or s[k] == "_"):
        k += 1
    return k, "int"


def lex(src):
    """Tokens as (kind, text, line). Kinds: kw name int float str char bytes sym NEWLINE INDENT DEDENT EOF."""
    toks = []
    n = len(src)
    indents = [0]
    opens = []
    continuing = False
    line_no = 1
    i = 0
    while i < n:
        end = src.find("\n", i)
        if end < 0:
            end = n
        line_start = i
        lead = i
        while lead < end and src[lead] in " \t":
            lead += 1
        if lead == end or src[lead] == "#" or (src[lead] == "\r" and lead + 1 == end):
            i = end + 1
            line_no += 1
            continue
        if "\t" in src[line_start:lead]:
            raise LexError("tab in indentation")
        k = len(toks)
        dot_name = (src[lead] == "." and lead + 1 < end
                    and (src[lead + 1].isalpha() or src[lead + 1] == "_"))
        if (not opens and not continuing and (dot_name or leading_operator(src, lead))
                and k >= 2 and toks[-1][0] == "NEWLINE"
                and toks[-2][0] not in ("INDENT", "DEDENT")):
            toks.pop()
            continuing = True
        if not opens and not continuing:
            width = lead - line_start
            if width > indents[-1]:
                if width != indents[-1] + UNIT:
                    raise LexError("indentation increases by %d" % (width - indents[-1]))
                indents.append(width)
                toks.append(("INDENT", "", line_no))
            else:
                while width < indents[-1]:
                    indents.pop()
                    toks.append(("DEDENT", "", line_no))
                if width != indents[-1]:
                    raise LexError("indentation matches no block")
        before = len(toks)
        p = lead
        while True:
            c = src[p] if p < n else "\n"
            if c == "\n":
                break
            if c in " \t\r":
                p += 1
            elif c == "#":
                while p < n and src[p] != "\n":
                    p += 1
            elif c == '"':
                if src.startswith('"""', p):
                    e = triple_end(src, p)
                else:
                    e = skip_string(src, p)
                toks.append(("str", "", line_no + src.count("\n", line_start, p)))
                p = e
            elif c == "r" and src.startswith('r"', p):
                if src.startswith('r"""', p):
                    e = triple_end(src, p + 1)
                else:
                    e = src.index('"', p + 2) + 1
                toks.append(("str", "", line_no))
                p = e
            elif c == "b" and src.startswith('b"', p):
                j = p + 2
                while src[j] != '"':
                    j += 2 if src[j] == "\\" else 1
                toks.append(("bytes", "", line_no))
                p = j + 1
            elif c == "'":
                p = char_end(src, p)
                toks.append(("char", "", line_no))
            elif c.isdigit():
                prev_dot = p > 0 and src[p - 1] == "." and (p < 2 or src[p - 2] != ".")
                e, kind = number_end(src, p, prev_dot)
                toks.append((kind, src[p:e], line_no))
                p = e
            elif is_name_start(c):
                e = p
                while e < n and is_name_char(src[e]):
                    e += 1
                word = src[p:e]
                if word == "uhwhatnow" and e < n and src[e] == "?":
                    e += 1
                    toks.append(("kw", "uhwhatnow?", line_no))
                elif word in KEYWORDS:
                    toks.append(("kw", word, line_no))
                else:
                    toks.append(("name", word, line_no))
                p = e
            else:
                sym = None
                if src.startswith("..=", p):
                    sym = "..="
                elif src.startswith("..", p):
                    sym = ".."
                elif src[p:p + 3] in ("<<=", ">>="):
                    sym = src[p:p + 3]
                elif src[p:p + 2] in TWO:
                    sym = src[p:p + 2]
                elif c in ONE:
                    sym = c
                if sym is None:
                    raise LexError("the character %r isn't part of AIL" % c)
                if sym in "([{" and len(sym) == 1:
                    opens.append(sym)
                elif sym in (")", "]", "}"):
                    if not opens:
                        raise LexError("unmatched %s" % sym)
                    opens.pop()
                toks.append(("sym", sym, line_no))
                p += len(sym)
        produced = len(toks) > before
        end = p  # the newline that ends the (possibly multi-line) logical line
        line_no += src.count("\n", line_start, end)
        if not opens:
            continues = False
            if toks:
                t = toks[-1]
                if t[0] == "sym":
                    continues = t[1] == "," or t[1] in BINARY_END
                elif t[0] == "kw":
                    continues = t[1] in ("and", "or")
            if produced and continues:
                continuing = True
            else:
                if produced or continuing:
                    toks.append(("NEWLINE", "", line_no))
                continuing = False
        i = end + 1
        line_no += 1
    if opens:
        raise LexError("unclosed bracket")
    while len(indents) > 1:
        indents.pop()
        toks.append(("DEDENT", "", line_no))
    toks.append(("EOF", "", line_no))
    return toks


# ---------------------------------------------------------------------------------------
# Grammar: a small EBNF reader and a recogniser that tracks every way a rule can match
# ---------------------------------------------------------------------------------------

TOKEN_CLASSES = {"NAME", "KEYWORD", "INT", "FLOAT", "CHAR", "BYTES", "STRING", "NEWLINE", "INDENT",
                 "DEDENT", "EOF", "AFTER_BLOCK", "ANY"}


def tokenize_grammar(text):
    text = re.sub(r"\(\*.*?\*\)", " ", text, flags=re.S)
    out = []
    pos = 0
    pat = re.compile(r"\s*(?:('(?:[^'\\]|\\.)*')|([A-Za-z_][A-Za-z_0-9]*)|(=|;|\||\{|\}|\[|\]|\(|\)))")
    while True:
        m = pat.match(text, pos)
        if not m:
            if text[pos:].strip():
                raise SystemExit("grammar.ebnf: cannot read near %r" % text[pos:pos + 30])
            break
        pos = m.end()
        if m.group(1):
            out.append(("term", m.group(1)[1:-1].replace("\\'", "'")))
        elif m.group(2):
            out.append(("id", m.group(2)))
        else:
            out.append(("op", m.group(3)))
    return out


def parse_grammar(text):
    toks = tokenize_grammar(text)
    rules = {}
    i = 0

    def peek():
        return toks[i] if i < len(toks) else ("eof", "")

    def alt():
        nonlocal i
        alts = [seq()]
        while peek() == ("op", "|"):
            i += 1
            alts.append(seq())
        return alts[0] if len(alts) == 1 else ("alt", alts)

    def seq():
        nonlocal i
        items = []
        while True:
            t = peek()
            if t[0] == "term":
                i += 1
                items.append(("term", t[1]))
            elif t[0] == "id":
                if i + 1 < len(toks) and toks[i + 1] == ("op", "="):
                    break
                i += 1
                items.append(("ref", t[1]))
            elif t == ("op", "{"):
                i += 1
                e = alt()
                expect("}")
                items.append(("rep", e))
            elif t == ("op", "["):
                i += 1
                e = alt()
                expect("]")
                items.append(("opt", e))
            elif t == ("op", "("):
                i += 1
                e = alt()
                expect(")")
                items.append(e)
            else:
                break
        if not items:
            raise SystemExit("grammar.ebnf: empty alternative near token %d" % i)
        return items[0] if len(items) == 1 else ("seq", items)

    def expect(op):
        nonlocal i
        if peek() != ("op", op):
            raise SystemExit("grammar.ebnf: expected %s, found %r" % (op, peek()))
        i += 1

    while i < len(toks):
        t = peek()
        if t[0] != "id":
            raise SystemExit("grammar.ebnf: a rule name is expected, found %r" % (t,))
        name = t[1]
        i += 1
        expect("=")
        rules[name] = alt()
        expect(";")
    for name, body in rules.items():
        check_refs(name, body, rules)
    return rules


def check_refs(name, node, rules):
    k = node[0]
    if k == "ref":
        if node[1] not in rules and node[1] not in TOKEN_CLASSES:
            raise SystemExit("grammar.ebnf: rule %s uses undefined %s" % (name, node[1]))
    elif k in ("seq", "alt"):
        for c in node[1]:
            check_refs(name, c, rules)
    elif k in ("rep", "opt"):
        check_refs(name, node[1], rules)


class Matcher:
    def __init__(self, rules, toks):
        self.rules = rules
        self.toks = toks
        self.memo = {}
        self.far = 0

    def term(self, text, pos):
        kind, val, _ = self.toks[pos]
        if text in KEYWORDS:
            return kind == "kw" and val == text
        if re.fullmatch(r"[A-Za-z_][A-Za-z_0-9]*", text):
            return kind == "name" and val == text
        return kind == "sym" and val == text

    def klass(self, name, pos):
        kind, val, _ = self.toks[pos]
        if name == "NAME":
            return kind == "name"
        if name == "KEYWORD":
            return kind == "kw"
        if name == "STRING":
            return kind == "str"
        if name in ("INT", "FLOAT", "CHAR", "BYTES"):
            return kind == name.lower()
        if name == "ANY":
            return kind in ("kw", "name", "int", "float", "str", "char", "bytes") or (
                kind == "sym" and val not in ("(", ")"))
        return kind == name

    def match(self, node, pos):
        """The set of positions where `node` can end, starting at `pos`."""
        k = node[0]
        if k == "term":
            if pos < len(self.toks):
                if pos > self.far:
                    self.far = pos
                if self.term(node[1], pos):
                    return {pos + 1}
            return set()
        if k == "ref":
            name = node[1]
            if name in TOKEN_CLASSES:
                if name == "AFTER_BLOCK":
                    return {pos} if pos > 0 and self.toks[pos - 1][0] == "DEDENT" else set()
                if pos < len(self.toks):
                    if pos > self.far:
                        self.far = pos
                    if self.klass(name, pos):
                        return {pos + 1}
                return set()
            key = (name, pos)
            got = self.memo.get(key)
            if got is None:
                self.memo[key] = set()  # a left-recursive use finds nothing, not a loop
                got = self.match(self.rules[name], pos)
                self.memo[key] = got
            return got
        if k == "seq":
            cur = {pos}
            for item in node[1]:
                nxt = set()
                for p in cur:
                    nxt |= self.match(item, p)
                cur = nxt
                if not cur:
                    return cur
            return cur
        if k == "alt":
            out = set()
            for item in node[1]:
                out |= self.match(item, pos)
            return out
        if k == "opt":
            return {pos} | self.match(node[1], pos)
        if k == "rep":
            seen = {pos}
            frontier = {pos}
            while frontier:
                nxt = set()
                for p in frontier:
                    nxt |= self.match(node[1], p)
                frontier = nxt - seen
                seen |= frontier
            return seen
        raise AssertionError(k)


def check_tokens(rules, toks):
    """None when the grammar accepts the tokens, else (line, text) where it got furthest."""
    m = Matcher(rules, toks)
    ends = m.match(("ref", "Module"), 0)
    if len(toks) in ends or (len(toks) - 1) in ends:
        return None
    k = min(m.far, len(toks) - 1)
    kind, val, line = toks[k]
    return line, val or kind


# ---------------------------------------------------------------------------------------
# Driver
# ---------------------------------------------------------------------------------------


def wanted(path, rel):
    stem = path[:-4]
    if os.path.exists(stem + ".err"):
        return False
    # tests/cases: a case with a `.rules` file and no `.out` is expected to be rejected
    if os.path.exists(stem + ".rules") and not os.path.exists(stem + ".out"):
        return False
    return not any(rel == d or rel.startswith(d + "/") for d in SKIP_DIRS)


def collect(targets):
    files = []
    for t in targets:
        if os.path.isfile(t):
            files.append(t)
            continue
        for dirpath, dirnames, names in os.walk(t):
            dirnames[:] = sorted(d for d in dirnames if d not in SKIP_NAMES)
            for name in sorted(names):
                if name.endswith(".ail"):
                    files.append(os.path.join(dirpath, name))
    return files


def run(targets, verbose):
    rules = parse_grammar(open(GRAMMAR, encoding="utf-8").read())
    ok = skipped = 0
    bad = []
    for path in collect(targets):
        rel = os.path.relpath(os.path.abspath(path), ROOT).replace(os.sep, "/")
        if not wanted(path, rel):
            skipped += 1
            if verbose:
                print("skip", rel)
            continue
        try:
            toks = lex(open(path, encoding="utf-8").read())
        except (LexError, UnicodeDecodeError, ValueError, IndexError) as e:
            bad.append("%s: lexer: %s" % (rel, e))
            continue
        r = check_tokens(rules, toks)
        if r is None:
            ok += 1
        else:
            bad.append("%s:%d: the grammar stops at `%s`" % (rel, r[0], r[1]))
    for b in bad:
        print(b)
    print("check-grammar: %d accepted, %d rejected, %d skipped" % (ok, len(bad), skipped))
    return 1 if bad else 0


SELF_OK = [
    "fn main()\n    print(1)\n",
    "fn add(a int, b int) int\n    a + b\n",
    "struct P\n    x int\n    y int = 0\n\nfn main()\n    p := P(x: 1)\n    print(p.x)\n",
    "enum E\n    A\n    B(n int)\n\nfn f(e E) int\n    match e\n        A => 1\n        B(n) => n\n",
    "fn main()\n    xs := [x * 2 for x in 0..3 if x > 0]\n    for i in 0..3\n        print(i)\n",
    "fn main()\n    v := f() else\n        return\n    print(v)\n",
    "type UserId = int\n",
]
SELF_BAD = [
    "fn main(\n    print(1)\n",
    "fn main()\n    x = = 2\n",
    "struct\n    x int\n",
]


def self_test():
    rules = parse_grammar(open(GRAMMAR, encoding="utf-8").read())
    fails = 0
    for src in SELF_OK:
        if check_tokens(rules, lex(src)) is not None:
            print("self-test: should accept:\n" + src)
            fails += 1
    for src in SELF_BAD:
        try:
            r = check_tokens(rules, lex(src))
        except LexError:
            continue
        if r is None:
            print("self-test: should reject:\n" + src)
            fails += 1
    print("check-grammar --self-test: %s" % ("ok" if not fails else "%d failed" % fails))
    return 1 if fails else 0


def main():
    args = sys.argv[1:]
    verbose = "--verbose" in args
    args = [a for a in args if a != "--verbose"]
    sys.setrecursionlimit(1000000)
    # Windows refuses a 512 MB thread stack (ValueError): take the largest size the host allows.
    for mb in (512, 256, 64):
        try:
            threading.stack_size(mb * 1024 * 1024)
            break
        except (ValueError, RuntimeError):
            continue
    result = []

    def go():
        try:
            go_inner()
        except SystemExit as e:
            print(e)
            result.append(1)

    def go_inner():
        if args == ["--self-test"]:
            result.append(self_test())
        else:
            targets = args or [os.path.join(ROOT, d) for d in DEFAULT_ROOTS]
            result.append(run([t for t in targets if os.path.exists(t)], verbose))

    t = threading.Thread(target=go)
    t.start()
    t.join()
    sys.exit(result[0] if result else 1)


if __name__ == "__main__":
    main()
