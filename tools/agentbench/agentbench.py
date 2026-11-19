#!/usr/bin/env python3
"""Agent end-to-end benchmark harness (ADR 0916): one model, one task, AIL arm vs C arm.

  agentbench.py gen TASK                       # fill cases/{visible,hidden}.jsonl from the task's ref.py
  agentbench.py docs TASK                      # check the two arms' doc bundles are within tolerance
  agentbench.py run TASK ARM TRIAL --model M   # ARM: ail-plain | ail-tools | ail-core-card | c (ail = ail-plain); or AGENTBENCH_MODEL
                [--max-usd X] [--trial-usd 4]  # one trial -> results/TASK/ARM-TRIAL/ (metrics.json, transcript.jsonl)
  agentbench.py batch PLAN --model M --max-usd X [--trial-usd 4] [--docs-policy read|reference]   # plans/PLAN.txt or a path, in order; resumable
  agentbench.py spend [RESULTS]                # total recorded spend (results/spend.jsonl)
  agentbench.py validate TASK [--arms A,B]     # solution/ passes every hidden case in every arm (or the listed arms, or task.json "arms"); a fix task's seed/ fails some
  agentbench.py score DIR                      # re-score a finished trial's workspace against the hidden cases
  agentbench.py report [RESULTS]               # markdown table over all trials
  agentbench.py trials [RESULTS] [--plan P] [--json]   # per trial: lookups, calls before the first write, failed compiles and tests after it
  agentbench.py audit [RESULTS] [--all]        # per trial: DOCS bytes received, paths outside the workspace, void reads
Run on a Kaleb machine only (ADR 0842): trials call a paid model and build with `ail` / `cc`.
"""
import json, os, shutil, subprocess, sys, tempfile, time, statistics, argparse, hashlib

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
ARMS = ("ail-plain", "ail-tools", "ail-core-card", "c", "rust", "c-nodocs", "rust-nodocs")  # ADR 0916, amended 2026-11-10: the AIL arm is split; "ail" in old results means ail-plain
NODOCS_ARMS = ("c-nodocs", "rust-nodocs")  # E14 review, decision 2(b): the language's own DOCS left out, as C in E13 behaved (it never read its card)
TOOL_ARMS = ("ail-tools", "ail-core-card")  # arms that get the kit (AGENTS.md, CLAUDE.md, .mcp.json) from arms/ARM/
LANG = {"ail-plain": "ail", "ail-tools": "ail", "ail-core-card": "ail", "c": "c", "rust": "rust", "c-nodocs": "c", "rust-nodocs": "rust"}  # seeds, build lines and docs are per language
ALIAS = {"ail": "ail-plain"}
def arm_name(a):
    a = ALIAS.get(a, a)
    if a not in ARMS: sys.exit(f"unknown arm {a!r}; one of {', '.join(ARMS)}")
    return a
PROMPT = ("You are in a fresh repository. Read SPEC.md, then read every file in DOCS/ in full before you write any code: "
          "a file longer than 24 KB is split into numbered parts, and every part has to be read, one call each. "
          "Then implement the task in {lang}. `./build` builds, `./test` runs the visible cases and prints the failures. "
          "Work alone until `./test` passes; hidden cases will also be run. Do not edit build, test or DOCS/.")
# --docs-policy (E14 review, decision 2): "read" is the prompt above; "reference" makes DOCS/ a reference to consult as needed,
# the same words for every arm. The policy is recorded in metrics.json and the report shows what each trial read (docs_bytes).
PROMPT_REFERENCE = ("You are in a fresh repository. Read SPEC.md, then implement the task in {lang}. DOCS/ is a reference for the language "
          "and its library: consult it as needed. `./build` builds, `./test` runs the visible cases and prints the failures. "
          "Work alone until `./test` passes; hidden cases will also be run. Do not edit build, test or DOCS/.")
DOCS_POLICIES = {"read": PROMPT, "reference": PROMPT_REFERENCE}

# Held-out tasks (E14 review, item 16) live outside the repository, in AGENTBENCH_HELDOUT (the Mac keeps them in
# ~/ail-heldout), so no lane that tunes AIL sees their hidden cases; a task found there wins over tasks/.
HELDOUT = os.environ.get("AGENTBENCH_HELDOUT")
def task_dir(t):
    if HELDOUT and os.path.isdir(os.path.join(HELDOUT, t)): return os.path.join(HELDOUT, t)
    return os.path.join(HERE, "tasks", t)
def own_or_base(t, name):  # a change task may carry its own oracle and solution; otherwise its base task's are used
    p = os.path.join(task_dir(t), name)
    return p if os.path.exists(p) else os.path.join(task_dir(load_task(t).get("base", t)), name)
def load_task(t):
    with open(os.path.join(task_dir(t), "task.json")) as f: d = json.load(f)
    d.setdefault("exe", t)
    return d  # "base": a task whose spec, oracle and case inputs this one extends (fix tasks)
def jsonl(path):
    with open(path) as f: return [json.loads(l) for l in f if l.strip()]

# --- cases ---------------------------------------------------------------
sys.path.insert(0, HERE)
from visible import run_case  # one runner for ./test and the hidden-case score
sys.path.pop(0)

def case_inputs(t):
    task = load_task(t); own = os.path.join(task_dir(t), "cases", "inputs.jsonl")
    base = case_inputs(task["base"]) if "base" in task else []
    return base + (jsonl(own) if os.path.exists(own) else [])

def cmd_gen(a):
    d = task_dir(a.task); ref = own_or_base(a.task, "ref.py")
    vis, hid = [], []
    for c in case_inputs(a.task):
        p = subprocess.run([sys.executable, "-I", ref] + c["args"], input=c["stdin"].encode(), capture_output=True)
        full = {"name": c["name"], "args": c["args"], "stdin": c["stdin"], "stdout": p.stdout.decode(), "code": p.returncode}
        if load_task(a.task).get("check_stderr"): full["stderr"] = p.stderr.decode()
        hid.append(full)
        if c.get("visible"): vis.append(full)
    for n, rows in (("visible", vis), ("hidden", hid)):
        with open(os.path.join(d, "cases", n + ".jsonl"), "w") as f:
            f.write("".join(json.dumps(r) + "\n" for r in rows))
    print(f"gen: {len(vis)} visible, {len(hid)} hidden")

# --- doc parity ----------------------------------------------------------
def doc_tokens(paths):
    tb = os.path.join(ROOT, "tools", "tokbench", "run.sh")
    out = subprocess.run([tb, "--json"] + [os.path.join(ROOT, p) for p in paths], capture_output=True, text=True, check=True).stdout
    return sum(r["mean"] for r in json.loads(out))  # mean of the offline tokenizers (ADR 0817)

def cmd_docs(a):
    t = load_task(a.task); others = [l for l in t["docs"] if l != "ail"]; n = {l: doc_tokens(t["docs"][l]) for l in ["ail"] + others}
    short = max((n["ail"] - n[l]) / n["ail"] for l in others)  # each other language may have more docs than AIL, never more than the tolerance fewer (ADR 0916)
    # ail-tools adds the arm's agent files on top of the same card; they are the treatment, so they do not count toward parity (ADR 0916, amended 2026-11-10).
    for arm in TOOL_ARMS: n[f"{arm} agent files"] = doc_tokens(arm_files(arm))
    # The plain arm's DOCS is the card through Modules (card_front): it must fit one part, or the arm reads a split file.
    front = len(card_front(os.path.join(ROOT, "docs", "language-card.md")).encode()) if "docs/language-card.md" in t["docs"]["ail"] else 0
    if front > DOC_PART_BYTES: print(f"ail-plain DOCS (card through Modules) is {front} bytes, over {DOC_PART_BYTES}: move a section past Modules"); sys.exit(1)
    print(n, f"largest shortfall against AIL {short:.1%}"); sys.exit(0 if short <= t["doc_token_tolerance"] else 1)

# --- one trial -----------------------------------------------------------
# Wrapper-logged metrics (ADR 0916, amended 2026-11-10): every ./build, ./test, shimmed `ail` call and MCP tool call appends one line to
# .bench/runs.jsonl ({"t", "k": build|test|tool|mcp, "name", "rc", "out"}) and keeps what it printed in .bench/out/, so
# nothing is self-reported. Nested calls (./test running the build) set BENCH_INNER and are not logged twice.
# "src" is a checksum of src/ at the call, so the metrics can count source versions rather than calls (ADR 0916, amended 2026-11-15).
LOG = "[ -n \"$BENCH_NOLOG\" ] || {{ s=$(find src -type f 2>/dev/null | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | cut -d' ' -f1); echo \"{{\\\"t\\\":$(date +%s),\\\"k\\\":\\\"{k}\\\",\\\"name\\\":\\\"{name}\\\",\\\"rc\\\":$rc,\\\"src_before\\\":\\\"$b\\\",\\\"src\\\":\\\"$s\\\",\\\"out\\\":\\\"out/$n.txt\\\"}}\" >> .bench/runs.jsonl; }}"  # BENCH_NOLOG: the harness's own closing ./test is not the agent's (E14 review R4)
HEAD = ("#!/bin/sh\ncd \"$(dirname \"$0\")\"\nexport BENCH_INNER=1\nn=$(ls .bench/out | wc -l | tr -d ' ')-{k}\n"
        "b=$(find src -type f 2>/dev/null | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | cut -d' ' -f1)\n")
# "src_before" is src/'s checksum when the call started and "src" when it ended: they differ when the build itself rewrote
# the source (repair on intake, ADR 1130), so a repaired draft is told apart from a clean first draft (E14 review R4).
SHIM_BUILD = HEAD + "{cmd} > .bench/out/$n.txt 2>&1\nrc=$?\ncat .bench/out/$n.txt\n" + LOG + "\nexit $rc\n"
SHIM_TEST = (HEAD + "{cmd} > .bench/out/$n.b 2>&1 || {{ {{ echo 'build failed:'; cat .bench/out/$n.b; }} > .bench/out/$n.txt; cat .bench/out/$n.txt; rm .bench/out/$n.b; rc=2; " + LOG
             + "; exit 2; }}\nrm -f .bench/out/$n.b\npython3 -I .bench/visible.py . > .bench/out/$n.txt 2>&1\nrc=$?\ncat .bench/out/$n.txt\n" + LOG + "\nexit $rc\n")
