package main

import (
	"fmt"
	"math"
	"slices"
	"strconv"
	"strings"
	"unicode"
)

const (
	maxDepth = 200
	eof      = rune(-1)
)

var (
	keywords   = map[string]bool{"let": true, "if": true, "else": true, "while": true, "fn": true, "return": true, "true": true, "false": true, "and": true, "or": true, "not": true}
	twoCharOps = map[string]bool{"==": true, "!=": true, "<=": true, ">=": true}
	escapes    = map[rune]rune{'n': '\n', '"': '"', '\\': '\\'}
)

const oneCharOps = "+-*/%<>=(){},"

type Pos struct{ Line, Col int }

func (p Pos) Position() Pos { return p }

type MiniError struct {
	Pos     Pos
	Message string
}

func (e *MiniError) Error() string {
	return fmt.Sprintf("error at %d:%d: %s", e.Pos.Line, e.Pos.Col, e.Message)
}

func errorf(pos Pos, format string, args ...any) error {
	return &MiniError{pos, fmt.Sprintf(format, args...)}
}

type TokenKind int

const (
	tokEnd TokenKind = iota
	tokInt
	tokStr
	tokIdent
	tokKeyword
	tokOp
)

type Token struct {
	kind TokenKind
	text string
	pos  Pos
}

func describe(tok Token) string {
	switch tok.kind {
	case tokEnd:
		return "end of input"
	case tokInt:
		return "number " + tok.text
	case tokStr:
		return "string"
	}
	return "'" + tok.text + "'"
}

type Lexer struct {
	src  []rune
	i    int
	line int
	col  int
}

func newLexer(source string) *Lexer {
	return &Lexer{src: []rune(source), line: 1, col: 1}
}

func (l *Lexer) peekAt(offset int) rune {
	if j := l.i + offset; j < len(l.src) {
		return l.src[j]
	}
	return eof
}

func (l *Lexer) peek() rune { return l.peekAt(0) }

func (l *Lexer) advance() rune {
	c := l.src[l.i]
	l.i++
	if c == '\n' {
		l.line++
		l.col = 1
	} else {
		l.col++
	}
	return c
}

func (l *Lexer) tokenize() ([]Token, error) {
	var tokens []Token
	for {
		l.skipSpaceAndComments()
		pos := Pos{l.line, l.col}
		c := l.peek()
		switch {
		case c == eof:
			return append(tokens, Token{tokEnd, "", pos}), nil
		case unicode.IsDigit(c):
			tokens = append(tokens, l.number(pos))
		case unicode.IsLetter(c) || c == '_':
			tokens = append(tokens, l.word(pos))
		case c == '"':
			tok, err := l.str(pos)
			if err != nil {
				return nil, err
			}
			tokens = append(tokens, tok)
		case twoCharOps[string([]rune{c, l.peekAt(1)})]:
			tokens = append(tokens, Token{tokOp, string([]rune{l.advance(), l.advance()}), pos})
		case strings.ContainsRune(oneCharOps, c):
			tokens = append(tokens, Token{tokOp, string(l.advance()), pos})
		default:
			return nil, errorf(pos, "unexpected character '%c'", c)
		}
	}
}

func (l *Lexer) skipSpaceAndComments() {
	for {
		c := l.peek()
		switch {
		case strings.ContainsRune(" \n\t\r", c):
			l.advance()
		case c == '/' && l.peekAt(1) == '/':
			for l.peek() != eof && l.peek() != '\n' {
				l.advance()
			}
		default:
			return
		}
	}
}

func (l *Lexer) number(pos Pos) Token {
	start := l.i
	for unicode.IsDigit(l.peek()) {
		l.advance()
	}
	return Token{tokInt, string(l.src[start:l.i]), pos}
}

func (l *Lexer) word(pos Pos) Token {
	start := l.i
	for c := l.peek(); unicode.IsLetter(c) || unicode.IsDigit(c) || c == '_'; c = l.peek() {
		l.advance()
	}
	text := string(l.src[start:l.i])
	if keywords[text] {
		return Token{tokKeyword, text, pos}
	}
	return Token{tokIdent, text, pos}
}

