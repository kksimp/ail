// Tokenizes small AIL snippets with the shipped grammar and checks the scopes.
// Run: npm install && npm test   (on a Kaleb machine; no VS Code needed)
"use strict";
const fs = require("fs");
const path = require("path");
const textmate = require("vscode-textmate");
const oniguruma = require("vscode-oniguruma");

const wasm = fs.readFileSync(require.resolve("vscode-oniguruma/release/onig.wasm")).buffer;
const lib = oniguruma.loadWASM(wasm).then(() => ({
  createOnigScanner: (p) => new oniguruma.OnigScanner(p),
  createOnigString: (s) => new oniguruma.OnigString(s),
}));
const registry = new textmate.Registry({
  onigLib: lib,
  loadGrammar: async (scope) =>
    scope === "source.ail"
      ? textmate.parseRawGrammar(
          fs.readFileSync(path.join(__dirname, "..", "syntaxes", "ail.tmLanguage.json"), "utf8"),
          "ail.tmLanguage.json"
        )
      : null,
});

// [source, token text, scope that must be on it]
const CASES = [
  ["fn main()", "fn", "keyword.declaration.function.ail"],
  ["fn main()", "main", "entity.name.function.ail"],
  ["fn Parser.peek(self) char?", "Parser", "entity.name.type.ail"],
  ["fn Parser.peek(self) char?", "peek", "entity.name.function.ail"],
  ["    x := 0x1F_ff", "0x1F_ff", "constant.numeric.hex.ail"],
  ["    x := 2.5e-3", "2.5e-3", "constant.numeric.float.ail"],
  ["    print(\"hi\")", "print", "entity.name.function.call.ail"],
  ["    s := \"a\\n\"", "\\n", "constant.character.escape.ail"],
  ["    s := \"a\\{b}\"", "\\{", "constant.character.escape.ail"],
  ["    s := \"n={count}\"", "count", "meta.interpolation.ail"],
  ["    s := r\"a\\b{c}\"", "a\\b{c}", "string.quoted.other.raw.ail"],
  ["    s := b\"\\x41\"", "\\x41", "constant.character.escape.ail"],
  ["    c := 'x'", "'x'", "string.quoted.single.ail"],
  ["# see lex.kw.1 here", "lex.kw.1", "constant.other.rule-id.ail"],
  // begin/end rules tokenize the marker and the body separately; both carry the scope
  ["## doc comment", "##", "comment.block.documentation.ail"],
  ["## doc comment", " doc comment", "comment.block.documentation.ail"],
  ["# plain", " plain", "comment.line.number-sign.ail"],
  ["struct Vec[T, const N int]", "const", "storage.modifier.const.ail"],
  ["@test", "@test", "entity.other.attribute-name.ail"],
  ["    if x and not y", "and", "keyword.operator.word.ail"],
  ["    return none", "none", "constant.language.ail"],
  ["struct Point", "Point", "entity.name.type.ail"],
  ["    x := n as int", "int", "support.type.primitive.ail"],
];

(async () => {
  const grammar = await registry.loadGrammar("source.ail");
  let failed = 0;
  for (const [src, text, scope] of CASES) {
    const { tokens } = grammar.tokenizeLine(src, textmate.INITIAL);
    const hit = tokens.find((t) => src.slice(t.startIndex, t.endIndex).includes(text) && t.scopes.includes(scope))
      || tokens.some((t) => t.scopes.includes(scope) && src.slice(t.startIndex, t.endIndex) === text);
    if (!hit) {
      failed++;
      console.error(`FAIL ${JSON.stringify(src)}: ${JSON.stringify(text)} lacks ${scope}`);
    }
  }
  // Multi-line string: the middle line is a string and `#` inside it is not a comment.
  let st = textmate.INITIAL;
  for (const line of ['t := """', "    a # not a comment", '    """']) {
    const r = grammar.tokenizeLine(line, st);
    st = r.ruleStack;
    if (line.includes("not a comment") && !r.tokens.every((t) => !t.scopes.some((s) => s.startsWith("comment"))))
      { failed++; console.error("FAIL: # inside a multi-line string was a comment"); }
  }
  // Format spec after `:` in an interpolation is not code (`#` there is not a comment).
  {
    const src = 'print("[{n:#>8x}]")';
    const { tokens } = grammar.tokenizeLine(src, textmate.INITIAL);
    if (tokens.some((t) => t.scopes.some((x) => x.startsWith("comment"))))
      { failed++; console.error("FAIL: format spec `#` was a comment"); }
  }
  // Every .ail file in the repository tokenizes with no illegal scope and ends back at the top level.
  // Files that exist to be rejected by the compiler (lexical errors) are listed in KNOWN_BAD.
  const KNOWN_BAD = new Set(fs.readFileSync(path.join(__dirname, "known_bad.txt"), "utf8").split("\n").filter((l) => l && !l.startsWith("#")));
  const repo = path.join(__dirname, "..", "..", "..");
  const walk = function* (d) {
    for (const e of fs.readdirSync(d, { withFileTypes: true })) {
      if (e.name === "node_modules" || e.name === ".git") continue;
      const p = path.join(d, e.name);
      if (e.isDirectory()) yield* walk(p);
      else if (e.name.endsWith(".ail")) yield p;
    }
  };
  let files = 0;
  for (const f of walk(repo)) {
    files++;
    const rel = path.relative(repo, f).split(path.sep).join("/");
    let s = textmate.INITIAL, bad = null;
    for (const [i, line] of fs.readFileSync(f, "utf8").split("\n").entries()) {
      const r = grammar.tokenizeLine(line, s);
      s = r.ruleStack;
      if (!bad && r.tokens.some((t) => t.scopes.some((x) => x.startsWith("invalid")))) bad = `${i + 1}: invalid scope`;
    }
    if (!bad && s.depth > 1) bad = "ends inside a string or interpolation";
    if (bad && !KNOWN_BAD.has(rel)) { failed++; console.error(`FAIL ${rel}:${bad}`); }
    if (!bad && KNOWN_BAD.has(rel)) { failed++; console.error(`FAIL ${rel}: now clean, remove it from test/known_bad.txt`); }
  }
  // The reserved words in compiler/syntax/lex.ail are all keywords in the grammar.
  const lex = fs.readFileSync(path.join(repo, "compiler", "syntax", "lex.ail"), "utf8");
  const block = lex.slice(lex.indexOf("KEYWORDS := {"), lex.indexOf("}", lex.indexOf("KEYWORDS := {")));
  const words = [...block.matchAll(/"([a-z]+)"/g)].map((m) => m[1]);
  if (words.length < 40) { failed++; console.error("FAIL: could not read KEYWORDS from lex.ail"); }
  for (const w of words) {
    const { tokens } = grammar.tokenizeLine(`${w} X`, textmate.INITIAL);
    if (!tokens.some((t) => t.scopes.some((x) => /^(keyword|storage|constant\.language|variable\.language)/.test(x))))
      { failed++; console.error(`FAIL: keyword ${w} has no keyword scope`); }
  }
  console.log(`${files} .ail files, ${words.length} keywords`);
  console.log(failed ? `${failed} failed` : "grammar ok");
  process.exit(failed ? 1 : 0);
})();
