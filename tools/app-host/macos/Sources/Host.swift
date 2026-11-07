// The generic SwiftUI host for std.app (ADR 0838): draws whatever view tree
// the AIL program sends and reports what the user does. It knows nothing
// about any one app.

import AppKit
import SwiftUI

/// The latest view from the program.
final class Model: ObservableObject {
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
        Edge.apply(nil)
        if !started {
            started = true
            Outbox.shared.send("start", Edge.insets(Edge.window(nil)))
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

@main
struct HostApp: App {
    @NSApplicationDelegateAdaptor(AppDelegate.self) var delegate
    @StateObject private var model = Model()

    var body: some Scene {
        WindowGroup {
            Root(model: model)
                .frame(minWidth: 320, minHeight: 200)
                .navigationTitle(model.title)
                .onAppear {
                    readViews(onView: { model.show(title: $0, focus: $1, tree: $2) },
                              onEnd: { NSApp.terminate(nil) })
                }
        }
        .defaultSize(width: 1000, height: 720)
    }
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
