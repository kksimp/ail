// The std.app host's side of the protocol (spec std.app.7-10, ADR 0838):
// newline-delimited JSON. Views arrive on standard input; events leave on
// standard output. The first line each way carries "v": 1.

import AppKit

/// A drawing step of a canvas (std.app.3).
enum Op {
    case str(x: Double, y: Double, text: String, font: String, size: Double, fg: Int, bold: Bool, italic: Bool, underline: Bool)
    case fill(x: Double, y: Double, w: Double, h: Double, color: Int)
    case stroke(x1: Double, y1: Double, x2: Double, y2: Double, color: Int, width: Double)
    case pic(x: Double, y: Double, w: Double, h: Double, src: String)
    case bitmap(x: Double, y: Double, w: Double, h: Double, id: String)
    case shade(x: Double, y: Double, w: Double, h: Double, color: Int, alpha: Double)
    case clip(x: Double, y: Double, w: Double, h: Double)
    case unclip
}

/// A node of the view tree (std.app.2), with its path in the tree as identity.
indirect enum Node {
    case label(String)
    case button(id: String, label: String)
    case field(id: String, value: String, hint: String)
    case toggle(id: String, label: String, on: Bool)
    case column([Node])
    case row([Node])
    case spacer
    case canvas(id: String, w: Double, h: Double, ops: [Op])
}

private func num(_ d: [String: Any], _ k: String) -> Double { (d[k] as? NSNumber)?.doubleValue ?? 0 }
private func int(_ d: [String: Any], _ k: String) -> Int { (d[k] as? NSNumber)?.intValue ?? 0 }
private func text(_ d: [String: Any], _ k: String) -> String { d[k] as? String ?? "" }
private func flag(_ d: [String: Any], _ k: String) -> Bool { (d[k] as? NSNumber)?.boolValue ?? false }

/// Reads a node from the derived JSON of `std.app.Node`: `{"kind": "Label", "text": ...}`.
func decodeNode(_ any: Any?) -> Node {
    guard let d = any as? [String: Any] else { return .label("?") }
    switch text(d, "kind") {
    case "Label": return .label(text(d, "text"))
    case "Button": return .button(id: text(d, "id"), label: text(d, "caption"))
    case "Field": return .field(id: text(d, "id"), value: text(d, "value"), hint: text(d, "hint"))
    case "Toggle": return .toggle(id: text(d, "id"), label: text(d, "caption"), on: flag(d, "on"))
    case "Column": return .column((d["kids"] as? [Any] ?? []).map(decodeNode))
    case "Row": return .row((d["kids"] as? [Any] ?? []).map(decodeNode))
    case "Spacer": return .spacer
    case "Canvas":
        return .canvas(id: text(d, "id"), w: num(d, "w"), h: num(d, "h"), ops: (d["ops"] as? [Any] ?? []).compactMap(decodeOp))
    case let k: return .label("unknown node \(k)")
    }
}

func decodeOp(_ any: Any) -> Op? {
    guard let d = any as? [String: Any] else { return nil }
    switch text(d, "kind") {
    case "Str":
        return .str(x: num(d, "x"), y: num(d, "y"), text: text(d, "text"), font: text(d, "font"), size: num(d, "size"),
                    fg: int(d, "fg"), bold: flag(d, "bold"), italic: flag(d, "italic"), underline: flag(d, "underline"))
    case "Fill": return .fill(x: num(d, "x"), y: num(d, "y"), w: num(d, "w"), h: num(d, "h"), color: int(d, "color"))
    case "Stroke":
        return .stroke(x1: num(d, "x1"), y1: num(d, "y1"), x2: num(d, "x2"), y2: num(d, "y2"), color: int(d, "color"), width: num(d, "width"))
    case "Pic": return .pic(x: num(d, "x"), y: num(d, "y"), w: num(d, "w"), h: num(d, "h"), src: text(d, "src"))
    case "Bitmap":
        let id = text(d, "id")
        Bitmaps.store(id, w: int(d, "pw"), h: int(d, "ph"), base64: text(d, "data"))
        return .bitmap(x: num(d, "x"), y: num(d, "y"), w: num(d, "w"), h: num(d, "h"), id: id)
    case "Shade": return .shade(x: num(d, "x"), y: num(d, "y"), w: num(d, "w"), h: num(d, "h"), color: int(d, "color"), alpha: num(d, "alpha"))
    case "Clip": return .clip(x: num(d, "x"), y: num(d, "y"), w: num(d, "w"), h: num(d, "h"))
    case "Unclip": return .unclip
    default: return nil
    }
}

/// Images sent as `Bitmap` ops, kept by id so a later view can omit the data (std.app.3).
enum Bitmaps {
    private static var images: [String: NSImage] = [:]
    private static let lock = NSLock()

