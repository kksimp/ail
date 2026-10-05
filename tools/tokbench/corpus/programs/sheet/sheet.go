package main

import (
	"cmp"
	"errors"
	"fmt"
	"maps"
	"slices"
	"strconv"
	"strings"
	"unicode"
	"unicode/utf8"
)

const (
	columns = "ABCDEFGH"
	maxRow  = 20
)

type Pos struct {
	col int
	row int
}

func (p Pos) String() string {
	return fmt.Sprintf("%c%d", columns[p.col], p.row)
}

func (p Pos) Compare(q Pos) int {
	return cmp.Or(cmp.Compare(p.col, q.col), cmp.Compare(p.row, q.row))
}

func parsePos(text string) (Pos, bool) {
	if len(text) < 2 {
		return Pos{}, false
	}
	col := strings.IndexByte(columns, text[0])
	row, err := strconv.ParseUint(text[1:], 10, 64)
	if col < 0 || err != nil || row < 1 || row > maxRow {
		return Pos{}, false
	}
	return Pos{col, int(row)}, true
}

func cellsIn(first, last Pos) []Pos {
	c0, c1 := min(first.col, last.col), max(first.col, last.col)
	r0, r1 := min(first.row, last.row), max(first.row, last.row)
	var cells []Pos
	for r := r0; r <= r1; r++ {
		for c := c0; c <= c1; c++ {
			cells = append(cells, Pos{c, r})
		}
	}
	return cells
}

func names(cells []Pos) string {
	parts := make([]string, len(cells))
	for i, p := range cells {
		parts[i] = p.String()
	}
	return strings.Join(parts, ", ")
}

type Expr interface{ expr() }

type Lit struct{ n int }

type Ref struct{ pos Pos }

type Neg struct{ operand Expr }

type Bin struct {
	op          string
	left, right Expr
}

type Agg struct {
	fn          string
	first, last Pos
}

type If struct {
	cmp                      string
	left, right, then, other Expr
}

func (Lit) expr() {}
func (Ref) expr() {}
func (Neg) expr() {}
func (Bin) expr() {}
func (Agg) expr() {}
func (If) expr()  {}

var aggregates = []string{"SUM", "MIN", "MAX", "COUNT", "AVG"}

const symbols = "+-*/(),:<>="

type TokenKind int

const (
	tokNum TokenKind = iota
	tokCell
	tokName
	tokSym
	tokEnd
)

type Token struct {
	kind TokenKind
	text string
}

func (t Token) describe() string {
	switch t.kind {
	case tokEnd:
		return "end of formula"
	case tokSym:
		return fmt.Sprintf("'%s'", t.text)
	}
	return t.text
}

func tokenize(src string) ([]Token, error) {
	var tokens []Token
	runes := []rune(src)
	for i := 0; i < len(runes); {
		c := runes[i]
		switch {
		case c == ' ':
			i++
		case unicode.IsDigit(c):
			start := i
			for i < len(runes) && unicode.IsDigit(runes[i]) {
				i++
			}
			tokens = append(tokens, Token{tokNum, string(runes[start:i])})
		case unicode.IsLetter(c):
			start := i
			for i < len(runes) && (unicode.IsLetter(runes[i]) || unicode.IsDigit(runes[i])) {
				i++
			}
			word := string(runes[start:i])
			if unicode.IsDigit(runes[i-1]) {
				if _, ok := parsePos(word); !ok {
					return nil, fmt.Errorf("bad cell name %s", word)
				}
				tokens = append(tokens, Token{tokCell, word})
			} else {
				tokens = append(tokens, Token{tokName, word})
			}
		case strings.ContainsRune(symbols, c):
			tokens = append(tokens, Token{tokSym, string(c)})
			i++
		default:
			return nil, fmt.Errorf("unexpected character '%c'", c)
		}
	}
	return append(tokens, Token{tokEnd, ""}), nil
}

type Parser struct {
	tokens []Token
	at     int
}

func (p *Parser) peek() Token {
	return p.tokens[p.at]
}

func (p *Parser) next() Token {
	tok := p.tokens[p.at]
	if tok.kind != tokEnd {
		p.at++
	}
	return tok
}

func (p *Parser) atSym(syms string) bool {
	tok := p.peek()
	return tok.kind == tokSym && strings.Contains(syms, tok.text)
}

func (p *Parser) expect(sym string) error {
	tok := p.next()
	if tok.kind != tokSym || tok.text != sym {
		return fmt.Errorf("expected '%s' but found %s", sym, tok.describe())
	}
	return nil
}

