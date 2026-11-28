package main

import (
	"crypto/hmac"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"os"
	"strconv"
	"strings"
)

func main() {
	n := 200000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	h, m := "seed", "key"
	for i := 0; i < n; i++ {
		s := sha256.Sum256([]byte(h + strconv.Itoa(i)))
		h = hex.EncodeToString(s[:])
		mac := hmac.New(sha256.New, []byte(h[:16]))
		mac.Write([]byte(m + strconv.Itoa(i)))
		m = hex.EncodeToString(mac.Sum(nil))
	}
	fmt.Println(h)
	fmt.Println(m)
	big := strings.Repeat("0123456789abcdef", 65536)
	d := ""
	for i := 0; i < n/10000; i++ {
		s := sha256.Sum256([]byte(d + big))
		d = hex.EncodeToString(s[:])
	}
	fmt.Println(d)
}
