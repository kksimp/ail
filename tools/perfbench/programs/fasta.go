// FASTA-style DNA: the same LCG and weights as fasta.ail.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
)

const width = 60

func main() {
	var n int64 = 100000
	if len(os.Args) > 1 {
		n, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	out := bufio.NewWriter(os.Stdout)
	defer out.Flush()
	const letters = "acgt"
	var seed int64 = 42
	var counts [4]int64
	var line [width + 1]byte
	used := 0
	for i := int64(0); i < n; i++ {
		seed = (seed*3877 + 29573) % 139968
		t := seed * 100 / 139968
		base := 3
		if t < 30 {
			base = 0
		} else if t < 50 {
			base = 1
		} else if t < 70 {
			base = 2
		}
		counts[base] += 1
		line[used] = letters[base]
		used++
		if used == width {
			line[used] = '\n'
			out.Write(line[:used+1])
			used = 0
		}
	}
	if used > 0 {
		line[used] = '\n'
		out.Write(line[:used+1])
	}
	fmt.Fprintf(out, "a %d c %d g %d t %d\n", counts[0], counts[1], counts[2], counts[3])
}