func (l *Lexer) str(pos Pos) (Token, error) {
	l.advance()
	var sb strings.Builder
	for {
		c := l.peek()
		if c == eof || c == '\n' {
			return Token{}, errorf(pos, "unterminated string")
		}
		l.advance()
		switch c {
		case '"':
			return Token{tokStr, sb.String(), pos}, nil
		case '\\':
			escapePos := Pos{l.line, l.col - 1}
			e := l.peek()
			if e == eof || e == '\n' {
				continue
			}
			escaped, ok := escapes[e]
			if !ok {
				return Token{}, errorf(escapePos, "unknown escape '\\%c'", e)
			}
			l.advance()
			sb.WriteRune(escaped)
		default:
			sb.WriteRune(c)
		}
	}
}

type Expr interface{ Position() Pos }

type Stmt interface{ Position() Pos }

type IntLit struct {
	Pos
	Value int64
}

type StrLit struct {
	Pos
	Value string
}

type BoolLit struct {
	Pos
	Value bool
}

type Var struct {
	Pos
	Name string
}

type Unary struct {
	Pos
	Op      string
	Operand Expr
}

type Binary struct {
	Pos
	Op          string
	Left, Right Expr
}

type Call struct {
	Pos
	Name string
	Args []Expr
}

type Let struct {
	Pos
	Name  string
	Value Expr
}

type Assign struct {
	Pos
	Name  string
	Value Expr
}

type If struct {
	Pos
	Cond      Expr
	Then      []Stmt
	Otherwise []Stmt
}

type While struct {
	Pos
	Cond Expr
	Body []Stmt
}

type FnDecl struct {
	Pos
	Name   string
	Params []string
	Body   []Stmt
}

type Return struct {
	Pos
	Value Expr
}

type ExprStmt struct {
	Pos
	Expr Expr
}

var binaryLevels = [][]string{
	{"or"},
	{"and"},
	{"==", "!="},
	{"<", "<=", ">", ">="},
	{"+", "-"},
	{"*", "/", "%"},
}

type Parser struct {
	tokens []Token
	i      int
}

func (p *Parser) peekAt(offset int) Token {
	return p.tokens[min(p.i+offset, len(p.tokens)-1)]
}

func (p *Parser) peek() Token { return p.peekAt(0) }

func (p *Parser) advance() Token {
	tok := p.peek()
	if tok.kind != tokEnd {
		p.i++
	}
	return tok
}

func (p *Parser) at(texts ...string) bool {
	tok := p.peek()
	return (tok.kind == tokOp || tok.kind == tokKeyword) && slices.Contains(texts, tok.text)
}

func (p *Parser) expect(text string) (Token, error) {
	if !p.at(text) {
		return Token{}, errorf(p.peek().pos, "expected '%s' but found %s", text, describe(p.peek()))
	}
	return p.advance(), nil
}

func (p *Parser) expectIdent() (string, error) {
	tok := p.peek()
	if tok.kind != tokIdent {
		return "", errorf(tok.pos, "expected identifier but found %s", describe(tok))
	}
	return p.advance().text, nil
}

func (p *Parser) program() ([]Stmt, error) {
	var stmts []Stmt
	for p.peek().kind != tokEnd {
		stmt, err := p.statement()
		if err != nil {
			return nil, err
		}
		stmts = append(stmts, stmt)
	}
	return stmts, nil
}

func (p *Parser) block() ([]Stmt, error) {
	if _, err := p.expect("{"); err != nil {
		return nil, err
	}
	var stmts []Stmt
	for !p.at("}") {
		if p.peek().kind == tokEnd {
			return nil, errorf(p.peek().pos, "expected '}' but found end of input")
		}
		stmt, err := p.statement()
		if err != nil {
			return nil, err
		}
		stmts = append(stmts, stmt)
	}
	p.advance()
	return stmts, nil
}