# ./test prints the build's own output when the build fails, as ./build does (E14 review: it used to hide it behind a second call).
# `ail` on PATH for the agent: logs one `tool` line per call (not `mcp`, which streams) and passes stdin through.
SHIM_AIL = ("#!/bin/sh\nreal='{real}'\nB=\"$(cd \"$(dirname \"$0\")/..\" && pwd)\"\n"
            "if [ -n \"$BENCH_INNER\" ] || [ \"$1\" = mcp ]; then exec \"$real\" \"$@\"; fi\n"
            "n=$(ls \"$B/out\" | wc -l | tr -d ' ')-ail\n"
            "on=1; for x in \"$@\"; do case \"$x\" in *src/*) on=1; break;; *.ail) on=0;; esac; done\n"
            "b=$(find \"$B/../src\" -type f 2>/dev/null | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | cut -d' ' -f1)\n"
            "\"$real\" \"$@\" > \"$B/out/$n.txt\" 2>&1\nrc=$?\ncat \"$B/out/$n.txt\"\n"
            "s=$(find \"$B/../src\" -type f 2>/dev/null | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | cut -d' ' -f1); echo \"{{\\\"t\\\":$(date +%s),\\\"k\\\":\\\"tool\\\",\\\"name\\\":\\\"ail $1\\\",\\\"rc\\\":$rc,\\\"onsrc\\\":$on,\\\"src_before\\\":\\\"$b\\\",\\\"src\\\":\\\"$s\\\",\\\"out\\\":\\\"out/$n.txt\\\"}}\" >> \"$B/runs.jsonl\"\nexit $rc\n")

# `sed -i` with no suffix is GNU's form; the Mac's BSD sed reads the next argument as the suffix, so in E13 the agents'
# `sed -i` failed on the Mac in every arm (240 of 480 calls). Every arm gets this `sed` on PATH: GNU sed (gsed) when the
# machine has it, else the system sed with a bare `-i` or `--in-place` given an empty suffix (E14 review, agent fluency).
# Both spellings of in-place work on either sed: GNU's bare `-i` and macOS's `-i ''` (E13: 269 and 259 calls). The shim
# drops an empty argument right after `-i`, then passes `-i` to GNU sed and `-i ''` to BSD sed (E14 review R4, 3.1).
SHIM_SED = ("#!/bin/sh\nreal='{real}'\ngnu={gnu}\nn=$#; i=0\n"
            "while [ $i -lt $n ]; do a=$1; shift; i=$((i+1))\n"
            "  if [ \"$a\" = -i ] || [ \"$a\" = --in-place ]; then\n"
            "    if [ $i -lt $n ] && [ -z \"$1\" ]; then shift; i=$((i+1)); fi\n"
            "    if [ $gnu = 1 ]; then set -- \"$@\" -i; else set -- \"$@\" -i ''; fi\n"
            "  else set -- \"$@\" \"$a\"; fi\n"
            "done\nexec \"$real\" \"$@\"\n")
def sed_shim():
    """The body of the workspace's `sed`: GNU sed (gsed, or the system sed when it is GNU's) or BSD sed, behind SHIM_SED."""
    real = shutil.which("gsed")
    if real: return SHIM_SED.format(real=real, gnu=1)
    real = shutil.which("sed") or "/usr/bin/sed"
    gnu = subprocess.run([real, "--version"], capture_output=True).returncode == 0
    return SHIM_SED.format(real=real, gnu=1 if gnu else 0)

# `cc`, `gcc`, `clang`, `rustc` and `cargo` on PATH for every arm log one `tool` line per call, as the `ail` shim does, so a
# C or Rust agent's direct compiles count like an AIL agent's `ail check` (E14 review R4 3.5): a call that names a file under
# src/ (or none) is on the program, one that names only a source file elsewhere is a scratch compile (onsrc 0).
SHIM_TOOL = ("#!/bin/sh\nreal='{real}'\nB=\"$(cd \"$(dirname \"$0\")/..\" && pwd)\"\n"
             "if [ -n \"$BENCH_INNER\" ]; then exec \"$real\" \"$@\"; fi\n"
             "n=$(ls \"$B/out\" | wc -l | tr -d ' ')-{name}\n"
             "on=1; for x in \"$@\"; do case \"$x\" in *src/*) on=1; break;; *.c|*.rs|*.h) on=0;; esac; done\n"
             "b=$(find \"$B/../src\" -type f 2>/dev/null | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | cut -d' ' -f1)\n"
             "\"$real\" \"$@\" > \"$B/out/$n.txt\" 2>&1\nrc=$?\ncat \"$B/out/$n.txt\"\n"
             "s=$(find \"$B/../src\" -type f 2>/dev/null | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | cut -d' ' -f1); "
             "echo \"{{\\\"t\\\":$(date +%s),\\\"k\\\":\\\"tool\\\",\\\"name\\\":\\\"{name}\\\",\\\"rc\\\":$rc,\\\"onsrc\\\":$on,\\\"src_before\\\":\\\"$b\\\",\\\"src\\\":\\\"$s\\\",\\\"out\\\":\\\"out/$n.txt\\\"}}\" >> \"$B/runs.jsonl\"\nexit $rc\n")
COMPILERS = ("cc", "gcc", "clang", "rustc", "cargo")

def arm_dir(arm): return os.path.join(HERE, "arms", arm)
def arm_files(arm):  # the arm's instruction file, relative to the repository root (what the doc-size check counts)
    p = os.path.join(arm_dir(arm), "AGENTS.md")
    return [os.path.relpath(p, ROOT)] if os.path.exists(p) else []

def real_ail():
    r = os.environ.get("AIL") or shutil.which("ail") or os.path.join(ROOT, "ail")
    if not os.path.exists(r): sys.exit("no `ail`: put it on PATH or set AIL (ADR 0842: run on a designer machine)")
    return os.path.abspath(r)

def write_exec(path, body):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w") as f: f.write(body)
    os.chmod(path, 0o755)

# --- docs over the agent's tool-output limit (ADR 0916, amended 2026-11-15b) ---
# The agent CLI cuts a tool result at about 30 KB and saves the rest to a file, so a `cat` of a larger card costs a second
# read (E13: 77 of 96 ail-plain and 59 of 96 c trials). Every arm's DOCS file above DOC_PART_BYTES is split at a heading
# (else a line) into NAME.part1.md, NAME.part2.md, ...; SPEC.md gets the same note in every arm. The text is unchanged.
DOC_PART_BYTES = 24000
DOCS_NOTE = ("\n\n## Reading DOCS\n\nRead every file in `DOCS/` in full before you write any code. A file longer than 24 KB is split "
             "into numbered parts (`NAME.part1.md`, `NAME.part2.md`, ...): read each part, in order, with its own call (a tool result "
             "over 30 KB is cut short), and do not stop at the first lines of a part.\n")

def docs_listing(ws):
    """The files of DOCS/ with their sizes, for SPEC.md: every arm is told the same way what there is to read."""
    d = os.path.join(ws, "DOCS"); names = sorted(os.listdir(d)) if os.path.isdir(d) else []
    if not names: return "\nFiles in `DOCS/`: none.\n"
    return "\nFiles in `DOCS/` (read all of them):\n" + "".join(f"- `{n}` ({os.path.getsize(os.path.join(d, n))} bytes)\n" for n in names)

def split_doc(text, limit=DOC_PART_BYTES):
    """Parts of at most `limit` bytes, each ending before a `#` heading where one fits, else at a line end, and never inside a
    ``` code block: a `# comment` in a code block is not a heading (E14 review: the AIL card's part 1 ended on an open fence)."""
    lines = text.splitlines(keepends=True); inside, fence = [], False
    for l in lines:  # inside[i]: line i is in a code block (its opening fence counts as inside, its closing one too)
        opens = l.lstrip().startswith("```")
        inside.append(fence or opens)
        if opens: fence = not fence
    parts, start, size, head, edge = [], 0, 0, None, None  # head, edge: absolute line indices to cut before
    for i, l in enumerate(lines):
        n = len(l.encode())
        if size + n > limit and i > start:
            k = head or edge or i
            parts.append("".join(lines[start:k])); start = k; size = sum(len(x.encode()) for x in lines[start:i])
            head = edge = None
            for j in range(start + 1, i):  # cut points among the carried lines
                if not inside[j] and not inside[j - 1]:
                    edge = j
                    if lines[j].startswith("#"): head = j
        if i > start and not inside[i] and not inside[i - 1]:
            edge = i
            if l.startswith("#"): head = i
        size += n
    if start < len(lines): parts.append("".join(lines[start:]))
    return parts

def put_doc(src, dst_dir):
    """Copies one doc into DOCS/, split into parts when it is over DOC_PART_BYTES. Returns the names written."""
    name = os.path.basename(src)
    with open(src, encoding="utf-8") as f: text = f.read()
    if len(text.encode()) <= DOC_PART_BYTES:
        shutil.copy(src, os.path.join(dst_dir, name)); return [name]
    stem, ext = os.path.splitext(name); out = []
    for i, part in enumerate(split_doc(text), 1):
        out.append(f"{stem}.part{i}{ext}")
        with open(os.path.join(dst_dir, out[-1]), "w", encoding="utf-8") as f: f.write(part)
    return out

# The plain arm's DOCS (E14 review item 15): the card through "Modules", the part its own text says to read before a first
# program, in one file under DOC_PART_BYTES, and a closing list of the other sections, which `ail card WORD` prints.
def card_front(path):
    import re
    with open(path, encoding="utf-8") as f: text = f.read()
    i = text.index("\n## Modules"); j = text.index("\n## ", i + 1)
    rest = re.findall(r"^## (.+)$", text[j:], re.M)
    note = ("## The rest of the card\n\nNot in this file; `ail card WORD` prints a section: "
            + ", ".join(f"`{first_word(h)}` ({h.replace('`', '')})" for h in rest) + ".\n")
    return text[:j + 1] + note
def first_word(h):
    """The first word of a heading that no earlier heading of the card holds, lower case, for `ail card` (tool.card.2)."""
    import re
    return re.findall(r"[a-z][a-z-]+", h.replace("`", "").lower())[0]

def seed_sum(t, lang):
    """The checksum the shims log for the untouched seed: the same shell pipeline, run in the seed directory."""
    d = os.path.join(task_dir(t), "seed", lang)
    return subprocess.run(["sh", "-c", "find src -type f 2>/dev/null | LC_ALL=C sort | xargs cat 2>/dev/null | cksum | cut -d' ' -f1"],
                          cwd=d, capture_output=True, text=True).stdout.strip()

