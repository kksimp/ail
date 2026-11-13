import os, shutil, tempfile


def parse_layer(name, text):
    pairs = {}
    bad = 0
    for line in text.splitlines():
        t = line.strip()
        if t == "" or t.startswith("#"):
            continue
        cells = t.split("=", 1)
        if len(cells) != 2 or cells[0].strip() == "":
            bad += 1
            continue
        pairs[cells[0].strip()] = cells[1].strip()
    return (name, pairs, bad)


def merge(layers):
    merged = {}
    bad = 0
    for _, pairs, b in layers:
        bad += b
        for k, v in pairs.items():
            merged[k] = v
    return ("merged", merged, bad)


def setting_int(layer, key, fallback):
    text = layer[1].get(key)
    if text is None:
        return fallback
    t = text.strip()
    if t.lstrip("-").isdigit() and t != "":
        return int(t)
    return fallback


def main():
    d = tempfile.mkdtemp(prefix="corpus_config_merge")
    names = ["defaults", "user", "override"]
    texts = [
        "# built-in defaults\nport = 8080\nhost = localhost\nworkers = 2\nlog = info\n",
        "workers = 8\nlog=debug\n\nthis line is junk\ncache_dir = /var/cache\n",
        "port = 9090\n= nameless\nworkers = many\nlog = warn\n",
    ]
    layers = []
    for n, t in zip(names, texts):
        file = os.path.join(d, n + ".conf")
        with open(file, "w", newline="") as f:
            f.write(t)
        with open(file, newline="") as f:
            layers.append(parse_layer(n, f.read()))
        os.remove(file)
    os.rmdir(d)
    origin = {}
    for name, pairs, _ in layers:
        for k in pairs:
            origin[k] = name
    cfg = merge(layers)
    print(f"{'key':<10}{'value':<12}layer")
    for k in sorted(cfg[1]):
        print(f"{k:<10}{cfg[1][k]:<12}{origin[k]}")
    port = setting_int(cfg, "port", 80)
    workers = setting_int(cfg, "workers", 1)
    retries = setting_int(cfg, "retries", 3)
    print(f"port={port} workers={workers} retries={retries}")
    print(f"{len(cfg[1])} keys, {cfg[2]} bad lines")


main()
