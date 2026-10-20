# tokbench: token measurement harness

Counts how many tokens source files cost under several real model tokenizers.
It implements layers 1–2 of AIL's token benchmark
([ADR 0029](../../docs/decisions/0029-token-efficiency-benchmarks.md)).
It's a measurement instrument, not part of AIL
([ADR 0070](../../docs/decisions/0070-token-bench-tool.md)).

## Run (one command)

**macOS / Linux**, from the repository root:
```sh
tools/tokbench/run.sh
```

**Windows (PowerShell)**, from the repository root:
```powershell
powershell -ExecutionPolicy Bypass -File tools\tokbench\run.ps1
```

That's all. The first run sets everything up automatically:
- a private Python environment in `tools/tokbench/.venv`, so nothing is
  installed system-wide;
- the tokenizer files, downloaded into `tools/tokbench/.cache`.

Later runs skip setup. It needs Python 3.10 or newer. To choose a specific
interpreter, set `PYTHON=/path/to/python`.

Options (the same for both scripts):

| Command | Does |
|---|---|
| `run.sh` | Measure every file under `corpus/` |
| `run.sh FILE ...` | Measure specific files (paths relative to where you run it) |
| `run.sh --json` | Machine-readable output |
| `run.sh --claude` | Also count with the **current** Claude tokenizer (optional, see below) |

### Optional: the current Claude tokenizer

The offline `claude-legacy` column is an **older** Claude tokenizer. Current
Claude tokenizers are only available through Anthropic's token-counting API,
which needs an Anthropic API key (`ANTHROPIC_API_KEY`, or `ant auth login`).
The counting calls aren't billed, but getting a key means creating an Anthropic
developer account. This is entirely optional. All AIL syntax decisions so far
were made with the offline tokenizers, which have agreed with each other on
every comparison. The Anthropic SDK is installed only when you pass `--claude`.

The API counts a whole message, so the tool subtracts the framing overhead.
Results may be off by about one token per file.

## Tokenizers

| Column | Source | Notes |
|---|---|---|
| `cl100k` | OpenAI `cl100k_base`, via the pinned `litellm` wheel | GPT-4 era |
| `o200k` | OpenAI `o200k_base`, via the same wheel | GPT-4o era |
| `claude-legacy` | `anthropic_tokenizer.json`, via the same wheel | Old Claude tokenizer. Use `--claude` for current models. |
| `mistral-tekken` | `tekken_240911.json`, via the pinned `mistral-common` wheel | Recent Mistral |
| `mistral-spm-v1` | `tokenizer.model.v1`, via the same wheel | Older SentencePiece |
| `claude:<model>` | Anthropic API `messages.count_tokens` | Only with `--claude` |

## Corpus

The language card has a budget of 8,000 tokens ([ADR 0276](../../docs/decisions/0276-language-card-budget-8000.md), amending ADR 0174); check it with `run.sh docs/language-card.md`.

| Directory | Contents |
|---|---|
| `corpus/r1/` | Round-1 block-style samples (A–D) plus Rust and Go references, as measured in [syntax-measurements-r1.md](../../docs/design/syntax-measurements-r1.md). They use candidate syntax from before later decisions. |
| `corpus/r2/` | Methods, trait conformance, generic constraints, and receiver-mutation variants. See [traits-generics-options.md](../../docs/design/traits-generics-options.md). |
| `corpus/r3/` | Operators, trait-typed parameters, dynamic dispatch, and attribute variants (ADRs 0077–0083). |
| `corpus/r4/` | Printing, interpolation, collection types and literals, `elif`, optional wrapping, and ranges (ADRs 0088–0093). |
| `corpus/r5/` | Concurrency (colorless vs. async/await), visibility (`pub` vs. capitalisation), and import styles. |
| `corpus/r8/` | Format specifications (`{x:>8}`, `{r:.2}`) vs `pad_left`/`to_hex`/`to_fixed` calls on a table-and-money program (ADR 0173). |
| `corpus/programs/NAME/` | Whole programs with Python twins (and Go, Rust, TypeScript, C or Swift ports for some), compared in [program-measurements-r2.md](../../docs/design/program-measurements-r2.md) and [-r3.md](../../docs/design/program-measurements-r3.md). The `.ail` files are `examples/NAME.ail` without comments, blank lines or `@test` functions: regenerate them with `python3 tools/tokbench/corpus_from_examples.py`. |
| `corpus/r9/` | One 4-dependency package manifest in seven formats: line per dependency, indented, dotted keys, TOML (two forms), JSON, AIL literal (ADR 0230). |
| `corpus/current/` | Samples in the syntax decided so far (as of 2026-10-04). `tokenize.ail` predates ADR 0089 (it still indexes a `str` directly) and also uses constructs that aren't decided yet: `while`, `char`, slicing, `+=`, `and`, methods that mutate a `mut` binding, and returning an error value. Corpus files hold no explanatory comments, because comments would distort the counts. |

## Attribution

`attribute.py` (measurement only) says where the tokens of the
`corpus/programs/` go: for each construct class (signature types, `fn`,
`self`, `:=`, `mut`, `use`, `else`, `?`, indentation, ...) it deletes that
class from every program and reports the difference, for AIL and for Python.
Run it after `run.sh` has set up once:

```sh
tools/tokbench/.venv/bin/python tools/tokbench/attribute.py [--json]
```

