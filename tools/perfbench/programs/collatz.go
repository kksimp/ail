// Collatz: total steps and the longest chain for starts 1 up to N (exclusive).
package main

import (
	"fmt"
	"os"
	"strconv"
)

func steps(n int64) int64 {
	var k int64 = 0
	for n != 1 {
		if n%2 == 0 {
			n = n / 2
		} else {
			n = 3*n + 1
		}
		k += 1
	}
	return k
}

func main() {
	var limit int64 = 30000
	if len(os.Args) > 1 {
		limit, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	var total int64 = 0
	var best int64 = 1
	var bestSteps int64 = 0
	for i := int64(1); i < limit; i++ {
		s := steps(i)
		total += s
		if s > bestSteps {
			best = i
			bestSteps = s
		}
	}
	fmt.Printf("total steps: %d\n", total)
	fmt.Printf("longest: %d with %d steps\n", best, bestSteps)
}