func (p *Parser) statement() (Stmt, error) {
	tok := p.peek()
	pos := tok.pos
	next := p.peekAt(1)
	switch {
	case p.at("let"):
		p.advance()
		name, err := p.expectIdent()
		if err != nil {
			return nil, err
		}
		if _, err := p.expect("="); err != nil {
			return nil, err
		}
		value, err := p.expression()
		if err != nil {
			return nil, err
		}
		return &Let{pos, name, value}, nil
	case p.at("if"):
		return p.ifStatement()
	case p.at("while"):
		p.advance()
		cond, err := p.expression()
		if err != nil {
			return nil, err
		}
		body, err := p.block()
		if err != nil {
			return nil, err
		}
		return &While{pos, cond, body}, nil
	case p.at("fn"):
		p.advance()
		name, err := p.expectIdent()
		if err != nil {
			return nil, err
		}
		if _, err := p.expect("("); err != nil {
			return nil, err
		}
		var params []string
		if !p.at(")") {
			for {
				param, err := p.expectIdent()
				if err != nil {
					return nil, err
				}
				params = append(params, param)
				if !p.at(",") {
					break
				}
				p.advance()
			}
		}
		if _, err := p.expect(")"); err != nil {
			return nil, err
		}
		body, err := p.block()
		if err != nil {
			return nil, err
		}
		return &FnDecl{pos, name, params, body}, nil
	case p.at("return"):
		p.advance()
		if p.at("}") {
			return &Return{pos, nil}, nil
		}
		value, err := p.expression()
		if err != nil {
			return nil, err
		}
		return &Return{pos, value}, nil
	case tok.kind == tokIdent && next.kind == tokOp && next.text == "=":
		p.advance()
		p.advance()
		value, err := p.expression()
		if err != nil {
			return nil, err
		}
		return &Assign{pos, tok.text, value}, nil
	}
	expr, err := p.expression()
	if err != nil {
		return nil, err
	}
	return &ExprStmt{pos, expr}, nil
}

func (p *Parser) ifStatement() (Stmt, error) {
	tok, err := p.expect("if")
	if err != nil {
		return nil, err
	}
	cond, err := p.expression()
	if err != nil {
		return nil, err
	}
	then, err := p.block()
	if err != nil {
		return nil, err
	}
	var otherwise []Stmt
	if p.at("else") {
		p.advance()
		if p.at("if") {
			nested, err := p.ifStatement()
			if err != nil {
				return nil, err
			}
			otherwise = []Stmt{nested}
		} else if otherwise, err = p.block(); err != nil {
			return nil, err
		}
	}
	return &If{tok.pos, cond, then, otherwise}, nil
}

func (p *Parser) expression() (Expr, error) { return p.binary(0) }

func (p *Parser) binary(level int) (Expr, error) {
	if level == len(binaryLevels) {
		return p.unary()
	}
	left, err := p.binary(level + 1)
	if err != nil {
		return nil, err
	}
	for p.at(binaryLevels[level]...) {
		op := p.advance()
		right, err := p.binary(level + 1)
		if err != nil {
			return nil, err
		}
		left = &Binary{op.pos, op.text, left, right}
	}
	return left, nil
}

func (p *Parser) unary() (Expr, error) {
	if p.at("-", "not") {
		op := p.advance()
		operand, err := p.unary()
		if err != nil {
			return nil, err
		}
		return &Unary{op.pos, op.text, operand}, nil
	}
	return p.primary()
}

func (p *Parser) primary() (Expr, error) {
	tok := p.advance()
	switch tok.kind {
	case tokInt:
		value, err := strconv.ParseInt(tok.text, 10, 64)
		if err != nil {
			return nil, errorf(tok.pos, "number too large")
		}
		return &IntLit{tok.pos, value}, nil
	case tokStr:
		return &StrLit{tok.pos, tok.text}, nil
	case tokIdent:
		if !p.at("(") {
			return &Var{tok.pos, tok.text}, nil
		}
		p.advance()
		var args []Expr
		if !p.at(")") {
			for {
				arg, err := p.expression()
				if err != nil {
					return nil, err
				}
				args = append(args, arg)
				if !p.at(",") {
					break
				}
				p.advance()
			}
		}
		if _, err := p.expect(")"); err != nil {
			return nil, err
		}
		return &Call{tok.pos, tok.text, args}, nil
	case tokKeyword:
		if tok.text == "true" || tok.text == "false" {
			return &BoolLit{tok.pos, tok.text == "true"}, nil
		}
	case tokOp:
		if tok.text == "(" {
			inner, err := p.expression()
			if err != nil {
				return nil, err
			}
			if _, err := p.expect(")"); err != nil {
				return nil, err
			}
			return inner, nil
		}
	}
	return nil, errorf(tok.pos, "expected expression but found %s", describe(tok))
}