func (p *Parser) cell() (Pos, error) {
	tok := p.next()
	if tok.kind != tokCell {
		return Pos{}, fmt.Errorf("expected a cell but found %s", tok.describe())
	}
	pos, _ := parsePos(tok.text)
	return pos, nil
}

func (p *Parser) formula() (Expr, error) {
	expr, err := p.sum()
	if err != nil {
		return nil, err
	}
	if p.peek().kind != tokEnd {
		return nil, fmt.Errorf("unexpected %s", p.peek().describe())
	}
	return expr, nil
}

func (p *Parser) sum() (Expr, error) {
	left, err := p.product()
	if err != nil {
		return nil, err
	}
	for p.atSym("+-") {
		op := p.next().text
		right, err := p.product()
		if err != nil {
			return nil, err
		}
		left = Bin{op, left, right}
	}
	return left, nil
}

func (p *Parser) product() (Expr, error) {
	left, err := p.unary()
	if err != nil {
		return nil, err
	}
	for p.atSym("*/") {
		op := p.next().text
		right, err := p.unary()
		if err != nil {
			return nil, err
		}
		left = Bin{op, left, right}
	}
	return left, nil
}

func (p *Parser) unary() (Expr, error) {
	if p.atSym("-") {
		p.next()
		operand, err := p.unary()
		if err != nil {
			return nil, err
		}
		return Neg{operand}, nil
	}
	return p.primary()
}

func (p *Parser) primary() (Expr, error) {
	tok := p.next()
	switch {
	case tok.kind == tokNum:
		n, err := strconv.Atoi(tok.text)
		if err != nil {
			return nil, fmt.Errorf("number too large: %s", tok.text)
		}
		return Lit{n}, nil
	case tok.kind == tokCell:
		pos, _ := parsePos(tok.text)
		return Ref{pos}, nil
	case tok.kind == tokName:
		return p.call(tok.text)
	case tok.kind == tokSym && tok.text == "(":
		inner, err := p.sum()
		if err != nil {
			return nil, err
		}
		if err := p.expect(")"); err != nil {
			return nil, err
		}
		return inner, nil
	}
	return nil, fmt.Errorf("unexpected %s", tok.describe())
}

func (p *Parser) call(name string) (Expr, error) {
	if slices.Contains(aggregates, name) {
		if err := p.expect("("); err != nil {
			return nil, err
		}
		first, err := p.cell()
		if err != nil {
			return nil, err
		}
		if err := p.expect(":"); err != nil {
			return nil, err
		}
		last, err := p.cell()
		if err != nil {
			return nil, err
		}
		if err := p.expect(")"); err != nil {
			return nil, err
		}
		return Agg{name, first, last}, nil
	}
	if name == "IF" {
		if err := p.expect("("); err != nil {
			return nil, err
		}
		left, err := p.sum()
		if err != nil {
			return nil, err
		}
		tok := p.next()
		if tok.kind != tokSym || !strings.Contains("<>=", tok.text) {
			return nil, fmt.Errorf("expected a comparison but found %s", tok.describe())
		}
		right, err := p.sum()
		if err != nil {
			return nil, err
		}
		if err := p.expect(","); err != nil {
			return nil, err
		}
		then, err := p.sum()
		if err != nil {
			return nil, err
		}
		if err := p.expect(","); err != nil {
			return nil, err
		}
		other, err := p.sum()
		if err != nil {
			return nil, err
		}
		if err := p.expect(")"); err != nil {
			return nil, err
		}
		return If{tok.text, left, right, then, other}, nil
	}
	return nil, fmt.Errorf("unknown function %s", name)
}

func parseFormula(src string) (Expr, error) {
	tokens, err := tokenize(src)
	if err != nil {
		return nil, err
	}
	return (&Parser{tokens: tokens}).formula()
}

func collectRefs(expr Expr, out map[Pos]bool) {
	switch e := expr.(type) {
	case Ref:
		out[e.pos] = true
	case Neg:
		collectRefs(e.operand, out)
	case Bin:
		collectRefs(e.left, out)
		collectRefs(e.right, out)
	case Agg:
		for _, pos := range cellsIn(e.first, e.last) {
			out[pos] = true
		}
	case If:
		for _, part := range []Expr{e.left, e.right, e.then, e.other} {
			collectRefs(part, out)
		}
	}
}

type Content interface{ content() }

type Number struct{ n int }

type Label struct{ text string }

type Formula struct {
	src  string
	expr Expr
}

