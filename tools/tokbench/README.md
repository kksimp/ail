# tokbench: token measurement harness

Counts how many tokens source files cost under several real model tokenizers.
It implements layers 1–2 of AIL's token benchmark
([ADR 0029](../../docs/decisions/0029-token-efficiency-benchmarks.md)).
It's a measurement instrument, not part of AIL
([ADR 0070](../../docs/decisions/0070-token-bench-tool.md)).

## Setup

```sh
cd tools/tokbench
python -m venv .venv && . .venv/bin/activate     # Windows: .venv\Scripts\activate
pip install -r requirements.txt
python fetch_tokenizers.py                       # once; extracts tokenizer files into .cache/
```

## Run

```sh
python tokbench.py corpus/r1/*.ail corpus/r1/*.rs corpus/r1/*.go
python tokbench.py corpus/current/*.ail
```

### With the current Claude tokenizer

The offline `claude-legacy` tokenizer is an **older** Claude tokenizer, so its
counts don't represent current Claude models. To get real current-Claude counts,
use Anthropic's token-counting API with your own credentials:

```sh
export ANTHROPIC_API_KEY=...        # or: ant auth login
python tokbench.py --claude corpus/r1/*.ail corpus/r1/*.rs corpus/r1/*.go
python tokbench.py --claude --model claude-sonnet-5-5 corpus/current/*.ail
```

Token counting is not billed as model usage, but it is rate-limited. The API
counts a whole message, so the tool subtracts the framing overhead. Results
may be off by about one token per file.

Add `--json` for machine-readable output.

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

| Directory | Contents |
|---|---|
| `corpus/r1/` | Round-1 block-style samples (A–D) plus Rust and Go references, as measured in [syntax-measurements-r1.md](../../docs/design/syntax-measurements-r1.md). They use candidate syntax from before later decisions. |
| `corpus/current/` | Samples in the syntax decided so far (as of 2026-10-04). `tokenize.ail` also uses constructs that aren't decided yet: `while`, `char`, slicing, `+=`, `and`, methods that mutate a `mut` binding, and returning an error value. Corpus files hold no explanatory comments, because comments would distort the counts. |
