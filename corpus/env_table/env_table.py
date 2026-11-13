DATA = [
    "# sample environment",
    "USER=ada",
    "HOME=/home/${USER}",
    "PATH=${HOME}/bin:/usr/bin",
    "EDITOR = vi",
    "",
    "LOG_DIR=${HOME}/logs",
    "LOG_FILE=${LOG_DIR}/${APP}.log",
    "APP=demo",
    "BAD LINE",
    "USER=grace",
    "GREETING=hello ${USER} from ${MISSING}!",
]


def expand(s, vars):
    out = ""
    rest = s
    while True:
        i = rest.find("${")
        if i < 0:
            break
        tailpart = rest[i + 2:]
        j = tailpart.find("}")
        if j < 0:
            break
        name = tailpart[:j]
        val = vars.get(name, "")
        out = out + rest[:i] + val
        rest = tailpart[j + 1:]
    return out + rest


def load(lines):
    vars = {}
    bad = 0
    for raw in lines:
        line = raw.strip()
        if line == "" or line.startswith("#"):
            continue
        eq = line.find("=")
        if eq < 0:
            bad += 1
            continue
        key = line[:eq].strip()
        if key == "" or " " in key:
            bad += 1
            continue
        vars[key] = expand(line[eq + 1:].strip(), vars)
    return vars, bad


vars, bad = load(DATA)
for k in sorted(vars):
    print(f"{k:<9}= {vars[k]}")
print(f"{len(vars)} variables, {bad} malformed lines")
refs = [v for v in vars.values() if "/home/ada" in v]
print(f"{len(refs)} values mention the home directory")
