from dataclasses import dataclass


class CommandError(Exception):
    def __str__(self) -> str:
        name = f"CommandError.{type(self).__name__}"
        fields = ", ".join(f'{k}: "{v}"' for k, v in vars(self).items())
        return f"{name}({fields})" if fields else name


class Empty(CommandError):
    pass


@dataclass
class UnknownVerb(CommandError):
    verb: str


@dataclass
class MissingArgument(CommandError):
    verb: str


@dataclass
class BadQuantity(CommandError):
    text: str


@dataclass
class Add:
    item: str
    qty: int


@dataclass
class Remove:
    item: str


@dataclass
class Show:
    pass


Command = Add | Remove | Show


@dataclass
class Entry:
    item: str
    qty: int


def parse(line: str) -> Command:
    words = line.split(" ")
    verb = words[0]
    if verb == "":
        raise Empty()
    if verb == "show":
        return Show()
    if len(words) < 2:
        raise MissingArgument(verb)
    if verb == "remove":
        return Remove(words[1])
    if verb != "add":
        raise UnknownVerb(verb)
    if len(words) < 3:
        raise MissingArgument(verb)
    try:
        qty = int(words[2])
    except ValueError:
        raise BadQuantity(words[2]) from None
    return Add(words[1], qty)


def apply(stock: list[Entry], cmd: Command) -> None:
    match cmd:
        case Add(item, qty):
            for entry in stock:
                if entry.item == item:
                    entry.qty += qty
                    return
            stock.append(Entry(item, qty))
        case Remove(item):
            for entry in stock:
                if entry.item == item:
                    entry.qty = 0
        case Show():
            for entry in stock:
                if entry.qty > 0:
                    print(f"  {entry.item}: {entry.qty}")


def total_added(lines: list[str]) -> int:
    total = 0
    for line in lines:
        try:
            cmd = parse(line)
        except CommandError as err:
            err.add_note(f'reading "{line}"')
            raise
        match cmd:
            case Add(_, qty):
                total += qty
    return total


lines = [
    "add milk 2",
    "add eggs 12",
    "add milk 1",
    "remove eggs",
    "add tea lots",
    "sell milk 1",
    "",
    "show",
]
stock = [Entry("bread", 1)]
for line in lines:
    try:
        cmd = parse(line)
    except CommandError as err:
        print(f"error: {err}")
        continue
    apply(stock, cmd)
print(f"added: {total_added(lines[0:4])}")