def prep(t, task, arm, ws):
    lang = LANG[arm]
    shutil.copytree(os.path.join(task_dir(t), "seed", lang), ws, dirs_exist_ok=True)
    with open(os.path.join(task_dir(t), "spec.md" if task["kind"] == "build" else "issue.md"), encoding="utf-8") as f: spec = f.read()
    os.makedirs(os.path.join(ws, "DOCS"), exist_ok=True); os.makedirs(os.path.join(ws, ".bench", "out"), exist_ok=True)
    if task["kind"] != "build": put_doc(os.path.join(task_dir(task["base"]), "spec.md"), os.path.join(ws, "DOCS"))
    with open(os.path.join(ws, ".bench", "exe"), "w") as f: f.write(task["exe"])
    for p in ([] if arm in NODOCS_ARMS else task["docs"][lang]):
        if arm == "ail-core-card" and p == "docs/language-card.md": p = "docs/language-card-core.md"  # the one doc this arm loads; the rest is `ail card TOPIC` and `ail ask std`
        if arm == "ail-plain" and p == "docs/language-card.md":  # the card through Modules, one file (item 15)
            with open(os.path.join(ws, "DOCS", "language-card.md"), "w", encoding="utf-8") as f: f.write(card_front(os.path.join(ROOT, p)))
            continue
        put_doc(os.path.join(ROOT, p), os.path.join(ws, "DOCS"))
    with open(os.path.join(ws, "SPEC.md"), "w", encoding="utf-8") as f: f.write(spec.rstrip("\n") + DOCS_NOTE + docs_listing(ws))  # after DOCS/ is filled
    with open(os.path.join(ws, ".bench", "seed"), "w") as f: f.write(seed_sum(t, lang))  # the untouched seed's src/ checksum
    shutil.copy(os.path.join(task_dir(t), "cases", "visible.jsonl"), os.path.join(ws, ".bench", "visible.jsonl"))
    cmd = " ".join(task["build"][lang])
    write_exec(os.path.join(ws, "build"), SHIM_BUILD.format(k="build", name="./build", cmd=cmd))
    write_exec(os.path.join(ws, "test"), SHIM_TEST.format(k="test", name="./test", cmd=cmd))
    shutil.copy(os.path.join(HERE, "visible.py"), os.path.join(ws, ".bench", "visible.py"))
    if lang == "ail": write_exec(os.path.join(ws, ".bench", "bin", "ail"), SHIM_AIL.format(real=real_ail()))
    write_exec(os.path.join(ws, ".bench", "bin", "sed"), sed_shim())  # every arm alike
    for c in COMPILERS:  # every arm alike; ./build's own compile runs inside BENCH_INNER and is not logged twice
        real = shutil.which(c)
        if real: write_exec(os.path.join(ws, ".bench", "bin", c), SHIM_TOOL.format(real=real, name=c))
    if arm in TOOL_ARMS:  # what `ail new` writes (ADR 1046, amended 2026-11-11), plus the MCP server through the logging proxy; agent files, never the prompt
        d = arm_dir(arm)
        for f in ("AGENTS.md", "CLAUDE.md"): shutil.copy(os.path.join(d, f), os.path.join(ws, f))
        with open(os.path.join(d, ".mcp.json")) as f: mcp = f.read()
        with open(os.path.join(ws, ".mcp.json"), "w") as f: f.write(mcp.replace("{me}", os.path.abspath(__file__)).replace("{ail}", real_ail()).replace("{bench}", os.path.join(os.path.abspath(ws), ".bench")))
    subprocess.run(["git", "init", "-q"], cwd=ws); subprocess.run(["git", "add", "-A"], cwd=ws)
    subprocess.run(["git", "-c", "user.name=b", "-c", "user.email=b@b", "commit", "-qm", "seed"], cwd=ws)

def agent_env(ws):
    # The host's CLAUDE* variables (a desktop or CLI session's organisation, account and entrypoint) would make the agent act
    # as part of that session and bill it; drop them all. agent_isolation() then sets only CLAUDE_CONFIG_DIR (ADR 0916, amended 2026-11-15).
    # The harness's own variables (AGENTBENCH_*, AIL*: the results folder and the tree with the hidden cases) are dropped too;
    # the `ail` shim names the real `ail`, which sets its own root (E14 review).
    e = {k: v for k, v in os.environ.items() if not k.startswith(("CLAUDE", "AGENTBENCH", "AIL"))}; e["PATH"] = os.path.join(os.path.abspath(ws), ".bench", "bin") + os.pathsep + e["PATH"]
    e["BENCH_DIR"] = os.path.join(os.path.abspath(ws), ".bench")
    # The agent's TMPDIR is a folder in its own workspace, never the job's temp folder (E14 and the final-run audit: that folder
    # held the whole tree with the hidden cases). Inside the workspace a path never voids a trial; cmd_run removes it at the end.
    e["TMPDIR"] = os.path.join(os.path.abspath(ws), ".bench", "tmp"); os.makedirs(e["TMPDIR"], exist_ok=True)
    return e

# --- spend caps (ADR 0916, amended 2026-11-15) --------------------------
# Every agent invocation appends {"task", "arm", "trial", "usd"} to results/spend.jsonl, which a rerun of a trial never deletes.
# --max-usd caps the whole ledger (pilot and batch together); --trial-usd caps one trial. Each invocation gets the CLI's
# --max-budget-usd set to what is left of both, and an invocation killed at its timeout (no result event) is charged its
# whole budget, so the recorded total never undercounts. A trial stopped by a cap is counted as a failure, never dropped.
def ledger_path(results): return os.path.join(results, "spend.jsonl")
def spent(results, key=None):
    p = ledger_path(results)
    if not os.path.exists(p): return 0.0
    return sum(r["usd"] for r in jsonl(p) if key is None or (r["task"], r["arm"], r["trial"]) == key)
def charge(results, key, usd):
    os.makedirs(results, exist_ok=True)
    with open(ledger_path(results), "a") as f: f.write(json.dumps({"t": int(time.time()), "task": key[0], "arm": key[1], "trial": key[2], "usd": round(usd, 4)}) + "\n")

def agent_isolation(out):
    """`out` is the trial's private folder outside the results (see cmd_run). The agent runs with a fresh config directory per trial: none of the machine's user settings, memory, CLAUDE.md, plugins
    or MCP servers reach it (equivalence rule 3), and with no stored sign-in it can only bill the API key in the environment."""
    d = os.path.join(out, "agent-config"); os.makedirs(d, exist_ok=True)
    return {"CLAUDE_CONFIG_DIR": d}

def agent(ws, prompt, model, task, cont, log, budget, out, arm):
    """One agent invocation. Returns (usd, stopped): stopped is "timeout", "budget" or None."""
    cmd = os.environ.get("AGENTBENCH_AGENT", "claude").split() + ["-p", "--model", model, "--output-format", "stream-json", "--verbose",
           "--dangerously-skip-permissions", "--max-turns", str(task["max_turns"]), "--max-budget-usd", f"{budget:.2f}", "--strict-mcp-config"]
    if arm in TOOL_ARMS: cmd += ["--mcp-config", os.path.join(os.path.abspath(ws), ".mcp.json")]
    if cont: cmd.append("--continue")
    before = usage(log)["cost_usd"] if os.path.exists(log) else 0.0; n0 = usage(log)["results"] if os.path.exists(log) else 0
    env = dict(agent_env(ws), **agent_isolation(out)); stopped = None
    with open(log, "a") as f:
        try: subprocess.run(cmd, input=prompt.encode(), stdout=f, cwd=ws, timeout=task["timeout_s"], env=env)
        except subprocess.TimeoutExpired: stopped = "timeout"
    u = usage(log)
    if u["results"] == n0: return budget, stopped or "no-result"  # no result event: charge the whole budget
    if u["api_error"]: stopped = "api-error"
    elif u["last_subtype"] == "error_max_budget_usd": stopped = "budget"
    return u["cost_usd"] - before, stopped

def usage(log):
    u = {"input": 0, "output": 0, "cache_read": 0, "cache_create": 0, "turns": 0, "cost_usd": 0.0, "results": 0, "last_subtype": None, "api_error": None}
    for l in open(log):
        try: e = json.loads(l)
        except ValueError: continue
        if e.get("type") == "result":  # one per agent invocation; cumulative for that call
            x = e.get("usage", {}); u["turns"] += e.get("num_turns", 0); u["cost_usd"] += e.get("total_cost_usd", 0.0)
            u["results"] += 1; u["last_subtype"] = e.get("subtype")
            if e.get("terminal_reason") == "api_error" or (e.get("is_error") and "API Error" in str(e.get("result", ""))):
                u["api_error"] = str(e.get("result") or e.get("terminal_reason"))[:300]  # the model was never reached: a harness fault, not a result
            u["input"] += x.get("input_tokens", 0); u["output"] += x.get("output_tokens", 0)
            u["cache_read"] += x.get("cache_read_input_tokens", 0); u["cache_create"] += x.get("cache_creation_input_tokens", 0)
    return u

def count_tokens(paths):
    """Tokens (tokbench mean, ADR 0817) of each file, in order: one tokbench call for the lot."""
    if not paths: return []
    tb = os.path.join(ROOT, "tools", "tokbench", "run.sh")
    out = subprocess.run([tb, "--json"] + list(paths), capture_output=True, text=True, check=True).stdout
    return [r["mean"] for r in json.loads(out)]

CHECKS = ("ail check", "ail_check", "./build")
AGENT_TOOLS = ("ail task", "ail context", "ail edit", "ail try", "ail fix", "ail impact")  # the kit's workflow tools, beyond lookups; any `mcp:` call counts too
def used_agent_tools(m): return any(n in AGENT_TOOLS or n.startswith("mcp:") for n in m.get("tool_calls", {}))
CHECKY = ("ail check", "ail build", "ail test", "ail run", "ail fix", "ail edit", "ail try", "ail task") + COMPILERS  # shimmed calls that compile or run the program

def src_sum(d):
    """The same checksum the shims log: POSIX cksum of src/'s files concatenated in byte order of their paths."""
    paths = sorted(os.path.relpath(os.path.join(r, f), os.path.dirname(d)) for r, _, fs in os.walk(d) for f in fs)
    data = b"".join(open(os.path.join(os.path.dirname(d), p), "rb").read() for p in paths)
    try: return subprocess.run(["cksum"], input=data, capture_output=True, text=False).stdout.split()[0].decode()
    except (OSError, IndexError): return ""

def on_src(r):
    """Whether a shimmed `ail` call was on the program: `onsrc` is 0 when it named a .ail file and no path under src/ (a scratch
    probe such as `ail run /tmp/t.ail`), so probes are not fix rounds in any arm (E14 review). Runs logged before the field count."""
    return r.get("onsrc", 1) == 1

def version_metrics(runs, seed=None):
    """Counts source versions, not calls (ADR 0916, amended 2026-11-15): an agent that runs `ail check`, `./build` and `./test`
    on one version has made one attempt, not three. fix_rounds: versions on which a build, test or checking call failed.
    versions_to_green: versions checked up to and including the first passing ./test. The untouched seed (checksum `seed`)
    is not an attempt: running ./test on it first reproduces the task, so it counts in neither (amended 2026-11-15b)."""
    checky = [r for r in runs if "src" in r and r["src"] != seed and (r["k"] in ("build", "test", "mcp") or (r["k"] == "tool" and r["name"] in CHECKY and on_src(r)))]
    order, failed = [], set()
    drafts, extra, extra_at_green, green_i = set(), 0, None, None  # drafts: what the agent wrote before a build rewrote it; counted as one more version in the secondary figure (ADR 0916 amended 2026-11-19)
    for r in checky:
        if r["src"] not in order: order.append(r["src"])
        sb = r.get("src_before")
        if sb and sb != r["src"] and sb != seed and sb not in order and sb not in drafts: drafts.add(sb); extra += 1
        if r["rc"] != 0: failed.add(r["src"])
        if green_i is None and r["k"] == "test" and r["rc"] == 0: green_i, extra_at_green = order.index(r["src"]) + 1, extra
    green = green_i
    return {"fix_rounds": len(failed) if checky else None, "versions_to_green": green, "versions": len(order),
            "versions_to_green_alt": None if green is None else green + extra_at_green, "versions_alt": len(order) + extra}  # a first "check" is whichever of these an agent runs first: the build runs the checker

