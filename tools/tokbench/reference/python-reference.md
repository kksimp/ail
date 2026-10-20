# Python reference card

Python 3.12 on one page, for an agent writing scripts and tools. Covers the
same areas as the AIL card: running, layout, functions, bindings, types,
classes and enums, absence and errors, generics, control flow, operators and
strings, lists, dicts and sets, mutation, tasks, contracts, tests, packages
and the system.

## Running programs

    python3 prog.py [ARGS...]         run a file; sys.argv[1:] are its args
    python3 -m pytest -q prog.py      run test_* functions and their asserts
    python3 -m py_compile prog.py     check syntax only
    python3 -m doctest prog.py        run examples in docstrings
    python3 -m mypy prog.py           optional static type check (third party)
    if __name__ == "__main__": main()   run main only when executed directly

## Layout

Indentation (4 spaces) is the block structure. A colon opens a block.
One statement per line. `#` starts a comment. Strings may be `"..."`,
`'...'`, `"""..."""` (multi-line) and `f"..."` (interpolation).

## Functions

    def area(w: float, h: float = 1.0) -> float:
        return w * h

    area(2, h=3)                  keyword arguments by name
    def tags(*names, **opts): ... variadic positional and keyword arguments
    lambda x: x + 1               anonymous function, one expression
    def f(xs: list[int]) -> None: no value returned
    @functools.cache              memoise a pure function

Functions are first-class values and closures capture variables by name.

## Bindings

    x = 3                         binding; rebinding is allowed
    x: int = 3                    annotation, checked only by mypy
    x, y = y, x                   unpacking
    first, *rest = xs             star unpacking
    global counter / nonlocal n   assign to an outer name

## Types

int (unbounded), float (IEEE 64-bit), bool, str (Unicode text),
bytes, None, list, tuple (immutable), dict, set, frozenset.
`int / int` gives a float; `int // int` floors. `divmod(a, b)` gives both.
`float("2.5")`, `int("12")`, `str(12)`. `int("x")` raises ValueError.
Booleans are ints: `True + 1 == 2`.

## Classes, dataclasses and enums

    @dataclass(frozen=True)
    class Point:
        x: int
        y: int = 0

    Point(1, y=2)                 construct by position or name
    p.x, p.y                      fields; frozen=True forbids assignment
    dataclasses.replace(p, y=5)   copy with changed fields

    class Shape:
        def area(self) -> float: ...     methods take self explicitly
        @staticmethod / @classmethod

    class Color(enum.Enum):
        RED = "red"
        BLUE = "blue"
    Color("red")                  lookup by value; Color.RED.value

Methods and attributes are found at run time; a misspelled name raises
AttributeError when the line runs, not before.

## Absence and errors

    value = d.get("key")          None when missing, no error
    value = d.get("key", 0)       default instead of None
    x if x is not None else 0    test for None explicitly
    try:
        n = int(text)
    except ValueError as err:
        print(f"bad number: {err}")
    else:
        use(n)
    finally:
        close()
    raise ValueError("message")   fail with an exception
    class BadRow(Exception): ...  custom exception class

Exceptions are not in the function signature. Each line that can fail must
be guarded by try or by a value check. Uncaught exceptions print a traceback
and exit with status 1.

## Generics and protocols

    from typing import TypeVar, Protocol
    T = TypeVar("T")
    def first(xs: list[T]) -> T: ...
    class Named(Protocol):
        def name(self) -> str: ...

Generics and protocols are checked by mypy, not by the interpreter.

## Control flow

    if n < 0: ... elif n == 0: ... else: ...
    while cond: ... break / continue
    for i, item in enumerate(xs): ...
    for i in range(len(xs)): ...   index loop
    for k in d: ...                 iterate keys
    match value:                    structural match (3.10+)
        case 200: ...
        case {"kind": "ok", "body": body}: ...
        case _: ...
    x = a if cond else b          conditional expression
    [f(x) for x in xs if x > 0]   comprehension

## Operators and strings

