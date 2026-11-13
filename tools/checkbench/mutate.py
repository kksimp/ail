"""checkbench mutators: one typical mistake per copy of a clean program (ADR 1111).

Each mutator is `fn(prog, rng) -> Mutant | None`. It picks one site with the seeded `rng`, so a run repeats.
A Mutant carries the new text and the span of lines (1-based, in the mutated text) where a good first
diagnostic would point. `silent_ok` marks a class the checker cannot catch in general (a same-typed swap, a
missing dedent): a clean check there is listed as a decided silent class, not counted as a miss.
"""

import re
from dataclasses import dataclass, field

KEYWORDS = {
    "fn", "struct", "enum", "trait", "impl", "if", "elif", "else", "match", "for", "in", "while", "loop", "break",
    "continue", "return", "mut", "pub", "use", "and", "or", "not", "true", "false", "none", "error", "par", "any",
    "with", "unique", "type", "self", "requires", "ensures", "invariant", "old", "result", "then", "test", "unsafe",
    "extern", "ref", "spawn", "todo", "yield", "is", "as", "const", "static", "let", "var", "defer",
}
PRELUDE_TYPES = {"int", "float", "bool", "str", "char", "bytes", "List", "Map", "Set", "Task", "Deque", "Heap"}
IDENT = r"[A-Za-z_][A-Za-z0-9_]*"


@dataclass
class Mutant:
    mutator: str
    group: str
    text: str
    lo: int
    hi: int
    what: str
    silent_ok: bool = False


@dataclass
class Prog:
    path: str
    text: str
    lines: list = field(default_factory=list)
    masks: list = field(default_factory=list)  # per line: set of column indexes inside a string or comment
    fns: list = field(default_factory=list)  # (header_idx, end_idx_exclusive, header_text)
    structs: dict = field(default_factory=dict)  # name -> [field names]
    types: set = field(default_factory=set)

    def __post_init__(self):
        self.lines = self.text.split("\n")
        self.masks = [string_mask(l) for l in self.lines]
        self.scan()

    def scan(self):
        n = len(self.lines)
        cur = None
        for i, l in enumerate(self.lines):
            if l and not l[0].isspace() and not l.startswith("#"):
                if cur is not None:
                    self.fns.append((cur, last_code(self.lines, cur, i), self.lines[cur]))
                    cur = None
                m = re.match(r"(?:@test\s+)?(?:pub\s+)?fn\s", l)
                if m:
                    cur = i
                m = re.match(r"(?:pub\s+)?(?:error\s+)?(struct|enum|trait)\s+(" + IDENT + ")", l)
                if m:
                    self.types.add(m.group(2))
                    if m.group(1) == "struct":
                        fs = []
                        for j in range(i + 1, n):
                            fl = self.lines[j]
                            if fl and not fl[0].isspace():
                                break
                            fm = re.match(r"    (" + IDENT + r") \S", fl)
                            if fm and fm.group(1) not in ("invariant",) and not fl.strip().startswith("#"):
                                fs.append(fm.group(1))
                        self.structs[m.group(2)] = fs
        if cur is not None:
            self.fns.append((cur, last_code(self.lines, cur, n), self.lines[cur]))

    def code(self, i):
        """Line i with strings and comments blanked (same length), for matching."""
        l = self.lines[i]
        m = self.masks[i]
        return "".join(" " if k in m else c for k, c in enumerate(l))

    def fn_of(self, i):
        for h, e, t in self.fns:
            if h < i < e:
                return h, e, t
        return None

    def body_lines(self):
        """Indexes of code lines inside a function body."""
        out = []
        for h, e, _ in self.fns:
            for i in range(h + 1, e):
                s = self.lines[i].strip()
                if s and not s.startswith("#"):
                    out.append(i)
        return out


def last_code(lines, start, end):
    """End (exclusive) of a top-level item: trailing blank and comment lines are not part of it."""
    e = end
    while e > start + 1 and (not lines[e - 1].strip() or lines[e - 1].lstrip().startswith("#")):
        e -= 1
    return e


