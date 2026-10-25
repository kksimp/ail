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

func font(_ family: String, _ size: Double, bold: Bool, italic: Bool) -> NSFont {
    let s = CGFloat(size > 0 ? size : 13)
    let w: NSFont.Weight = bold ? .bold : .regular
    var f: NSFont
    switch family {
    case "sans": f = NSFont.systemFont(ofSize: s, weight: w)
    case "serif":
        let sys = NSFont.systemFont(ofSize: s, weight: w)
        f = sys.fontDescriptor.withDesign(.serif).flatMap { NSFont(descriptor: $0, size: s) } ?? sys
    default: f = NSFont.monospacedSystemFont(ofSize: s, weight: w)
    }
    if italic { f = NSFontManager.shared.convert(f, toHaveTrait: .italicFontMask) }
    return f
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
    var canvasId = ""
    var ops: [Op] = []
    private var reported = CGSize.zero

    override var isFlipped: Bool { true }
    override var acceptsFirstResponder: Bool { true }

    override func setFrameSize(_ size: NSSize) {
        super.setFrameSize(size)
        if size != reported {
            reported = size
            let cell = ("M" as NSString).size(withAttributes: [.font: cellFont])
            let lineH = ceil(cellFont.ascender - cellFont.descender + cellFont.leading)
            Outbox.shared.send("resize", ["id": canvasId, "dx": size.width, "dy": size.height,
                                          "cell_w": cell.width, "cell_h": lineH])
        }
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
            case let .clip(x, y, w, h):
                g.saveGState()
                g.clip(to: CGRect(x: x, y: y, width: w, height: h))
            case .unclip:
                g.restoreGState()
            }
        }
    }

    private func mods(_ e: NSEvent) -> [String: Any] {
        let f = e.modifierFlags
        return ["ctrl": f.contains(.control), "alt": f.contains(.option), "shift": f.contains(.shift), "meta": f.contains(.command)]
    }

    override func mouseDown(with e: NSEvent) {
        window?.makeFirstResponder(self)
        click(e, "left")
    }
    override func rightMouseDown(with e: NSEvent) { click(e, "right") }
    override func otherMouseDown(with e: NSEvent) { click(e, "other") }

    private func click(_ e: NSEvent, _ button: String) {
        let p = convert(e.locationInWindow, from: nil)
        var f = mods(e)
        f["id"] = canvasId
        f["x"] = p.x
        f["y"] = p.y
        f["text"] = button
        Outbox.shared.send("click", f)
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
        var f = mods(e)
        f["id"] = canvasId
        f["text"] = keyName(e)
        if (f["text"] as? String ?? "").isEmpty { return }
        Outbox.shared.send("key", f)
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
