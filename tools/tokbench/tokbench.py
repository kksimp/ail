"""Count tokens of source files with several real model tokenizers.

Implements layers 1-2 of the AIL token benchmark (ADR 0029). This is a
measurement instrument, not part of AIL (ADR 0070).

Usage (normally via ./run.sh or .\run.ps1, which set everything up):
    python tokbench.py                     # every file under corpus/
    python tokbench.py FILE [FILE ...]
    python tokbench.py --claude [FILE ...] # add current-Claude counts via the API
    python tokbench.py --json [FILE ...]
    python tokbench.py --check-budgets     # the agent-facing pages against their budgets

Offline tokenizers need `python fetch_tokenizers.py` once first.

--claude uses Anthropic's token-counting endpoint (messages.count_tokens) with
your own credentials: ANTHROPIC_API_KEY, or a profile from `ant auth login`.
Token counting is not billed as model usage but is rate-limited.
"""

import argparse
import base64
import json
import os
import pathlib
import sys

HERE = pathlib.Path(__file__).resolve().parent
CACHE = HERE / ".cache"
DEFAULT_CLAUDE_MODEL = os.environ.get("TOKBENCH_CLAUDE_MODEL", "")  # set to the model to count with

# Token budgets for the agent-facing pages, checked by --check-budgets on the
# offline five-tokenizer mean (the mean column). Change budgets here only.
# The language card: ADR 1110 (16,000; core about 3,000, was ADR 0276's 8,000). The quick reference: 12,518. Both are
# the five-tokenizer mean, recorded in ADR 0817.
BUDGETS = {
    "docs/language-card.md": 16000,
    "docs/language-card-core.md": 3000,
    "docs/quick-reference.md": 12518,
}
REPO = HERE.parents[1]


def check_budgets():
    """Print each page's offline five-tokenizer mean and budget; return 1 if any is over."""
    counters = offline_tokenizers()
    over = False
    for path, budget in BUDGETS.items():
        text = (REPO / path).read_text()
        counts = [fn(text) for fn in counters.values()]
        mean = round(sum(counts) / len(counts), 1)
        ok = mean <= budget
        over = over or not ok
        verdict = "ok" if ok else f"OVER by {mean - budget:.1f}"
        print(f"{path}: {mean:.1f} tokens, budget {budget} ({verdict})")
    return 1 if over else 0


def offline_tokenizers():
    """Return {name: count_fn} for every offline tokenizer."""
    if not CACHE.is_dir():
        sys.exit("tokenizer files not found; run: python fetch_tokenizers.py")
    os.environ["TIKTOKEN_CACHE_DIR"] = str(CACHE)

    import sentencepiece
    import tiktoken
    from tokenizers import Tokenizer

    cl100k = tiktoken.get_encoding("cl100k_base")
    o200k = tiktoken.get_encoding("o200k_base")
    claude_legacy = Tokenizer.from_file(str(CACHE / "anthropic_tokenizer.json"))
    spm = sentencepiece.SentencePieceProcessor(model_file=str(CACHE / "tokenizer.model.v1"))

    data = json.loads((CACHE / "tekken_240911.json").read_text())
    cfg = data["config"]
    n = cfg["default_vocab_size"] - cfg.get("default_num_special_tokens", 1000)
    ranks = {base64.b64decode(t["token_bytes"]): t["rank"] for t in data["vocab"][:n]}
    tekken = tiktoken.Encoding("tekken", pat_str=cfg["pattern"], mergeable_ranks=ranks,
                               special_tokens={})

    return {
        "cl100k": lambda t: len(cl100k.encode(t)),
        "o200k": lambda t: len(o200k.encode(t)),
        "claude-legacy": lambda t: len(claude_legacy.encode(t).ids),
        "mistral-tekken": lambda t: len(tekken.encode(t)),
        "mistral-spm-v1": lambda t: len(spm.encode(t)),
    }


def claude_tokenizer(model):
    """Return a count function backed by the current Claude tokenizer.

    The endpoint counts a whole message, including fixed framing overhead.
    That overhead is removed by subtracting the count of a one-character
    message, so results can be off by about one token at the boundary.
    """
    try:
        import anthropic
    except ImportError:
        sys.exit("--claude needs the Anthropic SDK: pip install anthropic")

    client = anthropic.Anthropic()

    def raw(text):
        try:
            return client.messages.count_tokens(
                model=model,
                messages=[{"role": "user", "content": text}],
            ).input_tokens
        except anthropic.AuthenticationError:
            sys.exit("Claude counting: no valid credentials (set ANTHROPIC_API_KEY or run `ant auth login`)")
        except anthropic.NotFoundError:
            sys.exit(f"Claude counting: model {model!r} not found")
        except anthropic.RateLimitError:
            sys.exit("Claude counting: rate limited; retry shortly")
        except anthropic.APIStatusError as e:
            sys.exit(f"Claude counting: API error {e.status_code}: {e.message}")
        except anthropic.APIConnectionError:
            sys.exit("Claude counting: could not reach the API")
        except TypeError as e:
            # Raised by the SDK when no credential source can be resolved.
            if "authentication" not in str(e):
                raise
            sys.exit("Claude counting: no credentials found (set ANTHROPIC_API_KEY or run `ant auth login`)")

    overhead = raw("x") - 1
    return lambda t: raw(t) - overhead


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("files", nargs="*", type=pathlib.Path,
                    help="files to measure (default: every file under corpus/)")
    ap.add_argument("--claude", action="store_true", help="add current-Claude counts via the API")
    ap.add_argument("--model", default=DEFAULT_CLAUDE_MODEL, help="model for --claude (default: $TOKBENCH_CLAUDE_MODEL)")
    ap.add_argument("--json", action="store_true", help="print JSON instead of a table")
    ap.add_argument("--check-budgets", action="store_true",
                    help="measure the language card and quick reference against their budgets; exit 1 if one is over")
    args = ap.parse_args()

    if args.check_budgets:
        sys.exit(check_budgets())

    if not args.files:
        # Program folders also hold the inputs their programs read (data/, configs/):
        # only the program and its twin are measured there.
        args.files = sorted(p.relative_to(HERE) for p in (HERE / "corpus").rglob("*")
                            if p.is_file() and not p.name.startswith(".")
                            and (p.suffix in (".ail", ".py") or "programs" not in p.parts))
        os.chdir(HERE)

    counters = offline_tokenizers()
    if args.claude:
        if not args.model:
            sys.exit("--claude needs --model or $TOKBENCH_CLAUDE_MODEL")
        counters[f"claude:{args.model}"] = claude_tokenizer(args.model)

    rows = []
    for f in args.files:
        text = f.read_text()
        counts = {name: fn(text) for name, fn in counters.items()}
        rows.append({"file": str(f), "counts": counts,
                     "mean": round(sum(counts.values()) / len(counts), 1)})

    if args.json:
        print(json.dumps(rows, indent=2))
        return
    names = list(counters)
    width = max(len(r["file"]) for r in rows)
    print(f"{'file':<{width}}" + "".join(f"{n:>{max(len(n), 6) + 2}}" for n in names) + "    mean")
    for r in rows:
        print(f"{r['file']:<{width}}"
              + "".join(f"{r['counts'][n]:>{max(len(n), 6) + 2}}" for n in names)
              + f"{r['mean']:>8}")


if __name__ == "__main__":
    main()