def string_mask(line):
    """Columns inside string or char literals (quotes included) or a comment. Interpolations count as string."""
    mask = set()
    i, n = 0, len(line)
    while i < n:
        c = line[i]
        if c == "#":
            mask.update(range(i, n))
            break
        if c == '"' or (c == "r" and i + 1 < n and line[i + 1] == '"' and (i == 0 or not line[i - 1].isalnum())):
            start = i
            raw = c == "r"
            i += 2 if raw else 1
            while i < n and line[i] != '"':
                if line[i] == "\\" and not raw:
                    i += 1
                i += 1
            mask.update(range(start, min(i + 1, n)))
            i += 1
            continue
        if c == "'":
            j = line.find("'", i + 1)
            if j != -1 and j - i <= 4:
                mask.update(range(i, j + 1))
                i = j + 1
                continue
        i += 1
    return mask


def typo(name, rng, dist=1, avoid=()):
    """A misspelling of `name` at edit distance `dist` (swap, drop or double a letter) that is not in `avoid`."""
    for _ in range(20):
        s = name
        for _ in range(dist):
            k = rng.randrange(3) if len(s) > 2 else 2
            if k == 0 and len(s) >= 3:
                p = rng.randrange(1, len(s) - 1)
                if s[p] == s[p + 1]:
                    continue
                s = s[:p] + s[p + 1] + s[p] + s[p + 2:]
            elif k == 1 and len(s) >= 4:
                p = rng.randrange(1, len(s))
                s = s[:p] + s[p + 1:]
            else:
                p = rng.randrange(1, len(s))
                s = s[:p] + s[p] + s[p:]
        if s != name and s not in avoid and s not in KEYWORDS and re.fullmatch(IDENT, s):
            return s
    return None


def words(text):
    return set(re.findall(IDENT, text))


def finds(prog, pattern, lines=None, group=0):
    """All (line, start, end, match) of `pattern` in code (outside strings and comments)."""
    out = []
    for i in lines if lines is not None else range(len(prog.lines)):
        code = prog.code(i)
        for m in re.finditer(pattern, code):
            out.append((i, m.start(group), m.end(group), m))
    return out


def edit(prog, i, s, e, new):
    lines = list(prog.lines)
    lines[i] = lines[i][:s] + new + lines[i][e:]
    return "\n".join(lines)


def with_lines(prog, i, j, new_lines):
    lines = list(prog.lines)
    lines[i:j] = new_lines
    return "\n".join(lines)


def pick(rng, xs):
    return xs[rng.randrange(len(xs))] if xs else None


def mk(name, group, text, lo, hi, what, silent_ok=False):
    return Mutant(name, group, text, lo + 1, hi + 1, what, silent_ok)


# ---------------------------------------------------------------- names

def locals_bound(prog):
    """(decl line, name) for each `name :=` / `mut name :=` binding in a body."""
    out = []
    for i, s, e, m in finds(prog, r"^\s+(?:mut\s+)?(" + IDENT + r")\s*:=", prog.body_lines(), 1):
        if m.group(1) not in KEYWORDS and m.group(1) != "_":
            out.append((i, m.group(1)))
    return out


def later_uses(prog, i, name, require_read=True):
    f = prog.fn_of(i)
    if not f:
        return []
    out = []
    for j, s, e, m in finds(prog, r"(?<![\w.])" + re.escape(name) + r"(?![\w])", range(i + 1, f[1])):
        rest = prog.code(j)[e:]
        if require_read and re.match(r"\s*(?:[-+*/%]?=(?!=)|:=)", rest):
            continue
        if re.match(r"\s*:(?!=)", rest):  # a named argument `name: v`
            continue
        out.append((j, s, e))
    return out


def m_local(dist):
    def run(prog, rng):
        cands = []
        for i, name in locals_bound(prog):
            if len(name) < 3:
                continue
            for u in later_uses(prog, i, name):
                cands.append((u, name))
        c = pick(rng, cands)
        if not c:
            return None
        (j, s, e), name = c
        bad = typo(name, rng, dist, words(prog.text))
        if not bad:
            return None
        return mk(f"misspell_local{dist}", "names", edit(prog, j, s, e, bad), j, j, f"{name} -> {bad}")
    return run


def m_field(prog, rng):
    fields = {f for fs in prog.structs.values() for f in fs}
    cands = [(i, s, e, m.group(1)) for i, s, e, m in finds(prog, r"\.(" + IDENT + r")\b(?!\s*\()", prog.body_lines(), 1)
             if m.group(1) in fields and len(m.group(1)) >= 3]
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, name = c
    bad = typo(name, rng, 1, words(prog.text))
    return bad and mk("misspell_field", "names", edit(prog, i, s, e, bad), i, i, f".{name} -> .{bad}")


