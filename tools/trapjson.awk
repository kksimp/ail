# `ail run --json FILE` (trace.snap.4): turns the text trap report that a `--values --verbose`
# build prints (trace.snap.3, trace.frame.3) into the interpreter's one-line JSON object.
# Lines before the trap line pass through unchanged. Usage: awk -v main=FILE -f trapjson.awk

function esc(s,    i, c, o) {
    o = ""
    for (i = 1; i <= length(s); i++) {
        c = substr(s, i, 1)
        if (c == "\\") o = o "\\\\"
        else if (c == "\"") o = o "\\\""
        else if (c == "\t") o = o "\\t"
        else if (c == "\r") o = o "\\r"
        else o = o c
    }
    return o
}

# Splits "a = 1, b = [2, 3]" at the commas outside brackets and quotes that start a `name = `.
function pairs(s,    i, n, c, depth, q, cut, out, first, p, rest) {
    if (s == "") return "{}"
    out = ""; first = 1; depth = 0; q = 0; cut = 1; n = length(s)
    for (i = 1; i <= n + 1; i++) {
        c = substr(s, i, 1)
        if (i <= n) {
            if (q) {
                if (c == "\\") { i++; continue }
                if (c == "\"") q = 0
                continue
            }
            if (c == "\"") { q = 1; continue }
            if (c == "(" || c == "[" || c == "{") { depth++; continue }
            if (c == ")" || c == "]" || c == "}") { depth--; continue }
            if (!(c == "," && depth == 0 && substr(s, i + 1, 1) == " ")) continue
            rest = substr(s, i + 2)
            if (!match(rest, /^[A-Za-z_][A-Za-z0-9_]* = /)) continue
        }
        p = substr(s, cut, i - cut)
        cut = i + 2
        out = out (first ? "" : ",") one(p)
        first = 0
    }
    return "{" out "}"
}

function one(p,    k) {
    k = index(p, " = ")
    if (k == 0) return "\"" esc(p) "\":\"\""
    return "\"" esc(substr(p, 1, k - 1)) "\":\"" esc(substr(p, k + 3)) "\""
}

# "FILE:L:C" into file, line, col.
function place(s) {
    match(s, /:[0-9]+$/); pc = substr(s, RSTART + 1) + 0; s = substr(s, 1, RSTART - 1)
    match(s, /:[0-9]+$/); pl = substr(s, RSTART + 1) + 0; pf = substr(s, 1, RSTART - 1)
}

function flush_frame() {
    if (nf > 0) frames[nf] = frames[nf] ",\"locals\":" pairs(locs) "}"
}

BEGIN { head = 0; nf = 0 }

head == 0 {
    i = index($0, ": trap[")
    if (i > 0 && match(substr($0, 1, i - 1), /:[0-9]+:[0-9]+$/)) {
        place(substr($0, 1, i - 1))
        file = pf; line = pl; col = pc
        rest = substr($0, i + 7)
        j = index(rest, "]: ")
        rule = substr(rest, 1, j - 1); msg = substr(rest, j + 3)
        head = 1; src = ""; have_src = 0; cond = ""; have_cond = 0
        next
    }
    print
    next
}

/^  [0-9]+ \| / { if (!have_src) { src = substr($0, index($0, "| ") + 2); have_src = 1 }; next }
/^ +\| *\^/ { next }
/^  values: / { cond = substr($0, 11); have_cond = 1; next }
/^  at / {
    flush_frame()
    s = substr($0, 6)
    k = index(s, " in ")
    place(substr(s, 1, k - 1))
    f = pf
    if (f == "") f = main
    name = substr(s, k + 4)
    a = ""
    par = index(name, "(")
    if (par > 0) {
        a = substr(name, par + 1)
        if (substr(a, length(a), 1) == ")") a = substr(a, 1, length(a) - 1)
        name = substr(name, 1, par - 1)
    }
    nf++; locs = ""
    frames[nf] = "{\"fn\":\"" esc(name) "\",\"file\":\"" esc(f) "\",\"line\":" pl ",\"col\":" pc ",\"args\":" pairs(a)
    next
}
/^      [A-Za-z_]/ { locs = locs (locs == "" ? "" : ", ") substr($0, 7); next }
/^  … [0-9]+ more frames/ { match($0, /[0-9]+/); omitted = substr($0, RSTART, RLENGTH) + 0; next }

END {
    if (head == 0) exit
    flush_frame()
    o = "{\"api\":1,\"severity\":\"trap\",\"rule\":\"" esc(rule) "\",\"file\":\"" esc(file) "\",\"line\":" line ",\"col\":" col ",\"message\":\"" esc(msg) "\""
    if (have_src) o = o ",\"source\":\"" esc(src) "\""
    if (have_cond) o = o ",\"condition\":" pairs(cond)
    o = o ",\"stack\":["
    for (i = 1; i <= nf; i++) o = o (i > 1 ? "," : "") frames[i]
    print o "],\"omitted_frames\":" (omitted + 0) "}"
}
