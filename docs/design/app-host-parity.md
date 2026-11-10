# std.app host parity: macOS, Linux, Windows

Status: working table for v0.1.0 slice P6 (2026-11-10, ail main). One row per op, node, protocol line and event of the macOS host's `tools/app-host/macos/Sources/Protocol.swift`, with what `tools/app-host/linux/host.c` and `tools/app-host/windows/Host.cs` do. **Linux owners add the Linux column; the Windows lane owns the Windows column.** "yes" means implemented and covered by `tests/ui/host_conformance.txt` or a manual check named in the notes; "no" is a gap to fill before the tag; "chain" means the rule is in the 0.1.0 chain of PRs and not yet on main.

Conformance: `tests/ui/host_conformance.txt` is the script every host must pass without an AIL toolchain. `tests/ui/host_conformance.sh PLATFORM HOST...` plays it on macOS and Linux; `tools/app-host/windows/smoke.ps1` plays it on Windows and adds the Windows-only checks (surface mapping).

## Nodes and ops (std.app.2-3, .19)

| Item | Rule | macOS | Linux | Windows |
|---|---|---|---|---|
| Label, Button, Field, Toggle, Column, Row, Spacer, Canvas | std.app.2 | yes | | yes |
| Str, Fill, Stroke, Clip, Unclip | std.app.3 | yes | | yes |
| Pic, Bitmap | std.app.3 | yes | | yes |
| Shade | std.app.19 | yes | | yes |
| Str `underline`, `bold`, `italic` | std.app.3 | yes | | yes |

## Lines the program sends

| Line | Rule | macOS | Linux | Windows |
|---|---|---|---|---|
| `view` (title, focus) | std.app.7 | yes | | yes |
| `view` `hover` | std.app.16 | yes | | yes |
| `view` `edge` | std.app.25 | yes | | yes (ADR 1033) |
| `measure` | std.app.13 | yes | | yes |
| `popup` | std.app.18 | yes | | yes (ContextMenuStrip) |
| `pick` (file and folder) | std.app.22 | yes | | yes (OpenFileDialog, FolderBrowserDialog) |
| `clipboard` | std.app.26 | yes | | yes |
| `drag`, `zoom` | std.app.25 | yes | | yes |
| `surface`, `present`, `text` | std.gfx.7-8 | yes | | yes (named file mapping, ADR 1044) |
| `lines()` lines the host does not know are skipped | std.app.21 | yes | | yes |

## Events the host sends

| Event | Rule | macOS | Linux | Windows |
|---|---|---|---|---|
| `start` (screen_w, screen_h, scale) | std.app.31 | yes | | yes |
| `start`, `resize` insets (inset_left, inset_top) | std.app.25 | yes | | yes (0, 32) |
| `start`, `resize` inset_right | std.app.33 | not sent (0) | | yes (138) |
| `resize` (cell size, screen) | std.app.5, .31 | yes | | yes |
| `press`, `edit`, `submit`, `toggle` | std.app.8 | yes | | yes |
| `click` with `clicks` | std.app.17 | yes | | yes |
| `down`, `up`, pointer capture | std.app.27 | yes | | yes |
| `move`, `leave` | std.app.16 | yes | | yes |
| `scroll` | std.app.8 | yes | | yes |
| `key` with `repeat` and `code` | std.app.20 | yes | | yes |
| `keyup` | std.app.20 | yes | | yes |
| `paste` | std.app.23 | yes | | yes |
| `measured` | std.app.13 | yes | | yes |
| `popup` answer | std.app.18 | yes | | yes |
| `picked`, `pick_error` | std.app.22 | yes | | yes |
| `appearance` | std.app.30 | yes | | yes |
| `vsync` | std.gfx.8 | yes | | yes |
| `pad` | std.gfx.11 | yes (GameController) | | yes (XInput) |
| `close` | std.app.8 | yes | | yes |

## In the 0.1.0 chain (std.app.28, .29, .32, .33, .34)

Pulled into 0.1.0 on 2026-11-10; they land as one chain (order .32, .29, .28, .33, .34), each with all three hosts. Until a rule merges its cells read "chain".

| Rule | What | macOS | Linux | Windows |
|---|---|---|---|---|
| std.app.28 | font-family lists | chain | chain | chain |
| std.app.29 | web fonts `Font` op | chain | chain | chain |
| std.app.32 | `Round`, `Poly`, `Line` ops | chain | chain | chain |
| std.app.33 | `inset_right` (ADR 1033) | n/a (sends 0) | chain | chain |
| std.app.34 | `sans_line`, `Str` placement | chain | chain | chain |

## Windows notes

- **Surface (ADR 1044).** The program creates a named file mapping (`CreateFileMappingA`, name `Local\ail-gfx-PID-TICK`) and sends it as `"name"` in the `surface` line in place of `"path"`; the host opens it with `OpenFileMappingA` and `MapViewOfFile`. Nothing is unlinked. `vsync` is sent after `DwmFlush`, so frames follow the compositor.
- **Gamepads.** `xinput1_4.dll` (else `xinput9_1_0.dll`), polled once per drawn frame, with the XInput deadzones; connected pads are numbered in slot order from 0.
- **Edge mode.** The client area covers the caption (the frame stays: shadow, snapping, resize edges); the host draws minimise, maximise and close at the top-right and reports `inset_right` 138, `inset_top` 32.
- **Folder chooser.** The classic `FolderBrowserDialog`; the Vista `IFileDialog` picker is a later polish.
