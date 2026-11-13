def apply_filter(value, flt):
    if flt == "upper":
        return value.upper()
    if flt == "lower":
        return value.lower()
    return value


def fill(text, variables):
    out = ""
    missing = []
    rest = text
    while True:
        i = rest.find("[[")
        if i < 0:
            break
        tail = rest[i + 2:]
        j = tail.find("]]")
        if j < 0:
            break
        parts = tail[:j].split("|")
        key = parts[0].strip()
        val = f"<{key}?>"
        if key in variables:
            val = variables[key]
            if len(parts) > 1:
                val = apply_filter(val, parts[1].strip())
        else:
            missing.append(key)
        out = f"{out}{rest[:i]}{val}"
        rest = tail[j + 2:]
    out = f"{out}{rest}"
    return out, missing


def main():
    variables = {"name": "Ada Lovelace", "team": "Compilers", "city": "London"}
    letter = [
        "Dear [[name]],",
        "Welcome to the [[ team|upper ]] team in [[city|lower]].",
        "Your badge says [[badge]] and your desk is [[desk]].",
        "Unclosed [[marker stays as is.",
    ]
    for line in letter:
        out, missing = fill(line, variables)
        print(out)
        if missing:
            print("  missing: " + ", ".join(missing))


main()
