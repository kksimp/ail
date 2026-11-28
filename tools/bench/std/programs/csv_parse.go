package main

import (
	"encoding/csv"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func main() {
	n := 100000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	var sb strings.Builder
	for i := 0; i < n; i++ {
		fmt.Fprintf(&sb, "%d,\"name, %d\",%d,\"say \"\"hi\"\" %d\"\n", i, i, i%97, i%5)
	}
	rows, err := csv.NewReader(strings.NewReader(sb.String())).ReadAll()
	if err != nil {
		panic(err)
	}
	qty, chars := 0, 0
	for _, r := range rows {
		q, _ := strconv.Atoi(r[2])
		qty += q
		chars += len(r[1]) + len(r[3])
	}
	fmt.Printf("rows %d, qty %d, chars %d\n", len(rows), qty, chars)
}
