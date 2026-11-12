# How to

Short, checked answers to the things programs do most. `ail ask how WORDS`
finds one by its title or its words (`api.find.6`); every block is checked by
`tools/check-doc-snippets.py`. Each section is a `##` title, a `Words:` line the
search also reads, and one block.

## sort descending
Words: sort order largest first reverse biggest highest

`sorted(by:)` takes a key; negate a number to flip the order. For any other
type, sort and then reverse in place.

```ail fragment
xs := [3, 1, 2]
down := xs.sorted(by: |x| -x)
mut names := ["b", "a", "c"].sorted()
names.reverse()
print(down)
print(names)
```

## sort by key
Words: sort list order by field property custom comparator

```ail fragment
words := ["ccc", "a", "bb"]
print(words.sorted(by: |w| w.len()))
```

## loop with index
Words: for each element index position counter enumerate

```ail fragment
xs := ["a", "b"]
for i, x in xs.enumerate()
    print("{i}: {x}")
```

## read a file by lines
Words: read open text file lines each line

```ail fragment
text := fs.read_text("data.txt") else
    print("cannot read data.txt")
    return
for line in text.lines()
    print(line)
```

## read lines from standard input
Words: stdin standard input console lines read all

```ail fragment
for line in io.read_all().lines()
    print(line)
```

## write a file
Words: write save create text file

```ail fragment
fs.write_text("out.txt", "hello\n") else
    print("cannot write out.txt")
```

## string to int
Words: parse convert number integer text str int

`to_int()` gives an `int?`; `else` supplies the fallback.

```ail fragment
n := "42".to_int() else 0
print(n)
```

## build a string in a loop
Words: string builder concatenate append join pieces accumulate

There is no `+` for strings. Push the parts to a list and join once.

```ail fragment
mut parts := List[str]()
for i in 0..3
    parts.push("item {i}")
print(parts.join(", "))
```

## leave nested loops
Words: break out of nested loops both loops exit early find pair

`break` leaves one loop. Put the loops in a function and `return`.

```ail
fn find_pair(xs List[int], target int) (int, int)?
    for i in 0..xs.len()
        for j in 0..xs.len()
            if i != j and xs[i] + xs[j] == target
                return (i, j)
    none

fn main()
    print(find_pair([1, 2, 3], 5))
```

## count occurrences
Words: count frequency histogram tally words how many each

```ail fragment
counts := "a b a".split(" ").counts()
print(counts)
```

## group by key
Words: group multimap list per key bucket

```ail fragment
mut groups := Map[str, List[int]]()
groups.push("even", 2)
groups.push("odd", 1)
groups.push("even", 4)
print(groups)
```

## default for a missing map key
Words: map get default missing key lookup else zero

`m.get(k)` takes only the key and gives a `V?`.

```ail fragment
ages := {"ann": 31}
age := ages.get("bob") else 0
print(age)
```

## split and trim text
Words: split text separator trim whitespace tokens words parse csv

```ail fragment
parts := "a, b ,c".split(",")
for p in parts
    print(p.trim())
```

## wait some time
Words: sleep delay pause wait milliseconds

```ail fragment
time.sleep_ms(100)
```