`+ - * / // % **` arithmetic; `== != < <= > >=` comparison;
`and or not` boolean; `in` membership; `is` identity; `:=` assignment expression.
Strings: `s.split(",")`, `s.splitlines()`, `s.strip()`, `s.startswith("x")`,
`s.endswith("x")`, `"x" in s`, `s.replace(a, b)`, `", ".join(parts)`,
`s[1:4]` slices, `s[::-1]` reversed, `s.encode("utf-8")` to bytes.
f-strings: `f"{x:>8}"` right-align, `f"{x:08.3f}"` zero-pad float,
`f"{x!r}"` repr. Concatenation with `+` works only for str with str.

## Lists

    xs = [1, 2, 3]
    xs.append(4); xs.extend([5, 6]); xs.insert(0, 0)
    xs.pop(); xs.remove(3); del xs[0]
    xs[-1]                         last item; xs[10] raises IndexError
    xs[1:3]                        new list, copy of a slice
    sorted(xs, key=lambda x: -x)   new sorted list, stable
    xs.sort(reverse=True)          in place
    len(xs), sum(xs), min(xs), max(xs)
    list(zip(a, b))                pairs; zip stops at the shorter list
    [(i, x) for i, x in enumerate(xs)]

## Dicts and sets

    d = {"ann": 31}
    d["bob"] = 27                  add or replace
    d["cy"]                        raises KeyError when missing
    d.get("cy", 0)                 default when missing
    d.setdefault("dan", []).append(1)
    d.keys(), d.values(), d.items()  views in insertion order
    sorted(d, key=lambda k: -d[k])
    collections.Counter(words)     counts with missing keys as 0
    collections.defaultdict(int)   missing keys start at 0
    s = {1, 2}; s.add(3); s | t; s & t; s - t
    dict(zip(keys, values))

## Mutation and copies

Assignment binds a name to the same object. Mutable objects (list, dict,
set, dataclass without frozen) change for every name bound to them.
`copy.copy(x)` copies the top level; `copy.deepcopy(x)` copies all levels.
Function arguments are passed by object reference: appending to a list
argument changes the caller's list. Tuples, strings and ints do not change.

## Tasks and concurrency

    from concurrent.futures import ThreadPoolExecutor
    with ThreadPoolExecutor(max_workers=4) as pool:
        results = list(pool.map(work, items))
    import asyncio
    async def fetch(url): ...      await inside async functions only
    asyncio.run(main())
    threading.Lock()               guard shared state by hand
    queue.Queue()                  thread-safe channel

The global interpreter lock lets only one thread run Python bytecode at a
time; CPU-bound work needs processes (concurrent.futures.ProcessPoolExecutor).

## Contracts and checks

    assert x >= 0, "x must not be negative"    removed under python -O
    typing.assert_never(value)                 mypy: exhaustive match check

## Tests

    def test_sums():
        assert totals("a,1") == {"a": 1}
    python3 -m pytest -q prog.py       shows the failing assert and values
    pytest.raises(ValueError)           expect an exception
    pytest.mark.parametrize("x,y", [(1, 2), (3, 4)])   one test per row

## Files, processes and the system

    from pathlib import Path
    text = Path("in.txt").read_text(encoding="utf-8")   raises FileNotFoundError
    Path("out.txt").write_text(text)
    Path("dir").mkdir(parents=True, exist_ok=True)
    sorted(Path("src").rglob("*.py"))
    os.environ.get("HOME")         None when unset
    sys.argv                       program arguments
    subprocess.run(["git", "status"], capture_output=True, text=True,
                   timeout=5)      no shell; raises TimeoutExpired
    result.returncode, result.stdout, result.stderr
    urllib.request.urlopen(url, timeout=5)   HTTP(S); raises URLError
    json.loads(text); json.dumps(obj, separators=(",", ":"))
    time.time()                    wall clock in seconds
    random.seed(1); random.randint(1, 6)

## Packages and modules

A module is a .py file; a package is a directory with __init__.py.
`import a.b` and `from a.b import name` load modules once. Code at the top
level of a module runs on import. Third-party packages come from pip into a
virtual environment: `python3 -m venv .venv && .venv/bin/pip install pkg`.

