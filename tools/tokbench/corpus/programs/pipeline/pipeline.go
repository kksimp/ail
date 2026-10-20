// A three-stage pipeline over bounded channels: one goroutine produces
// numbers, one squares them, and `main` collects the results. Each stage
// closes its output when its input runs dry.
package main

import "fmt"

func produce(out chan<- int, n int) {
	for i := 1; i <= n; i++ {
		out <- i
	}
	close(out)
}

func square(input <-chan int, out chan<- int) {
	for x := range input {
		out <- x * x
	}
	close(out)
}

func main() {
	numbers := make(chan int, 2)
	squares := make(chan int, 2)
	go produce(numbers, 6)
	go square(numbers, squares)
	total := 0
	for s := range squares {
		fmt.Printf("square %d\n", s)
		total += s
	}
	fmt.Printf("total %d\n", total)
}
