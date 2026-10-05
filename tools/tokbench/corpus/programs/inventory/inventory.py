from dataclasses import dataclass
from enum import Enum, auto


class Category(Enum):
    FOOD = auto()
    TOOL = auto()
    TOY = auto()

    def label(self) -> str:
        match self:
            case Category.FOOD:
                return "food"
            case Category.TOOL:
                return "tool"
            case Category.TOY:
                return "toy"


@dataclass
class Item:
    name: str
    category: Category
    price: int
    qty: int

    def value(self) -> int:
        return self.price * self.qty


def restock(items: list[Item], minimum: int) -> int:
    added = 0
    for item in items:
        if item.qty < minimum:
            added += minimum - item.qty
            item.qty = minimum
    return added


items = [
    Item(name="apple", category=Category.FOOD, price=3, qty=10),
    Item(name="hammer", category=Category.TOOL, price=25, qty=1),
    Item(name="ball", category=Category.TOY, price=8, qty=0),
]
added = restock(items, 2)
total = 0
for item in items:
    print(f"{item.name} ({item.category.label()}): {item.qty} x {item.price}")
    total += item.value()
print(f"restocked {added}, stock value {total}")