def ws_seed(ws, t=None, lang=None):
    """The seed checksum of a workspace: .bench/seed, written by prep; for a trial run before it existed, recomputed from the task."""
    p = os.path.join(ws, ".bench", "seed")
    if os.path.exists(p): return open(p).read().strip() or None
    return seed_sum(t, lang) if t and lang else None

def toolchain_metrics(ws, tokens=True, seed=None):
    """The wrapper-logged figures of one trial (ADR 0916, amended 2026-11-10). `ws` is the trial workspace."""
    p = os.path.join(ws, ".bench", "runs.jsonl"); runs = jsonl(p) if os.path.exists(p) else []
    seed = seed or ws_seed(ws)
    calls = {}
    for r in runs:
        n = r["name"] if r["k"] != "mcp" else "mcp:" + r["name"]
        calls[n] = calls.get(n, 0) + 1
    first = next((r for r in runs if r["name"] in CHECKS and r.get("src") != seed), None)
    rewrites = sum(1 for r in runs if r.get("src_before") and r.get("src") and r["src_before"] != r["src"])
    scratch = sum(1 for r in runs if r["k"] == "tool" and r["name"] in CHECKY and not on_src(r))  # probes outside src/ (R4 3.5)
    green = next((i + 1 for i, r in enumerate(runs) if r["k"] == "test" and r["rc"] == 0), None)
    outs = [os.path.join(ws, ".bench", r["out"]) for r in runs if r.get("out") and os.path.exists(os.path.join(ws, ".bench", r["out"]))]
    return {**version_metrics(runs, seed), "seed_checked": any(r.get("src") == seed for r in runs) if seed else None, "first_check_clean": None if first is None else (first["rc"] == 0 and first.get("src_before", first["src"]) == first["src"]),
            "build_rewrites": rewrites, "scratch_compiles": scratch, "tool_calls": calls, "calls_total": len(runs),
            "toolchain_tokens": round(sum(count_tokens(outs)), 1) if tokens else None, "turns_to_green": green}

# --- the per-trial report (ADR 0916, amended 2026-11-15b): what E13 needed a transcript analysis for -------------------
# From the transcript (the agent's tool calls): lookups (reading SPEC.md or DOCS/, `ail ask|card|explain|api|doc|help`,
# `man`, `--help`, MCP lookup tools, web) and the calls made before the first write to src/. From runs.jsonl (wrapper-logged,
# by source version, the seed excluded): versions after the first write whose build or check failed, and versions that built
# but failed ./test.
import re
LOOKUP_CMD = re.compile(r"(^|[\s;&|(])(ail\s+(ask|card|explain|api|doc|help)\b|man\s|\S+\s+--help\b|ail\s+--help)")
DOC_PATH = re.compile(r"(^|[\s/'\"])(DOCS/|SPEC\.md)")
READERS = re.compile(r"(^|[\s;&|(])(cat|head|tail|sed\s+-n|less|more|grep|rg|awk|wc)\b")
WRITE_CMD = re.compile(r"(>\s*\S*src/|tee\s+\S*src/|sed\s+-i|perl\s+-[a-z]*i|python3?\b.*src/|ail\s+(edit|fix)\b|ail\s+check\s+--fix|patch\b|\bcp\b.*src/|\bmv\b.*src/)")

def classify(name, inp):
    """'lookup', 'write' or 'other' for one agent tool call."""
    if name in ("Write", "Edit", "MultiEdit", "NotebookEdit"):
        return "write" if "src/" in str(inp.get("file_path", "")) else "other"
    if name in ("Read", "Grep", "Glob"):
        return "lookup" if DOC_PATH.search(str(inp.get("file_path") or inp.get("path") or inp.get("pattern") or "")) else "other"
    if name in ("WebFetch", "WebSearch"): return "lookup"
    if name.startswith("mcp__"):
        tool = name.rsplit("__", 1)[-1]
        return "write" if tool in ("edit", "fix") else ("lookup" if tool in ("ask", "card", "explain", "api", "doc", "help", "context") else "other")
    if name == "Bash":
        c = str(inp.get("command", ""))
        if WRITE_CMD.search(c): return "write"
        if LOOKUP_CMD.search(c) or (DOC_PATH.search(c) and READERS.search(c)): return "lookup"
    return "other"

def transcript_calls(log):
    """The agent's tool calls in order, as (name, input), from a stream-json transcript."""
    calls = []
    if not os.path.exists(log): return calls
    for l in open(log, encoding="utf-8", errors="replace"):
        try: e = json.loads(l)
        except ValueError: continue
        if e.get("type") != "assistant": continue
        for c in (e.get("message") or {}).get("content") or []:
            if isinstance(c, dict) and c.get("type") == "tool_use": calls.append((c.get("name", "?"), c.get("input") or {}))
    return calls

def transcript_events(log):
    if not os.path.exists(log): return
    for l in open(log, encoding="utf-8", errors="replace"):
        try: yield json.loads(l)
        except ValueError: continue

def result_text(c):
    x = c.get("content")
    if isinstance(x, list): return "".join(b.get("text", "") for b in x if isinstance(b, dict))
    return x if isinstance(x, str) else ""

def docs_received(log):
    """Bytes of SPEC.md and DOCS/ the agent actually received: the tool results of calls that read them, as cut by the agent
    CLI (E14 review: in E13 no C trial from scratch received its card). {"docs_bytes", "docs_reads"}."""
    calls, got, n, before, wrote = {}, 0, 0, 0, False  # before: bytes received from calls made before the first write to src/
    for e in transcript_events(log):
        for c in (e.get("message") or {}).get("content") or []:
            if not isinstance(c, dict): continue
            if c.get("type") == "tool_use":
                name, inp = c.get("name", "?"), c.get("input") or {}
                calls[c.get("id")] = (name, inp, not wrote)
                if classify(name, inp) == "write": wrote = True
            elif c.get("type") == "tool_result" and c.get("tool_use_id") in calls:
                name, inp, early = calls[c["tool_use_id"]]
                if classify(name, inp) == "lookup" and DOC_PATH.search(json.dumps(inp)):
                    k = len(result_text(c).encode()); got += k; n += 1; before += k if early else 0
    return {"docs_bytes": got, "docs_reads": n, "docs_bytes_before_write": before}

# Price per million tokens by class, fitted exactly to E13's recorded costs (E14 review); the CLI's own total stays the charged cost.
PRICE = {"input": 2.0, "cache_create": 2.5, "cache_read": 0.1, "output": 10.0}
DOCS_BYTES_PER_TOKEN = 4.0  # an estimate, stated in the report
def cost_ex_docs(m):
    """Cost of a trial without its DOCS read (ADR 0916 amended 2026-11-19), an estimate: the DOCS tokens (bytes / DOCS_BYTES_PER_TOKEN) are
    written to the cache once at the cache-write price and re-read on each later turn at the cache-read price:
    cost - tokens * (cache_create + cache_read * (turns - 1)) / 1e6, never below the trial's output cost."""
    tok = m.get("docs_bytes", 0) / DOCS_BYTES_PER_TOKEN; t = max(m.get("turns", 1) - 1, 0)
    c = m.get("charged_usd", m["cost_usd"]) - tok * (PRICE["cache_create"] + PRICE["cache_read"] * t) / 1e6
    return max(c, m.get("usd_output", 0.0))
def cost_by_class(u): return {f"usd_{k}": round(u.get(k, 0) * v / 1e6, 4) for k, v in PRICE.items()}

# --- the transcript audit (E14 review): any path outside the trial's workspace, and the reads that void a trial ---
ABS_PATH = re.compile(r"(?<![\w.])(/[A-Za-z0-9_.@+-]+(?:/[A-Za-z0-9_.@+*-]*)*)")
SYSTEM_PATHS = ("/usr/", "/bin/", "/sbin/", "/opt/homebrew/", "/Library/Developer/", "/Applications/Xcode", "/dev/null", "/dev/std", "/etc/", "/System/", "/private/var/folders/", "/var/folders/")
VOIDING = re.compile(r"tools/agentbench|ail-heldout|hidden\.jsonl|inputs\.jsonl|/solution/|ref\.py|spend\.jsonl|metrics\.json|/trial-[A-Za-z0-9_]+/|/ail-e1[0-9]")
def trial_cwd(log):
    """The workspace the agent ran in, from the transcript's init event."""
    return next((e["cwd"] for e in transcript_events(log) if e.get("type") == "system" and e.get("cwd")), None)

def void_text(text, homes, own):
    """Whether one command reads outside the workspace or a VOIDING path. A relative path is resolved against the command's
    `cd` directory (the workspace at first): `../.bench` from `ws/src` stays inside and is never a void, `.bench/` itself
    neither; a path that leaves the workspace voids when the resolved path matches VOIDING (ADR 0916 amended 2026-11-17)."""
    ws = sorted(h for h in homes if h)[0] if homes else ""
    if VOIDING.search(" ".join(q for q in ABS_PATH.findall(text) if not own(q))): return True
    cwd, prev = ws, ""
    for tok in re.split(r"[\s;&|]+", text):
        if prev == "cd" and tok: cwd = os.path.normpath(tok if tok.startswith("/") else os.path.join(cwd, tok))
        elif tok.startswith("../") or tok == "..":
            q = os.path.normpath(os.path.join(cwd, tok))
            if not own(q) and VOIDING.search(q + "/"): return True
        prev = tok
    return False

def audit(log, ws=None):
    """{"outside": paths outside the workspace that the agent named, "void": the first call that read the harness, a hidden
    case, a solution or another trial, else None}. A voided trial is listed and left out of every table."""
    ws = ws or trial_cwd(log) or ""; homes = {ws, os.path.realpath(ws) if ws else "", ws[len("/private"):] if ws.startswith("/private/") else ""} - {""}
    own = lambda q: any(q == h or q.startswith(h.rstrip("/") + "/") for h in homes)
    outside, void = [], None
    for name, inp in transcript_calls(log):
        text = " ".join(str(v) for v in inp.values()) if isinstance(inp, dict) else str(inp)
        for q in ABS_PATH.findall(text):
            if own(q) or q.startswith(SYSTEM_PATHS) or q in ("/", "/tmp", "/tmp/"): continue
            if q not in outside: outside.append(q)
        rel = re.sub(r"(^|\s)\.\./", " PARENT/", text)
        bad = void_text(text, homes, own)
        if bad and void is None: void = f"{name}: {text[:200]}"
    return {"outside": outside[:30], "void": void}

