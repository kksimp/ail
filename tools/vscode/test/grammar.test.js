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
  ["## doc comment", "## doc comment", "comment.block.documentation.ail"],
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
  // Every .ail file in examples/ tokenizes without throwing and with no illegal scope.
  const dir = path.join(__dirname, "..", "..", "..", "examples");
  if (fs.existsSync(dir)) {
    for (const f of fs.readdirSync(dir).filter((n) => n.endsWith(".ail"))) {
      let s = textmate.INITIAL;
      for (const [i, line] of fs.readFileSync(path.join(dir, f), "utf8").split("\n").entries()) {
        const r = grammar.tokenizeLine(line, s);
        s = r.ruleStack;
        if (r.tokens.some((t) => t.scopes.some((x) => x.startsWith("invalid")))) {
          failed++; console.error(`FAIL ${f}:${i + 1}: invalid scope`);
        }
      }
    }
  }
  console.log(failed ? `${failed} failed` : "grammar ok");
  process.exit(failed ? 1 : 0);
})();
