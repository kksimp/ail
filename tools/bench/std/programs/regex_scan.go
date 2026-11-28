package main

import (
	"fmt"
	"os"
	"regexp"
	"strconv"
	"strings"
)

func main() {
	n := 100000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	w := []string{"alpha7", "beta42", "gamma", "delta1234", "eps", "zeta99", "eta12345", "theta", "iota3", "kappa808"}
	seed := 11
	ws := make([]string, 0, n)
	for i := 0; i < n; i++ {
		seed = (seed*3877 + 29573) % 139968
		ws = append(ws, w[seed%len(w)])
	}
	text := strings.Join(ws, " ")
	found := regexp.MustCompile(`[a-z]+[0-9]{2,4}`).FindAllString(text, -1)
	total := 0
	for _, m := range found {
		total += len(m)
	}
	swapped := regexp.MustCompile(`[aeiou]`).ReplaceAllString(text, "_")
	fmt.Printf("tokens %d, token bytes %d, replaced %d %s\n", len(found), total, len(swapped), swapped[:30])
}
