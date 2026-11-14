// Go twin of matmul1400.c: dense matrix multiply.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
)

func main() {
	out := bufio.NewWriter(os.Stdout)
	defer out.Flush()
	n := 120
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	a := make([]int, n*n)
	b := make([]int, n*n)
	c := make([]int, n*n)
	for i := 0; i < n; i++ {
		for j := 0; j < n; j++ {
			a[i*n+j] = (i*j + i + j) % 7
			b[i*n+j] = (i + 2*j) % 5
		}
	}
	for i := 0; i < n; i++ {
		for j := 0; j < n; j++ {
			s := 0
			for k := 0; k < n; k++ {
				s += a[i*n+k] * b[k*n+j]
			}
			c[i*n+j] = s
		}
	}
	trace := 0
	total := 0
	for i := 0; i < n; i++ {
		trace += c[i*n+i]
		for j := 0; j < n; j++ {
			total += c[i*n+j]
		}
	}
	fmt.Fprintf(out, "trace: %d\n", trace)
	fmt.Fprintf(out, "sum: %d\n", total)
}
