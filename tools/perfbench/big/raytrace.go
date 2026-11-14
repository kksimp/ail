// Go twin of raytrace.c: a small recursive ray tracer.
package main

import (
	"bufio"
	"fmt"
	"math"
	"os"
	"strconv"
)

type Vec struct{ x, y, z float64 }
type Sphere struct {
	c    Vec
	r    float64
	col  Vec
	refl float64
}

func V(x, y, z float64) Vec      { return Vec{x, y, z} }
func add(a, o Vec) Vec           { return V(a.x+o.x, a.y+o.y, a.z+o.z) }
func sub(a, o Vec) Vec           { return V(a.x-o.x, a.y-o.y, a.z-o.z) }
func scale(a Vec, k float64) Vec { return V(a.x*k, a.y*k, a.z*k) }
func dot(a, o Vec) float64       { return a.x*o.x + a.y*o.y + a.z*o.z }
func unit(a Vec) Vec {
	k := 1.0 / math.Sqrt(dot(a, a))
	return scale(a, k)
}

func hit(s *Sphere, o, d Vec) float64 {
	oc := sub(o, s.c)
	b := dot(oc, d)
	c := dot(oc, oc) - s.r*s.r
	disc := b*b - c
	if disc < 0.0 {
		return -1.0
	}
	sq := math.Sqrt(disc)
	t := -b - sq
	if t > 0.001 {
		return t
	}
	t2 := -b + sq
	if t2 > 0.001 {
		return t2
	}
	return -1.0
}

var spheres [4]Sphere

func trace(light, o, d Vec, depth int) Vec {
	best := -1.0
	idx := -1
	for i := 0; i < 4; i++ {
		t := hit(&spheres[i], o, d)
		if t > 0.0 && (best < 0.0 || t < best) {
			best = t
			idx = i
		}
	}
	if idx < 0 {
		k := 0.5 * (d.y + 1.0)
		return add(scale(V(1.0, 1.0, 1.0), 1.0-k), scale(V(0.5, 0.7, 1.0), k))
	}
	s := spheres[idx]
	p := add(o, scale(d, best))
	n := unit(sub(p, s.c))
	base := s.col
	if idx == 3 {
		parity := (int(p.x+100.0) + int(p.z+100.0)) % 2
		if parity == 0 {
			base = V(0.2, 0.2, 0.2)
		}
	}
	lit := false
	for i := 0; i < 4; i++ {
		if hit(&spheres[i], p, light) > 0.0 {
			lit = true
		}
	}
	diff := dot(n, light)
	if diff < 0.0 || lit {
		diff = 0.0
	}
	col := scale(base, 0.15+0.85*diff)
	if depth < 2 && s.refl > 0.0 {
		r := sub(d, scale(n, 2.0*dot(d, n)))
		rc := trace(light, p, r, depth+1)
		col = add(scale(col, 1.0-s.refl), scale(rc, s.refl))
	}
	return col
}

func chanv(v float64) int64 {
	c := v
	if c < 0.0 {
		c = 0.0
	}
	if c > 1.0 {
		c = 1.0
	}
	return int64(c * 255.0)
}

func main() {
	out := bufio.NewWriter(os.Stdout)
	defer out.Flush()
	var n int64 = 400
	if len(os.Args) > 1 {
		n, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	spheres[0] = Sphere{V(0.0, 0.0, -5.0), 1.0, V(0.9, 0.2, 0.2), 0.3}
	spheres[1] = Sphere{V(2.0, 0.2, -6.0), 1.2, V(0.2, 0.8, 0.3), 0.5}
	spheres[2] = Sphere{V(-2.2, -0.3, -4.5), 0.7, V(0.3, 0.3, 0.9), 0.2}
	spheres[3] = Sphere{V(0.0, -1001.0, -5.0), 1000.0, V(0.8, 0.8, 0.8), 0.25}
	light := unit(V(0.5, 1.0, 0.3))
	eye := V(0.0, 0.0, 0.0)
	var cks, total int64
	for j := int64(0); j < n; j++ {
		for i := int64(0); i < n; i++ {
			u := (float64(i)+0.5)/float64(n)*2.0 - 1.0
			v := 1.0 - (float64(j)+0.5)/float64(n)*2.0
			d := unit(V(u, v, -1.5))
			c := trace(light, eye, d, 0)
			r, g, b := chanv(c.x), chanv(c.y), chanv(c.z)
			total += r + g + b
			cks = (cks*31 + r*65536 + g*256 + b) % 1000000007
		}
	}
	fmt.Fprintf(out, "size %d\ntotal %d\nchecksum %d\n", n, total, cks)
}
