package main

import (
	"fmt"
	"os"
	"strconv"
	"strings"
)

type CommandError struct {
	Kind  string
	Field string
	Value string
}

func (e *CommandError) Error() string {
	if e.Field == "" {
		return "CommandError." + e.Kind
	}
	return fmt.Sprintf("CommandError.%s(%s: %q)", e.Kind, e.Field, e.Value)
}

type Command struct {
	Kind string
	Item string
	Qty  int
}

type Entry struct {
	Item string
	Qty  int
}

func parse(line string) (Command, error) {
	words := strings.Split(line, " ")
	verb := words[0]
	if verb == "" {
		return Command{}, &CommandError{Kind: "Empty"}
	}
	if verb == "show" {
		return Command{Kind: "show"}, nil
	}
	if len(words) < 2 {
		return Command{}, &CommandError{"MissingArgument", "verb", verb}
	}
	if verb == "remove" {
		return Command{Kind: "remove", Item: words[1]}, nil
	}
	if verb != "add" {
		return Command{}, &CommandError{"UnknownVerb", "verb", verb}
	}
	if len(words) < 3 {
		return Command{}, &CommandError{"MissingArgument", "verb", verb}
	}
	qty, err := strconv.Atoi(words[2])
	if err != nil {
		return Command{}, &CommandError{"BadQuantity", "text", words[2]}
	}
	return Command{Kind: "add", Item: words[1], Qty: qty}, nil
}

func apply(stock []Entry, cmd Command) []Entry {
	switch cmd.Kind {
	case "add":
		for i := range stock {
			if stock[i].Item == cmd.Item {
				stock[i].Qty += cmd.Qty
				return stock
			}
		}
		return append(stock, Entry{cmd.Item, cmd.Qty})
	case "remove":
		for i := range stock {
			if stock[i].Item == cmd.Item {
				stock[i].Qty = 0
			}
		}
	case "show":
		for _, e := range stock {
			if e.Qty > 0 {
				fmt.Printf("  %s: %d\n", e.Item, e.Qty)
			}
		}
	}
	return stock
}

func totalAdded(lines []string) (int, error) {
	total := 0
	for _, line := range lines {
		cmd, err := parse(line)
		if err != nil {
			return 0, fmt.Errorf("reading %q: %w", line, err)
		}
		if cmd.Kind == "add" {
			total += cmd.Qty
		}
	}
	return total, nil
}

func main() {
	lines := []string{
		"add milk 2",
		"add eggs 12",
		"add milk 1",
		"remove eggs",
		"add tea lots",
		"sell milk 1",
		"",
		"show",
	}
	stock := []Entry{{"bread", 1}}
	for _, line := range lines {
		cmd, err := parse(line)
		if err != nil {
			fmt.Println("error: " + err.Error())
			continue
		}
		stock = apply(stock, cmd)
	}
	added, err := totalAdded(lines[0:4])
	if err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
	fmt.Printf("added: %d\n", added)
}
