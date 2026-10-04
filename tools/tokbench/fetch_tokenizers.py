"""Fetch the offline tokenizer files used by tokbench.

The official download hosts for some tokenizers are not always reachable, so
the files are extracted from pinned PyPI packages that bundle them. Nothing
here is committed to the repository; files land in tools/tokbench/.cache/.

Usage:
    python fetch_tokenizers.py
"""

import pathlib
import subprocess
import sys
import tempfile
import zipfile

HERE = pathlib.Path(__file__).resolve().parent
CACHE = HERE / ".cache"

# Pinned so measurements are reproducible. Changing a pin changes results;
# record the change in docs/design when you do.
PACKAGES = ["litellm==1.104.0", "mistral-common==1.12.0"]

WANTED = {
    # tiktoken cache files (named by the SHA-1 of their source URL)
    "litellm/litellm_core_utils/tokenizers/9b5ad71b2ce5302211f9c61530b329a4922fc6a4",  # cl100k_base
    "litellm/litellm_core_utils/tokenizers/fb374d419588a4632f3f557e76b4b70aebbca790",  # o200k_base
    # an older Claude tokenizer (not representative of current Claude models)
    "litellm/litellm_core_utils/tokenizers/anthropic_tokenizer.json",
    # Mistral tokenizers
    "mistral_common/data/tekken_240911.json",
    "mistral_common/data/tokenizer.model.v1",
}


def main() -> int:
    CACHE.mkdir(exist_ok=True)
    with tempfile.TemporaryDirectory() as tmp:
        subprocess.run(
            [sys.executable, "-m", "pip", "download", "--quiet", "--no-deps",
             "--dest", tmp, *PACKAGES],
            check=True,
        )
        found = set()
        for wheel in pathlib.Path(tmp).glob("*.whl"):
            with zipfile.ZipFile(wheel) as z:
                for name in z.namelist():
                    if name in WANTED:
                        (CACHE / pathlib.Path(name).name).write_bytes(z.read(name))
                        found.add(name)
    missing = WANTED - found
    if missing:
        print("missing tokenizer files:", *sorted(missing), sep="\n  ", file=sys.stderr)
        return 1
    print(f"tokenizer files ready in {CACHE}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
