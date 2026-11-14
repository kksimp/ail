// Mandelbrot as text: the same shades and pixel mapping as mandelbrot.ail.
package main

import (
	"bufio"
	"os"
	"strconv"
)

const maxIter = 50

const shades = " .:-=+*#%@"

func escape(cr, ci float64) int64 {
	zr := 0.0
	zi := 0.0
	var k int64 = 0
	for k < maxIter {
		if zr*zr+zi*zi > 4.0 {
			break
		}
		t := zr*zr - zi*zi + cr
		zi = 2.0*zr*zi + ci
		zr = t
		k += 1
	}
	return k
}

func main() {
	var rows int64 = 160
	if len(os.Args) > 1 {
		rows, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	cols := rows * 2
	out := bufio.NewWriter(os.Stdout)
	defer out.Flush()
	line := make([]byte, cols+1)
	for y := int64(0); y < rows; y++ {
		ci := -1.2 + 2.4*float64(y)/float64(rows)
		for x := int64(0); x < cols; x++ {
			cr := -2.0 + 3.0*float64(x)/float64(cols)
			k := escape(cr, ci)
			line[x] = shades[k*9/maxIter]
		}
		line[cols] = '\n'
		out.Write(line)
	}
}
