type TokenKind int

const (
	Num TokenKind = iota
	Op
	LParen
	RParen
)

type Token struct {
	Kind  TokenKind
	Value int64
	Op    byte
}

func tokenize(src string) ([]Token, error) {
	var tokens []Token
	i := 0
	for i < len(src) {
		c := src[i]
		if c == ' ' || c == '\t' || c == '\n' {
			i++
			continue
		}
		if c >= '0' && c <= '9' {
			start := i
			for i < len(src) && src[i] >= '0' && src[i] <= '9' {
				i++
			}
			n, err := strconv.ParseInt(src[start:i], 10, 64)
			if err != nil {
				return nil, err
			}
			tokens = append(tokens, Token{Kind: Num, Value: n})
			continue
		}
		switch c {
		case '(':
			tokens = append(tokens, Token{Kind: LParen})
		case ')':
			tokens = append(tokens, Token{Kind: RParen})
		case '+', '-', '*', '/':
			tokens = append(tokens, Token{Kind: Op, Op: c})
		default:
			return nil, fmt.Errorf("unexpected %q at %d", c, i)
		}
		i++
	}
	return tokens, nil
}
