// The `Canvas` node (std.app.3-4): an AppKit view that draws the display
// list with Core Graphics and reports clicks, scrolling, keys and its size.

import AppKit
import SwiftUI

/// The monospace font a canvas's cell size is measured in (std.app.5).
let cellFont = NSFont.monospacedSystemFont(ofSize: 13, weight: .regular)

func color(_ c: Int) -> NSColor {
    NSColor(srgbRed: CGFloat((c >> 16) & 0xff) / 255, green: CGFloat((c >> 8) & 0xff) / 255,
            blue: CGFloat(c & 0xff) / 255, alpha: 1)
}

func font(_ family0: String, _ size: Double, bold: Bool, italic: Bool) -> NSFont {
    return fontUsed(family0, size, bold: bold, italic: italic).0
}

/// The installed font families, lower-cased, for std.app.28.
private let installedFamilies: Set<String> = Set(NSFontManager.shared.availableFontFamilies.map { $0.lowercased() })

/// Resolves a CSS `font-family` list (std.app.28): the first installed family wins,
/// else the list's first generic (`serif`, `mono` for monospace, `sans` otherwise).
/// Returns the host face or the installed family's name.
func resolveFamily(_ list: String) -> String {
    var generic = ""
    for part in list.split(separator: ",") {
        var f = part.trimmingCharacters(in: .whitespaces)
        if f.count >= 2, let q = f.first, (q == "\"" || q == "'"), f.last == q {
            f = String(f.dropFirst().dropLast()).trimmingCharacters(in: .whitespaces)
        }
        if f.isEmpty { continue }
        let low = f.lowercased()
        switch low {
        case "serif": if generic.isEmpty { generic = "serif" }
        case "monospace", "ui-monospace", "mono": if generic.isEmpty { generic = "mono" }
        case "sans-serif", "system-ui", "ui-sans-serif", "sans", "-apple-system", "blinkmacsystemfont":
            if generic.isEmpty { generic = "sans" }
        default:
            if installedFamilies.contains(low) {
                return NSFontManager.shared.availableFontFamilies.first { $0.lowercased() == low } ?? f
            }
        }
    }
    return generic.isEmpty ? "sans" : generic
}

/// The font a `Str` of this family is drawn in and the family the host chose (std.app.28).
func fontUsed(_ family0: String, _ size: Double, bold: Bool, italic: Bool) -> (NSFont, String) {
    let s = CGFloat(size > 0 ? size : 13)
    let w: NSFont.Weight = bold ? .bold : .regular
    var f: NSFont
    if family0.hasPrefix("web:") {
        if let wf = WebFonts.font(String(family0.dropFirst(4)), s) { return (wf, family0) }
        return (NSFont.systemFont(ofSize: s, weight: w), "sans")
    }
    let family = (family0 == "mono" || family0 == "sans" || family0 == "serif") ? family0 : resolveFamily(family0)
    switch family {
    case "sans": f = NSFont.systemFont(ofSize: s, weight: w)
    case "serif":
        let sys = NSFont.systemFont(ofSize: s, weight: w)
        f = sys.fontDescriptor.withDesign(.serif).flatMap { NSFont(descriptor: $0, size: s) } ?? sys
    case "mono": f = NSFont.monospacedSystemFont(ofSize: s, weight: w)
    default:
        var traits: NSFontDescriptor.SymbolicTraits = []
        if bold { traits.insert(.bold) }
        if italic { traits.insert(.italic) }
        let d = NSFontDescriptor(fontAttributes: [.family: family]).withSymbolicTraits(traits)
        f = NSFont(descriptor: d, size: s) ?? NSFont(descriptor: NSFontDescriptor(fontAttributes: [.family: family]), size: s)
            ?? NSFont.systemFont(ofSize: s, weight: w)
        return (f, family)
    }
    if italic { f = NSFontManager.shared.convert(f, toHaveTrait: .italicFontMask) }
    return (f, family)
}