def trial_report(out, t=None, arm=None):
    """One trial's lookups, calls before the first write, failed compiles and failed tests after the first write."""
    ws = os.path.join(out, "ws"); kinds = [classify(n, i) for n, i in transcript_calls(os.path.join(out, "transcript.jsonl"))]
    first_write = kinds.index("write") if "write" in kinds else None
    p = os.path.join(ws, ".bench", "runs.jsonl"); runs = jsonl(p) if os.path.exists(p) else []
    seed = ws_seed(ws, t, LANG.get(arm_name(arm)) if arm else None)
    bad_build, bad_test = set(), set()
    for r in runs:
        if "src" not in r or r["src"] == seed or r["rc"] == 0: continue
        compiles = r["k"] == "build" or (r["k"] == "tool" and r["name"] in CHECKY and on_src(r)) or (r["k"] == "mcp" and r["name"] in ("check", "build"))
        if r["k"] == "test":  # a ./test whose build failed printed "build failed": a compile failure, not a test failure
            o = os.path.join(ws, ".bench", r.get("out", ""))
            compiles = os.path.exists(o) and open(o, errors="replace").read().startswith("build failed")
            if not compiles: bad_test.add(r["src"])
        if compiles: bad_build.add(r["src"])
    return {"agent_calls": len(kinds), "lookups": kinds.count("lookup"), "lookups_before_write": kinds[:first_write].count("lookup") if first_write is not None else kinds.count("lookup"),
            "calls_before_write": first_write, "failed_compile_versions": len(bad_build), "failed_test_versions": len(bad_test - bad_build)}

def score_ws(t, arm, ws):
    task = load_task(t); lang = LANG[arm]
    import glob
    cmd = [real_ail() if (lang == "ail" and x == "ail") else x for x in task["build"][lang]]  # honours AIL=, as the shims do
    cmd = [y for x in cmd for y in (sorted(os.path.relpath(g, ws) for g in glob.glob(os.path.join(ws, x))) or [x] if "*" in x else [x])]  # `src/*.c`, as the shell shims expand it
    r = subprocess.run(cmd, cwd=ws, capture_output=True, text=True)
    st = structure_ok(task, lang, ws)
    if r.returncode: return {"built": False, "passed": 0, "total": len(jsonl(os.path.join(task_dir(t), "cases", "hidden.jsonl"))), "failed": [], "structure": st}
    res = [run_case(os.path.join(ws, task["exe"]), c) for c in jsonl(os.path.join(task_dir(t), "cases", "hidden.jsonl"))]
    return {"built": True, "passed": sum(x["ok"] for x in res), "total": len(res), "failed": [x["name"] for x in res if not x["ok"]], "structure": st}

EXT = {"ail": ".ail", "c": ".c", "rust": ".rs"}
def structure_ok(task, lang, ws):
    """A refactor task's mechanical check (ADR 0916, amended 2026-11-15): `src/main.EXT` at most `main_max_lines` lines and at
    least `min_modules` other source files of the language under src/ (C headers do not count); and no `forbid` regex matching
    the path (under the workspace) or the text of any file under src/ (a rename task). None when the task has no check."""
    want = task.get("structure")
    if not want: return None
    import re
    for r, _, fs in os.walk(os.path.join(ws, "src")):
        for f in fs:
            p = os.path.join(r, f); text = open(p, encoding="utf-8", errors="replace").read()
            if any(re.search(x, os.path.relpath(p, ws)) or re.search(x, text) for x in want.get("forbid", [])): return False
    if "main_max_lines" not in want: return True
    main = os.path.join(ws, "src", "main" + EXT[lang])
    if not os.path.exists(main): return False
    lines = sum(1 for l in open(main, encoding="utf-8", errors="replace") if l.strip())
    others = [f for r, _, fs in os.walk(os.path.join(ws, "src")) for f in fs if f.endswith(EXT[lang]) and os.path.join(r, f) != main]
    return lines <= want["main_max_lines"] and len(others) >= want["min_modules"]

def is_correct(sc): return sc["built"] and sc["passed"] == sc["total"] and sc.get("structure") is not False

RESULTS = os.environ.get("AGENTBENCH_RESULTS") or os.path.join(HERE, "results")  # a dry run with a fake agent writes elsewhere

def require_key():
    if "AGENTBENCH_AGENT" not in os.environ and not os.environ.get("ANTHROPIC_API_KEY"):
        sys.exit("ANTHROPIC_API_KEY is not set: counted trials bill the API key only, never a signed-in plan (ADR 0916)")

def trial_invalid(stops, m):
    """None, or why a finished trial is INVALID (rerun, never scored; ADR 0916 amended 2026-11-19): an API error, or an invocation that
    ended with no result event (a timeout or a dead connection) while no source version was ever checked: that would score 0
    versions, below the floor of 1."""
    if "api-error" in stops: return "API error: " + str(m.get("api_error"))
    if any(s in ("no-result", "timeout") for s in stops) and not m.get("versions"): return "no result and no version checked (" + ", ".join(s for s in stops if s) + ")"
    return None

def cmd_run(a):
    require_key()
    arm = arm_name(a.arm); task = load_task(a.task); key = (a.task, arm, str(a.trial)); mine = [0.0]  # this attempt's spend; an invalid attempt's spend stays in the ledger
    left = lambda: min(a.trial_usd - mine[0], a.max_usd - spent(RESULTS))
    if a.max_usd - spent(RESULTS) < 0.5: print(f"cap: ${spent(RESULTS):.2f} of ${a.max_usd:.2f} spent; trial not started"); return 3
    out = os.path.join(RESULTS, a.task, f"{arm}-{a.trial}")
    if os.path.exists(os.path.join(out, "invalid.json")): os.rename(out, f"{out}.invalid-{int(time.time())}")  # kept for the list of harness faults
    shutil.rmtree(out, ignore_errors=True); os.makedirs(out)
    # The agent works in a private folder outside the results and the tree, so `ls ..` shows no other trial and no hidden case
    # (E14 review); the workspace moves to out/ws when the trial ends. AGENTBENCH_WS_ROOT picks the parent (default the system temp).
    priv = tempfile.mkdtemp(prefix="trial-", dir=os.environ.get("AGENTBENCH_WS_ROOT") or "/tmp"); ws = os.path.join(priv, "ws"); prep(a.task, task, arm, ws)
    log = os.path.join(out, "transcript.jsonl"); prompt = DOCS_POLICIES[a.docs_policy].format(lang={"ail": "AIL (the `ail` language)", "c": "C", "rust": "Rust"}[LANG[arm]])
    t0 = time.time(); nudges = 0; stops = []
    usd, st = agent(ws, prompt, a.model, task, False, log, left(), priv, arm); charge(RESULTS, key, usd); mine[0] += usd; stops.append(st)
    while nudges < task["max_nudges"] and st is None and left() >= 0.25 and subprocess.run([os.path.join(ws, "test")], capture_output=True, env=dict(agent_env(ws), BENCH_NOLOG="1")).returncode != 0:
        nudges += 1  # a nudge stands in for a human saying "keep going"; counted as an intervention
        usd, st = agent(ws, "`./test` still fails. Keep going.", a.model, task, True, log, left(), priv, arm); charge(RESULTS, key, usd); mine[0] += usd; stops.append(st)
    shutil.rmtree(os.path.join(priv, "agent-config"), ignore_errors=True)  # session files duplicate the transcript
    shutil.rmtree(os.path.join(ws, ".bench", "tmp"), ignore_errors=True)
    elapsed = round(time.time() - t0, 1); ws_abs = os.path.realpath(ws)
    shutil.move(ws, os.path.join(out, "ws")); shutil.rmtree(priv, ignore_errors=True); ws = os.path.join(out, "ws")
    runs = jsonl(os.path.join(ws, ".bench", "runs.jsonl")) if os.path.exists(os.path.join(ws, ".bench", "runs.jsonl")) else []
    builds = [r for r in runs if r["k"] == "build"]; tests = [r for r in runs if r["k"] == "test"]
    m = {"task": a.task, "arm": arm, "trial": a.trial, "model": a.model, "elapsed_s": elapsed, "interventions": nudges,
         "build_failures": sum(r["rc"] != 0 for r in builds), "test_failures": sum(r["rc"] != 0 for r in tests),
         "debug_rounds": sum(r["rc"] != 0 for r in builds + tests), **usage(log), **toolchain_metrics(ws), **trial_report(out), "score": score_ws(a.task, arm, ws),
         "charged_usd": round(mine[0], 4), "stopped": next((s for s in stops if s), None),
         "docs_policy": a.docs_policy, "tree": tree_commit(), "agent_cli": agent_cli(), **docs_received(log), **cost_by_class(usage(log)), "audit": audit(log, ws_abs)}
    m["correct"] = is_correct(m["score"])
    why = trial_invalid(stops, m)
    if why:  # the model was never reached, or the API failed mid-trial, or the agent ended with no result and no version: INVALID, never scored, and rerun (ADR 0916: harness faults are listed, not counted)
        with open(os.path.join(out, "invalid.json"), "w") as f: json.dump(dict(m, invalid=why), f, indent=1)
        print(f"INVALID {key}: {why}"); return 4
    with open(os.path.join(out, "metrics.json"), "w") as f: json.dump(m, f, indent=1)
    print(json.dumps({k: m[k] for k in ("task", "arm", "trial", "elapsed_s", "output", "charged_usd", "stopped", "debug_rounds", "interventions", "first_check_clean", "turns_to_green", "fix_rounds", "versions_to_green", "lookups", "failed_compile_versions", "failed_test_versions", "toolchain_tokens", "correct")}))
    return 0

_CACHE = {}
def tree_commit():
    if "tree" not in _CACHE:
        r = subprocess.run(["git", "-C", ROOT, "rev-parse", "HEAD"], capture_output=True, text=True); _CACHE["tree"] = r.stdout.strip() or None
    return _CACHE["tree"]
def agent_cli():
    """The agent CLI's version, recorded per trial (E14 review); "fake" for a dry run."""
    if "cli" not in _CACHE:
        if "AGENTBENCH_AGENT" in os.environ: _CACHE["cli"] = "fake"
        else:
            r = subprocess.run(["claude", "--version"], capture_output=True, text=True, input=""); _CACHE["cli"] = (r.stdout.strip().splitlines() or [None])[0]
    return _CACHE["cli"]

def live_dirs(results):
    """(dir, files) of every folder under results that is not a kept invalid attempt (`*.invalid-TIMESTAMP`: renamed when a trial is rerun)."""
    return [(dp, fs) for dp, _, fs in os.walk(results) if ".invalid-" not in os.path.relpath(dp, results)]

def plan_path(name): return name if os.path.exists(name) else os.path.join(HERE, "plans", name + ".txt")
def plan_names():
    """Sealed tasks keep their real names out of the repository: a file `names.txt` in AGENTBENCH_HELDOUT maps the plan's placeholder
    ids to the real task ids, one `sealed1 realname` pair a line. Every plan row, result folder and report then uses the real id."""
    p = os.path.join(HELDOUT or "", "names.txt")
    return dict(l.split()[:2] for l in open(p) if l.strip() and not l.startswith("#")) if HELDOUT and os.path.exists(p) else {}
def plan(name):
    nm = plan_names()
    return [[nm.get(r[0], r[0])] + r[1:] for r in (l.split() for l in open(plan_path(name)) if l.strip() and not l.startswith(("#", "@")))]