    static func image(_ id: String) -> NSImage? {
        lock.lock(); defer { lock.unlock() }
        return images[id]
    }

    /// Decodes `R, G, B, A` bytes (non-premultiplied) to an image; empty data keeps what `id` has.
    static func store(_ id: String, w: Int, h: Int, base64: String) {
        guard !base64.isEmpty, w > 0, h > 0, let data = Data(base64Encoded: base64), data.count == w * h * 4 else { return }
        var px = [UInt8](data)
        for i in stride(from: 0, to: px.count, by: 4) {
            let a = UInt16(px[i + 3])
            for c in 0..<3 { px[i + c] = UInt8((UInt16(px[i + c]) * a + 127) / 255) }
        }
        guard let provider = CGDataProvider(data: Data(px) as CFData),
              let cg = CGImage(width: w, height: h, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: w * 4,
                               space: CGColorSpaceCreateDeviceRGB(),
                               bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.premultipliedLast.rawValue),
                               provider: provider, decode: nil, shouldInterpolate: true, intent: .defaultIntent)
        else { return }
        lock.lock(); defer { lock.unlock() }
        images[id] = NSImage(cgImage: cg, size: NSSize(width: w, height: h))
    }
}

/// Writes events to standard output, one JSON object per line (std.app.8).
final class Outbox {
    static let shared = Outbox()
    private let queue = DispatchQueue(label: "std.app.outbox")
    private var greeted = false

    func send(_ kind: String, _ fields: [String: Any] = [:]) {
        queue.async {
            var e = fields
            e["kind"] = kind
            if !self.greeted {
                e["v"] = 1
                self.greeted = true
            }
            guard let data = try? JSONSerialization.data(withJSONObject: e, options: [.sortedKeys]) else { return }
            FileHandle.standardOutput.write(data)
            FileHandle.standardOutput.write(Data([0x0a]))
        }
    }
}

/// std.app.13: the advance width in points of each requested run, in the font a `Str` op
/// of the same family, size and weight is drawn in, sent back as one `measured` event whose
/// `text` is the widths joined by commas.
func answerMeasure(_ runs: [[String: Any]]) {
    let widths = runs.map { r -> String in
        let f = font(text(r, "font"), num(r, "size"), bold: flag(r, "bold"), italic: flag(r, "italic"))
        let w = (text(r, "text") as NSString).size(withAttributes: [.font: f]).width
        return String(format: "%.3f", Double(w))
    }
    Outbox.shared.send("measured", ["text": widths.joined(separator: ",")])
}

/// Reads views from standard input on a background thread and hands each
/// one to `onView` on the main thread; `onEnd` runs when the input ends.
func readViews(onView: @escaping (String, String, Node) -> Void, onEnd: @escaping () -> Void) {
    Thread.detachNewThread {
        var first = true
        while let line = readLine(strippingNewline: true) {
            guard let data = line.data(using: .utf8),
                  let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                  let kind = obj["t"] as? String
            else { continue }
            if kind == "measure" {
                answerMeasure(obj["runs"] as? [[String: Any]] ?? [])
                continue
            }
            if kind == "surface" {
                // std.gfx.8 (ADR 0957): map the program's shared frame buffers.
                if !Surfaces.attach(obj) {
                    FileHandle.standardError.write("AIL App Host: cannot map the surface\n".data(using: .utf8)!)
                }
                continue
            }
            if kind == "present" {
                Surfaces.present(obj)
                continue
            }
            if kind == "popup" {
                // std.app.18: a native menu at (x, y) of the canvas `id`.
                let id = text(obj, "id"), x = num(obj, "x"), y = num(obj, "y")
                let items = obj["items"] as? [String] ?? []
                DispatchQueue.main.async { showPopup(id: id, x: x, y: y, items: items) }
                continue
            }
            if kind == "pick" {
                // std.app.22: a native chooser; answered with `picked`.
                let id = text(obj, "id"), prompt = text(obj, "prompt"), folders = flag(obj, "folders"), start = text(obj, "start")
                DispatchQueue.main.async { showPick(id: id, prompt: prompt, folders: folders, start: start) }
                continue
            }
            if kind == "drag" || kind == "zoom" {
                // std.app.25: in edge mode the app says when its strip is dragged or double-clicked.
                DispatchQueue.main.async { Edge.act(kind) }
                continue
            }
            if kind != "view" { continue }
            Hover.enabled = flag(obj, "hover")
            Edge.wanted = flag(obj, "edge")
            if first {
                first = false
                if (obj["v"] as? NSNumber)?.intValue != 1 {
                    FileHandle.standardError.write("AIL App Host: the program speaks another protocol version\n".data(using: .utf8)!)
                    break
                }
            }
            let title = obj["title"] as? String ?? "AIL"
            let focus = obj["focus"] as? String ?? ""
            let tree = decodeNode(obj["tree"])
            DispatchQueue.main.async { onView(title, focus, tree) }
        }
        DispatchQueue.main.async { onEnd() }
    }
}