struct CanvasRep: NSViewRepresentable {
    let id: String
    let ops: [Op]

    func makeNSView(context: Context) -> CanvasView {
        let v = CanvasView()
        v.canvasId = id
        v.ops = ops
        return v
    }

    func updateNSView(_ v: CanvasView, context: Context) {
        v.canvasId = id
        v.ops = ops
        v.needsDisplay = true
    }
}

final class CanvasView: NSView {
    /// Canvases by id, so a `popup` (std.app.18) finds the view to show its menu in.
    static var registry: [String: CanvasView] = [:]
    var canvasId = "" { didSet { CanvasView.registry[canvasId] = self } }
    private var lastMove = Date.distantPast
    var ops: [Op] = []
    private var reported = CGSize.zero

    override var isFlipped: Bool { true }
    override var acceptsFirstResponder: Bool { true }
    /// In edge mode the canvas lies under the title bar; only the app's `drag` line moves the window (std.app.25).
    override var mouseDownCanMoveWindow: Bool { false }

    override func setFrameSize(_ size: NSSize) {
        super.setFrameSize(size)
        if size != reported {
            reported = size
            reportSize()
        }
    }

    /// `resize` (std.app.5) with the edge-mode insets (std.app.25).
    func reportSize() {
        let size = reported
        let cell = ("M" as NSString).size(withAttributes: [.font: cellFont])
        let lineH = ceil(cellFont.ascender - cellFont.descender + cellFont.leading)
        var f = Edge.insets(window)
        f["id"] = canvasId
        f["dx"] = size.width
        f["dy"] = size.height
        f["cell_w"] = cell.width
        f["cell_h"] = lineH
        Outbox.shared.send("resize", f)
    }

    override func draw(_ dirty: NSRect) {
        guard let g = NSGraphicsContext.current?.cgContext else { return }
        NSColor.textBackgroundColor.setFill()
        bounds.fill()
        for op in ops {
            switch op {
            case let .fill(x, y, w, h, c):
                color(c).setFill()
                NSRect(x: x, y: y, width: w, height: h).fill()
            case let .str(x, y, s, family, size, fg, bold, italic, underline):
                var attrs: [NSAttributedString.Key: Any] = [.font: font(family, size, bold: bold, italic: italic),
                                                            .foregroundColor: color(fg)]
                if underline { attrs[.underlineStyle] = NSUnderlineStyle.single.rawValue }
                (s as NSString).draw(at: NSPoint(x: x, y: y), withAttributes: attrs)
            case let .stroke(x1, y1, x2, y2, c, width):
                let p = NSBezierPath()
                p.move(to: NSPoint(x: x1, y: y1))
                p.line(to: NSPoint(x: x2, y: y2))
                p.lineWidth = CGFloat(width > 0 ? width : 1)
                color(c).setStroke()
                p.stroke()
            case let .pic(x, y, w, h, src):
                let r = NSRect(x: x, y: y, width: w, height: h)
                if let img = NSImage(contentsOfFile: src) ?? URL(string: src).flatMap({ NSImage(contentsOf: $0) }) {
                    img.draw(in: r, from: .zero, operation: .sourceOver, fraction: 1, respectFlipped: true, hints: nil)
                } else {
                    NSColor.quaternaryLabelColor.setFill()
                    r.fill()
                }
            case let .bitmap(x, y, w, h, id):
                let r = NSRect(x: x, y: y, width: w, height: h)
                if let img = Bitmaps.image(id) {
                    img.draw(in: r, from: .zero, operation: .sourceOver, fraction: 1, respectFlipped: true, hints: nil)
                } else {
                    NSColor.quaternaryLabelColor.setFill()
                    r.fill()
                }
            case let .shade(x, y, w, h, c, a):
                // std.app.19: a fill with opacity `a`, clamped to 0...1.
                color(c).withAlphaComponent(CGFloat(min(max(a, 0), 1))).setFill()
                NSRect(x: x, y: y, width: w, height: h).fill(using: .sourceOver)
            case let .round(x, y, w, h, r, c, a, stroke):
                // std.app.32: a rounded rectangle; the radius is at most half the shorter side.
                let rr = CGFloat(min(max(r, 0), min(w, h) / 2))
                let p = NSBezierPath(roundedRect: NSRect(x: x, y: y, width: w, height: h), xRadius: rr, yRadius: rr)
                let tint = color(c).withAlphaComponent(CGFloat(min(max(a, 0), 1)))
                if stroke > 0 {
                    p.lineWidth = CGFloat(stroke)
                    tint.setStroke()
                    p.stroke()
                } else {
                    tint.setFill()
                    p.fill()
                }
            case let .poly(pts, c, a):
                if pts.count >= 6 {
                    let p = NSBezierPath()
                    p.windingRule = .nonZero
                    p.move(to: NSPoint(x: pts[0], y: pts[1]))
                    for k in 1..<(pts.count / 2) { p.line(to: NSPoint(x: pts[k * 2], y: pts[k * 2 + 1])) }
                    p.close()
                    color(c).withAlphaComponent(CGFloat(min(max(a, 0), 1))).setFill()
                    p.fill()
                }
            case let .line(pts, c, width):
                if pts.count >= 4 {
                    let p = NSBezierPath()
                    p.move(to: NSPoint(x: pts[0], y: pts[1]))
                    for k in 1..<(pts.count / 2) { p.line(to: NSPoint(x: pts[k * 2], y: pts[k * 2 + 1])) }
                    p.lineWidth = CGFloat(width > 0 ? width : 1)
                    p.lineCapStyle = .round
                    p.lineJoinStyle = .round
                    color(c).setStroke()
                    p.stroke()
                }
            case let .clip(x, y, w, h):
                g.saveGState()
                g.clip(to: CGRect(x: x, y: y, width: w, height: h))
            case .unclip:
                g.restoreGState()
            case .font:
                break
            }
        }
        Surfaces.draw(canvasId, in: g, bounds: bounds)
    }

