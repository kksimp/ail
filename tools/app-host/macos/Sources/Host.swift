// The generic SwiftUI host for std.app (ADR 0838): draws whatever view tree
// the AIL program sends and reports what the user does. It knows nothing
// about any one app.

import AppKit
import SwiftUI

/// The latest view from the program.
final class Model: ObservableObject {
    /// The one model: the AppDelegate starts the reader before any window exists.
    static let shared = Model()
    @Published var title = "AIL"
    @Published var focus = ""
    @Published var tree: Node = .label("")
    @Published var started = false
    /// What each field shows: the user's typing, until the program sends a
    /// different value than it sent last time.
    var fieldText: [String: String] = [:]
    var lastSent: [String: String] = [:]

    func show(title: String, focus: String, tree: Node) {
        self.title = title
        self.focus = focus
        sync(tree)
        self.tree = tree
        Edge.apply(self)
        Edge.own?.title = title
        if !started {
            started = true
            Outbox.shared.send("start", Edge.insets(Edge.window(nil)))
            Appearance.begin()
        }
    }

    /// True when the tree has a canvas that fills the window: the app draws
    /// its own chrome, so the host adds no frame and no gaps (ADR 0888, as
    /// the Windows host does).
    static func edge(_ n: Node) -> Bool {
        switch n {
        case let .canvas(_, w, h, _): return w <= 0 || h <= 0
        case let .column(kids), let .row(kids): return kids.contains(where: edge)
        default: return false
        }
    }

    private func sync(_ n: Node) {
        switch n {
        case let .field(id, value, _):
            if lastSent[id] != value {
                lastSent[id] = value
                fieldText[id] = value
            }
        case let .column(kids), let .row(kids): kids.forEach(sync)
        default: break
        }
    }
}

final class AppDelegate: NSObject, NSApplicationDelegate {
    func applicationDidFinishLaunching(_ n: Notification) {
        NSApp.setActivationPolicy(.regular)
        // AIL_APP_ICON names a PNG for the Dock icon (run.sh sets it for Garlic).
        if let path = ProcessInfo.processInfo.environment["AIL_APP_ICON"], let img = NSImage(contentsOfFile: path) {
            NSApp.applicationIconImage = img
        }
        NSApp.activate(ignoringOtherApps: true)
        // The program's first view must not wait for a window: on some systems (macOS 27) the
        // SwiftUI scene opens none, and the reader would never start. Read now, and open a window
        // ourselves if SwiftUI has not shown one after a moment.
        startReader()
        DispatchQueue.main.asyncAfter(deadline: .now() + 1.5) {
            if !NSApp.windows.contains(where: { $0.isVisible && $0.contentView != nil }) {
                openFallbackWindow()
            }
        }
        // A window restored from another display's position can open off screen.
        DispatchQueue.main.async {
            for w in NSApp.windows where !(NSScreen.screens.contains { $0.visibleFrame.intersects(w.frame) }) {
                w.center()
            }
        }
    }
    func applicationShouldTerminateAfterLastWindowClosed(_ app: NSApplication) -> Bool { true }
    func applicationWillTerminate(_ n: Notification) {
        Outbox.shared.send("close")
        Thread.sleep(forTimeInterval: 0.05)
    }
}

/// Starts the stdin reader once, whichever comes first: the delegate or a window's root.
func startReader() {
    if Reader.started { return }
    Reader.started = true
    readViews(onView: { Model.shared.show(title: $0, focus: $1, tree: $2) },
              onEnd: { NSApp.terminate(nil) })
}

/// A plain window holding the same root, for when the SwiftUI scene opens none.
func openFallbackWindow() {
    let win = NSWindow(contentRect: NSRect(x: 0, y: 0, width: 1000, height: 720),
                       styleMask: [.titled, .closable, .miniaturizable, .resizable],
                       backing: .buffered, defer: false)
    win.title = Model.shared.title
    win.minSize = NSSize(width: 320, height: 200)
    win.isReleasedWhenClosed = false
    let hosting = NSHostingView(rootView: Root(model: Model.shared))
    hosting.autoresizingMask = [.width, .height]
    win.contentView = hosting
    win.center()
    Edge.own = win
    win.makeKeyAndOrderFront(nil)
    NSApp.activate(ignoringOtherApps: true)
    FileHandle.standardError.write("AIL App Host: opened a window of its own (the SwiftUI scene showed none)\n".data(using: .utf8)!)
}

@main
struct HostApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var delegate
    @StateObject private var model = Model.shared

    var body: some Scene {
        WindowGroup {
            Root(model: model)
                .frame(minWidth: 320, minHeight: 200)
                .navigationTitle(model.title)
                .onAppear { startReader() }
        }
        .defaultSize(width: 1000, height: 720)
    }
}

/// The stdin reader runs once, whichever window's root appears first.
enum Reader {
    static var started = false
}

struct Root: View {
    @ObservedObject var model: Model
    @FocusState private var focused: String?

    var body: some View {
        let edge = Model.edge(model.tree)
        NodeView(node: model.tree, model: model, focused: $focused, gap: edge ? 0 : 6)
            .padding(edge ? 0 : 8)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
            // std.app.25: in edge mode the tree reaches under the (transparent) title bar,
            // which SwiftUI otherwise keeps clear as a safe area.
            .ignoresSafeArea(.container, edges: Edge.wanted ? .top : [])
            .onChange(of: model.focus) { id in
                if id.isEmpty { return }
                // std.app.4: a canvas named by `focus` takes the keys (its view
                // may still be building, so ask again on the next turn).
                if let c = CanvasView.registry[id] {
                    c.window?.makeFirstResponder(c)
                } else {
                    focused = id
                    DispatchQueue.main.async {
                        if let c = CanvasView.registry[id] { c.window?.makeFirstResponder(c) }
                    }
                }
            }
    }
}

struct NodeView: View {
    let node: Node
    @ObservedObject var model: Model
    var focused: FocusState<String?>.Binding
    /// The space between a column's or row's children.
    var gap: CGFloat = 6

    var body: some View {
        switch node {
        case let .label(s):
            Text(s).textSelection(.enabled).frame(maxWidth: .infinity, alignment: .leading)
        case let .button(id, label):
            Button(label) { Outbox.shared.send("press", ["id": id]) }
        case let .field(id, _, hint):
            TextField(hint, text: Binding(
                get: { model.fieldText[id] ?? "" },
                set: { v in
                    model.fieldText[id] = v
                    Outbox.shared.send("edit", ["id": id, "text": v])
                }))
                .textFieldStyle(.roundedBorder)
                .focused(focused, equals: id)
                .onSubmit { Outbox.shared.send("submit", ["id": id]) }
        case let .toggle(id, label, on):
            Toggle(label, isOn: Binding(
                get: { on },
                set: { v in Outbox.shared.send("toggle", ["id": id, "text": v ? "on" : "off"]) }))
        case let .column(kids):
            VStack(alignment: .leading, spacing: gap) {
                ForEach(Array(kids.enumerated()), id: \.offset) { _, k in NodeView(node: k, model: model, focused: focused, gap: gap) }
            }
        case let .row(kids):
            HStack(spacing: gap) {
                ForEach(Array(kids.enumerated()), id: \.offset) { _, k in NodeView(node: k, model: model, focused: focused, gap: gap) }
            }
        case .spacer:
            Spacer()
        case let .canvas(id, w, h, ops):
            CanvasRep(id: id, ops: ops)
                .frame(width: w > 0 ? w : nil, height: h > 0 ? h : nil)
                .frame(maxWidth: w > 0 ? nil : .infinity, maxHeight: h > 0 ? nil : .infinity)
        }
    }
}
