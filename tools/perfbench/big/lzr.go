// Go twin of lzr.c: LZ77 + adaptive binary range coder.
package main

import (
	"bufio"
	"fmt"
	"os"
	"strconv"
)

type ll = int64

var low, rng, cache, cacheSize ll = 0, 4294967295, 0, 1
var outb []ll
var nout ll = 0
var probs [2400]ll

func shiftLow() {
	if low < 4278190080 || low >= 4294967296 {
		carry, temp := low>>32, cache
		for {
			outb[nout] = (temp + carry) & 255
			nout++
			temp = 255
			cacheSize--
			if cacheSize == 0 {
				break
			}
		}
		cache = (low >> 24) & 255
	}
	cacheSize++
	low = (low & 16777215) << 8
}

func ebit(idx, b ll) {
	p := probs[idx]
	bound := (rng >> 11) * p
	if b == 0 {
		rng = bound
		probs[idx] = p + ((2048 - p) >> 5)
	} else {
		low += bound
		rng -= bound
		probs[idx] = p - (p >> 5)
	}
	for rng < 16777216 {
		rng <<= 8
		shiftLow()
	}
}

func edirect(v, nbits ll) {
	for i := nbits - 1; i >= 0; i-- {
		rng >>= 1
		if (v>>uint(i))&1 != 0 {
			low += rng
		}
		for rng < 16777216 {
			rng <<= 8
			shiftLow()
		}
	}
}

func etree(base, nbits, v ll) {
	var m ll = 1
	for i := nbits - 1; i >= 0; i-- {
		b := (v >> uint(i)) & 1
		ebit(base+m, b)
		m = m*2 + b
	}
}

var code, pos ll
var comp []ll

func dnext() ll {
	v := comp[pos]
	pos++
	return v
}

func dbit(idx ll) ll {
	p := probs[idx]
	bound := (rng >> 11) * p
	var r ll = 0
	if code < bound {
		rng = bound
		probs[idx] = p + ((2048 - p) >> 5)
	} else {
		code -= bound
		rng -= bound
		probs[idx] = p - (p >> 5)
		r = 1
	}
	if rng < 16777216 {
		rng <<= 8
		code = (code << 8) | dnext()
	}
	return r
}

func ddirect(nbits ll) ll {
	var v ll = 0
	for i := ll(0); i < nbits; i++ {
		rng >>= 1
		var b ll = 0
		if code >= rng {
			code -= rng
			b = 1
		}
		v = (v << 1) | b
		if rng < 16777216 {
			rng <<= 8
			code = (code << 8) | dnext()
		}
	}
	return v
}

func dtree(base, nbits ll) ll {
	var m, top ll = 1, 1
	for i := ll(0); i < nbits; i++ {
		m = m*2 + dbit(base+m)
		top *= 2
	}
	return m - top
}

func hash3(a, b, c ll) ll {
	return (((a*65536 + b*256 + c) * 2654435761) & 4294967295) >> 12
}

var head, prev, data []ll

func insertAt(p, n ll) {
	if p+2 < n {
		h := hash3(data[p], data[p+1], data[p+2])
		prev[p] = head[h]
		head[h] = p
	}
}

func find(i, n ll) ll {
	var bestLen, bestDist ll = 0, 0
	if i+2 < n {
		cand := head[hash3(data[i], data[i+1], data[i+2])]
		var chain ll = 0
		for cand >= 0 && i-cand <= 1048576 && chain < 32 {
			var l ll = 0
			for l < 66 && i+l < n && data[cand+l] == data[i+l] {
				l++
			}
			if l > bestLen {
				bestLen = l
				bestDist = i - cand
			}
			cand = prev[cand]
			chain++
		}
	}
	if bestLen < 3 {
		return 0
	}
	return bestLen*2097152 + bestDist
}

func distSlot(d ll) ll {
	if d < 4 {
		return d
	}
	var n ll = 0
	t := d
	for t > 1 {
		t >>= 1
		n++
	}
	return 2*n + ((d >> uint(n-1)) & 1)
}

func minLL(a, b ll) ll {
	if a < b {
		return a
	}
	return b
}

func main() {
	out := bufio.NewWriter(os.Stdout)
	defer out.Flush()
	var n ll = 12000000
	if len(os.Args) > 1 {
		n, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	data = make([]ll, n+16)
	var length, seed ll = 0, 7
	for length < n {
		seed = (seed*3877 + 29573) % 139968
		k := seed % 32768
		for j := ll(0); j < 3+k%7; j++ {
			data[length] = 97 + (k*7+j*3+k/64)%26
			length++
		}
		data[length] = 32
		length++
	}
	n = length
	head = make([]ll, 1048576)
	prev = make([]ll, n)
	for i := range head {
		head[i] = -1
	}
	for i := range prev {
		prev[i] = -1
	}
	outb = make([]ll, n+1024)
	for i := range probs {
		probs[i] = 1024
	}
	var ins, st, i ll = 0, 0, 0
	for i < n {
		for ins < i {
			insertAt(ins, n)
			ins++
		}
		m := find(i, n)
		if m > 0 && i+1 < n {
			for ins < i+1 {
				insertAt(ins, n)
				ins++
			}
			if find(i+1, n)/2097152 > m/2097152+1 {
				m = 0
			}
		}
		if m > 0 {
			l := m/2097152 - 3
			d := m%2097152 - 1
			ebit(st, 1)
			etree(2, 6, l)
			slot := distSlot(d)
			etree(66+minLL(l, 3)*64, 6, slot)
			if slot >= 4 {
				nb := (slot >> 1) - 1
				edirect(d-((2+(slot&1))<<uint(nb)), nb)
			}
			st = 1
			i += l + 3
		} else {
			var ctx ll = 0
			if i > 0 {
				ctx = data[i-1] >> 5
			}
			ebit(st, 0)
			etree(322+ctx*256, 8, data[i])
			st = 0
			i++
		}
	}
	for k := 0; k < 5; k++ {
		shiftLow()
	}
	clen := nout
	// decode
	comp = outb
	pos = 1
	code = 0
	rng = 4294967295
	for k := 0; k < 4; k++ {
		code = (code << 8) | dnext()
	}
	for k := range probs {
		probs[k] = 1024
	}
	back := make([]ll, n+80)
	var nb2 ll = 0
	st = 0
	for nb2 < n {
		if dbit(st) == 0 {
			var ctx ll = 0
			if nb2 > 0 {
				ctx = back[nb2-1] >> 5
			}
			back[nb2] = dtree(322+ctx*256, 8)
			nb2++
			st = 0
		} else {
			l := dtree(2, 6)
			slot := dtree(66+minLL(l, 3)*64, 6)
			dist := slot
			if slot >= 4 {
				nb := (slot >> 1) - 1
				dist = ((2 + (slot & 1)) << uint(nb)) + ddirect(nb)
			}
			start := nb2 - (dist + 1)
			for k := ll(0); k < l+3; k++ {
				back[nb2] = back[start+k]
				nb2++
			}
			st = 1
		}
	}
	var cks ll = 0
	for k := ll(0); k < clen; k++ {
		cks = (cks*31 + outb[k]) % 1000000007
	}
	ok := nb2 == n
	for k := ll(0); ok && k < n; k++ {
		ok = back[k] == data[k]
	}
	s := "false"
	if ok {
		s = "true"
	}
	fmt.Fprintf(out, "input %d\ncompressed %d\npermille %d\nchecksum %d\nroundtrip %s\n", n, clen, clen*1000/n, cks, s)
}