## More on functions and arguments

    def connect(host: str, port: int = 80, *, timeout: float = 5.0) -> None:
        ...                        arguments after * are keyword-only
    def log(*parts: object) -> None:
        print(*parts, sep=" ")
    def apply(f, *args, **kwargs):
        return f(*args, **kwargs)  forward any arguments unchanged
    def counter():
        n = 0
        def step() -> int:
            nonlocal n
            n += 1
            return n
        return step                closure keeps n between calls
    functools.partial(int, base=16)  a function with some arguments fixed
    functools.reduce(lambda a, b: a + b, xs, 0)  fold a list to one value
    map(str, xs), filter(None, xs)  lazy; wrap in list() to materialise
    sum(x * x for x in xs)           generator expression, no list built
    def chunks(xs, n):
        for i in range(0, len(xs), n):
            yield xs[i:i + n]          generator: produces values one by one

Default argument values are evaluated once, when the def runs. A mutable
default (`def f(xs=[])`) is shared by every call; use None and create the list
inside the body.

## More on classes

    class Account:
        count = 0                    class attribute shared by all instances
        def __init__(self, owner: str, balance: int = 0) -> None:
            self.owner = owner
            self.balance = balance
            Account.count += 1
        def __repr__(self) -> str:
            return f"Account({self.owner!r}, {self.balance})"
        def __eq__(self, other: object) -> bool:
            return isinstance(other, Account) and self.owner == other.owner
        def deposit(self, amount: int) -> None:
            if amount <= 0:
                raise ValueError("amount must be positive")
            self.balance += amount

    class Savings(Account):          inheritance; super() calls the parent
        def deposit(self, amount: int) -> None:
            super().deposit(amount)

    isinstance(acct, Account)        type test, includes subclasses
    hasattr(acct, "balance")         attribute test
    getattr(acct, "owner", "?")      read with a default
    __slots__ = ("x", "y")           fixed attributes, less memory

Dunder methods give operators meaning: `__add__` for +, `__lt__` for <,
`__len__` for len(), `__iter__` for for loops, `__enter__` and `__exit__`
for with statements.

## Dataclasses in depth

    @dataclass(order=True)
    class Task:
        priority: int
        name: str = field(compare=False)
        tags: list[str] = field(default_factory=list)

    tasks = sorted([Task(2, "b"), Task(1, "a")])  ordering from the fields
    asdict(task)                       dict of fields, nested dataclasses too
    fields(task)                       tuple of Field objects
    Task(1, "a") == Task(1, "a")       equality from the fields

A dataclass with default_factory gives each instance its own list. A plain
default of a list raises ValueError at class definition.

## Enums in depth

    class Status(enum.Enum):
        OK = 200
        MISSING = 404

    Status.OK.name                    "OK"
    Status(404)                       Status.MISSING; ValueError if unknown
    [s.name for s in Status]          iteration in definition order
    class Level(enum.IntEnum): LOW = 1; HIGH = 2   compares as an int
    class Mode(enum.Flag): R = 4; W = 2; X = 1     bit flags, combine with |

## Errors in depth

    class ConfigError(Exception):
        def __init__(self, key: str) -> None:
            super().__init__(f"missing key {key}")
            self.key = key

    try:
        value = config["port"]
    except KeyError as err:
        raise ConfigError("port") from err   keep the cause in the traceback

    with open("log.txt", "a") as fh:  close the file when the block ends
        fh.write("done\n")

    contextlib.suppress(FileNotFoundError)   ignore one exception type
    exc.args, str(exc)                       message text
    traceback.format_exc()                   the full traceback as a string

Common exceptions: KeyError (dict key), IndexError (list index), ValueError
(bad value), TypeError (wrong type in an operation), AttributeError (no such
attribute), ZeroDivisionError, FileNotFoundError, PermissionError,
TimeoutExpired (subprocess), URLError (urllib). A traceback names the file
and line of the failing expression and then the line itself.