def m_method(prog, rng):
    cands = [(i, s, e, m.group(1)) for i, s, e, m in finds(prog, r"\.(" + IDENT + r")\(", prog.body_lines(), 1)
             if len(m.group(1)) >= 3]
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, name = c
    bad = typo(name, rng, 1, words(prog.text))
    return bad and mk("misspell_method", "names", edit(prog, i, s, e, bad), i, i, f".{name}( -> .{bad}(")


def m_fn(prog, rng):
    names = set()
    for _, _, h in prog.fns:
        m = re.match(r"(?:@test\s+)?(?:pub\s+)?fn\s+(" + IDENT + r")\(", h)
        if m and m.group(1) != "main":
            names.add(m.group(1))
    cands = [(i, s, e, m.group(1)) for i, s, e, m in finds(prog, r"(?<![\w.])(" + IDENT + r")\(", prog.body_lines(), 1)
             if m.group(1) in names and len(m.group(1)) >= 3]
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, name = c
    bad = typo(name, rng, 1, words(prog.text))
    return bad and mk("misspell_fn", "names", edit(prog, i, s, e, bad), i, i, f"{name}( -> {bad}(")


def m_type(prog, rng):
    cands = []
    for i, s, e, m in finds(prog, r"(?<![\w.])(" + IDENT + r")\b", None, 1):
        name = m.group(1)
        if name not in prog.types or len(name) < 3:
            continue
        l = prog.lines[i]
        if re.match(r"(?:pub\s+)?(?:error\s+)?(struct|enum|trait)\s", l):
            continue  # the declaration itself
        if re.match(r"(?:@test\s+)?(?:pub\s+)?fn\s+" + re.escape(name) + r"\.", l) and s < l.index("."):
            continue  # a method's receiver: a typo there is a different class
        cands.append((i, s, e, name))
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, name = c
    bad = typo(name, rng, 1, words(prog.text))
    return bad and mk("misspell_type", "names", edit(prog, i, s, e, bad), i, i, f"{name} -> {bad}")


def m_keyword_local(prog, rng):
    cands = [(i, n) for i, n in locals_bound(prog) if later_uses(prog, i, n, False)]
    c = pick(rng, cands)
    if not c:
        return None
    i, name = c
    kw = pick(rng, ["type", "match", "error", "any", "with", "in"])
    lines = list(prog.lines)
    f = prog.fn_of(i)
    rx = re.compile(r"(?<![\w.])" + re.escape(name) + r"(?![\w])(?!\s*:(?!=))")
    for j in range(i, f[1]):
        code = prog.code(j)
        out, last = [], 0
        for m in rx.finditer(code):
            out.append(lines[j][last:m.start()] + kw)
            last = m.end()
        lines[j] = "".join(out) + lines[j][last:]
    return mk("keyword_local", "names", "\n".join(lines), i, i, f"local {name} renamed {kw}")


# ---------------------------------------------------------------- types

def m_int_for_float(prog, rng):
    cands = finds(prog, r"(?<![\w.])(\d+)\.(\d+)(?![\w.])", prog.body_lines())
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, m = c
    return mk("int_for_float", "types", edit(prog, i, s, e, m.group(1)), i, i, f"{m.group(0)} -> {m.group(1)}")


def m_drop_to_float(prog, rng):
    c = pick(rng, finds(prog, r"\.to_float\(\)", prog.body_lines()))
    if not c:
        return None
    i, s, e, _ = c
    return mk("drop_to_float", "types", edit(prog, i, s, e, ""), i, i, "dropped .to_float()")


ARG = r"(?:\"[^\"]*\"|-?\d+(?:\.\d+)?|" + IDENT + r"(?:\.\w+)*)"


