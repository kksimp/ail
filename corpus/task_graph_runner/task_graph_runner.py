class Job:
    def __init__(self, name, weight, deps):
        self.name = name
        self.weight = weight
        self.deps = deps


def parse_job(spec):
    parts = spec.split(":")
    deps = parts[2].split(",") if len(parts) > 2 and parts[2] != "" else []
    return Job(parts[0], int(parts[1]), deps)


def compute(weight, inputs):
    v = weight * 31
    for x in inputs:
        v = (v * 7 + x) % 10007
    return v


def ready_wave(jobs, finished):
    wave = []
    for j in jobs:
        if j.name in finished:
            continue
        if all(d in finished for d in j.deps):
            wave.append(j)
    return sorted(wave, key=lambda j: j.name)


def main():
    specs = ["pack:3:build,docs", "build:5:compile,assets", "compile:8:", "assets:2:", "docs:4:compile", "deploy:1:pack,lint", "lint:6:ghost", "a:1:b", "b:1:a"]
    jobs = [parse_job(s) for s in specs]
    finished = set()
    values = {}
    rnd = 1
    while True:
        wave = ready_wave(jobs, finished)
        if not wave:
            break
        results = []
        for job in wave:
            inputs = [values[d] for d in job.deps]
            results.append((job.name, compute(job.weight, inputs)))
        for name, v in results:
            values[name] = v
            finished.add(name)
        print(f"wave {rnd}: {' '.join(j.name for j in wave)}")
        rnd += 1
    for name in sorted(values):
        print(f"{name:<8}{values[name]:>6}")
    stuck = [j.name for j in jobs if j.name not in finished]
    print(f"ran {len(finished)} of {len(jobs)} jobs, stuck: {','.join(stuck)}")


main()
