# 1044. `std.gfx` on Windows: the shared surface is a named file mapping

- **Status:** Accepted
- **Date:** 2026-11-10
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.gfx.8`, `std.gfx.11` ([std-gfx.md](../spec/std-gfx.md))
- **Amends:** [ADR 0957](0957-shared-surface-transport.md)
- **Requested by:** the designer (2026-11-10): the three hosts at feature parity for v0.1.0.

## Context

`host.run` (ADR 0957) makes its shared frame buffers with the C library's `mkstemp`, `ftruncate` and `mmap`, and the host maps the file by path. Windows has none of the three in `ucrtbase.dll`, so `host.run` could not draw on the Windows host, which also had no `surface`, `present`, `text` or `vsync` handling. The Windows host also sent no `pad` events (ADR 0985).

## Decision

1. **Transport.** On Windows the program creates a named, pagefile-backed file mapping with kernel32 (`CreateFileMappingA` on `INVALID_HANDLE_VALUE`, name `Local\ail-gfx-PID-TICK`, `MapViewOfFile`), with the layout of std.gfx.8 (guard, two buffers, guard). The `surface` line carries `"name"` in place of `"path"`. The host opens it with `OpenFileMappingA` and maps it read-only. The program keeps its handle until the process ends; nothing is unlinked. macOS and Linux keep `path`.
2. **Pacing.** After a frame is painted the host waits for the compositor with `DwmFlush` (a 16 ms sleep if DWM is off), polls the gamepads and sends `vsync`, so a game runs at the display's refresh rate as on the Mac.
3. **Drawing.** The host draws the buffer with GDI+ straight from the mapping (a `Bitmap` over the shared bytes, R and B swapped by a colour matrix, nearest-neighbour scaling), and `text` items over it in Consolas.
4. **Gamepads.** `xinput1_4.dll` (else `xinput9_1_0.dll`), mapped to the bits of std.gfx.11 with XInput's recommended deadzones, connected pads numbered from 0 in slot order.

## Rejected alternatives

- **A file under `%TEMP%` mapped by both sides** (`CreateFileW` and `CreateFileMapping`): leaves a file behind if either side dies, and needs the delete-on-close dance the named mapping avoids.
- **Sending pixels as base64 on the pipe:** the cost ADR 0957 exists to avoid.
- **Inheriting a handle into the host process:** the host is started by `proc.start`, which has no handle inheritance.
- **`MapViewOfFile3` placeholders:** Windows 10 1803 and later only, and not needed here.

## Consequences

`host.pixels` keeps its private buffer on Windows. `tools/app-host/windows/smoke.ps1` checks the mapping and `vsync`. A game's first frame and the gamepad need a manual run on ail-win (a pad plugged in).
