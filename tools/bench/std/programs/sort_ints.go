package main

import (
	"fmt"
	"os"
	"sort"
	"strconv"
)

func main() {
	n := 1000000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	seed := 42
	xs := make([]int, 0, n)
	for i := 0; i < n; i++ {
		seed = (seed*3877 + 29573) % 139968
		xs = append(xs, seed*7+len(xs)%13)
	}
	sort.Ints(xs)
	chk := 0
	for _, x := range xs {
		chk = (chk*31 + x) % 1000000007
	}
	ss := make([]string, 0, n/4)
	for j := 0; j < n/4; j++ {
		seed = (seed*3877 + 29573) % 139968
		ss = append(ss, fmt.Sprintf("k%d-%d", seed, j%1000))
	}
	sort.Strings(ss)
	fmt.Printf("ints %d, check %d, first %d, last %d\n", len(xs), chk, xs[0], xs[n-1])
	fmt.Printf("strings %d, first %s, last %s\n", len(ss), ss[0], ss[len(ss)-1])
}
