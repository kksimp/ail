package main

import (
	"bytes"
	"compress/gzip"
	"fmt"
	"hash/crc32"
	"io"
	"os"
	"strconv"
	"strings"
)

func main() {
	n := 400000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	w := []string{"the", "of", "and", "to", "in", "fox", "jumps", "lazy", "dog", "river", "stone", "moss"}
	seed := 7
	ws := make([]string, 0, n)
	for i := 0; i < n; i++ {
		seed = (seed*3877 + 29573) % 139968
		ws = append(ws, w[seed%len(w)])
	}
	text := []byte(strings.Join(ws, " "))
	var buf bytes.Buffer
	zw := gzip.NewWriter(&buf)
	zw.Write(text)
	zw.Close()
	zr, err := gzip.NewReader(&buf)
	if err != nil {
		panic(err)
	}
	back, _ := io.ReadAll(zr)
	fmt.Printf("bytes %d, crc %d, round trip %v\n", len(text), crc32.ChecksumIEEE(text), bytes.Equal(back, text))
}