type Broken struct {
	src string
	msg string
}

func (Number) content()  {}
func (Label) content()   {}
func (Formula) content() {}
func (Broken) content()  {}

type CellError string

func (e CellError) Error() string {
	return string(e)
}

const (
	errValue CellError = "#VALUE!"
	errCycle CellError = "#CYCLE!"
	errDiv0  CellError = "#DIV/0!"
	errParse CellError = "#PARSE!"
)

type Value any

func show(value Value) string {
	return fmt.Sprint(value)
}

func div(a, b int) (int, error) {
	if b == 0 {
		return 0, errDiv0
	}
	return a / b, nil
}

func sum(nums []int) int {
	total := 0
	for _, n := range nums {
		total += n
	}
	return total
}

func parseContent(text string) (Content, error) {
	if text == "" {
		return nil, nil
	}
	if strings.HasPrefix(text, "=") {
		src := strings.TrimSpace(text[1:])
		expr, err := parseFormula(src)
		if err != nil {
			return Broken{src, err.Error()}, nil
		}
		return Formula{src, expr}, nil
	}
	if len(text) >= 2 && strings.HasPrefix(text, `"`) && strings.HasSuffix(text, `"`) {
		return Label{text[1 : len(text)-1]}, nil
	}
	n, err := strconv.Atoi(text)
	if err != nil {
		return nil, fmt.Errorf("bad content '%s'", text)
	}
	return Number{n}, nil
}

func parseLine(line string) (Pos, Content, error) {
	name, rest, found := strings.Cut(line, "=")
	if !found {
		return Pos{}, nil, errors.New("missing '='")
	}
	name = strings.TrimSpace(name)
	pos, ok := parsePos(name)
	if !ok {
		return Pos{}, nil, fmt.Errorf("bad cell name '%s'", name)
	}
	content, err := parseContent(strings.TrimSpace(rest))
	return pos, content, err
}

type Sheet struct {
	cells  map[Pos]Content
	memo   map[Pos]Value
	active map[Pos]bool
}

func NewSheet() *Sheet {
	return &Sheet{
		cells:  map[Pos]Content{},
		memo:   map[Pos]Value{},
		active: map[Pos]bool{},
	}
}

func (s *Sheet) apply(lines []string) {
	for i, line := range lines {
		pos, content, err := parseLine(line)
		if err != nil {
			fmt.Printf("line %d: %v: %s\n", i+1, err, line)
			continue
		}
		if content == nil {
			delete(s.cells, pos)
		} else {
			s.cells[pos] = content
		}
	}
	s.memo = map[Pos]Value{}
}

func (s *Sheet) value(pos Pos) Value {
	if v, ok := s.memo[pos]; ok {
		return v
	}
	switch c := s.cells[pos].(type) {
	case nil:
		return 0
	case Number:
		return c.n
	case Label:
		return c.text
	case Broken:
		return errParse
	case Formula:
		if s.active[pos] {
			return errCycle
		}
		s.active[pos] = true
		var result Value
		n, err := s.eval(c.expr)
		if err != nil {
			result = err
		} else {
			result = n
		}
		delete(s.active, pos)
		s.memo[pos] = result
		return result
	}
	panic("unreachable")
}

func (s *Sheet) number(pos Pos) (int, error) {
	switch v := s.value(pos).(type) {
	case int:
		return v, nil
	case CellError:
		return 0, v
	}
	return 0, errValue
}

func (s *Sheet) numbers(first, last Pos) ([]int, error) {
	var found []int
	for _, pos := range cellsIn(first, last) {
		if _, ok := s.cells[pos]; !ok {
			continue
		}
		switch v := s.value(pos).(type) {
		case CellError:
			return nil, v
		case int:
			found = append(found, v)
		}
	}
	return found, nil
}

