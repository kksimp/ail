"""Copy examples/NAME.ail into corpus/programs/NAME/NAME.ail for measurement.

Measurement only (ADR 0070). Comments, blank lines and `@test` functions are
removed, as in the hand-made copies of earlier rounds, so the AIL file holds
the same program as its Python twin. The agent-tool programs (ADR 0217) keep
their tests and blank lines, as their twins do; only comment lines go.

Usage (from the repository root): python3 tools/tokbench/corpus_from_examples.py [SRC_DIR]
SRC_DIR defaults to examples/; a program is copied only if its corpus directory exists.
"""

import pathlib
import re
import sys

HERE = pathlib.Path(__file__).resolve().parent
AGENT = {"logscan", "tasks", "build", "snapdiff", "csvreport", "scan", "tail", "merge", "digest", "runner",
         "logwatch", "backup", "poller", "supervise", "report_dates", "hexdump", "route", "perms", "csv2json", "matrix",
         "digest_lines", "regex_codes", "redact_mail", "gz_check", "shift_dates", "days_left", "frames", "hex_base64",
         "task_sum", "chan_pipe"}
# Round 5 (lane 224): every tool-driving program keeps its tests and blank lines, like its twin.
AGENT_PREFIX = "tool_"


def strip(src):
    out = []
    in_test = False
    for line in src.split("\n"):
        if line.startswith("@test "):
            in_test = True
            continue
        if in_test and (line.startswith(" ") or not line.strip()):
            continue
        in_test = False
        if not line.strip() or re.match(r"^\s*#", line):
            continue
        out.append(line)
    return "\n".join(out) + "\n"


def main():
    src_dir = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else "examples")
    for d in sorted((HERE / "corpus" / "programs").iterdir()):
        f = src_dir / f"{d.name}.ail"
        if d.is_dir() and f.exists():
            src = f.read_text()
            if d.name in AGENT or d.name.startswith(AGENT_PREFIX):
                text = "".join(l for l in src.splitlines(keepends=True) if not l.lstrip().startswith("#"))
            else:
                text = strip(src)
            (d / f"{d.name}.ail").write_text(text)
            print(f"{f} -> {d / (d.name + '.ail')}")


if __name__ == "__main__":
    main()
