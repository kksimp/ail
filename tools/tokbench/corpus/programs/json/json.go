package main

import (
	"fmt"
	"log"
	"reflect"
	"strconv"
	"strings"
	"unicode/utf16"
)

const (
	digits     = "0123456789"
	whitespace = " \t\n\r"
	eof        = -1
)

var escapes = map[rune]rune{
	'"':  '"',
	'\\': '\\',
	'/':  '/',
	'b':  '\b',
	'f':  '\f',
	'n':  '\n',
	'r':  '\r',
	't':  '\t',
}

var kinds = []string{"object", "array", "string", "number", "boolean", "null"}

type Value any

var literals = []struct {
	word  string
	value Value
}{
	{"true", true},
	{"false", false},
	{"null", nil},
}

type Member struct {
	Key   string
	Value Value
}

type Object []Member

func (o Object) get(key string) (Value, bool) {
	for _, m := range o {
		if m.Key == key {
			return m.Value, true
		}
	}
	return nil, false
}

type ParseError struct {
	Line, Col int
	Msg       string
}

func (e *ParseError) Error() string {
	return fmt.Sprintf("parse error at %d:%d: %s", e.Line, e.Col, e.Msg)
}

type Parser struct {
	text []rune
	pos  int
}

func (p *Parser) failAt(at int, format string, args ...any) error {
	line, col := 1, 1
	for _, c := range p.text[:at] {
		if c == '\n' {
			line++
			col = 1
		} else {
			col++
		}
	}
	return &ParseError{line, col, fmt.Sprintf(format, args...)}
}

func (p *Parser) fail(format string, args ...any) error {
	return p.failAt(p.pos, format, args...)
}

func (p *Parser) atEnd() bool {
	return p.pos >= len(p.text)
}

func (p *Parser) peek() rune {
	if p.atEnd() {
		return eof
	}
	return p.text[p.pos]
}

func (p *Parser) unexpected() error {
	if p.atEnd() {
		return p.fail("unexpected end of input")
	}
	return p.fail("unexpected character '%c'", p.peek())
}

func (p *Parser) skipWhitespace() {
	for strings.ContainsRune(whitespace, p.peek()) {
		p.pos++
	}
}

func (p *Parser) parseDocument() (Value, error) {
	value, err := p.parseValue()
	if err != nil {
		return nil, err
	}
	p.skipWhitespace()
	if !p.atEnd() {
		return nil, p.fail("extra data after value")
	}
	return value, nil
}

func (p *Parser) parseValue() (Value, error) {
	p.skipWhitespace()
	c := p.peek()
	switch {
	case c == '{':
		return p.parseObject()
	case c == '[':
		return p.parseArray()
	case c == '"':
		return p.parseString()
	case c == '-' || strings.ContainsRune(digits, c):
		return p.parseNumber()
	}
	for _, lit := range literals {
		if p.takeWord(lit.word) {
			return lit.value, nil
		}
	}
	return nil, p.unexpected()
}

func (p *Parser) takeWord(word string) bool {
	end := p.pos + len(word)
	if end > len(p.text) || string(p.text[p.pos:end]) != word {
		return false
	}
	p.pos = end
	return true
}

func (p *Parser) parseObject() (Value, error) {
	p.pos++
	members := Object{}
	p.skipWhitespace()
	if p.peek() == '}' {
		p.pos++
		return members, nil
	}
	for {
		p.skipWhitespace()
		if p.peek() != '"' {
			return nil, p.unexpected()
		}
		key, err := p.parseString()
		if err != nil {
			return nil, err
		}
		p.skipWhitespace()
		if p.peek() != ':' {
			if p.atEnd() {
				return nil, p.unexpected()
			}
			return nil, p.fail("expected ':' after object key")
		}
		p.pos++
		value, err := p.parseValue()
		if err != nil {
			return nil, err
		}
		members = append(members, Member{key, value})
		done, err := p.endOfContainer('}')
		if err != nil {
			return nil, err
		}
		if done {
			return members, nil
		}
	}
}

func (p *Parser) parseArray() (Value, error) {
	p.pos++
	items := []Value{}
	p.skipWhitespace()
	if p.peek() == ']' {
		p.pos++
		return items, nil
	}
	for {
		item, err := p.parseValue()
		if err != nil {
			return nil, err
		}
		items = append(items, item)
		done, err := p.endOfContainer(']')
		if err != nil {
			return nil, err
		}
		if done {
			return items, nil
		}
	}
}

func (p *Parser) endOfContainer(closer rune) (bool, error) {
	p.skipWhitespace()
	c := p.peek()
	if c == closer {
		p.pos++
		return true, nil
	}
	if c != ',' {
		return false, p.unexpected()
	}
	comma := p.pos
	p.pos++
	p.skipWhitespace()
	if c := p.peek(); c == ']' || c == '}' {
		return false, p.failAt(comma, "trailing comma")
	}
	return false, nil
}