def directives(name):
    """A plan's `@` lines (ADR 0916 amended 2026-11-18): `@max-usd X` and `@trial-usd X` lower the caps the flags set (never raise them),
    `@estimate-usd X` is the expected cost of a trial, `@agent-cli V` pins the agent CLI version, `@stage N KIND [ARG]` is a stop
    checked once every trial numbered N or lower has a result (KIND harness or futility)."""
    d = {"stages": []}
    for l in open(plan_path(name)):
        w = l.split()
        if not w or w[0][0] != "@": continue
        if w[0] == "@stage": d["stages"].append((int(w[1]), w[2], float(w[3]) if len(w) > 3 else None))
        else: d[w[0][1:]] = w[1]
    return d

def stage_check(results, rows, n, kind, arg, est):
    """The stop decision after round n of a plan. Returns None to go on, or the reason to stop."""
    ms = [json.load(open(os.path.join(dp, "metrics.json"))) for dp, fs in live_dirs(results) if "metrics.json" in fs]
    keep = {(t, arm_name(arm), trial) for t, arm, trial in rows if int(trial) <= n}
    for m in ms: m["arm"] = arm_name(m["arm"])
    ms = [m for m in ms if (m["task"], m["arm"], str(m["trial"])) in keep]
    inv = [dp for dp, fs in live_dirs(results) if "invalid.json" in fs and "metrics.json" not in fs]
    if kind == "harness":
        void = [m for m in ms if (m.get("audit") or {}).get("void")]
        nodocs = [] if "AGENTBENCH_AGENT" in os.environ else [m for m in ms if m["arm"] in ("c", "ail-plain") and not m.get("docs_bytes")]  # both arms; the fake agent of a dry run reads no DOCS
        cost = statistics.mean(m.get("charged_usd", m["cost_usd"]) for m in ms) if ms else 0.0
        if inv: return f"{len(inv)} invalid trial(s): " + ", ".join(os.path.relpath(d, results) for d in sorted(inv)[:5])
        if void: return f"{len(void)} void trial(s), first {void[0]['task']} {void[0]['arm']} {void[0]['trial']}"
        if nodocs: return f"{len(nodocs)} trial(s) received no DOCS, first {nodocs[0]['task']} {nodocs[0]['arm']} {nodocs[0]['trial']}"
        if est and cost > 1.3 * est: return f"mean cost per trial ${cost:.3f} is over 130% of the ${est:.3f} estimate"
        return None
    if kind == "futility":
        g = [m for m in ms if m["arm"] == "ail-plain"]; c = [m for m in ms if m["arm"] == "c"]
        if not g or not c: return None
        gap = statistics.mean(all_versions(g)) - statistics.mean(all_versions(c))
        if gap > arg: return f"futility: pooled mean versions of ail-plain is {gap:+.2f} above c's after round {n} (limit {arg:+.2f})"
        return None
    sys.exit(f"unknown stage kind {kind!r}")

def cmd_batch(a):
    """Runs a plan's pre-numbered trials in order. A trial with metrics.json is done and skipped, so a stopped batch resumes.
    Stops before the first trial that the --max-usd cap leaves less than $0.50 for."""
    require_key(); rows = plan(a.plan); d = directives(a.plan)
    if "max-usd" in d: a.max_usd = min(a.max_usd, float(d["max-usd"]))  # a plan can lower the caps, never raise them
    if "trial-usd" in d: a.trial_usd = min(a.trial_usd, float(d["trial-usd"]))
    if "agent-cli" in d:
        have = agent_cli() or ""
        if have != "fake" and have.split(" ")[0] != d["agent-cli"]:
            sys.exit(f"plan pins agent CLI {d['agent-cli']}, found {have or 'none'}; install the pinned version (the pin is part of the plan)")
    if "model-sha256" in d and hashlib.sha256(a.model.encode()).hexdigest() != d["model-sha256"]:
        sys.exit("plan pins the model by hash (@model-sha256, so no model name is committed): --model is not the pinned one")
    est = float(d["estimate-usd"]) if "estimate-usd" in d else None
    pending = sorted(d["stages"]); ignore = set(a.ignore_stage or [])
    done = lambda t, arm, trial: os.path.exists(os.path.join(RESULTS, t, f"{arm_name(arm)}-{trial}", "metrics.json"))
    def stage_stop(upto):  # check every stage whose round is over: all rows numbered <= upto have a result
        while pending and pending[0][0] <= upto:
            sn, kind, arg = pending.pop(0)
            if sn in ignore: continue
            why = stage_check(RESULTS, rows, sn, kind, arg, est)
            if why: return f"stage {sn} ({kind}) stop: {why}. Read the results, then rerun with --ignore-stage {sn} to go on, or stop here."
        return None
    for i, (t, arm, trial) in enumerate(rows, 1):
        if pending and int(trial) > pending[0][0]:
            if all(done(*r) for r in rows if int(r[2]) <= pending[0][0]):  # a trial that never produced metrics (INVALID) blocks the stage, and the batch has already stopped on it
                msg = stage_stop(int(trial) - 1)
                if msg: print(msg); print(f"spent ${spent(RESULTS):.2f}"); return 5
        if done(t, arm, trial): continue
        print(f"[{i}/{len(rows)}] {t} {arm} {trial}  spent so far ${spent(RESULTS):.2f} of ${a.max_usd:.2f}", flush=True)
        r = cmd_run(argparse.Namespace(task=t, arm=arm, trial=trial, model=a.model, max_usd=a.max_usd, trial_usd=a.trial_usd, docs_policy=a.docs_policy))
        if r == 4: print(f"batch stopped: trial {i} hit an API error (INVALID, not scored); fix the setup and rerun to resume"); print(f"spent ${spent(RESULTS):.2f}"); return 4
        if r == 3 and a.max_usd - spent(RESULTS) < 0.5: print(f"batch stopped at the cap after {i - 1} of {len(rows)} trials"); break
    else:
        msg = stage_stop(10 ** 9) if pending and all(done(*r) for r in rows) else None
        if msg: print(msg); print(f"spent ${spent(RESULTS):.2f}"); return 5
    print(f"spent ${spent(RESULTS):.2f}")

def cmd_spend(a):
    print(f"${spent(a.results):.2f}")

# --- the MCP logging proxy (ail-tools arm): .mcp.json starts `agentbench.py mcpproxy`, which runs the real `ail mcp` ---
def cmd_mcpproxy(a):
    import threading
    bench = os.environ["BENCH_DIR"]; lock = threading.Lock(); pending = {}
    child = subprocess.Popen([a.ail, "mcp"], stdin=subprocess.PIPE, stdout=subprocess.PIPE, env=dict(os.environ, BENCH_INNER="1"))
    def note(line, fn):
        try: e = json.loads(line)
        except ValueError: return
        fn(e)
    def up(e):  # agent -> server
        if e.get("method") == "tools/call":
            with lock: pending[e.get("id")] = e.get("params", {}).get("name", "?")
    def down(e):  # server -> agent
        with lock: name = pending.pop(e.get("id"), None)
        if name is None or "id" not in e: return
        text = json.dumps(e.get("result", e.get("error")), ensure_ascii=False)
        content = (e.get("result") or {}).get("content") or []
        text = "".join(c.get("text", "") for c in content) or text
        with lock:
            n = f"{len(os.listdir(os.path.join(bench, 'out')))}-mcp"
            with open(os.path.join(bench, "out", n + ".txt"), "w") as f: f.write(text)
            bad = "error" in e or (e.get("result") or {}).get("isError", False)
            with open(os.path.join(bench, "runs.jsonl"), "a") as f:
                f.write(json.dumps({"t": int(time.time()), "k": "mcp", "name": name, "rc": 1 if bad else 0, "src": src_sum(os.path.join(bench, "..", "src")), "out": f"out/{n}.txt"}) + "\n")
    def pump_up():
        for line in sys.stdin.buffer:
            note(line, up); child.stdin.write(line); child.stdin.flush()
        child.stdin.close()
    th = threading.Thread(target=pump_up, daemon=True); th.start()
    for line in child.stdout:
        note(line, down); sys.stdout.buffer.write(line); sys.stdout.buffer.flush()
    sys.exit(child.wait())

def cmd_score(a):
    m = json.load(open(os.path.join(a.dir, "metrics.json")))
    m["score"] = score_ws(m["task"], arm_name(m["arm"]), os.path.join(a.dir, "ws")); m["correct"] = is_correct(m["score"])
    json.dump(m, open(os.path.join(a.dir, "metrics.json"), "w"), indent=1); print(m["score"])

def cmd_validate(a):
    """Every arm's solution/ must pass all hidden cases; a fix task's seed/ must fail some, then be fixed by solution/.
    Only the arms named by --arms (comma list) or the task's "arms" key are checked: a final-run task has solutions for ail-plain and c only.
    Then each arm's workspace is built with the solution in place and driven through its shims, so a trial's logging is known to work."""
    task = load_task(a.task); sol = own_or_base(a.task, "solution"); ok = True
    arms = [arm_name(x) for x in a.arms.split(",")] if a.arms else task.get("arms") or ARMS  # --arms, else the task's own "arms" list, else every arm
    for arm in arms:
        lang = LANG[arm]
        for what, src in (("solution", os.path.join(sol, lang)), ("seed", os.path.join(task_dir(a.task), "seed", lang))):
            if what == "seed" and task["kind"] == "build": continue
            with tempfile.TemporaryDirectory() as ws:
                shutil.copytree(src, ws, dirs_exist_ok=True); sc = score_ws(a.task, arm, ws)
            if task["kind"] == "refactor" and what == "seed":  # behaviour already right; only the structure check fails
                good = sc["built"] and sc["passed"] == sc["total"] and sc["structure"] is False
            else:
                good = sc["built"] and (sc["passed"] == sc["total"] and sc["structure"] is not False) == (what == "solution")
            ok &= good; print(("ok  " if good else "FAIL"), arm, what, f"{sc['passed']}/{sc['total']}", "fails:", sc["failed"])
        with tempfile.TemporaryDirectory() as tmp:
            ws = os.path.join(tmp, "ws"); prep(a.task, task, arm, ws)
            shutil.rmtree(os.path.join(ws, "src")); shutil.copytree(os.path.join(sol, lang), ws, dirs_exist_ok=True)
            env = agent_env(ws); rcs = []
            if lang == "ail": rcs.append(subprocess.run([os.path.join(ws, ".bench", "bin", "ail"), "check", "src/main.ail"], cwd=ws, env=env, capture_output=True).returncode)
            rcs += [subprocess.run([os.path.join(ws, x)], cwd=ws, env=env, capture_output=True).returncode for x in ("build", "test")]
            m = toolchain_metrics(ws); names = set(m["tool_calls"])
            want = {"./build", "./test"} | ({"ail check"} if lang == "ail" else set())
            files = {"AGENTS.md", "CLAUDE.md", ".mcp.json"}
            good = (not any(rcs) and want <= names and m["first_check_clean"] is True and m["turns_to_green"] is not None
                    and m["toolchain_tokens"] is not None and m["fix_rounds"] == 0 and m["versions_to_green"] == 1 and all(os.path.exists(os.path.join(ws, f)) == (arm in TOOL_ARMS) for f in files))
            ok &= good; print(("ok  " if good else "FAIL"), arm, "shims", "rc", rcs, "calls", m["tool_calls"], "first_check_clean", m["first_check_clean"],
                              "turns_to_green", m["turns_to_green"], "toolchain_tokens", m["toolchain_tokens"])
    sys.exit(0 if ok else 1)