## Strings in depth

    s.find("x") -> -1 when missing      s.index("x") raises ValueError
    s.count("ab")                        non-overlapping matches
    s.zfill(4)                           pad with zeros on the left
    s.ljust(10), s.rjust(10), s.center(10)
    s.title(), s.upper(), s.lower(), s.casefold()
    s.isdigit(), s.isalpha(), s.isspace()
    s.partition("=") -> (before, sep, after)
    s.removeprefix("v"), s.removesuffix(".txt")   Python 3.9+
    "".join(reversed(s))                 reverse without slicing
    s.split()                            split on any whitespace run
    s.encode("utf-8").decode("utf-8")    round trip through bytes
    ord("a") == 97, chr(97) == "a"
    unicodedata.normalize("NFC", s)     canonical form for comparison

Strings are immutable: `s[0] = "x"` raises TypeError; build a new string.
Building a string in a loop is best done with a list and "".join.

## Numbers in depth

    round(2.5) == 2                     banker's rounding to even
    round(2.675, 2)                      2.67: binary floats are inexact
    decimal.Decimal("0.10") + decimal.Decimal("0.20")   exact decimal sums
    math.floor(x), math.ceil(x), math.isclose(a, b)
    abs(-3), pow(2, 10, 1000)            modular power
    int("ff", 16), format(255, "x")      hex in and out
    float("nan") != float("nan")         NaN never equals anything
    math.inf, -math.inf                  infinities

Integer overflow does not exist: ints grow as needed, so a loop that
multiplies can use a lot of memory but never wraps.

## Lists, dicts and sets in depth

    xs[::2]                              every second item
    xs[::-1]                             reversed copy
    xs.index(3); xs.count(3)             first position; number of matches
    list(itertools.chain(a, b))          concatenate iterables
    itertools.groupby(sorted(xs))         consecutive equal items
    heapq.nsmallest(3, xs)               the three smallest without a full sort
    bisect.bisect_left(sorted_xs, 4)    position in a sorted list
    d | other                            merged dict (3.9+); right side wins
    {k: v for k, v in d.items() if v}   dict comprehension
    {x for x in xs}                      set comprehension
    frozenset(xs)                        hashable set, usable as a dict key
    d.pop("key", None)                   remove and return, or None
    d.update(other); d.clear()
    collections.OrderedDict              order matters for equality here
    collections.deque(maxlen=3)          fixed-size window, O(1) at both ends

A dict key must be hashable: str, int, tuple of hashables. A list is not
hashable; `d[[1, 2]] = x` raises TypeError.

## Iteration and generators in depth

    for i, (name, age) in enumerate(people, start=1): ...
    for a, b in zip(xs, xs[1:]): ...     pairs of neighbours
    any(x < 0 for x in xs), all(xs)      short-circuit tests
    next(iter(xs), None)                  first item or None
    itertools.islice(gen, 10)            first ten values of a generator
    itertools.product("ab", repeat=2)   cartesian product
    yield from other_gen()               delegate to another generator

Iterating a generator uses it up; a second loop over the same generator
produces nothing. Store it in a list when it is needed twice.

## Files and text in depth

    with open("data.csv", newline="", encoding="utf-8") as fh:
        for row in csv.reader(fh):       parsed rows, quotes handled
            print(row)
    csv.DictReader(fh)                   rows as dicts keyed by the header
    csv.writer(out).writerow(row)
    Path("a").stat().st_size            size in bytes
    Path("a").stat().st_mtime           modified time, seconds since epoch
    Path("a").exists(), Path("a").is_dir()
    Path("a").unlink()                  remove a file
    shutil.copy(a, b); shutil.rmtree(d)
    tempfile.TemporaryDirectory()       removed on exit
    glob.glob("src/**/*.py", recursive=True)

## Processes in depth

    proc = subprocess.run(["sort"], input=text, capture_output=True,
                          text=True, timeout=5)
    proc.returncode != 0 is a failed command, not an exception
    subprocess.run(cmd, check=True)     raises CalledProcessError on failure
    shutil.which("git")                  path of an executable or None
    os.getpid(), os.cpu_count()
    sys.exit(2)                          exit with a status
    signal.alarm(5)                      interrupt after five seconds (Unix)

