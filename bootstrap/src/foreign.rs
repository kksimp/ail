//! Words and calls from other languages (gram.foreign.1, ADR 0262): what an
//! AI that writes AIL as if it were Python, Rust, Go or TypeScript reaches
//! for, and the AIL spelling to offer instead. The tables only name the
//! mistake; the parser and the checker decide where it is reported and
//! whether the replacement is safe as a fix.

/// What to do with the foreign word itself.
#[derive(Clone, Copy)]
pub enum Fix {
    /// No mechanical fix: the help says what to write.
    None,
    /// Replace the word.
    Replace(&'static str),
    /// Delete the word (and the blank after it).
    Delete,
}

/// A foreign word, the message that names it and the AIL way.
pub struct Word {
    pub msg: &'static str,
    pub help: &'static str,
    pub fix: Fix,
}

const fn w(msg: &'static str, help: &'static str, fix: Fix) -> Option<Word> {
    Some(Word { msg, help, fix })
}

/// A word that starts a declaration in another language.
pub fn decl_word(word: &str) -> Option<Word> {
    match word {
        "def" | "func" | "function" | "fun" => {
            w("AIL declares functions with `fn`", "write `fn name(a int) int`", Fix::Replace("fn"))
        }
        "class" => w(
            "AIL has no `class`",
            "declare a `struct` with its fields, then methods as `fn Name.method(self)`",
            Fix::Replace("struct"),
        ),
        "interface" | "protocol" => {
            w("AIL has no `interface`", "a set of methods a type provides is a `trait`", Fix::Replace("trait"))
        }
        "import" => w("AIL imports with `use`", "write `use std.fs`", Fix::Replace("use")),
        "async" => w(
            "AIL has no `async`",
            "tasks are colorless: start any function with `spawn f(x)` and wait with `t.wait()`",
            Fix::Delete,
        ),
        "public" | "export" => w("AIL exports with `pub`", "write `pub` before the declaration", Fix::Replace("pub")),
        "private" | "protected" | "static" | "final" | "abstract" | "override" | "sealed" | "internal" => w(
            "AIL has no access or storage modifiers on declarations",
            "a declaration is private to its module unless it is marked `pub`",
            Fix::Delete,
        ),
        "package" | "namespace" | "module" => w(
            "AIL modules are directories, not declared in the file",
            "put the file in a directory and import it elsewhere with `use dir`",
            Fix::None,
        ),
        _ => None,
    }
}

/// A word that starts a statement in another language and means nothing
/// here, when it isn't followed by something that makes it a name.
pub fn stmt_word(word: &str) -> Option<Word> {
    const ERRORS: &str = "errors are values: declare `fn f() T ! MyError`, fail with `return Variant`, and handle a call with `?` or `else` (spec chapter 08)";
    match word {
        "try" | "catch" | "except" | "finally" => w("AIL has no exceptions", ERRORS, Fix::None),
        "throw" | "raise" => w("AIL has no `throw`", ERRORS, Fix::None),
        "switch" | "case" | "default" => {
            w("AIL has no `switch`", "write `match value` with one `pattern => result` arm per line", Fix::None)
        }
        "pass" => w(
            "AIL has no `pass`",
            "a block needs a statement; write `todo` for code you haven't written yet",
            Fix::Replace("todo"),
        ),
        "await" => w(
            "AIL has no `await`",
            "tasks are colorless: `t := spawn f(x)` starts one and `t.wait()` waits for it",
            Fix::None,
        ),
        "loop" => w("AIL has no `loop`", "write `while true`", Fix::None),
        "elseif" | "elsif" | "else_if" => w("AIL writes `elseif` as `elif`", "write `elif`", Fix::Replace("elif")),
        "unless" | "until" | "yield" | "lambda" | "del" | "global" | "nonlocal" | "goto" | "do" => w(
            "this keyword from another language has no AIL form",
            "see the language card for the statements AIL has",
            Fix::None,
        ),
        _ => None,
    }
}

/// An unknown name that is another language's spelling of a value: the
/// replacement and the help.
pub fn name_word(name: &str) -> Option<(&'static str, &'static str)> {
    Some(match name {
        "True" => ("true", "booleans are `true` and `false`"),
        "False" => ("false", "booleans are `true` and `false`"),
        "None" | "nil" | "null" | "undefined" | "NULL" => {
            ("none", "the absent value is `none`, and the type must say it can be absent: `int?`")
        }
        "println" | "puts" | "echo" | "print_line" | "printline" | "writeln" => {
            ("print", "`print(value)` writes a value and a newline")
        }
        _ => return None,
    })
}

/// A call to a function other languages have: the template that rewrites it
/// (`Anchor::Call`), an example of the result, and the help.
pub struct CallHint {
    pub template: &'static str,
    pub example: &'static str,
    pub help: &'static str,
}

pub fn call_hint(name: &str, args: usize) -> Option<CallHint> {
    let h = |template, example, help| Some(CallHint { template, example, help });
    match (name, args) {
        ("len" | "length" | "size", 1) => h("$r1.len()", "xs.len()", "`len` is a method: write `xs.len()`"),
        ("str" | "string" | "toString", 1) => {
            h("\"{$1}\"", "\"{n}\"", "interpolate to get text: `\"{n}\"`, or write `n.to_str()`")
        }
        ("printf", n) if n >= 1 => h(
            "print($%)",
            "print(\"n is {n}\")",
            "build the text with interpolation, `print(\"n is {n}\")`; a width or precision goes after a colon: `{n:>6}`, `{x:.2}`",
        ),
        ("sqrt", 1) => h("$r1.sqrt()", "x.sqrt()", "`sqrt` is a method: write `x.sqrt()`"),
        ("floor", 1) => h("$r1.floor()", "x.floor()", "`floor` is a method: write `x.floor()`"),
        ("ceil", 1) => h("$r1.ceil()", "x.ceil()", "`ceil` is a method: write `x.ceil()`"),
        ("round", 1) => h("$r1.round()", "x.round()", "`round` is a method: write `x.round()`"),
        ("abs", 1) => h("$r1.abs()", "n.abs()", "`abs` is a method: write `n.abs()`"),
        ("max", 2) => h("$r1.max($2)", "a.max(b)", "`max` is a method: write `a.max(b)`"),
        ("min", 2) => h("$r1.min($2)", "a.min(b)", "`min` is a method: write `a.min(b)`"),
        ("sum", 1) => h("$r1.sum()", "xs.sum()", "`sum` is a method: write `xs.sum()`"),
        ("sorted", 1) => h("$r1.sorted()", "xs.sorted()", "`sorted` is a method: write `xs.sorted()`"),
        ("range", 1) => h("0..$1", "0..n", "a range is written `0..n` (excludes `n`) or `1..=n` (includes it)"),
        ("range", 2) => h("$1..$2", "a..b", "a range is written `a..b` (excludes `b`) or `a..=b` (includes it)"),
        _ => None,
    }
}

/// A type name from another language and the AIL type for it.
pub fn type_word(name: &str) -> Option<&'static str> {
    Some(match name {
        "Integer" | "Int" | "long" | "Long" | "short" | "Number" | "number" | "integer" | "int32" | "int64"
        | "size_t" => "int",
        "String" | "string" | "std::string" | "Str" | "text" => "str",
        "Boolean" | "boolean" | "Bool" => "bool",
        "double" | "Double" | "float64" | "Float" | "float32" | "real" => "float",
        "Vec" | "Array" | "array" | "list" | "ArrayList" | "vector" => "List",
        "dict" | "Dict" | "HashMap" | "Dictionary" | "map" | "object" => "Map",
        "HashSet" | "set" | "TreeSet" => "Set",
        "Character" | "rune" => "char",
        _ => return None,
    })
}

