// Dense matrix multiply: the same entries and checks as matmul.ail.
package main

import (
	"fmt"
	"os"
	"strconv"
)

func main() {
	n := 120
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	a := make([]int64, n*n)
	b := make([]int64, n*n)
	c := make([]int64, n*n)
	nn := int64(n)
	for i := int64(0); i < nn; i++ {
		for j := int64(0); j < nn; j++ {
			a[i*nn+j] = (i*j + i + j) % 7
			b[i*nn+j] = (i + 2*j) % 5
		}
	}
	for i := int64(0); i < nn; i++ {
		for j := int64(0); j < nn; j++ {
			var s int64 = 0
			for k := int64(0); k < nn; k++ {
				s += a[i*nn+k] * b[k*nn+j]
			}
			c[i*nn+j] = s
		}
	}
	var trace int64 = 0
	var total int64 = 0
	for i := int64(0); i < nn; i++ {
		trace += c[i*nn+i]
		for j := int64(0); j < nn; j++ {
			total += c[i*nn+j]
		}
	}
	fmt.Printf("trace: %d\n", trace)
	fmt.Printf("sum: %d\n", total)
}
