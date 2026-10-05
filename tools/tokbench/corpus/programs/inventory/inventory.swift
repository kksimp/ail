enum Category {
    case food, tool, toy

    var label: String {
        switch self {
        case .food: return "food"
        case .tool: return "tool"
        case .toy: return "toy"
        }
    }
}

struct Item {
    let name: String
    let category: Category
    let price: Int
    var qty: Int

    var value: Int { price * qty }
}

func restock(_ items: inout [Item], min: Int) -> Int {
    var added = 0
    for i in items.indices where items[i].qty < min {
        added += min - items[i].qty
        items[i].qty = min
    }
    return added
}

var items = [
    Item(name: "apple", category: .food, price: 3, qty: 10),
    Item(name: "hammer", category: .tool, price: 25, qty: 1),
    Item(name: "ball", category: .toy, price: 8, qty: 0),
]
let added = restock(&items, min: 2)
var total = 0
for item in items {
    print("\(item.name) (\(item.category.label)): \(item.qty) x \(item.price)")
    total += item.value
}
print("restocked \(added), stock value \(total)")
