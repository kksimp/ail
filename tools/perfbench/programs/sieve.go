// Sieve of Eratosthenes: the same count, sum and largest as sieve.ail.
package main

import (
	"fmt"
	"os"
	"strconv"
)

func main() {
	var n int64 = 1000000
	if len(os.Args) > 1 {
		n, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	isPrime := make([]byte, n)
	for k := int64(0); k < n; k++ {
		isPrime[k] = 1
	}
	isPrime[0] = 0
	isPrime[1] = 0
	for i := int64(2); i*i < n; i++ {
		if isPrime[i] != 0 {
			for j := i * i; j < n; j += i {
				isPrime[j] = 0
			}
		}
	}
	var count int64 = 0
	var total int64 = 0
	var largest int64 = 0
	for k := int64(0); k < n; k++ {
		if isPrime[k] != 0 {
			count += 1
			total += k
			largest = k
		}
	}
	fmt.Printf("primes below %d: %d\n", n, count)
	fmt.Printf("sum: %d\n", total)
	fmt.Printf("largest: %d\n", largest)
}