/// std.app.16: whether the program wants `move` and `leave` events.
enum Hover {
    static var enabled = false
}

/// std.app.25 (ADR 0981): edge mode. The app draws up to the window's top edge; the
/// title bar is hidden, the traffic lights stay, and the app is told the room they take.
enum Edge {
    static var wanted = false
    static var applied = false
    static var lastEvent: NSEvent?

    /// The window controls' room in points: their right edge plus a gap, and the title bar's height.
    static func insets(_ w: NSWindow?) -> [String: Any] {
        guard applied, let w = w else { return ["inset_left": 0, "inset_top": 0] }
        let right = w.standardWindowButton(.zoomButton)?.frame.maxX ?? 70
        let top = w.frame.height - w.contentLayoutRect.height
        return ["inset_left": right + 8, "inset_top": max(top, 0)]
    }

    /// The app's window: the one a canvas sits in, else the key or first visible one.
    static func window(_ w: NSWindow?) -> NSWindow? {
        w ?? CanvasView.registry.values.first?.window ?? NSApp.keyWindow
            ?? NSApp.windows.first(where: { $0.isVisible && $0.contentView != nil })
    }

    /// Hides the title bar once the app asks for edge mode, and keeps it hidden: SwiftUI
    /// sets the title again on every update. The first time, every canvas reports its
    /// size again so the app learns the insets.
    static func apply(_ w: NSWindow?) {
        guard wanted, let w = window(w) else { return }
        let first = !applied
        applied = true
        if !w.styleMask.contains(.fullSizeContentView) { w.styleMask.insert(.fullSizeContentView) }
        w.titlebarAppearsTransparent = true
        w.titleVisibility = .hidden
        w.toolbar = nil
        w.titlebarSeparatorStyle = .none
        w.isMovableByWindowBackground = false
        // The title bar's material still tints the content under it; hide it and keep the buttons.
        if let frame = w.contentView?.superview {
            for v in frame.subviews where String(describing: type(of: v)).contains("Titlebar") {
                hideMaterial(v)
            }
        }
        if first {
            FileHandle.standardError.write("AIL App Host: edge mode on (std.app.25)\n".data(using: .utf8)!)
            for v in CanvasView.registry.values { v.reportSize() }
        }
    }

    static func hideMaterial(_ v: NSView) {
        for s in v.subviews {
            if s is NSVisualEffectView { s.isHidden = true } else { hideMaterial(s) }
        }
    }

    static func act(_ kind: String) {
        guard let w = window(nil) else { return }
        if kind == "zoom" { w.zoom(nil); return }
        if let e = lastEvent { w.performDrag(with: e) }
    }
}

/// std.app.18: shows `items` as a menu and answers with a `popup` event whose `text` is the
/// chosen label, or empty if the menu was dismissed.
func showPopup(id: String, x: Double, y: Double, items: [String]) {
    guard let view = CanvasView.registry[id] else {
        Outbox.shared.send("popup", ["id": id, "text": ""])
        return
    }
    let picker = MenuPicker()
    let menu = NSMenu()
    for label in items {
        let item = NSMenuItem(title: label, action: #selector(MenuPicker.pick(_:)), keyEquivalent: "")
        item.target = picker
        menu.addItem(item)
    }
    menu.popUp(positioning: nil, at: NSPoint(x: x, y: y), in: view)
    Outbox.shared.send("popup", ["id": id, "text": picker.chosen])
}

final class MenuPicker: NSObject {
    var chosen = ""
    @objc func pick(_ sender: NSMenuItem) { chosen = sender.title }
}

/// std.app.22: shows an open panel for a folder or a file and answers with a `picked` event
/// whose `text` is the chosen path, empty if the user cancelled.
func showPick(id: String, prompt: String, folders: Bool, start: String) {
    let panel = NSOpenPanel()
    panel.canChooseDirectories = folders
    panel.canChooseFiles = !folders
    panel.allowsMultipleSelection = false
    panel.prompt = "Choose"
    panel.message = prompt
    if !start.isEmpty { panel.directoryURL = URL(fileURLWithPath: start) }
    let finish: (NSApplication.ModalResponse) -> Void = { r in
        let path = r == .OK ? (panel.url?.path ?? "") : ""
        Outbox.shared.send("picked", ["id": id, "text": path])
    }
    if let w = NSApp.keyWindow {
        panel.beginSheetModal(for: w, completionHandler: finish)
    } else {
        panel.begin(completionHandler: finish)
    }
}
