// Binary trees: the same arena build and check as binarytrees.ail.
package main

import (
	"fmt"
	"os"
	"strconv"
)

var left []int32
var right []int32
var used int32

func make_(depth int) int32 {
	node := used
	used += 1
	left[node] = -1
	right[node] = -1
	if depth > 0 {
		l := make_(depth - 1)
		r := make_(depth - 1)
		left[node] = l
		right[node] = r
	}
	return node
}

func check(node int32) int64 {
	if left[node] < 0 {
		return 1
	}
	return 1 + check(left[node]) + check(right[node])
}

func main() {
	maxDepth := 12
	if len(os.Args) > 1 {
		maxDepth, _ = strconv.Atoi(os.Args[1])
	}
	minDepth := 4
	cap_ := 1 << (maxDepth + 2)
	left = make([]int32, cap_)
	right = make([]int32, cap_)
	for d := minDepth; d <= maxDepth; d += 2 {
		iters := int64(1) << (maxDepth - d + minDepth)
		var total int64 = 0
		for t := int64(0); t < iters; t++ {
			used = 0
			root := make_(d)
			total += check(root)
		}
		fmt.Printf("%d trees of depth %d check: %d\n", iters, d, total)
	}
}