def m_swap_args(prog, rng):
    cands = []
    for i, s, e, m in finds(prog, r"(?<![\w.])(" + IDENT + r")\((" + ARG + r"), (" + ARG + r")\)", prog.body_lines()):
        a, b = m.group(2), m.group(3)
        # different kinds only: a text literal against a number or a name
        ka = "s" if a.startswith('"') else "n" if re.match(r"-?\d", a) else "v"
        kb = "s" if b.startswith('"') else "n" if re.match(r"-?\d", b) else "v"
        if ka != kb and "s" in (ka, kb):
            cands.append((i, m.start(2), m.end(3), a, b))
    # the code mask blanks strings; take the arguments from the real line
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, _, _ = c
    real = prog.lines[i][s:e]
    mm = re.fullmatch(r"(" + ARG + r"), (" + ARG + r")", real)
    if not mm:
        return None
    return mk("swap_args", "types", edit(prog, i, s, e, f"{mm.group(2)}, {mm.group(1)}"), i, i, f"swapped {real}")


def fn_ret(header):
    m = re.search(r"\)\s*([^#!]*?)\s*(?:!.*)?$", header.split("#")[0].rstrip())
    return m.group(1).strip() if m else ""


def m_return_wrong(prog, rng):
    cands = []
    for h, e, t in prog.fns:
        ret = fn_ret(t)
        if ret not in ("int", "float", "bool"):
            continue
        for i in range(h + 1, e):
            m = re.match(r"(\s+return )(.+)$", prog.lines[i])
            if m and "#" not in m.group(2):
                cands.append((i, m))
    c = pick(rng, cands)
    if not c:
        return None
    i, m = c
    return mk("return_wrong_type", "types", with_lines(prog, i, i + 1, [m.group(1) + '"none"']), i, i, "return a str")


def m_opt_as_value(prog, rng):
    """`x := f() else FALLBACK` with the fallback dropped: a `T?` (or `T ! E`) used as a `T`."""
    cands = [(i, m) for i, s, e, m in finds(prog, r"^(\s+(?:mut\s+)?" + IDENT + r"\s*:=\s*.+?)\s+else\s+(?!return\b|continue\b|break\b)(\S.*)$", prog.body_lines())]
    c = pick(rng, cands)
    if not c:
        return None
    i, m = c
    end = m.end(1)
    return mk("opt_as_value", "types", with_lines(prog, i, i + 1, [prog.lines[i][:end]]), i, i + 3, "dropped `else fallback`")


# ---------------------------------------------------------------- errors

def m_drop_q(prog, rng):
    cands = [c for c in finds(prog, r"\)(\?)(?![.\w])", prog.body_lines(), 1)
             if re.match(r"\s+(?:mut\s+)?" + IDENT + r"\s*:=", prog.lines[c[0]])]
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, _ = c
    return mk("drop_q", "errors", edit(prog, i, s, e, ""), i, i + 3, "dropped `?` in a binding")


def m_ignore_result(prog, rng):
    cands = [c for c in finds(prog, r"\)(\?)\s*$", prog.body_lines(), 1)
             if re.match(r"\s+" + IDENT + r"[\w.]*\(", prog.lines[c[0]])]
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, _ = c
    return mk("ignore_result", "errors", edit(prog, i, s, e, ""), i, i, "dropped `?` on a statement")


def m_drop_else(prog, rng):
    cands = finds(prog, r"\s+else\s+(?:return|continue|break)\b.*$", prog.body_lines())
    cands = [c for c in cands if ":=" in prog.code(c[0])[:c[1]]]
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, _ = c
    return mk("drop_else", "errors", edit(prog, i, s, len(prog.lines[i]), ""), i, i + 3, "dropped `else return/continue`")


def m_add_q(prog, rng):
    cands = []
    for h, e, t in prog.fns:
        if "!" in t.split("#")[0] or t.startswith("@test"):
            continue
        for i, s, ee, m in finds(prog, r"^\s+(?:mut\s+)?" + IDENT + r"\s*:=\s*[\w.]+\([^()]*\)()\s*$", range(h + 1, e), 1):
            cands.append((i, s))
    c = pick(rng, cands)
    if not c:
        return None
    i, s = c
    return mk("add_q", "errors", edit(prog, i, s, s, "?"), i, i, "`?` in a function that cannot fail")


# ---------------------------------------------------------------- structure

def indent(l):
    return len(l) - len(l.lstrip(" "))


def block_end(prog, i):
    """End (exclusive) of the block headed by line i: the lines indented deeper than it."""
    ind = indent(prog.lines[i])
    j = i + 1
    while j < len(prog.lines) and (not prog.lines[j].strip() or indent(prog.lines[j]) > ind):
        j += 1
    while j > i + 1 and not prog.lines[j - 1].strip():
        j -= 1
    return j


