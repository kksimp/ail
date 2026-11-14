// Word frequencies: the same generator and vocabulary as wordfreq.ail, with a
// real hash map from word to count (open addressing, linear probing, FNV-1a,
// doubling at 3/4 load), like the Map[str, int] the AIL program uses.
package main

import (
	"fmt"
	"os"
	"strconv"
)

var WORDS = [...]string{"the", "of", "and", "to", "in", "fox",
	"jumps", "lazy", "dog", "river", "stone", "moss"}

const NWORDS = 12

type Slot struct {
	key   string
	used  bool
	count int64
}

type Map struct {
	slots []Slot
	cap   uint64
	len   uint64
}

func hashStr(s string) uint64 {
	var h uint64 = 1469598103934665603
	for i := 0; i < len(s); i++ {
		h = (h ^ uint64(s[i])) * 1099511628211
	}
	return h
}

func find(slots []Slot, cap uint64, key string) *Slot {
	i := hashStr(key) & (cap - 1)
	for slots[i].used && slots[i].key != key {
		i = (i + 1) & (cap - 1)
	}
	return &slots[i]
}

func grow(m *Map) {
	ncap := m.cap * 2
	ns := make([]Slot, ncap)
	for i := uint64(0); i < m.cap; i++ {
		if m.slots[i].used {
			*find(ns, ncap, m.slots[i].key) = m.slots[i]
		}
	}
	m.slots = ns
	m.cap = ncap
}

func mapGet(m *Map, key string) int64 {
	s := find(m.slots, m.cap, key)
	if s.used {
		return s.count
	}
	return 0
}

func mapSet(m *Map, key string, count int64) {
	s := find(m.slots, m.cap, key)
	if !s.used {
		if (m.len+1)*4 > m.cap*3 {
			grow(m)
			s = find(m.slots, m.cap, key)
		}
		s.key = key
		s.used = true
		m.len += 1
	}
	s.count = count
}

func main() {
	var n int64 = 200000
	if len(os.Args) > 1 {
		n, _ = strconv.ParseInt(os.Args[1], 10, 64)
	}
	counts := Map{make([]Slot, 8), 8, 0}
	var seed int64 = 7
	for i := int64(0); i < n; i++ {
		seed = (seed*3877 + 29573) % 139968
		w := WORDS[seed%NWORDS]
		c := mapGet(&counts, w)
		mapSet(&counts, w, c+1)
	}
	for k := 0; k < NWORDS; k++ {
		fmt.Printf("%s %d\n", WORDS[k], mapGet(&counts, WORDS[k]))
	}
	fmt.Printf("distinct %d\n", counts.len)
}