type Value any

func typeName(v Value) string {
	switch v.(type) {
	case bool:
		return "bool"
	case int64:
		return "int"
	case string:
		return "str"
	}
	return "nil"
}

func show(v Value) string {
	if v == nil {
		return "nil"
	}
	return fmt.Sprint(v)
}

type returnSignal struct{ value Value }

type Env struct {
	scopes []map[string]Value
	base   int
}

func (e *Env) find(name string) map[string]Value {
	for i := len(e.scopes) - 1; i >= e.base; i-- {
		if _, ok := e.scopes[i][name]; ok {
			return e.scopes[i]
		}
	}
	if _, ok := e.scopes[0][name]; ok {
		return e.scopes[0]
	}
	return nil
}

type Interpreter struct {
	env       Env
	functions map[string]*FnDecl
	depth     int
}

func newInterpreter() *Interpreter {
	return &Interpreter{
		env:       Env{scopes: []map[string]Value{{}}},
		functions: map[string]*FnDecl{},
	}
}

func (in *Interpreter) run(program []Stmt) error {
	_, err := in.executeAll(program)
	return err
}

func (in *Interpreter) executeAll(body []Stmt) (*returnSignal, error) {
	for _, stmt := range body {
		if ret, err := in.execute(stmt); ret != nil || err != nil {
			return ret, err
		}
	}
	return nil, nil
}

func (in *Interpreter) executeBlock(body []Stmt) (*returnSignal, error) {
	in.env.scopes = append(in.env.scopes, map[string]Value{})
	ret, err := in.executeAll(body)
	in.env.scopes = in.env.scopes[:len(in.env.scopes)-1]
	return ret, err
}

func (in *Interpreter) execute(stmt Stmt) (*returnSignal, error) {
	switch s := stmt.(type) {
	case *Let:
		value, err := in.evaluate(s.Value)
		if err != nil {
			return nil, err
		}
		in.env.scopes[len(in.env.scopes)-1][s.Name] = value
	case *Assign:
		scope := in.env.find(s.Name)
		if scope == nil {
			return nil, errorf(s.Pos, "undefined variable '%s'", s.Name)
		}
		value, err := in.evaluate(s.Value)
		if err != nil {
			return nil, err
		}
		scope[s.Name] = value
	case *If:
		cond, err := in.condition(s.Cond)
		if err != nil {
			return nil, err
		}
		if cond {
			return in.executeBlock(s.Then)
		}
		if s.Otherwise != nil {
			return in.executeBlock(s.Otherwise)
		}
	case *While:
		for {
			cond, err := in.condition(s.Cond)
			if err != nil || !cond {
				return nil, err
			}
			if ret, err := in.executeBlock(s.Body); ret != nil || err != nil {
				return ret, err
			}
		}
	case *FnDecl:
		in.functions[s.Name] = s
	case *Return:
		var value Value
		if s.Value != nil {
			var err error
			if value, err = in.evaluate(s.Value); err != nil {
				return nil, err
			}
		}
		return &returnSignal{value}, nil
	case *ExprStmt:
		_, err := in.evaluate(s.Expr)
		return nil, err
	}
	return nil, nil
}

func (in *Interpreter) condition(expr Expr) (bool, error) {
	v, err := in.evaluate(expr)
	if err != nil {
		return false, err
	}
	b, ok := v.(bool)
	if !ok {
		return false, errorf(expr.Position(), "type mismatch: condition must be bool, got %s", typeName(v))
	}
	return b, nil
}

