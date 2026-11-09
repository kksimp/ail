// std.gfx shared surfaces (ADR 0957): the program writes RGBA8 frames into a
// file-backed shared mapping; the host maps the same file by path and shows the
// buffer named by each `present` line, so pixels never go through JSON. The
// surface belongs to the canvas of the same `id`. Each drawn frame is answered
// by a `vsync` event with the seconds since the previous one.

import AppKit
import Foundation

final class Mapped {
    let base: UnsafeMutableRawPointer
    /// Bytes of the mapping before `base` (the program's guard, `offset` in the surface line).
    let skip: Int
    let bytes: Int
    let w: Int
    let h: Int
    let stride: Int
    let count: Int
    var current = 0
    var lastDraw = Date()

    init(base: UnsafeMutableRawPointer, skip: Int, bytes: Int, w: Int, h: Int, stride: Int, count: Int) {
        self.base = base
        self.skip = skip
        self.bytes = bytes
        self.w = w
        self.h = h
        self.stride = stride
        self.count = count
    }

    deinit { munmap(base - skip, bytes) }

    /// The buffer `current` as an image that reads the shared bytes in place.
    func image() -> CGImage? {
        let offset = current * stride * h
        guard let provider = CGDataProvider(dataInfo: nil, data: base + offset, size: stride * h, releaseData: { _, _, _ in })
        else { return nil }
        return CGImage(width: w, height: h, bitsPerComponent: 8, bitsPerPixel: 32, bytesPerRow: stride,
                       space: CGColorSpaceCreateDeviceRGB(),
                       bitmapInfo: CGBitmapInfo(rawValue: CGImageAlphaInfo.noneSkipLast.rawValue),
                       provider: provider, decode: nil, shouldInterpolate: false, intent: .defaultIntent)
    }
}

private func number(_ d: [String: Any], _ k: String) -> Double { (d[k] as? NSNumber)?.doubleValue ?? 0 }
private func str(_ d: [String: Any], _ k: String) -> String { d[k] as? String ?? "" }

enum Surfaces {
    static var maps: [String: Mapped] = [:]
    /// The text items of each surface's canvas (std.gfx.7), drawn over its pixels.
    static var texts: [String: [[String: Any]]] = [:]

    /// `{"t":"text","id":I,"items":[{"x","y","s","size","color"}]}`: replaces the surface's text.
    static func setText(_ obj: [String: Any]) {
        let id = str(obj, "id")
        let items = obj["items"] as? [[String: Any]] ?? []
        // The reader thread parses; the dictionary is only touched on the main thread, where it is drawn.
        DispatchQueue.main.async {
            texts[id] = items
            CanvasView.registry[id]?.needsDisplay = true
        }
    }

    /// `{"t":"surface","id":I,"w":W,"h":H,"stride":S,"count":N,"path":P}` (std.gfx.8): map the file.
    /// Returns false if it can't be mapped; the program then sees no `vsync` and fails with `no_host`.
    static func attach(_ obj: [String: Any]) -> Bool {
        let id = str(obj, "id")
        let w = Int(number(obj, "w")), h = Int(number(obj, "h"))
        let stride = Int(number(obj, "stride")), count = max(Int(number(obj, "count")), 1)
        let path = str(obj, "path")
        let offset = Int(number(obj, "offset"))
        let bytes = stride * h * count + offset
        guard w > 0, h > 0, stride >= w * 4, offset >= 0, bytes > 0 else { return false }
        let fd = open(path, O_RDWR)
        if fd < 0 { return false }
        defer { close(fd) }
        let p = mmap(nil, bytes, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0)
        guard let base = p, base != MAP_FAILED else { return false }
        maps[id] = Mapped(base: base + offset, skip: offset, bytes: bytes, w: w, h: h, stride: stride, count: count)
        // A game reads the keyboard at once: the canvas takes the focus, and the
        // window is sized so the canvas is exactly the surface, one point per pixel
        // (std.gfx.6).
        DispatchQueue.main.async {
            guard let v = CanvasView.registry[id], let win = v.window else { return }
            win.makeFirstResponder(v)
            fitWindow(win, to: v, width: CGFloat(w), height: CGFloat(h))
            report(id, v, w, h)
        }
        return true
    }

