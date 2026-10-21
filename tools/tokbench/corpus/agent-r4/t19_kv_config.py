cfg = {}
for line in open("app.conf"):
    line = line.strip()
    if line and not line.startswith("#"):
        k, v = line.split("=", 1)
        cfg[k.strip()] = v.strip()
print(cfg.get("port", "8080"))