func (p *Parser) parseString() (string, error) {
	start := p.pos
	p.pos++
	var sb strings.Builder
	for {
		if p.atEnd() || p.peek() == '\n' {
			return "", p.failAt(start, "unterminated string")
		}
		switch c := p.peek(); {
		case c == '"':
			p.pos++
			return sb.String(), nil
		case c == '\\':
			r, err := p.parseEscape(start)
			if err != nil {
				return "", err
			}
			sb.WriteRune(r)
		case c < 0x20:
			return "", p.fail("control character in string")
		default:
			sb.WriteRune(c)
			p.pos++
		}
	}
}

func (p *Parser) parseEscape(start int) (rune, error) {
	escape := p.pos
	p.pos++
	if p.atEnd() {
		return 0, p.failAt(start, "unterminated string")
	}
	c := p.peek()
	p.pos++
	if r, ok := escapes[c]; ok {
		return r, nil
	}
	if c != 'u' {
		return 0, p.failAt(escape, "invalid escape '\\%c'", c)
	}
	end := p.pos + 4
	if end > len(p.text) {
		return 0, p.failAt(escape, "invalid unicode escape")
	}
	code, err := strconv.ParseUint(string(p.text[p.pos:end]), 16, 16)
	if err != nil {
		return 0, p.failAt(escape, "invalid unicode escape")
	}
	if utf16.IsSurrogate(rune(code)) {
		return 0, p.failAt(escape, "surrogate escapes are not supported")
	}
	p.pos = end
	return rune(code), nil
}

func (p *Parser) skipDigits() int {
	start := p.pos
	for strings.ContainsRune(digits, p.peek()) {
		p.pos++
	}
	return p.pos - start
}

func (p *Parser) parseNumber() (Value, error) {
	start := p.pos
	if p.peek() == '-' {
		p.pos++
	}
	if p.peek() == '0' {
		p.pos++
	} else if p.skipDigits() == 0 {
		return nil, p.fail("invalid number")
	}
	isFloat := false
	if p.peek() == '.' {
		p.pos++
		if p.skipDigits() == 0 {
			return nil, p.fail("invalid number")
		}
		isFloat = true
	}
	if p.peek() == 'e' || p.peek() == 'E' {
		p.pos++
		if p.peek() == '+' || p.peek() == '-' {
			p.pos++
		}
		if p.skipDigits() == 0 {
			return nil, p.fail("invalid number")
		}
		isFloat = true
	}
	text := string(p.text[start:p.pos])
	if isFloat {
		x, err := strconv.ParseFloat(text, 64)
		if err != nil {
			return nil, p.failAt(start, "invalid number")
		}
		return x, nil
	}
	n, err := strconv.Atoi(text)
	if err != nil {
		return nil, p.failAt(start, "number out of range")
	}
	return n, nil
}

func parse(text string) (Value, error) {
	p := &Parser{text: []rune(text)}
	return p.parseDocument()
}

func quote(s string) string {
	var sb strings.Builder
	sb.WriteByte('"')
	for _, c := range s {
		switch c {
		case '"':
			sb.WriteString(`\"`)
		case '\\':
			sb.WriteString(`\\`)
		case '\n':
			sb.WriteString(`\n`)
		case '\r':
			sb.WriteString(`\r`)
		case '\t':
			sb.WriteString(`\t`)
		case '\b':
			sb.WriteString(`\b`)
		case '\f':
			sb.WriteString(`\f`)
		default:
			if c < 0x20 {
				fmt.Fprintf(&sb, `\u%04x`, c)
			} else {
				sb.WriteRune(c)
			}
		}
	}
	sb.WriteByte('"')
	return sb.String()
}

func formatFloat(x float64) string {
	s := strconv.FormatFloat(x, 'f', -1, 64)
	if !strings.Contains(s, ".") {
		s += ".0"
	}
	return s
}

func scalar(v Value) string {
	switch v := v.(type) {
	case nil:
		return "null"
	case string:
		return quote(v)
	case float64:
		return formatFloat(v)
	default:
		return fmt.Sprint(v)
	}
}

func compact(v Value) string {
	switch v := v.(type) {
	case Object:
		members := make([]string, len(v))
		for i, m := range v {
			members[i] = quote(m.Key) + ":" + compact(m.Value)
		}
		return "{" + strings.Join(members, ",") + "}"
	case []Value:
		items := make([]string, len(v))
		for i, item := range v {
			items[i] = compact(item)
		}
		return "[" + strings.Join(items, ",") + "]"
	default:
		return scalar(v)
	}
}

func pretty(v Value, level int) string {
	pad := strings.Repeat("  ", level+1)
	closing := strings.Repeat("  ", level)
	switch v := v.(type) {
	case Object:
		if len(v) == 0 {
			return "{}"
		}
		members := make([]string, len(v))
		for i, m := range v {
			members[i] = pad + quote(m.Key) + ": " + pretty(m.Value, level+1)
		}
		return "{\n" + strings.Join(members, ",\n") + "\n" + closing + "}"
	case []Value:
		if len(v) == 0 {
			return "[]"
		}
		items := make([]string, len(v))
		for i, item := range v {
			items[i] = pad + pretty(item, level+1)
		}
		return "[\n" + strings.Join(items, ",\n") + "\n" + closing + "]"
	default:
		return scalar(v)
	}
}