/// Help for another language's call that has no one-line rewrite.
pub fn call_help(name: &str) -> Option<&'static str> {
    Some(match name {
        "int" | "Integer" | "parseInt" | "atoi" => {
            "convert text with `s.to_int()`, which gives `int?` (`none` if it isn't a number), and a float with `x.to_int()`"
        }
        "float" | "parseFloat" => "convert with `s.to_float()` (gives `float?`) or `n.to_float()`",
        "input" | "readline" | "scanf" | "scan" => {
            "read input with `use std.io` and `io.read_line()`, which gives `str?` (`none` at end of input)"
        }
        "printf" | "fmt.Printf" | "format" => {
            "build the text with interpolation, `print(\"n is {n}\")`; a width or precision goes after a colon: `{n:>6}`, `{x:.2}`"
        }
        "enumerate" => "loop over the indexes and index the list: `for i in 0..xs.len()` then `xs[i]`",
        "zip" => "loop over the indexes of the shorter list: `for i in 0..a.len()` then `a[i]` and `b[i]`",
        "list" | "dict" | "set" | "array" | "vec" | "Vec" => {
            "build collections with literals or typed constructors: `[1, 2]`, `{\"a\": 1}`, `Set[int]()`, `List[int]()`"
        }
        "isinstance" | "typeof" | "type" => {
            "AIL is statically typed; use `match` on an enum or a trait method instead of a type test"
        }
        "assert" => "`assert` works inside an `@test` function (spec chapter 19)",
        _ => return None,
    })
}