# --- `visible` is the in-workspace test runner (called by ./test) --------
def cmd_visible(a):
    import visible; sys.exit(visible.main(a.dir))

def trial_dirs(results, plan_name=None):
    """(task, arm, trial, dir) of every scored trial, or only a plan's."""
    rows = []
    for dp, fs in live_dirs(results):
        if "metrics.json" not in fs: continue
        m = json.load(open(os.path.join(dp, "metrics.json"))); rows.append((m["task"], arm_name(m["arm"]), str(m["trial"]), dp))
    if plan_name:
        keep = {(t, arm_name(arm), trial) for t, arm, trial in plan(plan_name)}; rows = [r for r in rows if r[:3] in keep]
    return sorted(rows, key=lambda r: (r[0], ARMS.index(r[1]), int(r[2]) if r[2].isdigit() else r[2]))

def cmd_trials(a):
    """The per-trial report, recomputed from each trial's transcript and runs.jsonl (so it works on E13's archive too)."""
    out = []
    for t, arm, trial, d in trial_dirs(a.results, a.plan):
        m = json.load(open(os.path.join(d, "metrics.json"))); r = trial_report(d, t, arm)
        r.update(version_metrics(jsonl(os.path.join(d, "ws", ".bench", "runs.jsonl")) if os.path.exists(os.path.join(d, "ws", ".bench", "runs.jsonl")) else [], ws_seed(os.path.join(d, "ws"), t, LANG[arm])))
        out.append({"task": t, "arm": arm, "trial": trial, "correct": m["correct"], "turns": m.get("turns"), "usd": m.get("charged_usd", m.get("cost_usd")), **r})
    if a.json: print(json.dumps(out, indent=1)); return 0
    cols = ("task", "arm", "trial", "correct", "turns", "usd", "lookups", "lookups_before_write", "calls_before_write", "failed_compile_versions", "failed_test_versions", "fix_rounds", "versions_to_green")
    print("| " + " | ".join(cols) + " |\n|" + "---|" * len(cols))
    for r in out: print("| " + " | ".join("-" if r[c] is None else str(r[c]) for c in cols) + " |")
    print("\n| task | arm | trials | mean lookups | mean calls before first write | mean failed compiles after first write | mean failed tests after first write |\n|---|---|--:|--:|--:|--:|--:|")
    mean = lambda xs: f"{statistics.mean(xs):.2f}" if xs else "-"
    for t in sorted({r["task"] for r in out}):
        for arm in ARMS:
            g = [r for r in out if r["task"] == t and r["arm"] == arm]
            if g: print(f"| {t} | {arm} | {len(g)} | {mean([r['lookups'] for r in g])} | {mean([r['calls_before_write'] for r in g if r['calls_before_write'] is not None])} | "
                        f"{mean([r['failed_compile_versions'] for r in g])} | {mean([r['failed_test_versions'] for r in g])} |")
    return 0

def cmd_audit(a):
    """The transcript audit and DOCS bytes of every trial in a results folder, recomputed (E13's archive too)."""
    bad = 0
    for t, arm, trial, d in trial_dirs(a.results):
        log = os.path.join(d, "transcript.jsonl"); r = audit(log); b = docs_received(log)
        if r["void"]: bad += 1
        if r["void"] or r["outside"] or a.all:
            print(f"{t} {arm} {trial}: docs {b['docs_bytes']} B in {b['docs_reads']} reads; VOID {r['void'][:200] if r['void'] else '-'}; outside {r['outside'][:8]}")
    print(f"void trials: {bad}")
    return 0

def med(xs, fmt=".1f"):
    xs = [x for x in xs if x is not None]
    return f"{statistics.median(xs):{fmt}}" if xs else "-"

def task_class(t):
    """from scratch, change, or held out (reported apart): the classes of the primary measure (ADR 0916, 2026-11-15c)."""
    try: d = load_task(t)
    except OSError: return "unknown"
    return "held out" if d.get("heldout") else ("from scratch" if d["kind"] == "build" else "change")

def boot_diff(xs, ys, n=2000):
    """The 95% interval of mean(xs) - mean(ys), bootstrapped over trials with a fixed seed; None when either is empty."""
    if not xs or not ys: return None
    import random
    rnd = random.Random(916); d = []
    for _ in range(n):
        a = [rnd.choice(xs) for _ in xs]; b = [rnd.choice(ys) for _ in ys]
        d.append(statistics.mean(a) - statistics.mean(b))
    d.sort(); return d[int(0.025 * n)], d[int(0.975 * n) - 1]

def all_versions(g, alt=False):
    """Every scored trial: versions to green, or all versions checked when it never went green; at least 1 (a scored trial that
    produced no version is not a better one than a single failed version: zero-version trials with no result are rerun, see
    trial_invalid). alt=True is the secondary figure: a build that rewrote the draft counts as one more version."""
    def one(m):
        if alt and m.get("versions_to_green_alt") is not None: return m["versions_to_green_alt"]
        if m.get("versions_to_green") is not None: return m["versions_to_green"]
        v = m.get("versions_alt") if alt and m.get("versions_alt") is not None else m.get("versions", m.get("fix_rounds"))
        return None if v is None else max(v, 1)
    return [x for x in map(one, g) if x is not None]

POOLED_MARGIN = 0.10  # ADR 0916 amended 2026-11-18: the target is "level on tries": the pooled upper bound of the gap under +0.10 versions

def boot_diff_strat(a, c, n=2000):
    """The 95% interval of mean(a) - mean(c) over all classes, resampling within each class (trials are not exchangeable across classes).
    a and c map class -> list of versions. Fixed seed. Classes with trials in only one arm are left out."""
    import random
    cls = [k for k in a if a[k] and c.get(k)]
    if not cls: return None
    rnd = random.Random(916); d = []
    for _ in range(n):
        xa = [x for k in cls for x in (rnd.choice(a[k]) for _ in a[k])]; xc = [x for k in cls for x in (rnd.choice(c[k]) for _ in c[k])]
        d.append(statistics.mean(xa) - statistics.mean(xc))
    d.sort(); return d[int(0.025 * n)], d[int(0.975 * n) - 1]

def correct_not_lower(g, c):
    """ADR 0916 amended 2026-11-19: AIL is "as correct" when the 95% bootstrap interval of (AIL correct share - C correct share)
    includes zero or lies above it (like the tries rule, by interval, so one trial's difference in a cell of 24 is not a loss).
    Returns (ok, interval); with no trials in either arm, (True, None)."""
    iv = boot_diff([int(m["correct"]) for m in g], [int(m["correct"]) for m in c])
    return (iv is None or iv[1] >= 0), iv

def boot_diff_correct_strat(g, c, classes, n=2000):
    """The same interval over all classes, resampled within class."""
    a = {cl: [int(m["correct"]) for m in g if task_class(m["task"]) == cl] for cl in classes}
    b = {cl: [int(m["correct"]) for m in c if task_class(m["task"]) == cl] for cl in classes}
    return boot_diff_strat(a, b, n)

def target_table(ms, calls, arm="ail-plain"):
    """The pooled line and the verdict against the target of ADR 0916 amended 2026-11-18: no class lost, pooled upper bound of the
    tries gap under +POOLED_MARGIN, correct share not lower by interval (pooled and per class, see correct_not_lower), cost per trial
    lower. `calls` is {class: call} from the per-class table."""
    g = [m for m in ms if m["arm"] == arm]; c = [m for m in ms if m["arm"] == "c"]
    if not g or not c: return
    classes = sorted({task_class(m["task"]) for m in ms})
    by = lambda xs: {cl: all_versions([m for m in xs if task_class(m["task"]) == cl]) for cl in classes}
    iv = boot_diff_strat(by(g), by(c))
    cost = lambda xs: statistics.mean(m.get("charged_usd", m["cost_usd"]) for m in xs)
    share = lambda xs: sum(m["correct"] for m in xs) / len(xs)
    civ = boot_diff_correct_strat(g, c, classes); cls_ok = {}
    for cl in classes:
        cg = [m for m in g if task_class(m["task"]) == cl]; cc = [m for m in c if task_class(m["task"]) == cl]
        if cg and cc: cls_ok[cl] = correct_not_lower(cg, cc)
    bad = [f"{cl} ({v[1][0]:+.2f} to {v[1][1]:+.2f})" for cl, v in cls_ok.items() if not v[0]]
    ok = {"no class lost": "lost" not in calls.values(),
          f"pooled upper bound under {POOLED_MARGIN:+.2f}": bool(iv) and iv[1] < POOLED_MARGIN,
          f"correct share not shown to be lower, pooled (95% interval of the difference {'-' if not civ else f'{civ[0]:+.2f} to {civ[1]:+.2f}'}; includes zero or is above it)": civ is None or civ[1] >= 0,
          "correct share not shown to be lower in any class" + (f" (shown lower: {', '.join(bad)})" if bad else ""): not bad,
          "cost per trial lower": cost(g) < cost(c)}
    ivs = "-" if not iv else f"{iv[0]:+.2f} to {iv[1]:+.2f}"
    print(f"Pooled over all classes ({arm} against c, bootstrap resampled within class): {len(g)} and {len(c)} trials; "
          f"mean versions {statistics.mean(all_versions(g)):.2f} against {statistics.mean(all_versions(c)):.2f}; "
          f"95% interval of the difference {ivs}; "
          f"correct {sum(m['correct'] for m in g)}/{len(g)} against {sum(m['correct'] for m in c)}/{len(c)}, "
          f"95% interval of the difference in share {'-' if not civ else f'{civ[0]:+.2f} to {civ[1]:+.2f}'}; "
          f"cost per trial ${cost(g):.3f} against ${cost(c):.3f}.\n")
    print("Target of the final run (level on tries, cheaper, not shown to be less correct): " + ("**PASSED**" if all(ok.values()) else "**NOT MET**") + "\n")
    for k, v in ok.items(): print(f"- {k}: {'yes' if v else 'NO'}")
    print()
    secondary_table(ms, classes, arm)