func (s *Sheet) eval(expr Expr) (int, error) {
	switch e := expr.(type) {
	case Lit:
		return e.n, nil
	case Ref:
		return s.number(e.pos)
	case Neg:
		n, err := s.eval(e.operand)
		return -n, err
	case Bin:
		a, err := s.eval(e.left)
		if err != nil {
			return 0, err
		}
		b, err := s.eval(e.right)
		if err != nil {
			return 0, err
		}
		switch e.op {
		case "+":
			return a + b, nil
		case "-":
			return a - b, nil
		case "*":
			return a * b, nil
		}
		return div(a, b)
	case Agg:
		nums, err := s.numbers(e.first, e.last)
		if err != nil {
			return 0, err
		}
		switch e.fn {
		case "SUM":
			return sum(nums), nil
		case "COUNT":
			return len(nums), nil
		case "AVG":
			return div(sum(nums), len(nums))
		}
		if len(nums) == 0 {
			return 0, nil
		}
		if e.fn == "MIN" {
			return slices.Min(nums), nil
		}
		return slices.Max(nums), nil
	case If:
		a, err := s.eval(e.left)
		if err != nil {
			return 0, err
		}
		b, err := s.eval(e.right)
		if err != nil {
			return 0, err
		}
		var holds bool
		switch e.cmp {
		case "<":
			holds = a < b
		case ">":
			holds = a > b
		default:
			holds = a == b
		}
		if holds {
			return s.eval(e.then)
		}
		return s.eval(e.other)
	}
	panic("unreachable")
}

func (s *Sheet) display(pos Pos) string {
	if _, ok := s.cells[pos]; !ok {
		return ""
	}
	return show(s.value(pos))
}

func (s *Sheet) snapshot() map[Pos]string {
	snap := map[Pos]string{}
	for pos := range s.cells {
		snap[pos] = s.display(pos)
	}
	return snap
}

func (s *Sheet) formulas() []Pos {
	var found []Pos
	for pos, content := range s.cells {
		switch content.(type) {
		case Formula, Broken:
			found = append(found, pos)
		}
	}
	slices.SortFunc(found, Pos.Compare)
	return found
}

func (s *Sheet) deps(pos Pos) []Pos {
	formula, ok := s.cells[pos].(Formula)
	if !ok {
		return nil
	}
	refs := map[Pos]bool{}
	collectRefs(formula.expr, refs)
	return slices.SortedFunc(maps.Keys(refs), Pos.Compare)
}

func (s *Sheet) evalOrder() (order, blocked []Pos) {
	formulas := s.formulas()
	isFormula := map[Pos]bool{}
	for _, pos := range formulas {
		isFormula[pos] = true
	}
	waits := map[Pos][]Pos{}
	for _, pos := range formulas {
		for _, dep := range s.deps(pos) {
			if isFormula[dep] {
				waits[pos] = append(waits[pos], dep)
			}
		}
	}
	done := map[Pos]bool{}
	ready := func(pos Pos) bool {
		return !done[pos] && !slices.ContainsFunc(waits[pos], func(dep Pos) bool { return !done[dep] })
	}
	for {
		i := slices.IndexFunc(formulas, ready)
		if i < 0 {
			break
		}
		order = append(order, formulas[i])
		done[formulas[i]] = true
	}
	for _, pos := range formulas {
		if !done[pos] {
			blocked = append(blocked, pos)
		}
	}
	return order, blocked
}

func (s *Sheet) table() []string {
	colSet, rowSet := map[int]bool{}, map[int]bool{}
	for pos := range s.cells {
		colSet[pos.col] = true
		rowSet[pos.row] = true
	}
	cols := slices.Sorted(maps.Keys(colSet))
	rows := slices.Sorted(maps.Keys(rowSet))
	grid := [][]string{{""}}
	for _, c := range cols {
		grid[0] = append(grid[0], string(columns[c]))
	}
	for _, r := range rows {
		line := []string{strconv.Itoa(r)}
		for _, c := range cols {
			line = append(line, s.display(Pos{c, r}))
		}
		grid = append(grid, line)
	}
	widths := make([]int, len(grid[0]))
	for _, line := range grid {
		for i, text := range line {
			widths[i] = max(widths[i], utf8.RuneCountInString(text))
		}
	}
	var lines []string
	for k, line := range grid {
		padded := make([]string, len(line))
		for i, text := range line {
			padded[i] = fmt.Sprintf("%*s", widths[i], text)
		}
		lines = append(lines, strings.Join(padded, " | "))
		if k == 0 {
			dashes := make([]string, len(widths))
			for i, w := range widths {
				dashes[i] = strings.Repeat("-", w)
			}
			lines = append(lines, strings.Join(dashes, "-+-"))
		}
	}
	return lines
}

func (s *Sheet) printTable() {
	for _, line := range s.table() {
		fmt.Println(line)
	}
}

func (s *Sheet) printDeps() {
	fmt.Println("dependencies:")
	for _, pos := range s.formulas() {
		if broken, ok := s.cells[pos].(Broken); ok {
			fmt.Printf("  %s <- %s %s\n", pos, errParse, broken.msg)
			continue
		}
		fmt.Printf("  %s <- %s\n", pos, cmp.Or(names(s.deps(pos)), "(none)"))
	}
}