func parsePath(path string) ([]any, bool) {
	var steps []any
	for _, part := range strings.Split(path, ".") {
		pieces := strings.Split(part, "[")
		if pieces[0] != "" {
			steps = append(steps, pieces[0])
		}
		for _, piece := range pieces[1:] {
			inner := strings.Split(piece, "]")
			if len(inner) != 2 || inner[1] != "" {
				return nil, false
			}
			n, err := strconv.Atoi(inner[0])
			if err != nil {
				return nil, false
			}
			steps = append(steps, n)
		}
	}
	return steps, true
}

func query(doc Value, steps []any) (Value, bool) {
	current := doc
	for _, step := range steps {
		switch step := step.(type) {
		case int:
			items, ok := current.([]Value)
			if !ok || step < 0 || step >= len(items) {
				return nil, false
			}
			current = items[step]
		case string:
			members, ok := current.(Object)
			if !ok {
				return nil, false
			}
			if current, ok = members.get(step); !ok {
				return nil, false
			}
		}
	}
	return current, true
}

type Stats struct {
	counts   map[string]int
	maxDepth int
	total    float64
}

func kindOf(v Value) string {
	switch v.(type) {
	case Object:
		return "object"
	case []Value:
		return "array"
	case string:
		return "string"
	case bool:
		return "boolean"
	case nil:
		return "null"
	default:
		return "number"
	}
}

func walk(v Value, depth int, stats *Stats) {
	stats.counts[kindOf(v)]++
	switch v := v.(type) {
	case int:
		stats.total += float64(v)
	case float64:
		stats.total += v
	case Object:
		stats.maxDepth = max(stats.maxDepth, depth)
		for _, m := range v {
			walk(m.Value, depth+1, stats)
		}
	case []Value:
		stats.maxDepth = max(stats.maxDepth, depth)
		for _, item := range v {
			walk(item, depth+1, stats)
		}
	}
}

const document = `
{
  "meta": {"version": 3, "count": 3, "generated": "2026-10-05", "archived": null},
  "users": [
    {
      "id": 1,
      "name": "Ann \"Annie\" Lee",
      "email": "ann@example.com",
      "active": true,
      "score": 3.5,
      "tags": ["admin", "dev", "ops"],
      "address": {"city": "Z\u00fcrich", "zip": "8001"}
    },
    {
      "id": 2,
      "name": "Jos\u00e9 Garc\u00eda",
      "active": false,
      "score": 1e3,
      "tags": [],
      "note": "line one\nline two\ttabbed \/ slash \\ backslash"
    },
    {
      "id": 3,
      "name": "\u674E\u96F7",
      "active": true,
      "score": -0.25,
      "tags": ["guest"],
      "balance": -42,
      "ratio": 2.5E-3,
      "prefs": {"theme": "dark", "langs": ["en", "zh"], "beta": {}},
      "flags": "\u0007\b\f\r"
    }
  ],
  "snowman": "\u2603"
}
`

var queries = []string{
	"users[1].name",
	"meta.count",
	"users[0].tags[2]",
	"users[2].prefs",
	"meta.archived",
	"users[1].email",
	"users[3]",
	"meta[0]",
	"users.name",
	"users[x]",
}

var badInputs = []string{
	`{"name": "Ann", "age": 30,}`,
	`{"name" "Ann"}`,
	"[\n  \"a\",\n  \"b\n]",
	`{"path": "C:\qdir"}`,
	"[1, 2] [3]",
	"[1, @]",
	`{"a": {"b": [1, 2`,
	"[-]",
	"{\n  \"x\": 1.,\n}",
	`"bad \u12G4"`,
	"",
}

func main() {
	doc, err := parse(document)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Println("== pretty ==")
	fmt.Println(pretty(doc, 0))

	fmt.Println("== compact ==")
	text := compact(doc)
	fmt.Println(text)
	reparsed, err := parse(text)
	if err == nil && reflect.DeepEqual(reparsed, doc) {
		fmt.Println("round trip: ok")
	} else {
		fmt.Println("round trip: MISMATCH")
	}

	fmt.Println("== queries ==")
	for _, path := range queries {
		steps, ok := parsePath(path)
		if !ok {
			fmt.Printf("bad path: %s\n", path)
			continue
		}
		if result, ok := query(doc, steps); ok {
			fmt.Printf("%s = %s\n", path, compact(result))
		} else {
			fmt.Printf("not found: %s\n", path)
		}
	}

	fmt.Println("== stats ==")
	stats := &Stats{counts: map[string]int{}}
	walk(doc, 1, stats)
	for _, kind := range kinds {
		fmt.Printf("%s: %d\n", kind, stats.counts[kind])
	}
	fmt.Printf("max depth: %d\n", stats.maxDepth)
	fmt.Printf("sum of numbers: %s\n", formatFloat(stats.total))

	fmt.Println("== errors ==")
	for i, text := range badInputs {
		value, err := parse(text)
		if err != nil {
			fmt.Printf("%d: %v\n", i+1, err)
			continue
		}
		fmt.Printf("%d: ok %s\n", i+1, compact(value))
	}
}
