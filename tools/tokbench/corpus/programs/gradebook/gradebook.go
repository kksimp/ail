package main

import "fmt"

type Grade int

const (
	A Grade = iota
	B
	C
	F
)

func (g Grade) Label() string {
	switch g {
	case A:
		return "excellent"
	case B:
		return "good"
	case C:
		return "passing"
	default:
		return "failing"
	}
}

type Student struct {
	Name  string
	Score int
}

func (s Student) Grade() Grade {
	if s.Score >= 90 {
		return A
	} else if s.Score >= 80 {
		return B
	} else if s.Score >= 70 {
		return C
	}
	return F
}

func main() {
	students := []Student{
		{Name: "Ada", Score: 95},
		{Name: "Linus", Score: 82},
		{Name: "Grace", Score: 67},
	}
	total := 0
	for _, s := range students {
		fmt.Printf("%s: %d (%s)\n", s.Name, s.Score, s.Grade().Label())
		total += s.Score
	}
	fmt.Printf("class average: %d\n", total/len(students))
}