    private func mods(_ e: NSEvent) -> [String: Any] {
        let f = e.modifierFlags
        return ["ctrl": f.contains(.control), "alt": f.contains(.option), "shift": f.contains(.shift), "meta": f.contains(.command)]
    }

    override func mouseDown(with e: NSEvent) {
        window?.makeFirstResponder(self)
        Edge.lastEvent = e
        // A drag moves the window only once the app asks (std.app.25).
        if Edge.applied { window?.isMovable = false }
        press(e, "left")
    }
    override func rightMouseDown(with e: NSEvent) { press(e, "right") }
    override func otherMouseDown(with e: NSEvent) { press(e, "other") }

    // std.app.27 (ADR 0983): `down` before `click`, `up` on release, `move` while held.
    // AppKit delivers the drag and the release to the view that got the press.
    private func press(_ e: NSEvent, _ button: String) {
        Outbox.shared.send("down", pointer(e, button))
        Outbox.shared.send("click", pointer(e, button))
    }
    override func mouseUp(with e: NSEvent) { Outbox.shared.send("up", pointer(e, "left")) }
    override func rightMouseUp(with e: NSEvent) { Outbox.shared.send("up", pointer(e, "right")) }
    override func otherMouseUp(with e: NSEvent) { Outbox.shared.send("up", pointer(e, "other")) }
    override func mouseDragged(with e: NSEvent) { dragged(e) }
    override func rightMouseDragged(with e: NSEvent) { dragged(e) }
    override func otherMouseDragged(with e: NSEvent) { dragged(e) }

    private func dragged(_ e: NSEvent) {
        guard Date().timeIntervalSince(lastMove) >= 0.016 else { return }
        lastMove = Date()
        let p = convert(e.locationInWindow, from: nil)
        Outbox.shared.send("move", ["id": canvasId, "x": p.x, "y": p.y])
    }

