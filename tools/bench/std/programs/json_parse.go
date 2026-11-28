package main

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"strings"
)

type rec struct {
	ID   int      `json:"id"`
	Name string   `json:"name"`
	Tags []string `json:"tags"`
	Ok   bool     `json:"ok"`
}

func total(r []rec) (int, int) {
	ids, oks := 0, 0
	for _, x := range r {
		ids += x.ID
		if x.Ok {
			oks++
		}
	}
	return ids, oks
}

func main() {
	n := 50000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	parts := make([]string, 0, n)
	for i := 0; i < n; i++ {
		flag := "false"
		if i%3 == 0 {
			flag = "true"
		}
		parts = append(parts, fmt.Sprintf(`{"id":%d,"name":"item-%d","tags":["a","b%d"],"ok":%s}`, i, i, i%7, flag))
	}
	text := "[" + strings.Join(parts, ",") + "]"
	var doc []rec
	if err := json.Unmarshal([]byte(text), &doc); err != nil {
		panic(err)
	}
	ids, oks := total(doc)
	out, _ := json.Marshal(doc)
	var again []rec
	if err := json.Unmarshal(out, &again); err != nil {
		panic(err)
	}
	ids2, oks2 := total(again)
	fmt.Printf("records %d, id sum %d, ok %d, round trip %v\n", n, ids, oks, ids == ids2 && oks == oks2)
}