func (in *Interpreter) evaluate(expr Expr) (Value, error) {
	switch e := expr.(type) {
	case *IntLit:
		return e.Value, nil
	case *StrLit:
		return e.Value, nil
	case *BoolLit:
		return e.Value, nil
	case *Var:
		scope := in.env.find(e.Name)
		if scope == nil {
			return nil, errorf(e.Pos, "undefined variable '%s'", e.Name)
		}
		return scope[e.Name], nil
	case *Unary:
		v, err := in.evaluate(e.Operand)
		if err != nil {
			return nil, err
		}
		switch x := v.(type) {
		case int64:
			if e.Op == "-" {
				if x == math.MinInt64 {
					return nil, errorf(e.Pos, "integer overflow")
				}
				return -x, nil
			}
		case bool:
			if e.Op == "not" {
				return !x, nil
			}
		}
		return nil, errorf(e.Pos, "type mismatch: cannot apply '%s' to %s", e.Op, typeName(v))
	case *Binary:
		if e.Op == "and" || e.Op == "or" {
			return in.logical(e)
		}
		left, err := in.evaluate(e.Left)
		if err != nil {
			return nil, err
		}
		right, err := in.evaluate(e.Right)
		if err != nil {
			return nil, err
		}
		return in.binary(e.Pos, e.Op, left, right)
	case *Call:
		args := make([]Value, len(e.Args))
		for i, arg := range e.Args {
			v, err := in.evaluate(arg)
			if err != nil {
				return nil, err
			}
			args[i] = v
		}
		return in.call(e.Pos, e.Name, args)
	}
	panic(fmt.Sprintf("unknown expression %T", expr))
}

func (in *Interpreter) boolOperand(e *Binary, operand Expr) (bool, error) {
	v, err := in.evaluate(operand)
	if err != nil {
		return false, err
	}
	b, ok := v.(bool)
	if !ok {
		return false, errorf(e.Pos, "type mismatch: '%s' needs bool operands, got %s", e.Op, typeName(v))
	}
	return b, nil
}

func (in *Interpreter) logical(e *Binary) (Value, error) {
	a, err := in.boolOperand(e, e.Left)
	if err != nil {
		return nil, err
	}
	if (e.Op == "and" && !a) || (e.Op == "or" && a) {
		return a, nil
	}
	return in.boolOperand(e, e.Right)
}

func (in *Interpreter) binary(pos Pos, op string, a, b Value) (Value, error) {
	switch op {
	case "==":
		return a == b, nil
	case "!=":
		return a != b, nil
	}
	if x, ok := a.(string); ok && op == "+" {
		if y, ok := b.(string); ok {
			return x + y, nil
		}
	}
	x, xok := a.(int64)
	y, yok := b.(int64)
	if !xok || !yok {
		return nil, errorf(pos, "type mismatch: cannot apply '%s' to %s and %s", op, typeName(a), typeName(b))
	}
	var result int64
	var overflow bool
	switch op {
	case "+":
		result = x + y
		overflow = (y > 0 && result < x) || (y < 0 && result > x)
	case "-":
		result = x - y
		overflow = (y > 0 && result > x) || (y < 0 && result < x)
	case "*":
		result = x * y
		overflow = x != 0 && (result/x != y || (x == -1 && y == math.MinInt64))
	case "/", "%":
		if y == 0 {
			return nil, errorf(pos, "division by zero")
		}
		if op == "/" {
			result = x / y
			overflow = x == math.MinInt64 && y == -1
		} else {
			result = x % y
		}
	case "<":
		return x < y, nil
	case "<=":
		return x <= y, nil
	case ">":
		return x > y, nil
	case ">=":
		return x >= y, nil
	default:
		panic("unknown operator " + op)
	}
	if overflow {
		return nil, errorf(pos, "integer overflow")
	}
	return result, nil
}

func (in *Interpreter) call(pos Pos, name string, args []Value) (Value, error) {
	if name == "print" || name == "str" {
		if len(args) != 1 {
			return nil, errorf(pos, "wrong number of arguments: '%s' takes 1 but got %d", name, len(args))
		}
		if name == "str" {
			return show(args[0]), nil
		}
		fmt.Println(show(args[0]))
		return nil, nil
	}
	fn, ok := in.functions[name]
	if !ok {
		return nil, errorf(pos, "undefined function '%s'", name)
	}
	if len(args) != len(fn.Params) {
		return nil, errorf(pos, "wrong number of arguments: '%s' takes %d but got %d", name, len(fn.Params), len(args))
	}
	if in.depth == maxDepth {
		return nil, errorf(pos, "stack overflow")
	}
	savedBase := in.env.base
	in.env.base = len(in.env.scopes)
	scope := make(map[string]Value, len(args))
	for i, param := range fn.Params {
		scope[param] = args[i]
	}
	in.env.scopes = append(in.env.scopes, scope)
	in.depth++
	ret, err := in.executeAll(fn.Body)
	in.depth--
	in.env.scopes = in.env.scopes[:in.env.base]
	in.env.base = savedBase
	if err != nil || ret == nil {
		return nil, err
	}
	return ret.value, nil
}