## Time, randomness and determinism

    datetime.datetime.now(datetime.timezone.utc)   wall clock, UTC
    datetime.datetime.fromtimestamp(ts, tz=datetime.timezone.utc)
    datetime.date(2026, 10, 20) + datetime.timedelta(days=3)
    time.perf_counter()                  elapsed time measurement
    random.Random(42).randint(1, 6)     a generator with its own seed
    uuid.uuid4()                         random identifier (not repeatable)
    hashlib.sha256(data).hexdigest()     digest of bytes

Results that depend on the clock or on random numbers differ between runs
unless the seed or the clock is passed in as an argument.

## Typing in depth

    from typing import Literal, TypedDict, Optional, Union, Callable
    Mode = Literal["r", "w"]
    class Row(TypedDict):
        name: str
        age: int
    def open_it(m: Mode) -> Optional[str]: ...
    handler: Callable[[int], str]
    Union[int, str]                      or int | str (3.10+)
    cast(int, value)                     tell the checker to trust a value

Annotations do nothing at run time. A wrong annotation runs fine until a
call with the wrong type fails somewhere else, possibly far away.

## Testing in depth

    @pytest.fixture
    def db():
        conn = connect(":memory:")
        yield conn                       setup, value, then teardown
        conn.close()
    def test_insert(db):
        assert db.count() == 0
    pytest.approx(0.3) == 0.1 + 0.2      floats compared with a tolerance
    with pytest.raises(KeyError, match="port"): ...
    capsys fixture                       captures printed output
    tmp_path fixture                     a fresh temporary directory per test
    python3 -m pytest -q -x              stop at the first failure
    python3 -m pytest -k sums            only tests whose names match

A failing assert shows the expression and the values on both sides. Pytest
rewrites asserts to show intermediate values, so plain assert is enough.

## Packages in depth

    project/
        pyproject.toml                   name, version, dependencies
        src/app/__init__.py              package marker
        src/app/core.py                  module app.core
        tests/test_core.py               run with pytest from the root
    from app.core import compute         absolute import from the package
    from . import helpers                relative import inside a package
    python3 -m app                       run the package's __main__.py

Dependencies are pinned in a lock file by pip-tools or uv. Without a lock
file, two installs of the same project may get different versions.

## Worked examples

Sum amounts per customer from CSV text:

    from collections import defaultdict
    def totals(csv: str) -> dict[str, float]:
        out: dict[str, float] = defaultdict(float)
        for line in csv.splitlines():
            name, amount = line.split(",")
            out[name] += float(amount)
        return dict(out)

Retry a fetch with a time limit, reporting failures as values:

    def fetch(url: str, retries: int = 2) -> str | None:
        for _ in range(retries + 1):
            try:
                with urllib.request.urlopen(url, timeout=5) as r:
                    return r.read().decode("utf-8")
            except (urllib.error.URLError, TimeoutError):
                continue
        return None

Run a list of jobs with a time limit each:

    def run_job(cmd: list[str], limit_s: float) -> str:
        try:
            r = subprocess.run(cmd, capture_output=True, text=True, timeout=limit_s)
        except subprocess.TimeoutExpired:
            return f"timed out after {limit_s} s"
        except FileNotFoundError:
            return "could not start"
        return "ok" if r.returncode == 0 else f"exit {r.returncode}: {r.stderr.strip()}"

Read a JSON config with defaults and a typed result:

    @dataclass
    class Config:
        url: str
        retries: int = 2

    def load(text: str) -> Config:
        cfg = json.loads(text)
        return Config(url=cfg["url"], retries=int(cfg.get("retries", 2)))

Parse a log line into a record, skipping bad lines:

    @dataclass
    class Entry:
        level: str
        message: str

    def parse(line: str) -> Entry | None:
        parts = line.split(" ", 2)
        if len(parts) < 3:
            return None
        return Entry(level=parts[1], message=parts[2])

Count words per line of text:

    def word_count(text: str) -> int:
        return sum(len(line.split()) for line in text.splitlines())

Average a list, zero when empty:

    def average(xs: list[int]) -> int:
        if not xs:
            return 0
        return sum(xs) // len(xs)
