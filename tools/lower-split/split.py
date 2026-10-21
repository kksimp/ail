#!/usr/bin/env python3
"""Split compiler/mir/lower.ail (ADR 0823), half 1: the dispatchers.

Run from the repository root on any batch head; it is regenerated at the
split's queue slot rather than rebased. Pure moves:

1. Every multi-line arm of `lower_expr`'s `match e` becomes a function
   named after the arm, and the arm calls it.
2. Every multi-line arm of `lower_method`'s `match r.ty` likewise; the
   method's closing `unsupported(...)` is appended to each extracted
   function, so falling out of the arm behaves as before.
3. The operators and number helpers move to lower_ops.ail.

Then `ail fmt` formats the touched files. Behaviour is unchanged: check with
check.sh (MIR dumps and perfbench executables, before and after).
"""
import re
import sys
from pathlib import Path

MIR = Path("compiler/mir")
LOWER = MIR / "lower.ail"

# Arm of lower_expr -> (function, file, doc).
EXPR_ARMS = {
    "Int": ("lower_int_lit", "lower.ail", "An integer literal, as a `float` where one is wanted."),
    "Name": ("lower_name", "lower.ail", "A name: a local, a constant, a field of `self`, a function, a global or a bare variant."),
    "Binary": ("lower_binary_expr", "lower_ops.ail", "A binary operation (`and` and `or` short-circuit)."),
    "Call": ("lower_call_expr", "lower.ail", "A call `f(...)`, or a variant built by calling it."),
    "Index": ("lower_index", "lower_list.ail", "`xs[i]`, `xs[a..b]`, and `m[k]` on a map."),
    "Member": ("lower_member_expr", "lower_struct.ail", "`x.f`, `E.V`, or a qualified constant."),
    "OrElse": ("lower_or_else", "lower_err.ail", "`e else ...` (err.else.1-6)."),
}
# Arm of lower_method's `match r.ty` -> (function, file, doc).
METHOD_ARMS = {
    "Int": ("int_method", "lower_ops.ail", "A method of an `int`: `to_float`, `abs`, `min`, `max`, the bit operations."),
    "Float": ("float_method", "lower_ops.ail", "A method of a `float` (type.num.9, type.num.16)."),
    "Str": ("str_method", "lower_str.ail", "A method of a `str`."),
    "List": ("list_method", "lower_list.ail", "A method of a list: `len`, `join`, `push` and the edits."),
}
# Top-level items moved whole, by name, to lower_ops.ail.
OPS_ITEMS = [
    "BIT_OPS", "MATH1", "MATH2", "wants_float", "is_int_lit", "operand_want",
    "Lower.lower_unary", "int_text", "op_name", "op_symbol", "is_comparison",
    "Lower.binary", "Lower.short_circuit", "Lower.int_abs", "Lower.int_minmax",
]
OPS_HEAD = (
    "## Operators and number methods in the native subset: unary and binary\n"
    "## operators, short-circuit `and`/`or`, and the methods of `int` and `float`\n"
    "## (split from lower.ail, ADR 0823).\nuse ast\n"
)

# Context variables of each dispatcher, with their types.
EXPR_CTX = {"e": "ast.Expr", "want": "Ty?"}
METHOD_CTX = {
    "r": "Val", "name": "str", "npos": "ast.Pos", "vs": "List[Val]",
    "args": "List[ast.Arg]", "recv": "ast.Expr",
}
BUILTIN = {"str", "int", "bool", "float", "List", "Map", "Set", "Ty", "Val"}


def ast_variants():
    text = (Path("compiler/ast/ast.ail")).read_text()
    m = re.search(r"^pub enum Expr\n((?:    .*\n|\s*\n)+)", text, re.M)
    out = {}
    for line in m.group(1).splitlines():
        v = re.match(r"^    (\w+)\((.*)\)\s*$", line)
        if v:
            fields = []
            for f in split_top(v.group(2)):
                n, t = f.strip().split(" ", 1)
                fields.append((n, qualify(t)))
            out[v.group(1)] = fields
    return out


def split_top(s):
    parts, depth, cur = [], 0, ""
    for ch in s:
        if ch in "[(":
            depth += 1
        elif ch in "])":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append(cur)
            cur = ""
        else:
            cur += ch
    if cur.strip():
        parts.append(cur)
    return parts


def qualify(t):
    return re.sub(r"\b([A-Z]\w*)\b", lambda m: m.group(1) if m.group(1) in BUILTIN else "ast." + m.group(1), t)


def item_bounds(lines, start):
    """The end (exclusive) of the top-level item starting at `start`."""
    i = start + 1
    while i < len(lines) and (lines[i] == "" or lines[i][0] in " \t"):
        i += 1
    # Trailing blank lines belong between items.
    while i > start + 1 and lines[i - 1] == "":
        i -= 1
    return i


def doc_start(lines, start):
    while start > 0 and lines[start - 1].startswith("##"):
        start -= 1
    return start


def find_item(lines, name):
    pat = re.compile(r"^(pub )?(fn %s\(|%s :=|struct %s$)" % (re.escape(name), re.escape(name), re.escape(name)))
    for i, l in enumerate(lines):
        if pat.match(l):
            return i
    return None


