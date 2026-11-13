# compiler: syntax

## syntax
```
pub fn syntax.tokens_text(lexed syntax.Lexed) str  ## One line per token, then one per comment, then one per diagnostic.
pub fn syntax.diag_lines(diags List[syntax.Diag], mut out List[str]) [mutates]
pub fn syntax.one_per_line(diags List[syntax.Diag]) List[syntax.Diag]  ## One syntax diagnostic per line (gram.rec.1, ADR 0263): the leftmost one
pub fn syntax.quote(s str) str  ## A string in JSON, as the prelude's `json_quote` writes it.
pub fn syntax.ast_text(lexed syntax.Lexed, parsed syntax.Parsed) str  ## The tree of `ail api --ast` (api.ast.1-2): the program as one line of
pub fn syntax.report_text(path str, text str, diags List[syntax.Diag]) str  ## The syntax diagnostics of one file as `ail check` prints them (the text
pub fn syntax.keywords() List[str]  ## The reserved words, sorted: what `ail help --keywords` prints for editors (tool.help.3).
pub enum syntax.Tok  ## A token's kind and value.
    Ident(name str)
    Kw(word str)
    Sym(text str)
    Str(value str)
    Bytes(hex str)  ## `b"..."`, its bytes as lowercase hex (lex.bytes.1).
    Interp(parts List[syntax.Part])  ## A string literal containing `{expression}` parts (lex.str.2).
    Char(value char)  ## `'a'` (lex.str.4)
    Int(value int)
    IntMin  ## `9223372036854775808`, which fits only after a `-` (lex.num.3).
    Float(value float)  ## `1.5`, `1e3` (lex.num.2)
    Newline
    Indent
    Dedent
    Eof
pub enum syntax.Part  ## A piece of an interpolated string: literal text, or the tokens of one
    Lit(text str)
    Code(tokens List[syntax.Token])
    Fmt(tokens List[syntax.Token], spec syntax.FormatSpec)
pub struct syntax.Token
    pub tok syntax.Tok
    pub pos ast.Pos
pub struct syntax.FormatSpec  ## `[[fill]align][+][0][width][,][.precision][x|b|e]` after the `:` in `{expression:spec}`
    pub fill char?
    pub align char?  ## `<`, `>` or `^`.
    pub width int?
    pub precision int?  ## Decimal places of a float (`.N`).
    pub hex bool  ## `x`: lowercase hexadecimal for an int.
    pub flags str  ## The other marks, each at most once: `+` (always a sign), `0` (zero padding), `,`
pub struct syntax.Diag  ## A mistake: the rule it breaks and where. The parser keeps the message
    pub rule str
    pub pos ast.Pos
    pub msg str
    pub fixed bool
    pub help str
    pub fix str
    pub edits List[fix.Edit]  ## The edits of the fix, when it has them (ADR 0145).
pub struct syntax.Comment  ## A comment (lex.comment.1): `text` runs from the `#` to the end of the line.
    pub pos ast.Pos
    pub text str
pub struct syntax.Lexed  ## The tokens of a file, its diagnostics in the order found, and its comments.
    pub tokens List[syntax.Token]
    pub diags List[syntax.Diag]
    pub comments List[syntax.Comment]
pub fn syntax.lex(src str) syntax.Lexed  ## Lex `src` (lex.*). A file that indents by 2 or 3 spaces is lexed again
pub struct syntax.Parsed  ## The tree of a file and the diagnostics found making it, in order.
    pub program ast.Program
    pub diags List[syntax.Diag]
pub fn syntax.generic_names(tokens List[syntax.Token]) Set[str]  ## Names declared generic (`fn f[`, `struct S[`, `enum E[`): after one of
pub fn syntax.parse(tokens List[syntax.Token]) syntax.Parsed  ## Parse a file's tokens (the lexer's, ending in `Eof`).
pub fn syntax.parse_with(tokens List[syntax.Token], extra Set[str]) syntax.Parsed  ## `parse` of a file whose module's other files declare generic names too:
```

trimmed to fit 2000 tokens (~1203); dropped: private items
