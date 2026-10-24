# 0888. `std.app` host for Windows: WinForms and GDI+ in C#, built with the framework's csc

- **Status:** Accepted
- **Date:** 2026-10-24
- **Decided by:** AI (delegated, ADR 0076); Windows host requested by the designer (item 307)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.7`–`10`, `std.app.13`
- **Builds:** [ADR 0838](0838-std-app-native-windows.md), [ADR 0009](0009-custom-tooling-policy.md)

## Context

ADR 0838 deferred hosts for other systems. The protocol is the same
newline-delimited JSON, so a Windows host is a renderer only; apps don't change.

## Decision

1. `tools/app-host/windows/Host.cs` is a single-file generic host: WinForms
   controls (`Label`, `Button`, `TextBox`, `CheckBox`, `TableLayoutPanel`) for the
   tree and one GDI+ `CanvasView` control for `Canvas`. It speaks protocol v1
   exactly as the macOS host does, including `measure` (std.app.13).
2. It is built by `tools/app-host/windows/run.ps1`, which compiles with the
   `csc.exe` that ships with the .NET Framework 4 in `%WINDIR%`. No SDK, no
   package, nothing to install (ADR 0071). The source is C# 5.
3. A view of an unchanged shape updates controls in place, so typing keeps focus;
   a new shape rebuilds the controls and restores focus by id.
4. `host_path` also returns `tools/app-host/windows/build/AILAppHost.exe` under
   `AIL_ROOT` when it exists.

## Consequences

- Windows gets std.app with no new dependency. Fonts are Segoe UI (`sans`) and
  Consolas (`mono`); GDI+ text is not hardware accelerated, fine at app scale.
- Windows on ARM is untested; the build targets x64.

## Rejected alternatives

- **Direct2D and DirectWrite from C.** Faster canvas, but needs a C toolchain on
  the VM (none installed) and interop for every control; GDI+ is enough for the
  cell-based canvases in use.
- **WPF.** Heavier layout system, a separate XAML build step, and its text
  measuring differs from the canvas's.
- **A new compiler install (MSVC, .NET SDK).** Violates the no-setup rule.
- **Writing the host in AIL.** Needs native FFI callbacks (`ffi.decl.3`).
