# Indentation examples (OQ-026)

> **Non-normative design aid,** 2026-10-04. The same code shown with each
> candidate indentation, so the designer can choose by sight. Costs come from
> [syntax-measurements-r1.md](syntax-measurements-r1.md#addendum-visible-indentation-markers-designers-question)
> (mean across five tokenizers, relative to 4 spaces).

## Option 1 — 4 spaces (cost: baseline)

```
fn tokenize(src str) Result[List[Token], LexError]
    tokens := List[Token]()
    mut i := 0
    while i < src.len
        c := src[i]
        if c.is_space()
            i += 1
            continue
        tokens.push(lex_one(c)?)
        i += 1
    Ok(tokens)
```

### How an editor can display 4 spaces with indentation guides

The `│` lines are drawn by the editor. They're **not** in the file.

```
fn tokenize(src str) Result[List[Token], LexError]
│   tokens := List[Token]()
│   mut i := 0
│   while i < src.len
│   │   c := src[i]
│   │   if c.is_space()
│   │   │   i += 1
│   │   │   continue
│   │   tokens.push(lex_one(c)?)
│   │   i += 1
│   Ok(tokens)
```

## Option 2 — 2 spaces (cost: −0.1%)

```
fn tokenize(src str) Result[List[Token], LexError]
  tokens := List[Token]()
  mut i := 0
  while i < src.len
    c := src[i]
    if c.is_space()
      i += 1
      continue
    tokens.push(lex_one(c)?)
    i += 1
  Ok(tokens)
```

## Option 3 — Tabs (cost: +6.0%)

These are real tab characters, so their displayed width depends on the viewer.

```
fn tokenize(src str) Result[List[Token], LexError]
	tokens := List[Token]()
	mut i := 0
	while i < src.len
		c := src[i]
		if c.is_space()
			i += 1
			continue
		tokens.push(lex_one(c)?)
		i += 1
	Ok(tokens)
```

## Option 4 — Periods, one per level (cost: +0.5%)

```
fn tokenize(src str) Result[List[Token], LexError]
.tokens := List[Token]()
.mut i := 0
.while i < src.len
..c := src[i]
..if c.is_space()
...i += 1
...continue
..tokens.push(lex_one(c)?)
..i += 1
.Ok(tokens)
```

Note how the leading periods sit next to `c.is_space()` (member access) and
would sit next to `..` in range expressions such as `src[start..i]`.
