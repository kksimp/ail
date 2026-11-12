# 0929. `fs.read_range` and `fs.trash`: partial reads and recoverable delete

- **Status:** Accepted
- **Date:** 2026-11-04
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** Standard library (`std.fs`)
- **Spec rules:** std.fs.25, std.fs.26

## Context

Finding duplicates cheaply means hashing the first block of same-size files before whole files, and `read_bytes` reads everything. A tool that deletes the user's files should be able to do so recoverably; `fs.remove` is permanent.

## Options considered

### Option A — `read_range` and `trash` in std.fs (chosen)
- **Pros:** reads cost bytes asked for; the trash call is correct per OS (volume trash, put-back info, name collisions).
- **Cons:** `trash` has three OS arms.

### Option B — `read_bytes` then slice; `fs.rename` into `~/.Trash`
- **Pros:** no new API.
- **Cons:** reads whole files; a rename fails across volumes, skips put-back info and collisions.

## Decision

Add `fs.read_range(path, offset, len)` (`std.fs.25`) and `fs.trash(path)` (`std.fs.26`). `trash` never deletes permanently and never copies; `fs.remove` remains the permanent call.

## Rationale

Recoverable by default is the safe behaviour for any app that removes user data, so no permanent-delete default is offered.

## Consequences

- Husk's only delete path is `fs.trash`.
- Native only (ADR 0865); the macOS arm first.
- Windows arm (2026-11-05): `SHFileOperationW` with `FO_DELETE` and `FOF_ALLOWUNDO`, from `SHELL32.dll`, which joins the import table only for programs that call `fs.trash`. The path is made full with `GetFullPathNameW` first, because the call deletes outright what is not a full path. `FOF_NOCONFIRMATION` is left off so an item too large for the bin is refused and never destroyed; `FOF_SILENT`, `FOF_NOERRORUI` and `FOF_WANTNUKEWARNINGS` keep the call quiet.

## Rejected alternatives

Option B is wrong on other volumes and loses put-back.

For the Windows arm: writing `$Recycle.Bin` entries (`$I` and `$R` files under the user's SID) by hand relies on an undocumented format that Explorer's Restore reads; `IFileOperation` is the newer API but needs COM initialised on the thread, which the runtime does not do.