    /// `{"t":"present","id":I,"buf":K}`: show buffer K at the next draw.
    static func present(_ obj: [String: Any]) {
        let id = str(obj, "id")
        guard let m = maps[id] else { return }
        m.current = min(max(Int(number(obj, "buf")), 0), m.count - 1)
        DispatchQueue.main.async { CanvasView.registry[id]?.needsDisplay = true }
    }

    /// Draws the surface of canvas `id` over `bounds` and reports the vsync.
    static func draw(_ id: String, in g: CGContext, bounds: CGRect) {
        guard let m = maps[id], let img = m.image() else { return }
        g.saveGState()
        // The canvas is flipped (y down); an image is drawn y up.
        g.translateBy(x: 0, y: bounds.height)
        g.scaleBy(x: 1, y: -1)
        g.interpolationQuality = .none
        g.draw(img, in: bounds)
        g.restoreGState()
        // Text positions are in surface pixels; the canvas may scale them.
        let sx = bounds.width / CGFloat(m.w)
        for t in texts[id] ?? [] {
            let size = CGFloat(number(t, "size") > 0 ? number(t, "size") : 12) * sx
            let attrs: [NSAttributedString.Key: Any] = [
                .font: NSFont.monospacedSystemFont(ofSize: size, weight: .regular),
                .foregroundColor: color(Int(number(t, "color"))),
            ]
            (str(t, "s") as NSString).draw(at: NSPoint(x: CGFloat(number(t, "x")) * sx, y: CGFloat(number(t, "y")) * sx),
                                           withAttributes: attrs)
        }
        Pads.poll()
        let now = Date()
        let dt = now.timeIntervalSince(m.lastDraw)
        m.lastDraw = now
        Outbox.shared.send("vsync", ["id": id, "dx": dt, "text": String(m.current)])
    }
}

/// Resizes `win` by the difference between the canvas and the size the surface wants,
/// so the controls and padding around the canvas keep their size.
private func fitWindow(_ win: NSWindow, to v: NSView, width: CGFloat, height: CGFloat) {
    guard let content = win.contentView else { return }
    content.layoutSubtreeIfNeeded()
    let dx = width - v.bounds.width, dy = height - v.bounds.height
    if abs(dx) < 0.5 && abs(dy) < 0.5 { return }
    win.setContentSize(NSSize(width: content.bounds.width + dx, height: content.bounds.height + dy))
    content.layoutSubtreeIfNeeded()
}

/// With `AIL_GFX_REPORT=FILE`, appends one line to FILE once the window has settled:
/// the surface size, the canvas size in points, the backing scale and the canvas in
/// pixels. tools/app-host/macos/gfx-size-check.sh reads it.
private func report(_ id: String, _ v: NSView, _ w: Int, _ h: Int) {
    guard let path = ProcessInfo.processInfo.environment["AIL_GFX_REPORT"], !path.isEmpty else { return }
    DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
        let scale = v.window?.backingScaleFactor ?? 1
        let line = String(format: "gfx-surface id=%@ surface=%dx%d canvas=%dx%d scale=%g pixels=%dx%d\n", id, w, h,
                          Int(v.bounds.width.rounded()), Int(v.bounds.height.rounded()), Double(scale),
                          Int((v.bounds.width * scale).rounded()), Int((v.bounds.height * scale).rounded()))
        if let data = line.data(using: .utf8) {
            if let fh = FileHandle(forWritingAtPath: path) {
                fh.seekToEndOfFile()
                fh.write(data)
                fh.closeFile()
            } else {
                FileManager.default.createFile(atPath: path, contents: data)
            }
        }
    }
}
