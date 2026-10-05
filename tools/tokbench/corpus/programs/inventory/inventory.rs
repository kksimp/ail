#[derive(Clone, Copy)]
enum Category {
    Food,
    Tool,
    Toy,
}

impl Category {
    fn label(self) -> &'static str {
        match self {
            Category::Food => "food",
            Category::Tool => "tool",
            Category::Toy => "toy",
        }
    }
}

struct Item {
    name: String,
    category: Category,
    price: i64,
    qty: i64,
}

impl Item {
    fn value(&self) -> i64 {
        self.price * self.qty
    }
}

fn restock(items: &mut [Item], min: i64) -> i64 {
    let mut added = 0;
    for item in items.iter_mut() {
        if item.qty < min {
            added += min - item.qty;
            item.qty = min;
        }
    }
    added
}

fn main() {
    let mut items = vec![
        Item { name: "apple".to_string(), category: Category::Food, price: 3, qty: 10 },
        Item { name: "hammer".to_string(), category: Category::Tool, price: 25, qty: 1 },
        Item { name: "ball".to_string(), category: Category::Toy, price: 8, qty: 0 },
    ];
    let added = restock(&mut items, 2);
    let mut total = 0;
    for item in &items {
        println!("{} ({}): {} x {}", item.name, item.category.label(), item.qty, item.price);
        total += item.value();
    }
    println!("restocked {added}, stock value {total}");
}