def m_delete_arm(prog, rng):
    cands = []
    for i, s, e, m in finds(prog, r"^\s+(?:.*[:=]\s*)?match\s.+$", prog.body_lines()):
        ind = indent(prog.lines[i]) + 4
        end = block_end(prog, i)
        arms = [j for j in range(i + 1, end) if indent(prog.lines[j]) == ind and "=>" in prog.code(j)]
        if len(arms) < 2 or any(re.match(r"\s+_\s*(?:if\b.*)?=>", prog.lines[j]) for j in arms):
            continue
        if any(re.match(r"\s+" + IDENT + r"\s*=>", prog.lines[j]) for j in arms):
            continue  # a binding arm catches all
        for j in arms:
            cands.append((i, j, block_end(prog, j)))
    c = pick(rng, cands)
    if not c:
        return None
    i, j, k = c
    return mk("delete_arm", "structure", with_lines(prog, j, k, []), i, i, f"deleted arm `{prog.lines[j].strip()[:30]}`")


def ctor_sites(prog):
    out = []
    for name, fs in prog.structs.items():
        if not fs:
            continue
        for i, s, e, m in finds(prog, r"(?<![\w.])" + name + r"\(([^()]*)\)", prog.body_lines()):
            real = prog.lines[i][m.start(1):m.end(1)]
            parts = [p.strip() for p in real.split(",")] if real.strip() else []
            if parts and all(re.match(IDENT + r":", p) for p in parts) and '"' not in real:
                out.append((i, m.start(1), m.end(1), parts, name))
    return out


def m_delete_field(prog, rng):
    c = pick(rng, [x for x in ctor_sites(prog) if len(x[3]) >= 2])
    if not c:
        return None
    i, s, e, parts, name = c
    k = rng.randrange(len(parts))
    left = parts[:k] + parts[k + 1:]
    return mk("delete_field", "structure", edit(prog, i, s, e, ", ".join(left)), i, i, f"{name}: dropped `{parts[k]}`")


def m_unknown_field(prog, rng):
    c = pick(rng, ctor_sites(prog))
    if not c:
        return None
    i, s, e, parts, name = c
    return mk("unknown_field", "structure", edit(prog, i, s, e, ", ".join(parts + ["colour: 0"])), i, i, f"{name}: added colour")


def m_arg_count(prog, rng):
    arity = {}
    for _, _, h in prog.fns:
        m = re.match(r"(?:@test\s+)?(?:pub\s+)?fn\s+(" + IDENT + r")\(([^)]*)\)", h)
        if m and m.group(2).strip():
            arity[m.group(1)] = m.group(2).count(",") + 1
    cands = []
    for i, s, e, m in finds(prog, r"(?<![\w.])(" + IDENT + r")\(([^()]*)\)", prog.body_lines()):
        n = arity.get(m.group(1))
        real = prog.lines[i][m.start(2):m.end(2)]
        if n and real.strip() and real.count(",") + 1 == n and '"' not in real:
            cands.append((i, m.start(2), m.end(2), real, m.group(1)))
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, real, name = c
    parts = real.split(",")
    return mk("arg_count", "structure", edit(prog, i, s, e, ",".join(parts[:-1]).strip()), i, i, f"{name}: dropped last argument")


def m_remove_mut(prog, rng):
    cands = []
    for i, s, e, m in finds(prog, r"^(\s+)(mut )(" + IDENT + r")\s*:=", prog.body_lines()):
        name = m.group(3)
        f = prog.fn_of(i)
        for j in range(i + 1, f[1]):
            if re.search(r"(?<![\w.])" + name + r"(?:\s*(?:[-+*/%]?=)(?!=)|\.(?:push|insert|remove|pop|sort|clear|append)\w*\()", prog.code(j)):
                cands.append((i, m.start(2), m.end(2), j, name))
                break
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, j, name = c
    return mk("remove_mut", "structure", edit(prog, i, s, e, ""), i, j, f"removed `mut` from {name}")


