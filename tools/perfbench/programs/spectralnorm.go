// Spectral norm: the same power iteration as spectralnorm.ail.
package main

import (
	"fmt"
	"math"
	"os"
	"strconv"
)

func evalA(i, j int64) float64 {
	d := (i+j)*(i+j+1)/2 + i + 1
	return 1.0 / float64(d)
}

func mulAv(v, out []float64, n int64) {
	for i := int64(0); i < n; i++ {
		sum := 0.0
		for j := int64(0); j < n; j++ {
			sum += evalA(i, j) * v[j]
		}
		out[i] = sum
	}
}

func mulAtv(v, out []float64, n int64) {
	for i := int64(0); i < n; i++ {
		sum := 0.0
		for j := int64(0); j < n; j++ {
			sum += evalA(j, i) * v[j]
		}
		out[i] = sum
	}
}

func mulAta(v, out, tmp []float64, n int64) {
	mulAv(v, tmp, n)
	mulAtv(tmp, out, n)
}

func main() {
	var n int64 = 200
	if len(os.Args) > 1 {
		n, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	u := make([]float64, n)
	v := make([]float64, n)
	tmp := make([]float64, n)
	for i := int64(0); i < n; i++ {
		u[i] = 1.0
		v[i] = 0.0
	}
	for k := 0; k < 10; k++ {
		mulAta(u, v, tmp, n)
		mulAta(v, u, tmp, n)
	}
	vbv := 0.0
	vv := 0.0
	for i := int64(0); i < n; i++ {
		vbv += u[i] * v[i]
		vv += v[i] * v[i]
	}
	fmt.Printf("%.9f\n", math.Sqrt(vbv/vv))
}