func (s *Sheet) printParseErrors() {
	for _, pos := range s.formulas() {
		if broken, ok := s.cells[pos].(Broken); ok {
			fmt.Printf("parse error in %s: %s in '%s'\n", pos, broken.msg, broken.src)
		}
	}
}

func (s *Sheet) printOrder() {
	order, blocked := s.evalOrder()
	fmt.Printf("order: %s\n", names(order))
	if len(blocked) > 0 {
		fmt.Printf("blocked by a cycle: %s\n", names(blocked))
	}
}

func printChanges(before, after map[Pos]string) {
	all := maps.Clone(before)
	maps.Copy(all, after)
	var changed []string
	for _, pos := range slices.SortedFunc(maps.Keys(all), Pos.Compare) {
		was, now := before[pos], after[pos]
		if was != now {
			changed = append(changed, fmt.Sprintf("%s %s -> %s", pos, cmp.Or(was, "(empty)"), cmp.Or(now, "(empty)")))
		}
	}
	if len(changed) == 0 {
		fmt.Println("changed: nothing")
		return
	}
	fmt.Println("changed:")
	for _, line := range changed {
		fmt.Printf("  %s\n", line)
	}
}

const budget = `A1 = "Item"
B1 = "Qty"
C1 = "Price"
D1 = "Total"
E1 = "Share"
A2 = "Pens"
B2 = 10
C2 = 2
D2 = =B2 * C2
E2 = =D2 * 100 / D7
A3 = "Paper"
B3 = 5
C3 = 7
D3 = =B3 * C3
E3 = =D3 * 100 / D7
A4 = "Ink"
B4 = 2
C4 = 25
D4 = =B4*C4
E4 = =D4 * 100 / D7
A5 = "Stapler"
B5 = 1
C5 = 15
D5 = =B5 * C5
E5 = =D5 * 100 / D7
A7 = "Sum"
B7 = =SUM(B2:B5)
D7 = =SUM(D2:D5)
E7 = =SUM(E2:E5)
A8 = "Average"
C8 = =AVG(C2:C5)
D8 = =AVG(D2:D5)
A9 = "Range"
C9 = =MAX(C2:C5) - MIN(C2:C5)
D9 = =MAX(D5:D2) - MIN(D2:D5)
A10 = "Numbers"
B10 = =COUNT(A1:C5)
A11 = "Budget"
D11 = 200
A12 = "Left"
D12 = =D11 - D7
A13 = "Status"
D13 = =IF(D7 > D11, -1, IF(D7 = D11, 0, 1))
E13 = =IF(D12 < 100, -(2 * (D11 / 3) - D12) / 3, 0) + -B5
G15 = "end"
I3 = 5
B6 12
C6 = twelve`

var edits = []struct {
	title string
	lines []string
}{
	{"change a value", []string{"B2 = 20"}},
	{"introduce a cycle", []string{"D11 = =D12 + 50", "G15 = =G15 + 1"}},
	{"break the cycle", []string{"D11 = 250", "G15 = "}},
	{"divide by zero", []string{"C5 = 0", "B11 = =D11 / (B5 - 1)", "B12 = =AVG(F1:F3)"}},
	{"parse errors", []string{
		"D4 = =B4 * (C4 + 1",
		"B12 = =SUMM(B2:B5)",
		"B13 = =B2 $ 3",
		"C11 = =IF(B2, 1, 2)",
		"C12 = =Z9 + 1",
		"C13 = =3 4",
	}},
	{"fix the formula, then reference a text cell", []string{
		"D4 = =B4 * (C4 + 1)",
		"B11 = =A2 * 2",
		"B12 = =SUM(A2:A5) + COUNT(A2:B5)",
		"B13 = =IF(A3 = 1, 1, 0)",
	}},
}

func main() {
	sheet := NewSheet()
	fmt.Println("== budget")
	sheet.apply(strings.Split(budget, "\n"))
	sheet.printTable()
	sheet.printDeps()
	sheet.printOrder()
	for i, edit := range edits {
		fmt.Println()
		fmt.Printf("== edit %d: %s\n", i+1, edit.title)
		for _, line := range edit.lines {
			fmt.Printf("  %s\n", line)
		}
		before := sheet.snapshot()
		sheet.apply(edit.lines)
		sheet.printTable()
		printChanges(before, sheet.snapshot())
		sheet.printParseErrors()
		sheet.printOrder()
	}
	fmt.Println()
	fmt.Println("== final")
	sheet.printDeps()
}