def m_shadow(prog, rng):
    cands = []
    for i, s, e, m in finds(prog, r"^\s+(" + IDENT + r")( = )", prog.body_lines()):
        name = m.group(1)
        f = prog.fn_of(i)
        if f and any(re.match(r"\s+mut " + name + r"\s*:=", prog.lines[k]) for k in range(f[0], i)):
            cands.append((i, m.start(2), m.end(2), name))
    c = pick(rng, cands)
    if not c:
        return None
    i, s, e, name = c
    return mk("shadow", "structure", edit(prog, i, s, e, " := "), i, i, f"`{name} =` -> `{name} :=`")


def m_delete_tail(prog, rng):
    cands = []
    for h, e, t in prog.fns:
        ret = fn_ret(t)
        if not ret or t.startswith("@test"):
            continue
        last = e - 1
        if indent(prog.lines[last]) == 4 and not re.match(r"\s+(return|if|for|while|match|elif|else)\b", prog.lines[last]) \
                and not prog.lines[last].rstrip().endswith((",", "(", "[", "{")):
            if last - 1 > h and indent(prog.lines[last - 1]) == 4 and not prog.lines[last - 1].rstrip().endswith((",", "(", "[")):
                cands.append((h, last))
    c = pick(rng, cands)
    if not c:
        return None
    h, last = c
    return mk("delete_tail", "structure", with_lines(prog, last, last + 1, []), h, last, "deleted the result line")


# ---------------------------------------------------------------- layout

def m_indent2(prog, rng):
    cands = [i for i in prog.body_lines() if indent(prog.lines[i]) >= 8]
    i = pick(rng, cands)
    if i is None:
        return None
    return mk("indent2", "layout", with_lines(prog, i, i + 1, [prog.lines[i][2:]]), i - 1, i, "2 spaces less indent")


def m_tab(prog, rng):
    i = pick(rng, prog.body_lines())
    if i is None:
        return None
    return mk("tab", "layout", with_lines(prog, i, i + 1, ["\t" + prog.lines[i][4:]]), i, i, "a tab for 4 spaces")


def m_missing_dedent(prog, rng):
    cands = []
    for i in prog.body_lines():
        prev = i - 1
        while prev >= 0 and not prog.lines[prev].strip():
            prev -= 1
        if indent(prog.lines[prev]) > indent(prog.lines[i]) >= 4 and not re.match(r"\s+(elif|else)\b", prog.lines[i]) \
                and "=>" not in prog.code(i) and not prog.lines[i].strip().startswith(("]", ")", "}")):
            cands.append(i)
    i = pick(rng, cands)
    if i is None:
        return None
    return mk("missing_dedent", "layout", with_lines(prog, i, i + 1, ["    " + prog.lines[i]]), i, i, "line not dedented", True)


def m_unclosed(ch, name):
    def run(prog, rng):
        cands = []
        for i in prog.body_lines():
            l = prog.lines[i]
            if ch == '"':
                k = l.rfind('"')
                if k > 0 and (k - 1) in prog.masks[i] and l.count('"') % 2 == 0 and "#" not in l:
                    cands.append((i, k))
            else:
                code = prog.code(i)
                k = code.rfind(ch)
                if k != -1 and code.count(ch) == code.count({")": "(", "]": "["}[ch]):
                    cands.append((i, k))
        c = pick(rng, cands)
        if not c:
            return None
        i, k = c
        return mk(name, "layout", edit(prog, i, k, k + 1, ""), i, i + 1, f"removed `{ch}`")
    return run


# ---------------------------------------------------------------- strings (corpus-lane reds, 2026-11-13)

def m_bare_brace(prog, rng):
    """A C, Go or JSON snippet in a string: a bare `{` where `\\{` was meant."""
    cands = []
    for i in prog.body_lines():
        m = re.match(r'^(\s+print\(")([^"{}\\]*)("\)\s*)$', prog.lines[i])
        if m:
            cands.append((i, m))
    c = pick(rng, cands)
    if not c:
        return None
    i, m = c
    new = m.group(1) + m.group(2) + ' { "ok": 1 }' + m.group(3)
    return mk("bare_brace", "strings", with_lines(prog, i, i + 1, [new]), i, i, "JSON `{` in a string")


def m_brace_return(prog, rng):
    cands = []
    for i in prog.body_lines():
        m = re.match(r'^(\s+print\(")([^"{}\\]*)("\)\s*)$', prog.lines[i])
        if m:
            cands.append((i, m))
    c = pick(rng, cands)
    if not c:
        return None
    i, m = c
    new = m.group(1) + m.group(2) + " fn f() { return 1 }" + m.group(3)
    return mk("brace_return", "strings", with_lines(prog, i, i + 1, [new]), i, i, "C code in a string")


