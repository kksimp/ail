import subprocess

PLAN = """
echo build ok 2000
false x 2000
sleep 5 200
no-such-tool x 2000
echo tests ok 2000
"""


def parse(line):
    words = line.split(" ")
    if len(words) < 2:
        return None
    return words[0], words[1:-1], int(words[-1])


def run_step(cmd, args, limit):
    try:
        r = subprocess.run([cmd, *args], capture_output=True, text=True, timeout=limit / 1000)
    except subprocess.TimeoutExpired:
        print(f"{cmd}: timed_out")
        return False
    except FileNotFoundError:
        print(f"{cmd}: not_found")
        return False
    print(f"{cmd}: {r.stdout.strip() if r.returncode == 0 else f'exit {r.returncode}'}")
    return r.returncode == 0


def main():
    bad = 0
    for line in PLAN.splitlines():
        step = parse(line)
        if step is None:
            continue
        if not run_step(*step):
            bad += 1
    print(f"{bad} steps failed")


def test_parses_a_step():
    assert parse("sleep 5 200")[2] == 200
    assert parse("x") is None


if __name__ == "__main__":
    main()
