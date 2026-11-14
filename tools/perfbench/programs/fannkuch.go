// Fannkuch-redux: the same permutation walk as fannkuch.ail.
package main

import (
	"fmt"
	"os"
	"strconv"
)

func main() {
	n := 8
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	perm1 := make([]int32, n)
	perm := make([]int32, n)
	count := make([]int32, n)
	for i := 0; i < n; i++ {
		perm1[i] = int32(i)
		count[i] = 0
	}
	var maxflips int32 = 0
	var checksum int64 = 0
	var permcount int64 = 0
	r := n
	done := false
	for !done {
		for r != 1 {
			count[r-1] = int32(r)
			r -= 1
		}
		for i := 0; i < n; i++ {
			perm[i] = perm1[i]
		}
		var flips int32 = 0
		k := perm[0]
		for k != 0 {
			i := 0
			j := int(k)
			for i < j {
				t := perm[i]
				perm[i] = perm[j]
				perm[j] = t
				i += 1
				j -= 1
			}
			flips += 1
			k = perm[0]
		}
		if flips > maxflips {
			maxflips = flips
		}
		if permcount%2 == 0 {
			checksum += int64(flips)
		} else {
			checksum -= int64(flips)
		}
		advanced := false
		for !advanced {
			if r == n {
				done = true
				break
			}
			first := perm1[0]
			for i := 0; i < r; i++ {
				perm1[i] = perm1[i+1]
			}
			perm1[r] = first
			count[r] -= 1
			if count[r] > 0 {
				advanced = true
			} else {
				r += 1
			}
		}
		permcount += 1
	}
	fmt.Printf("%d\n", checksum)
	fmt.Printf("Pfannkuchen(%d) = %d\n", n, maxflips)
}