# ---------------------------------------------------------------- habits (foreign table)

def sub_once(name, pattern, repl, lines_of=None, span_extra=0, cond=None):
    def run(prog, rng):
        rows = prog.body_lines() if lines_of is None else lines_of(prog)
        cands = [c for c in finds(prog, pattern, rows) if cond is None or cond(prog, c)]
        c = pick(rng, cands)
        if not c:
            return None
        i, s, e, m = c
        real = prog.lines[i][s:e]
        new = re.sub(pattern, repl, real, count=1) if not callable(repl) else repl(real)
        if new == real:
            return None
        return mk(name, "habits", edit(prog, i, s, e, new), i, i + span_extra, f"`{real.strip()[:30]}` -> `{new.strip()[:30]}`")
    run.__name__ = name
    return run


def header_rows(prog):
    return [h for h, _, t in prog.fns if not t.startswith("@test")]


def let_cond(prog, c):
    return prog.lines[c[0]][:c[1]].strip() == ""


HABITS = [
    sub_once("h_else_if", r"^(\s+)elif\b", r"\1else if"),
    sub_once("h_and", r" and ", " && "),
    sub_once("h_or", r" or ", " || "),
    sub_once("h_not", r"(?<![\w.])not (?=[\w(])", "!"),
    sub_once("h_incr", r"(" + IDENT + r") \+= 1\s*$", r"\1++"),
    sub_once("h_len", r"(?<![\w.])(" + IDENT + r")\.len\(\)", r"len(\1)"),
    sub_once("h_let", r"(?<!\S)(" + IDENT + r") := ", r"let \1 = ", cond=let_cond),
    sub_once("h_let_mut", r"(?<!\S)mut (" + IDENT + r") := ", r"let mut \1 = ", cond=let_cond),
    sub_once("h_var", r"(?<!\S)mut (" + IDENT + r") := ", r"var \1 = ", cond=let_cond),
    sub_once("h_nil", r"(?<![\w.])none\b", "nil"),
    sub_once("h_None", r"(?<![\w.])none\b", "None"),
    sub_once("h_True", r"(?<![\w.])true\b", "True"),
    sub_once("h_as", r"(?<![\w.])(" + IDENT + r")\.to_float\(\)", r"\1 as float"),
    sub_once("h_append", r"\.push\(", ".append("),
    sub_once("h_range", r"\bfor (" + IDENT + r") in 0\.\.(" + IDENT + r")\s*$", r"for \1 in range(\2)"),
    sub_once("h_arrow", r"\)\s+(" + IDENT + r"(?:\[[^\]]*\])?\??)\s*$", r") -> \1", header_rows),
    sub_once("h_colon_param", r"\((" + IDENT + r") (int|str|float|bool)\b", r"(\1: \2", header_rows),
    sub_once("h_def", r"^fn ", "def ", header_rows),
    sub_once("h_println", r"(?<![\w.])print\(", "println("),
    sub_once("h_console", r"(?<![\w.])print\(", "console.log("),
    sub_once("h_this", r"(?<![\w.])self\.", "this."),
    sub_once("h_semicolon", r"\)\s*$", ");"),
    sub_once("h_colon_if", r"^(\s+(?:if|while|for) .+?)\s*$", r"\1:"),
    sub_once("h_brace_if", r"^(\s+(?:if|while) .+?)\s*$", r"\1 {"),
    sub_once("h_switch", r"^(\s+)match\b", r"\1switch"),
    # batch 8 review probes (2026-11-13)
    sub_once("h_lifetime", r"^((?:pub\s+)?fn\s+" + IDENT + r")\(", r"\1<'a>(", header_rows),
    sub_once("h_usize", r"(\(|, )(" + IDENT + r") int\b", r"\1\2 usize", header_rows),
    sub_once("h_named_eq", r"\((" + IDENT + r"): ", r"(\1 = "),
    sub_once("h_iter_map", r"\.(map|filter)\(", r".iter().\1("),
    sub_once("h_generic_angle", r"\b(List|Map|Set)\[([^\[\]]+)\]", r"\1<\2>"),
]


