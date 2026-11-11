# std.app host parity: macOS, Linux, Windows

Status: working table for v0.1.0 slice P6 (2026-11-10, ail main). One row per op, node, protocol line and event of the macOS host's `tools/app-host/macos/Sources/Protocol.swift`, with what `tools/app-host/linux/host.c` and `tools/app-host/windows/Host.cs` do. **Linux owners add the Linux column; the Windows lane owns the Windows column.** "yes" means implemented and covered by `tests/ui/host_conformance.txt` or a manual check named in the notes; "no" is a gap to fill before the tag; "not on main" means the rule is not in `docs/spec/` (std.app.28, .29, .32, .34 are 0.2 work, [ADR 1035](../decisions/1035-v0-1-0-freeze-and-release-candidate.md)).

Conformance: `tests/ui/host_conformance.txt` is the script every host must pass without an AIL toolchain. `tests/ui/host_conformance.sh PLATFORM HOST...` plays it on macOS and Linux; `tools/app-host/windows/smoke.ps1` plays it on Windows and adds the Windows-only checks (surface mapping).

## Nodes and ops (std.app.2-3, .19)

| Item | Rule | macOS | Linux | Windows |
|---|---|---|---|---|
| Label, Button, Field, Toggle, Column, Row, Spacer, Canvas | std.app.2 | yes | yes | yes |
| Str, Fill, Stroke, Clip, Unclip | std.app.3 | yes | yes | yes |
| Pic, Bitmap | std.app.3 | yes | yes | yes |
| Shade | std.app.19 | yes | yes | yes |
| Str `underline`, `bold`, `italic` | std.app.3 | yes | yes | yes |

## Lines the program sends

| Line | Rule | macOS | Linux | Windows |
|---|---|---|---|---|
| `view` (title, focus) | std.app.7 | yes | yes | yes |
| `view` `hover` | std.app.16 | yes | yes | yes |
| `view` `edge` | std.app.25 | yes | yes (title bar hidden, ADR 1033) | yes (ADR 1033) |
| `measure` | std.app.13 | yes | yes | yes |
| `popup` | std.app.18 | yes | yes (GtkMenu) | yes (ContextMenuStrip) |
| `pick` (file and folder) | std.app.22 | yes | yes (GtkFileChooserDialog) | yes (OpenFileDialog, FolderBrowserDialog) |
| `clipboard` | std.app.26 | yes | yes | yes |
| `drag`, `zoom` | std.app.25 | yes | yes | yes |
| `surface`, `present`, `text` | std.gfx.7-8 | yes | yes (shm file via `path`) | yes (named file mapping, ADR 1044) |
| `lines()` lines the host does not know are skipped | std.app.21 | yes | yes | yes |

## Events the host sends

| Event | Rule | macOS | Linux | Windows |
|---|---|---|---|---|
| `start` (screen_w, screen_h, scale) | std.app.31 | yes | yes | yes |
| `start`, `resize` insets (inset_left, inset_top) | std.app.25 | yes | yes (0, 32 in edge mode) | yes (0, 32) |
| `start`, `resize` inset_right | std.app.33 | not sent (0) | yes (138 in edge mode) | yes (138) |
| `resize` (cell size, screen) | std.app.5, .31 | yes | yes | yes |
| `press`, `edit`, `submit`, `toggle` | std.app.8 | yes | yes | yes |
| `click` with `clicks` | std.app.17 | yes | yes | yes |
| `down`, `up`, pointer capture | std.app.27 | yes | yes (GTK implicit grab) | yes |
| `move`, `leave` | std.app.16 | yes | yes | yes |
| `scroll` | std.app.8 | yes | yes | yes |
| `key` with `repeat` and `code` | std.app.20 | yes | yes | yes |
| `keyup` | std.app.20 | yes | yes | yes |
| `paste` | std.app.23 | yes | yes | yes |
| `measured` | std.app.13 | yes | yes | yes |
| `popup` answer | std.app.18 | yes | yes (GtkMenu) | yes |
| `picked`, `pick_error` | std.app.22 | yes | yes (`pick_error` never sent) | yes |
| `appearance` | std.app.30 | yes | yes | yes |
| `vsync` | std.gfx.8 | yes | yes (after each drawn frame) | yes |
| `pad` | std.gfx.11 | yes (GameController) | yes (evdev) | yes (XInput) |
| `close` | std.app.8 | yes | yes | yes |

## Landed after this table was started (#1149-#1153)

std.app.28, .29, .32 and .34 are on main now. Each host got its half in the same chain; the Linux half (`Round`, `Poly`, `Line`, font lists, `sans_line`) is verified by the ail-linux run named in the PR, not by `tests/ui/host_conformance.txt`.

## Windows notes

- **Surface (ADR 1044).** The program creates a named file mapping (`CreateFileMappingA`, name `Local\ail-gfx-PID-TICK`) and sends it as `"name"` in the `surface` line in place of `"path"`; the host opens it with `OpenFileMappingA` and `MapViewOfFile`. Nothing is unlinked. `vsync` is sent after `DwmFlush`, so frames follow the compositor.
- **Gamepads.** `xinput1_4.dll` (else `xinput9_1_0.dll`), polled once per drawn frame, with the XInput deadzones; connected pads are numbered in slot order from 0.
- **Edge mode.** The client area covers the caption (the frame stays: shadow, snapping, resize edges); the host draws minimise, maximise and close at the top-right and reports `inset_right` 138, `inset_top` 32.
- **Folder chooser.** The classic `FolderBrowserDialog`; the Vista `IFileDialog` picker is a later polish.

## Linux notes

- **Surface.** The program's file is mapped `MAP_SHARED` by `path` as on macOS; each frame is converted from RGBA to cairo's pixel order and painted scaled with nearest-neighbour filtering. A `vsync` follows each drawn frame, which GTK's frame clock paces. `AIL_GFX_REPORT=FILE` appends the same size line the macOS check reads.
- **Gamepads.** evdev, `/dev/input/event*`, rescanned every 2 s and polled once per drawn frame, up to four pads, with a small stick dead zone. The user must be in the `input` group (or the devices otherwise readable); an unreadable device is skipped without a message. Layout is the common Xbox-style one (xpad, hid-generic): `BTN_SOUTH` is `a`, `BTN_EAST` `b`, `BTN_NORTH` `x`, `BTN_WEST` `y`, hat or `BTN_DPAD_*` for the d-pad.
- **Edge mode.** The window is undecorated while the app asks for edge mode. The host draws minimise, maximise and close at the top-right (46 by 32 points each) and resize grips on the edges and corners; `inset_right` is 138 and `inset_top` 32. Resize grips are on all four edges and corners; the controls sit above them.
- **Menus and choosers.** `popup` uses a `GtkMenu` and answers after the item's `activate`; `pick` uses `GtkFileChooserDialog`, modal and asynchronous.
- **Conformance.** `tools/app-host/linux/smoke.sh` builds the host, plays `tests/ui/host_conformance.txt` under `xvfb-run` when there is no display, then maps a surface and expects a `vsync`.