def take_items(lines, names):
    taken = []
    for n in names:
        i = find_item(lines, n)
        if i is None:
            sys.exit(f"split.py: no top-level item {n} in lower.ail")
        a, b = doc_start(lines, i), item_bounds(lines, i)
        taken.append(lines[a:b])
        del lines[a:b]
        while a < len(lines) and a > 0 and lines[a] == "" and lines[a - 1] == "":
            del lines[a]
    return taken


def snake(v):
    return re.sub(r"(?<!^)([A-Z])", r"_\1", v).lower()


def extract_arms(lines, fn_prefix, match_line, ctx, arms, variants, fallback, kind):
    """Extract multi-line arms of the `match` at `match_line` inside the
    function starting with `fn_prefix`; returns {file: [new item lines]}."""
    start = next(i for i, l in enumerate(lines) if l.startswith(fn_prefix))
    end = item_bounds(lines, start)
    m = next(i for i in range(start, end) if lines[i] == match_line)
    ind = len(match_line) - len(match_line.lstrip()) + 4
    out = {}
    i = m + 1
    while i < end:
        line = lines[i]
        arm = re.match(r"^ {%d}(\w+)(\((.*)\))? =>\s*$" % ind, line)
        if not arm:
            if line.strip() and len(line) - len(line.lstrip()) < ind:
                break
            i += 1
            continue
        j = i + 1
        while j < end and (lines[j] == "" or len(lines[j]) - len(lines[j].lstrip()) > ind):
            j += 1
        while lines[j - 1] == "":
            j -= 1
        variant, pat = arm.group(1), arm.group(3)
        body = [l[ind:] if l else "" for l in lines[i + 1:j]]
        fname, ffile, doc = arms.get(variant, (f"lower_{snake(variant)}_{kind}", "lower.ail", f"The `{variant}` arm of `lower_{kind}`."))
        params, call_args = [], []
        if pat is not None:
            names = [p.strip() for p in split_top(pat)]
            fields = variants.get(variant) if kind == "expr" else None
            for k, n in enumerate(names):
                if n == "_":
                    continue
                if fields is not None:
                    t = fields[k][1]
                elif kind == "method" and variant in ("List", "Set", "Opt") and k == 0:
                    t = "Ty"
                else:
                    sys.exit(f"split.py: don't know the type of `{n}` in arm {variant}")
                params.append(f"{n} {t}")
                call_args.append(n)
        bound = {p.split(" ")[0] for p in params}
        text = "\n".join(body)
        for c, t in ctx.items():
            if c not in bound and re.search(r"\b%s\b" % re.escape(c), text):
                params.append(f"{c} {t}")
                call_args.append(c)
        fn = [f"## {doc}", f"fn Lower.{fname}(mut self, {', '.join(params)}) Val?"]
        fn += body
        if fallback and not re.match(r"^    return ", body[-1]):
            fn.append("    " + fallback)
        out.setdefault(ffile, []).append(fn)
        call = f"{fname}({', '.join(call_args)})"
        head = f"{' ' * ind}{variant}{arm.group(2) or ''} => "
        lines[i:j] = [head + ("return " + call if kind == "method" else call)]
        end -= (j - i) - 1
        i += 1
    return out


def main():
    if not LOWER.exists():
        sys.exit("split.py: run from the repository root")
    variants = ast_variants()
    lines = LOWER.read_text().split("\n")
    new = {}
    for f, items in extract_arms(lines, "fn Lower.lower_expr(", "    match e", EXPR_CTX, EXPR_ARMS, variants, None, "expr").items():
        new.setdefault(f, []).extend(items)
    fallback = 'unsupported(npos, "`.{name}()` on `{r.ty.name()}`")'
    for f, items in extract_arms(lines, "fn Lower.lower_method(", "    match r.ty", METHOD_CTX, METHOD_ARMS, variants, fallback, "method").items():
        new.setdefault(f, []).extend(items)
    ops = take_items(lines, OPS_ITEMS)
    # Functions that stay in lower.ail go right after lower_expr.
    stay = new.pop("lower.ail", [])
    at = item_bounds(lines, next(i for i, l in enumerate(lines) if l.startswith("fn Lower.lower_expr(")))
    block = []
    for fn in stay:
        block += [""] + fn
    lines[at:at] = block
    LOWER.write_text("\n".join(lines))
    ops_path = MIR / "lower_ops.ail"
    if ops_path.exists():
        sys.exit("split.py: lower_ops.ail exists already")
    chunks = [OPS_HEAD.rstrip("\n")]
    for item in ops + new.pop("lower_ops.ail", []):
        chunks.append("\n".join(item))
    ops_path.write_text("\n\n".join(chunks) + "\n")
    for f, fns in new.items():
        p = MIR / f
        text = p.read_text().rstrip("\n")
        for fn in fns:
            text += "\n\n" + "\n".join(fn)
        p.write_text(text + "\n")
    print("split.py: moved", sum(len(x) for x in ops), "lines of operators; extracted arms into",
          ", ".join(sorted(set(["lower.ail", "lower_ops.ail"] + list(new)))))


main()