    private func pointer(_ e: NSEvent, _ button: String) -> [String: Any] {
        let p = convert(e.locationInWindow, from: nil)
        var f = mods(e)
        f["id"] = canvasId
        f["x"] = p.x
        f["y"] = p.y
        f["text"] = button
        f["clicks"] = min(max(e.clickCount, 1), 3)
        return f
    }

    // std.app.16: pointer position, at most once per 16 ms, only while the program asks.
    override func updateTrackingAreas() {
        super.updateTrackingAreas()
        for t in trackingAreas { removeTrackingArea(t) }
        addTrackingArea(NSTrackingArea(rect: bounds, options: [.mouseMoved, .mouseEnteredAndExited, .activeInKeyWindow, .inVisibleRect],
                                       owner: self, userInfo: nil))
    }

    override func mouseMoved(with e: NSEvent) {
        guard Hover.enabled, Date().timeIntervalSince(lastMove) >= 0.016 else { return }
        lastMove = Date()
        let p = convert(e.locationInWindow, from: nil)
        Outbox.shared.send("move", ["id": canvasId, "x": p.x, "y": p.y])
    }

    override func mouseExited(with e: NSEvent) {
        if Hover.enabled { Outbox.shared.send("leave", ["id": canvasId]) }
    }

    override func scrollWheel(with e: NSEvent) {
        // Points, positive when the content should move up (scrolling down).
        let k: CGFloat = e.hasPreciseScrollingDeltas ? 1 : 16
        var f = mods(e)
        f["id"] = canvasId
        f["dx"] = -e.scrollingDeltaX * k
        f["dy"] = -e.scrollingDeltaY * k
        Outbox.shared.send("scroll", f)
    }

    override func keyDown(with e: NSEvent) {
        // std.app.23 (ADR 0978): Command-V brings the clipboard's text as `paste`.
        if e.modifierFlags.contains(.command), e.keyCode == 9,
           let s = NSPasteboard.general.string(forType: .string) {
            Outbox.shared.send("paste", ["id": canvasId, "text": s])
            return
        }
        var f = mods(e)
        f["id"] = canvasId
        f["text"] = keyName(e)
        if (f["text"] as? String ?? "").isEmpty { return }
        f["repeat"] = e.isARepeat
        f["code"] = e.keyCode
        Outbox.shared.send("key", f)
    }

    // std.app.20 (ADR 0956): key release, with the same name and hardware code.
    override func keyUp(with e: NSEvent) {
        var f = mods(e)
        f["id"] = canvasId
        f["text"] = keyName(e)
        if (f["text"] as? String ?? "").isEmpty { return }
        f["code"] = e.keyCode
        Outbox.shared.send("keyup", f)
    }

    /// The key as std.ui names it (std.ui.7): a name for special keys, else
    /// the typed character, unshifted while Control or Command is held.
    private func keyName(_ e: NSEvent) -> String {
        switch e.keyCode {
        case 36, 76: return "enter"
        case 48: return "tab"
        case 51: return "backspace"
        case 117: return "delete"
        case 53: return "escape"
        case 126: return "up"
        case 125: return "down"
        case 123: return "left"
        case 124: return "right"
        case 115: return "home"
        case 119: return "end"
        case 116: return "pageup"
        case 121: return "pagedown"
        case 122: return "f1"
        case 120: return "f2"
        case 99: return "f3"
        case 118: return "f4"
        case 96: return "f5"
        case 97: return "f6"
        case 98: return "f7"
        case 100: return "f8"
        case 101: return "f9"
        case 109: return "f10"
        case 103: return "f11"
        case 111: return "f12"
        default: break
        }
        let chord = e.modifierFlags.contains(.control) || e.modifierFlags.contains(.command)
        return (chord ? e.charactersIgnoringModifiers : e.characters) ?? ""
    }
}
