seed = 11
parts = []
levels = ["info", "warn", "error", "debug"]
for i in range(200000):
    seed = (seed * 3877 + 29573) % 139968
    parts.append(f"2026-10-21T07:{i % 60:02} {levels[seed % 4]} user={seed % 97} ms={seed % 1000} GET /api/v{seed % 5}/items")
text = "\n".join(parts)
by_level = {}
by_user = {}
total = 0
for line in text.splitlines():
    f = line.split(" ")
    by_level[f[1]] = by_level.get(f[1], 0) + 1
    by_user[f[2]] = by_user.get(f[2], 0) + 1
    total += int(f[3][3:])
for l in sorted(by_level):
    print(f"{l} {by_level[l]}")
print(f"users {len(by_user)} total_ms {total}")
