// std.app.30 (ADR 1008): tell the program whether the system is in dark or light
// mode, once after `start` and again whenever the setting changes.

import AppKit

enum Appearance {
    private static var watch: NSKeyValueObservation?
    private static var last = ""

    /// "dark" or "light", as the app's effective appearance says.
    static func current() -> String {
        let name = NSApp.effectiveAppearance.bestMatch(from: [.darkAqua, .aqua])
        return name == .darkAqua ? "dark" : "light"
    }

    /// Sends the current mode, unless it is the one last sent.
    static func send() {
        let now = current()
        if now == last { return }
        last = now
        Outbox.shared.send("appearance", ["text": now])
    }

    /// Sends the mode now and on every change.
    static func begin() {
        send()
        watch = NSApp.observe(\.effectiveAppearance, options: []) { _, _ in
            DispatchQueue.main.async { send() }
        }
    }
}
