enum Category {
  Food,
  Tool,
  Toy,
}

function label(c: Category): string {
  switch (c) {
    case Category.Food:
      return "food";
    case Category.Tool:
      return "tool";
    case Category.Toy:
      return "toy";
  }
}

interface Item {
  name: string;
  category: Category;
  price: number;
  qty: number;
}

function value(item: Item): number {
  return item.price * item.qty;
}

function restock(items: Item[], min: number): number {
  let added = 0;
  for (const item of items) {
    if (item.qty < min) {
      added += min - item.qty;
      item.qty = min;
    }
  }
  return added;
}

const items: Item[] = [
  { name: "apple", category: Category.Food, price: 3, qty: 10 },
  { name: "hammer", category: Category.Tool, price: 25, qty: 1 },
  { name: "ball", category: Category.Toy, price: 8, qty: 0 },
];
const added = restock(items, 2);
let total = 0;
for (const item of items) {
  console.log(`${item.name} (${label(item.category)}): ${item.qty} x ${item.price}`);
  total += value(item);
}
console.log(`restocked ${added}, stock value ${total}`);
