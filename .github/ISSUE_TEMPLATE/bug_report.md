---
name: Bug report
about: The compiler, a tool or a program built with AIL does the wrong thing
labels: bug
---

<!-- Security problem? Do not open an issue: see SECURITY.md. -->

## What happened

<!-- What you ran, what you expected, what you got. Paste the full diagnostic or trap, including its rule ID. -->

## Smallest program that shows it

```ail
fn main()
    print("replace me")
```

Command: `./ail run bug.ail`

## Where

- Commit: <!-- git rev-parse HEAD -->
- Seed: <!-- first line of tools/release/SEED_TAG -->
- Host: <!-- e.g. macOS 15 arm64, Ubuntu 24.04 x86-64, Windows 11 x86-64 -->
- Target, if not the host: <!-- e.g. --target x86_64-windows -->

## Spec rule

<!-- The rule ID the behaviour breaks, if you know it (e.g. expr.match.3). `./ail explain RULE` prints a rule. -->