def h_derive(prog, rng):
    """`#[derive(Debug)]` above a struct: Rust's attribute, a comment in AIL (silent unless warned)."""
    cands = [i for i, l in enumerate(prog.lines) if re.match(r"(?:pub\s+)?struct\s", l)]
    i = pick(rng, cands)
    if i is None:
        return None
    return mk("h_derive", "habits", with_lines(prog, i, i, ["#[derive(Debug, Clone)]"]), i, i, "`#[derive]` above a struct")


def h_format(prog, rng):
    """`"text {x}"` written `format!("text {}", x)`."""
    cands = []
    for i in prog.body_lines():
        l = prog.lines[i]
        for m in re.finditer(r'"([^"{}\\]*)\{(' + IDENT + r')\}([^"{}\\]*)"', l):
            if m.start() in prog.masks[i]:
                cands.append((i, m))
    c = pick(rng, cands)
    if not c:
        return None
    i, m = c
    new = f'format!("{m.group(1)}{{}}{m.group(3)}", {m.group(2)})'
    return mk("h_format", "habits", edit(prog, i, m.start(), m.end(), new), i, i, "format! macro")


def h_dollar(prog, rng):
    """`"{x}"` written `"${x}"` (JavaScript template, shell)."""
    cands = []
    for i in prog.body_lines():
        l = prog.lines[i]
        for m in re.finditer(r"(?<![\\$])\{(" + IDENT + r")\}", l):
            if m.start() in prog.masks[i]:
                cands.append((i, m.start()))
    c = pick(rng, cands)
    if not c:
        return None
    i, s = c
    return mk("h_dollar", "habits", edit(prog, i, s, s, "$"), i, i, "`{x}` -> `${x}`")


def h_plus_str(prog, rng):
    """`"text {x}"` written `"text " + x`."""
    cands = []
    for i in prog.body_lines():
        l = prog.lines[i]
        for m in re.finditer(r'"([^"{}\\]+)\{(' + IDENT + r')\}"', l):
            if m.start() in prog.masks[i]:
                cands.append((i, m))
    c = pick(rng, cands)
    if not c:
        return None
    i, m = c
    new = f'"{m.group(1)}" + {m.group(2)}'
    return mk("h_plus_str", "habits", edit(prog, i, m.start(), m.end(), new), i, i, "string + name")


def h_slash_comment(prog, rng):
    cands = [i for i, l in enumerate(prog.lines) if re.match(r"\s*# ", l)]
    i = pick(rng, cands)
    if i is None:
        return None
    return mk("h_slash_comment", "habits", with_lines(prog, i, i + 1, [prog.lines[i].replace("#", "//", 1)]), i, i, "`#` -> `//`")



MUTATORS = {
    "misspell_local1": m_local(1),
    "misspell_local2": m_local(2),
    "misspell_field": m_field,
    "misspell_method": m_method,
    "misspell_fn": m_fn,
    "misspell_type": m_type,
    "keyword_local": m_keyword_local,
    "int_for_float": m_int_for_float,
    "drop_to_float": m_drop_to_float,
    "swap_args": m_swap_args,
    "return_wrong_type": m_return_wrong,
    "opt_as_value": m_opt_as_value,
    "drop_q": m_drop_q,
    "ignore_result": m_ignore_result,
    "drop_else": m_drop_else,
    "add_q": m_add_q,
    "delete_arm": m_delete_arm,
    "delete_field": m_delete_field,
    "unknown_field": m_unknown_field,
    "arg_count": m_arg_count,
    "remove_mut": m_remove_mut,
    "shadow": m_shadow,
    "delete_tail": m_delete_tail,
    "indent2": m_indent2,
    "tab": m_tab,
    "missing_dedent": m_missing_dedent,
    "unclosed_paren": m_unclosed(")", "unclosed_paren"),
    "unclosed_bracket": m_unclosed("]", "unclosed_bracket"),
    "unclosed_quote": m_unclosed('"', "unclosed_quote"),
    "bare_brace": m_bare_brace,
    "brace_return": m_brace_return,
    "h_dollar": h_dollar,
    "h_plus_str": h_plus_str,
    "h_slash_comment": h_slash_comment,
    "h_derive": h_derive,
    "h_format": h_format,
}
for _h in HABITS:
    MUTATORS[_h.__name__] = _h
