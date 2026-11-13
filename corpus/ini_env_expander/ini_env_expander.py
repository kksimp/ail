import os, tempfile


def parse_ini(text):
    out = []
    section = ""
    for raw in text.splitlines():
        line = raw.strip()
        if line == "" or line.startswith("#"):
            continue
        if line.startswith("[") and line.endswith("]"):
            section = line[1:-1]
            continue
        at = line.find("=")
        if at < 0:
            continue
        out.append((section, line[:at].strip(), line[at + 1:].strip()))
    return out


def expand(text, env, conf, depth):
    if depth > 4:
        return "<loop>"
    out = ""
    rest = text
    while True:
        i = rest.find("${")
        if i < 0:
            break
        after = rest[i + 2:]
        j = after.find("}")
        if j < 0:
            break
        out = out + rest[:i]
        spec = after[:j]
        rest = after[j + 1:]
        name = spec
        fallback = ""
        k = spec.find(":-")
        if k >= 0:
            name = spec[:k]
            fallback = spec[k + 2:]
        if name in env:
            out = out + env[name]
        elif name in conf:
            out = out + expand(conf[name], env, conf, depth + 1)
        else:
            out = out + fallback
    return out + rest


def main():
    d = tempfile.mkdtemp(prefix="corpus_ini_env_expander")
    p = os.path.join(d, "app.ini")
    lines = [
        "# deployment settings",
        "[paths]",
        "root = ${HOME_DIR:-/srv}/app",
        "logs = ${root}/logs",
        "cache = ${CACHE_DIR}/cache-${ENV_NAME:-dev}",
        "[service]",
        "url = http://${HOST:-localhost}:${PORT:-80}/${ENV_NAME}",
        "broken = ${missing_close",
        "ping = ${pong}",
        "pong = ${ping}",
    ]
    with open(p, "w", newline="") as f:
        f.write("\n".join(lines))
    with open(p, newline="") as f:
        entries = parse_ini(f.read())
    env = {"HOME_DIR": "/home/ci", "ENV_NAME": "prod", "PORT": "8443"}
    conf = {}
    for sec, name, raw in entries:
        conf[name] = raw
    print(f"{len(entries)} entries, {len(env)} env vars")
    for sec, name, raw in entries:
        print(f"[{sec}] {name} = {expand(raw, env, conf, 0)}")
    os.remove(p)
    os.rmdir(d)
    print(f"cleaned up: {str(not os.path.exists(d)).lower()}")


main()
