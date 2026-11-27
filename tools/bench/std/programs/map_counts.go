package main

import (
	"fmt"
	"os"
	"strconv"
)

func main() {
	n := 1000000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	seed := 5
	a := map[int]int{}
	b := map[string]int{}
	for i := 0; i < n; i++ {
		seed = (seed*3877 + 29573) % 139968
		a[seed%50021]++
		b["key"+strconv.Itoa(seed%20011)]++
	}
	sq, sq2 := 0, 0
	for k, v := range a {
		sq += v*v + k%7
	}
	for k, v := range b {
		sq2 += v*v + len(k)
	}
	fmt.Printf("int keys %d, check %d\n", len(a), sq)
	fmt.Printf("str keys %d, check %d\n", len(b), sq2)
}
