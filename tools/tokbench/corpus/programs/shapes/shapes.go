package main

import (
	"cmp"
	"fmt"
)

type Shape interface {
	Area() int
	Name() string
}

func describe(s Shape) string {
	return fmt.Sprintf("%s with area %d", s.Name(), s.Area())
}

type Circle struct{ R int }

func (c Circle) Area() int      { return 3 * c.R * c.R }
func (c Circle) Name() string   { return "circle" }
func (c Circle) String() string { return fmt.Sprintf("Circle(r: %d)", c.R) }

type Rect struct{ W, H int }

func (r Rect) Area() int { return r.W * r.H }

func (r Rect) Name() string {
	if r.W == r.H {
		return "square"
	}
	return "rectangle"
}

type Size struct {
	Area int
	Name string
}

func (s Size) Less(o Size) bool {
	if s.Area != o.Area {
		return s.Area < o.Area
	}
	return s.Name < o.Name
}

func (s Size) String() string { return fmt.Sprintf("Size(area: %d, name: %q)", s.Area, s.Name) }

type Drawing struct {
	Title  string
	Shapes []Shape
}

func (d *Drawing) Add(s Shape) { d.Shapes = append(d.Shapes, s) }

func (d *Drawing) Biggest() (Size, bool) {
	if len(d.Shapes) == 0 {
		return Size{}, false
	}
	best := Size{0, ""}
	for _, s := range d.Shapes {
		size := Size{s.Area(), s.Name()}
		if best.Less(size) {
			best = size
		}
	}
	return best, true
}

func largest[T cmp.Ordered](items []T) T {
	best := items[0]
	for _, x := range items {
		if x > best {
			best = x
		}
	}
	return best
}

func main() {
	d := Drawing{Title: "plan", Shapes: []Shape{Circle{2}, Rect{3, 4}}}
	d.Add(Rect{4, 4})
	for _, s := range d.Shapes {
		fmt.Println(describe(s))
	}
	if big, ok := d.Biggest(); ok {
		fmt.Printf("biggest: %v\n", big)
	}
	fmt.Println(largest([]int{3, 9, 4}))
	fmt.Println(largest([]string{"pear", "fig", "apple"}))
	fmt.Println(Rect{2, 3} == Rect{2, 3})
	fmt.Println(d.Shapes[0])
}