type script struct{ name, source string }

var scripts = []script{
	{"fibonacci", `// The first sixteen Fibonacci numbers, on one line
fn fib(n) {
    if n < 2 {
        return n
    }
    return fib(n - 1) + fib(n - 2)
}
let line = ""
let i = 0
while i <= 15 {
    if i > 0 {
        line = line + " "
    }
    line = line + str(fib(i))
    i = i + 1
}
print(line)
`},
	{"factorial", `fn fact(n) {
    if n <= 1 {
        return 1
    }
    return n * fact(n - 1)
}
fn fact_loop(n) {
    let result = 1
    while n > 1 {
        result = result * n
        n = n - 1
    }
    return result
}
print(fact(10))
print(fact(20))
print(fact(12) == fact_loop(12))
print(fact(21))
`},
	{"fizzbuzz", `let n = 1
let out = ""
while n <= 15 {
    if n % 15 == 0 {
        out = out + "FizzBuzz"
    } else if n % 3 == 0 {
        out = out + "Fizz"
    } else {
        if n % 5 == 0 {
            out = out + "Buzz"
        } else {
            out = out + str(n)
        }
    }
    if n < 15 { out = out + "," }
    n = n + 1
}
print(out)
`},
	{"strings", `fn repeat(s, times) {
    let out = ""
    while times > 0 {
        out = out + s
        times = times - 1
    }
    return out
}
let row = 1
while row <= 4 {
    print(repeat(" ", 4 - row) + repeat("*", 2 * row - 1))
    row = row + 1
}
print("she said \"hi\"\nand left")
print("ab" + "cd" == "abcd")
print(1 == "1")
print(true != 1)
print(-7 / 2)
print(-7 % 2)
print(not (3 > 2) or 2 >= 2)
print(false and missing)
print(true or missing())
fn nothing() { return }
print(nothing())
`},
	{"even and odd", `fn is_even(n) {
    if n == 0 { return true }
    return is_odd(n - 1)
}
fn is_odd(n) {
    if n == 0 { return false }
    return is_even(n - 1)
}
print(is_even(10))
print(is_odd(7))
print(is_even(7))
print(is_odd(150))
`},
	{"scopes", `let x = "global"
let count = 0
fn bump() {
    count = count + 1
    let x = "local"
    return x
}
if true {
    let x = "block"
    print(x)
    count = 10
}
print(x)
print(bump())
print(count)
`},
	{"bad character", `let a = 1
let b = a # 2
`},
	{"unterminated string", `print("ok")
print("hello)
`},
	{"missing paren", `fn add(a, b {
    return a + b
}
`},
	{"undefined variable", `fn peek() {
    return secret
}
let secret = 1
fn outer() {
    let hidden = 2
    return inner()
}
fn inner() { return hidden }
print(peek())
print(outer())
`},
	{"type mismatch", `let total = 5
print("total: " + str(total))
let flag = true
print(total + flag)
`},
	{"division by zero", `fn average(sum, count) {
    return sum / count
}
print(average(10, 2))
print(average(7, 0))
`},
	{"undefined function", `fn square(n) { return n * n }
print(square(4))
print(sqaure(5))
`},
	{"wrong arguments", `fn add(a, b) { return a + b }
print(add(1, 2))
print(add(1, 2, 3))
`},
	{"stack overflow", `fn down(n) {
    return down(n + 1)
}
print("going down")
down(0)
`},
	{"bad condition", `let n = 3
while n {
    n = n - 1
}
`},
}

func runScript(source string) error {
	tokens, err := newLexer(source).tokenize()
	if err != nil {
		return err
	}
	program, err := (&Parser{tokens: tokens}).program()
	if err != nil {
		return err
	}
	return newInterpreter().run(program)
}

func main() {
	for _, s := range scripts {
		fmt.Printf("== %s ==\n", s.name)
		if err := runScript(s.source); err != nil {
			fmt.Println(err)
		}
	}
}
