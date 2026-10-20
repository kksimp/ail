import subprocess

JOBS = [
    ("echo", ["hello"], 1000),
    ("sh", ["-c", "echo oops >&2; exit 3"], 1000),
    ("sleep", ["5"], 200),
    ("no-such-tool-xyz", [], 1000),
]


def run_job(cmd, args, limit):
    try:
        r = subprocess.run([cmd, *args], input="", capture_output=True, text=True, timeout=limit / 1000)
    except subprocess.TimeoutExpired:
        return f"timed out after {limit} ms"
    except FileNotFoundError:
        return "could not start"
    if r.returncode == 0:
        return "ok"
    return f"exit {r.returncode}: {r.stderr.strip()}"


def main():
    failed = 0
    for cmd, args, limit in JOBS:
        result = run_job(cmd, args, limit)
        if result != "ok":
            failed += 1
        print(f"{cmd:<18}{result}")
    print(f"{failed} of {len(JOBS)} jobs failed")


if __name__ == "__main__":
    main()
