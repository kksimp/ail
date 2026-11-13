LEVELS = 4


def height(key):
    h = (key * 2654435761) % 4294967296
    lv = 1
    while lv < LEVELS and h % 2 == 0:
        lv += 1
        h //= 2
    return lv


keys = [-1]
links = [-1] * LEVELS
count = 0


def before(key, level, start):
    cur = start
    while True:
        nx = links[cur * LEVELS + level]
        if nx == -1:
            break
        if keys[nx] >= key:
            break
        cur = nx
    return cur


def insert(key):
    global count
    update = [0] * LEVELS
    cur = 0
    for i in range(LEVELS):
        lv = LEVELS - 1 - i
        cur = before(key, lv, cur)
        update[lv] = cur
    nxt = links[cur * LEVELS]
    if nxt != -1 and keys[nxt] == key:
        return False
    ident = len(keys)
    h = height(key)
    keys.append(key)
    links.extend([-1] * LEVELS)
    for lv2 in range(h):
        links[ident * LEVELS + lv2] = links[update[lv2] * LEVELS + lv2]
        links[update[lv2] * LEVELS + lv2] = ident
    count += 1
    return True


def contains(key):
    cur = 0
    for i in range(LEVELS):
        cur = before(key, LEVELS - 1 - i, cur)
    nxt = links[cur * LEVELS]
    if nxt == -1:
        return False
    return keys[nxt] == key


def to_list():
    out = []
    cur = links[0]
    while cur != -1:
        out.append(keys[cur])
        cur = links[cur * LEVELS]
    return out


def level_size(level):
    n = 0
    cur = links[level]
    while cur != -1:
        n += 1
        cur = links[cur * LEVELS + level]
    return n


dupes = 0
for k in [42, 7, 19, 88, 3, 56, 7, 71, 25, 64, 11, 90, 42, 33]:
    if not insert(k):
        dupes += 1
print(f"keys: {to_list()}")
print(f"count {count}, duplicates {dupes}")
for lv in range(LEVELS):
    print(f"level {lv}: {level_size(lv)} nodes")
for probe in [19, 20, 90, 1, 100]:
    print(f"contains {probe}: {str(contains(probe)).lower()}")
