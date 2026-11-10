# 1008. `std.app` system appearance event

- **Status:** Accepted
- **Date:** 2026-11-09
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library; GUI
- **Spec rules:** `std.app.30` ([std-app.md](../spec/std-app.md))
- **Amends:** [ADR 0838](0838-std-app-native-windows.md)
- **Requested by:** the code editor (`kksimp/ail-edit`), which should follow the system's light or dark theme.

## Context

A program draws its own colours on canvases, so it can't pick up the system theme from native controls. It has no way to ask whether the system is dark or light, and no signal when the user switches.

## Decision

A new event kind, `appearance`, with `text` `dark` or `light`. Each host sends it once right after `start`, and again whenever the system setting changes (and only when the value changed). It needs no new `Event` field. The app may answer with a new view, as for any event. All three hosts implement it:

- macOS: `NSApp.effectiveAppearance`, observed with key-value observing.
- Linux (GTK): GNOME's `color-scheme` setting when present, else `gtk-application-prefer-dark-theme` or a theme name that contains "dark"; watched through `notify` and `changed` signals.
- Windows: `AppsUseLightTheme` in the current user's Personalize registry key, watched through `SystemEvents.UserPreferenceChanged`.

A host that can't read the setting sends nothing, so a program keeps a default theme.

## Pointer events

The ADR was also to add a canvas pointer `move` and a `drag`. `std.app` already has both in effect: `move` (std.app.16, with hover on) and, with the pointer captured by `down`, `move` until `up` (std.app.27). A drag is a `move` between `down` and `up`; the program has the start position and can take `dx` and `dy` from successive `move`s. Nothing is added.

## Rejected alternatives

- **A request line (`{"t":"appearance"}`) the app sends to ask.** A round trip for a value the host already knows, and the app must also handle the change event.
- **Put `appearance` on `start`.** `start` carries only the edge insets, and a change later needs its own event anyway.
- **Send it every resize.** Noise for a rarely changing value.
- **Derive it in the app from the window colour.** The window colour comes from the app's own first fill.

## Consequences

One more event kind for hosts; headless `script` passes `appearance` like any other. A fake-host test (`tests/app/appearance_host.ail`, `tests/cases/std_app_appearance.ail`) checks that `run_host` delivers it and that a view follows.