def secondary_table(ms, classes, arm="ail-plain"):
    """Two lines declared before the run (ADR 0916 amended 2026-11-19), reported beside the verdict and never part of it:
    (a) tries both ways: mean versions as the verdict counts them, and with a build that rewrote the draft counted as one more version;
    (b) cost both ways: per trial in total, and without the DOCS read (cost_ex_docs: an estimate, formula in its docstring)."""
    print("Secondary figures (not part of the verdict). Versions: the verdict's count, then with a build that rewrote the source counted as one more version. "
          f"Cost per trial: total, then without the DOCS read, estimated as total minus DOCS tokens (bytes / {DOCS_BYTES_PER_TOKEN:g}) x (cache-write price + cache-read price x (turns - 1)), never below the output cost.\n")
    print("| class | arm | trials | mean versions | mean versions, rewrite counted | cost per trial USD | cost per trial without the DOCS read |\n|---|---|--:|--:|--:|--:|--:|")
    mean = lambda xs: f"{statistics.mean(xs):.2f}" if xs else "-"
    for cl in classes + ["all classes"]:
        for ar in (arm, "c"):
            g = [m for m in ms if m["arm"] == ar and (cl == "all classes" or task_class(m["task"]) == cl)]
            if not g: continue
            print(f"| {cl} | {ar} | {len(g)} | {mean(all_versions(g))} | {mean(all_versions(g, True))} | "
                  f"{statistics.mean(m.get('charged_usd', m['cost_usd']) for m in g):.3f} | {statistics.mean(cost_ex_docs(m) for m in g):.3f} |")
    g = [m for m in ms if m["arm"] == arm]; c = [m for m in ms if m["arm"] == "c"]
    by = lambda xs: {cl: all_versions([m for m in xs if task_class(m["task"]) == cl], True) for cl in classes}
    iv = boot_diff_strat(by(g), by(c))
    print(f"\nPooled gap in versions with a rewrite counted (95% interval, resampled within class): {'-' if not iv else f'{iv[0]:+.2f} to {iv[1]:+.2f}'}.\n")

def primary_table(ms):
    """The primary measure (Kaleb 2026-11-15, ADR 0916 amended 2026-11-15c items 9 and 10): iterations to working code. Per
    arm and task class: trials correct on the hidden cases; the share whose first version was green; among the correct
    trials, the mean source versions up to the first passing ./test (versions to green) and the mean fix rounds; the
    versions column averages ALL scored trials (a trial that never went green counts every version it checked, else its fix
    rounds when it predates `versions`), with the correct-only mean beside it (E14 review: a correct-only mean hides the failures); the
    builds that rewrote the source (repair on intake); and against C, the 95% interval of the difference in mean versions
    to green with the call it gives: won (interval below zero and success no lower), lost (above zero), else tied."""
    print("Primary measure: iterations to working code (fewer is better).\n")
    print("| class | arm | trials | correct | first version green | mean versions, all trials | mean versions to green (correct only) | mean fix rounds (correct) | "
          "builds that rewrote src | scratch compiles per trial | DOCS KB before first write (median) | vs c: 95% interval of the difference | call |\n|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|---|")
    mean = lambda xs: f"{statistics.mean(xs):.2f}" if xs else "-"
    vtg = lambda g: [m["versions_to_green"] for m in g if m["correct"] and m.get("versions_to_green") is not None]
    def vall(g):  # every scored trial: versions to green, or all versions checked when it never went green
        v = [m["versions_to_green"] if m.get("versions_to_green") is not None else m.get("versions", m.get("fix_rounds")) for m in g]
        return [x for x in v if x is not None]
    calls = {}
    for cl in ("from scratch", "change", "held out"):
        cg = [m for m in ms if m["arm"] == "c" and task_class(m["task"]) == cl]
        for arm in ARMS:
            g = [m for m in ms if m["arm"] == arm and task_class(m["task"]) == cl]
            if not g: continue
            ok = [m for m in g if m["correct"]]
            iv = boot_diff(vall(g), vall(cg)) if arm != "c" else None
            call = "-"
            if iv:
                lo, hi = iv; better_ok = correct_not_lower(g, cg)[0]  # by interval (ADR 0916 amended 2026-11-19), as the target's correctness line
                call = "won" if hi < 0 and better_ok else ("lost" if lo > 0 else "tied")
                if arm == "ail-plain": calls[cl] = call
            print(f"| {cl} | {arm} | {len(g)} | {len(ok)}/{len(g)} | {sum(m.get('versions_to_green') == 1 for m in g)}/{len(g)} | "
                  f"{mean(vall(g))} | {mean(vtg(g))} | {mean([m['fix_rounds'] for m in ok if m.get('fix_rounds') is not None])} | "
                  f"{sum(m.get('build_rewrites') or 0 for m in g)} | {mean([m['scratch_compiles'] for m in g if 'scratch_compiles' in m])} | "
                  f"{med([m['docs_bytes_before_write'] / 1000 for m in g if 'docs_bytes_before_write' in m])} | {'-' if not iv else f'{iv[0]:+.2f} to {iv[1]:+.2f}'} | {call} |")
    print("\nA pilot makes no call: its cells are too small; the column is for the full run.\n")
    target_table(ms, calls)

def cmd_report(a):
    ms = [json.load(open(os.path.join(dp, "metrics.json"))) for dp, fs in live_dirs(a.results) if "metrics.json" in fs]
    bad = [dp for dp, fs in live_dirs(a.results) if "invalid.json" in fs and "metrics.json" not in fs]
    if bad: print("Invalid (API error, not scored, rerun pending): " + ", ".join(os.path.relpath(d, a.results) for d in sorted(bad)) + "\n")
    for m in ms: m["arm"] = arm_name(m["arm"])
    void = [m for m in ms if (m.get("audit") or {}).get("void")]
    if void: print("Void (read the harness, a hidden case or another trial; left out below):\n" + "".join(f"- {m['task']} {m['arm']} {m['trial']}: {m['audit']['void'][:160]}\n" for m in void))
    ms = [m for m in ms if m not in void]
    if a.plan:  # only the trials a plan numbered, so pilots in the same results folder stay out of a counted table
        rows = plan(a.plan); keep = {(t, arm_name(arm), trial) for t, arm, trial in rows}
        missing = sorted({t for t, _, _ in rows if not os.path.isdir(task_dir(t))})
        if missing: sys.exit(f"report: {len(missing)} task(s) of the plan cannot be loaded ({', '.join(missing)}); with sealed tasks set AGENTBENCH_HELDOUT (names.txt maps the plan's ids). Not reporting under an unknown class.")
        ms = [m for m in ms if (m["task"], m["arm"], str(m["trial"])) in keep]
        # An unfinished round is dropped from the verdict (ADR 0916 amended 2026-11-19): the plan is trial-major, so if the cap or a stop ends
        # the run mid-round, that round's trials are left out of every figure below (a half-round would favour whichever cells ran).
        have = {(m["task"], m["arm"], str(m["trial"])) for m in ms} | {(m["task"], m["arm"], str(m["trial"])) for m in void}
        first_open = min((int(tr) for t, arm, tr in rows if (t, arm_name(arm), tr) not in have), default=None)
        if first_open is not None:
            cut = [m for m in ms if int(m["trial"]) >= first_open]; ms = [m for m in ms if int(m["trial"]) < first_open]
            print(f"Unfinished round: round {first_open} has missing trials, so rounds {first_open} and later ({len(cut)} scored trial(s)) are left out of the tables and the verdict.\n")
    unknown = sorted({m["task"] for m in ms if task_class(m["task"]) == "unknown"})
    if unknown: sys.exit(f"report: no class for task(s) {', '.join(unknown)} (task folder not found)")
    primary_table(ms)
    print("| task | arm | trials | correct | stopped by a cap | median cost USD | median out tokens | median fix rounds | median versions to green | median debug rounds (calls) | median min | interventions | first check clean | median toolchain tokens | median calls to green | used agent tools | median DOCS KB received | median USD cache write / read / output |\n|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|")
    for t in sorted({m["task"] for m in ms}):
        for arm in ARMS:
            g = [m for m in ms if m["task"] == t and m["arm"] == arm]
            if g:
                fc = [m["first_check_clean"] for m in g if m.get("first_check_clean") is not None]
                print(f"| {t} | {arm} | {len(g)} | {sum(m['correct'] for m in g)}/{len(g)} | {sum(bool(m.get('stopped')) for m in g)} | "
                      f"{statistics.median(m.get('charged_usd', m['cost_usd']) for m in g):.2f} | {statistics.median(m['output'] for m in g):.0f} | "
                      f"{med(m.get('fix_rounds') for m in g)} | {med(m.get('versions_to_green') for m in g)} | "
                      f"{statistics.median(m['debug_rounds'] for m in g):.1f} | {statistics.median(m['elapsed_s'] for m in g)/60:.1f} | {sum(m['interventions'] for m in g)} | "
                      f"{sum(fc)}/{len(fc)} | {med(m.get('toolchain_tokens') for m in g)} | {med(m.get('turns_to_green') for m in g)} | {sum(used_agent_tools(m) for m in g)}/{len(g)} | "
                      f"{med([m['docs_bytes'] / 1000 for m in g if 'docs_bytes' in m])} | "
                      f"{med([m['usd_cache_create'] for m in g if 'usd_cache_create' in m], '.3f')} / {med([m['usd_cache_read'] for m in g if 'usd_cache_read' in m], '.3f')} / {med([m['usd_output'] for m in g if 'usd_output' in m], '.3f')} |")

def main():
    p = argparse.ArgumentParser(); s = p.add_subparsers(dest="c", required=True)
    for n, args in (("gen", ["task"]), ("docs", ["task"]), ("run", ["task", "arm", "trial"]), ("batch", ["plan"]), ("score", ["dir"]), ("validate", ["task"]),
                    ("visible", ["dir"]), ("report", []), ("trials", []), ("spend", []), ("mcpproxy", []), ("audit", [])):
        q = s.add_parser(n)
        for x in args: q.add_argument(x)
        if n in ("run", "batch"):
            q.add_argument("--model", default=os.environ.get("AGENTBENCH_MODEL"), required="AGENTBENCH_MODEL" not in os.environ)
            q.add_argument("--max-usd", type=float, required=True, help="hard cap on the whole spend ledger, results/spend.jsonl")
            q.add_argument("--trial-usd", type=float, default=4.0, help="per-trial ceiling; a trial stopped by it counts as a failure")
            if n == "batch": q.add_argument("--ignore-stage", type=int, action="append", help="go on past this plan stop (a stop is the designer's decision; see the plan's @stage lines)")
            q.add_argument("--docs-policy", choices=sorted(DOCS_POLICIES), default="read", help="read (E14, the designer 2026-11-15, ADR 0916 item 14): read DOCS/ first; reference: consult DOCS/ as needed")
        if n == "mcpproxy": q.add_argument("--ail", default=os.environ.get("AIL", "ail"))
        if n in ("report", "trials", "spend", "audit"): q.add_argument("results", nargs="?", default=RESULTS)
        if n == "validate": q.add_argument("--arms", help="comma list, e.g. ail-plain,c; default the task.json \"arms\" list, else every arm")
        if n == "audit": q.add_argument("--all", action="store_true", help="also list trials with nothing to report")
        if n in ("report", "trials"): q.add_argument("--plan", help="report only the trials of plans/PLAN.txt")
        if n == "trials": q.add_argument("--json", action="store_true", help="one object per trial")
    a = p.parse_args(); r = globals()["cmd_" + a.c](a); sys.exit(r if isinstance(r, int) else 0)

if __name__ == "__main__": main()
