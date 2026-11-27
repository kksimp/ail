# TBD. `fs.watch`: changes under a directory as a channel of paths

- **Status:** Accepted
- **Date:** 2026-11-27
- **Decided by:** AI (delegated, [ADR 0076](0076-delegated-decisions.md))
- **Topic:** Standard library, `std.fs`
- **Spec rules:** [std.fs.39-41](../spec/17-std.md)

## Context

Agents and tools (rebuild on save, sync, live reload) want to know when files change. Polling `fs.scan` costs a
full tree walk per look. Every target has a kernel interface for this: inotify (Linux), FSEvents (macOS) and
`ReadDirectoryChangesW` (Windows). The std gap plan (wave 3) asks for `fs.watch(dir) Channel[str]`.

## Decision

`fs.watch(dir) Channel[str] ! IoError` watches `dir` recursively. Each value is a path, `dir` as given joined with
`/` and the entry's path below it. The receiver reads the entry again to learn what happened; there is no event
kind. Lost events give `dir` itself (std.fs.40). Removing or moving `dir` gives `dir` and ends the channel, and
`close` stops the watch (std.fs.41). It needs `fs.read`.

The runtime (`compiler/rt/native/fswatch.ail`) feeds the channel the way signal channels are fed (ADR 1011): the
channel's `source` word holds a watch state, and a receiver that finds it empty asks the system for events, waiting
at most 100 ms a round. `sig_is` accepts the watch state and `sig_poll`, `sig_pump` and `sig_close` hand it on, so
`proc.ail`'s hooks are unchanged.

- **Linux:** a non-blocking inotify descriptor with one watch per directory, added for new directories as their
  `IN_CREATE` arrives (and their entries queued, as they may predate the watch); a directory moved away loses its
  watches. `IN_Q_OVERFLOW` gives `dir`.
- **macOS:** CoreServices is opened with `dlopen` (the executable imports only libSystem) and an FSEvents stream with
  file events, `NoDefer` and `WatchRoot`, 20 ms latency, runs on a dispatch queue. Its callback runs on a thread AIL
  did not start, so it is a plain runtime function (AAPCS64-compliant, as every AIL function is on arm64) that only
  copies the paths into a ring outside the heap under a spin lock and wakes the receiver, as the signal handler
  does; it allocates nothing and uses only `for` loops. Paths are reported under the real path (`/private/tmp`), so
  that prefix is replaced by `dir` as given. `MustScanSubDirs` and the dropped flags give `dir`; `RootChanged` ends
  the channel. The ring (256 KiB) is not freed on close, since a callback in flight may still write it.
- **Windows:** the directory opened with `FILE_FLAG_OVERLAPPED`, one `ReadDirectoryChangesW` read over the subtree
  at a time into a 64 KiB buffer outside the heap, waited on with its event. A zero-byte completion (overflow) gives
  `dir`; a failed read (the directory went away) ends the channel. `\` in names becomes `/`.

## Rejected alternatives

- **Values with an event kind** (`Change(path, kind)`): the three systems disagree on kinds (FSEvents coalesces
  created, modified and removed into one flag set; rename pairs differ), so a kind would be wrong in some cases on
  some host. Reading the entry again is correct everywhere and keeps the type `Channel[str]` the plan names.
- **Polling `fs.scan` in AIL:** portable and simple, but costs a tree walk per look and misses changes inside the
  interval's resolution; speed (ADR 1077, third) favours the kernel interfaces.
- **kqueue on macOS:** needs a descriptor per file to see writes and does not name the changed entry in a directory;
  large trees exhaust descriptors.
- **A C callback through `ffi.cb` (ADR 1074):** its entry traps on threads AIL did not start (ffi.cb.4), which is
  where FSEvents calls; the callback here touches no collected memory, so it needs no such entry.
- **Non-recursive watching:** inotify alone is non-recursive, but FSEvents and `ReadDirectoryChangesW` watch trees
  natively and agents expect a project directory to be watched whole.
