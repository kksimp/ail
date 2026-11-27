package main

import (
	"encoding/xml"
	"fmt"
	"os"
	"strconv"
	"strings"
)

type item struct {
	ID   int    `xml:"id,attr"`
	Name string `xml:"name"`
	V    int    `xml:"v"`
}

type root struct {
	Items []item `xml:"item"`
}

func main() {
	n := 50000
	if len(os.Args) > 1 {
		n, _ = strconv.Atoi(os.Args[1])
	}
	var sb strings.Builder
	sb.WriteString("<root>")
	for i := 0; i < n; i++ {
		fmt.Fprintf(&sb, `<item id="%d" kind="k%d"><name>item %d</name><v>%d</v></item>`, i, i%5, i, i%97)
	}
	sb.WriteString("</root>")
	var r root
	if err := xml.Unmarshal([]byte(sb.String()), &r); err != nil {
		panic(err)
	}
	ids, vs, nc := 0, 0, 0
	for _, it := range r.Items {
		ids += it.ID
		vs += it.V
		nc += len(it.Name)
	}
	fmt.Printf("items %d, ids %d, v %d, name chars %d\n", len(r.Items), ids, vs, nc)
}
