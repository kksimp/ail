// Go twin of vm.c: a stack VM counting primes.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
)

var code [256]int64
var ncode = 0

func emit(op, arg int64) {
	code[ncode] = op
	ncode++
	code[ncode] = arg
	ncode++
}

func here() int { return ncode / 2 }

func build(limit int64) {
	emit(0, 2)
	emit(2, 0)
	emit(0, 0)
	emit(2, 1)
	top := here()
	emit(1, 0)
	emit(0, limit)
	emit(6, 0)
	jzEnd := here()
	emit(8, 0)
	emit(0, 2)
	emit(2, 2)
	inner := here()
	emit(1, 2)
	emit(1, 2)
	emit(4, 0)
	emit(1, 0)
	emit(0, 1)
	emit(3, 0)
	emit(6, 0)
	jzPrime := here()
	emit(8, 0)
	emit(1, 0)
	emit(1, 2)
	emit(5, 0)
	jzNext := here()
	emit(8, 0)
	emit(1, 2)
	emit(0, 1)
	emit(3, 0)
	emit(2, 2)
	emit(7, int64(inner))
	prime := here()
	emit(1, 1)
	emit(0, 1)
	emit(3, 0)
	emit(2, 1)
	next := here()
	emit(1, 0)
	emit(0, 1)
	emit(3, 0)
	emit(2, 0)
	emit(7, int64(top))
	end := here()
	emit(9, 0)
	code[jzEnd*2+1] = int64(end)
	code[jzPrime*2+1] = int64(prime)
	code[jzNext*2+1] = int64(next)
}

func run(vars *[3]int64) int64 {
	var st [64]int64
	var sp, pc, steps int64
	for {
		op, arg := code[pc*2], code[pc*2+1]
		pc++
		steps++
		switch op {
		case 0:
			st[sp] = arg
			sp++
		case 1:
			st[sp] = vars[arg]
			sp++
		case 2:
			sp--
			vars[arg] = st[sp]
		case 3:
			sp--
			st[sp-1] = st[sp-1] + st[sp]
		case 4:
			sp--
			st[sp-1] = st[sp-1] * st[sp]
		case 5:
			sp--
			st[sp-1] = st[sp-1] % st[sp]
		case 6:
			sp--
			if st[sp-1] < st[sp] {
				st[sp-1] = 1
			} else {
				st[sp-1] = 0
			}
		case 7:
			pc = arg
		case 8:
			sp--
			if st[sp] == 0 {
				pc = arg
			}
		case 9:
			return steps
		default:
			panic("bad op")
		}
	}
}

func main() {
	out := bufio.NewWriter(os.Stdout)
	defer out.Flush()
	var limit int64 = 200000
	if len(os.Args) > 1 {
		limit, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	build(limit)
	var vars [3]int64
	steps := run(&vars)
	fmt.Fprintf(out, "primes %d\nsteps %d\n", vars[1], steps)
}
