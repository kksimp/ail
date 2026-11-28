package main

import (
	"fmt"
	"net/url"
	"os"
	"strconv"
)

func main() {
	n := 100000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	ports, chars := 0, 0
	for i := 0; i < n; i++ {
		u, err := url.Parse(fmt.Sprintf("https://host%d.example.com:%d/a/b%d?q=%d&r=x#f%d", i%50, 8000+i%100, i, i, i))
		if err != nil {
			panic(err)
		}
		p, _ := strconv.Atoi(u.Port())
		ports += p
		chars += len(u.Hostname()) + len(u.Path) + len(u.RawQuery) + len(u.Fragment)
	}
	fmt.Printf("urls %d, ports %d, chars %d\n", n, ports, chars)
}
