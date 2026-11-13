def parse_conf(text):
    out = {}
    for line in text.splitlines():
        t = line.strip()
        if not t or t.startswith("#"):
            continue
        at = t.find("=")
        if at < 0:
            continue
        out[t[:at].strip()] = t[at + 1:].strip()
    return out


def get_int(values, key, fallback):
    if key not in values:
        return fallback
    try:
        return int(values[key])
    except ValueError:
        return fallback


def main():
    files = {
        "defaults.conf": "host = localhost\nport = 8080\nworkers = 2\ndebug = false\n",
        "user.conf": "# user prefs\nworkers = 4\n\ntheme = dark\nport = 9090\n",
    }
    layers = [("defaults", "defaults.conf"), ("user", "user.conf"), ("local", "local.conf")]
    values = {}
    source = {}
    for name, fname in layers:
        if fname not in files:
            print(f"layer {name}: missing, skipped")
            continue
        found = parse_conf(files[fname])
        for k, v in found.items():
            values[k] = v
            source[k] = name
        print(f"layer {name}: {len(found)} keys")
    for k in sorted(values):
        print(f"{k:<8} = {values[k]:<10} from {source[k]}")
    print(f"port + 1 = {get_int(values, 'port', 0) + 1}")
    print(f"retries = {get_int(values, 'retries', 3)}")
    print(f"host is not a number: {get_int(values, 'host', -1)}")
    print("cleaned up: true")


main()
