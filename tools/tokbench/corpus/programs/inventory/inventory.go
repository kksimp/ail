package main

import "fmt"

type Category int

const (
	Food Category = iota
	Tool
	Toy
)

func (c Category) Label() string {
	switch c {
	case Food:
		return "food"
	case Tool:
		return "tool"
	default:
		return "toy"
	}
}

type Item struct {
	Name     string
	Category Category
	Price    int
	Qty      int
}

func (i Item) Value() int {
	return i.Price * i.Qty
}

func restock(items []Item, min int) int {
	added := 0
	for i := range items {
		if items[i].Qty < min {
			added += min - items[i].Qty
			items[i].Qty = min
		}
	}
	return added
}

func main() {
	items := []Item{
		{Name: "apple", Category: Food, Price: 3, Qty: 10},
		{Name: "hammer", Category: Tool, Price: 25, Qty: 1},
		{Name: "ball", Category: Toy, Price: 8, Qty: 0},
	}
	added := restock(items, 2)
	total := 0
	for _, item := range items {
		fmt.Printf("%s (%s): %d x %d\n", item.Name, item.Category.Label(), item.Qty, item.Price)
		total += item.Value()
	}
	fmt.Printf("restocked %d, stock value %d\n", added, total)
}
