// std.gfx shared surfaces (ADR 0957): the program writes RGBA8 frames into a
// file-backed shared mapping; the host maps the same file by path and shows the
// buffer named by each `present` line, so pixels never go through JSON. The
// surface belongs to the canvas of the same `id`. Each drawn frame is answered
// by a `vsync` event with the seconds since the previous one.

import AppKit
import Foundation

final class Mapped {
    let base: UnsafeMutableRawPointer
    let bytes: Int
    let w: Int
    let h: Int
    let stride: Int
    let count: Int
    var current = 0
    var lastDraw = Date()

    init(base: UnsafeMutableRawPointer, bytes: Int, w: Int, h: Int, stride: Int, count: Int) {
        self.base = base
        self.bytes = bytes
        self.w = w
        self.h = h
        self.stride = stride
        self.count = count
    }

    deinit { munmap(base, bytes) }

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

    /// `{"t":"surface","id":I,"w":W,"h":H,"stride":S,"count":N,"path":P}` (std.gfx.8): map the file.
    /// Returns false if it can't be mapped; the program then sees no `vsync` and fails with `no_host`.
    static func attach(_ obj: [String: Any]) -> Bool {
        let id = str(obj, "id")
        let w = Int(number(obj, "w")), h = Int(number(obj, "h"))
        let stride = Int(number(obj, "stride")), count = max(Int(number(obj, "count")), 1)
        let path = str(obj, "path")
        let bytes = stride * h * count
        guard w > 0, h > 0, stride >= w * 4, bytes > 0 else { return false }
        let fd = open(path, O_RDWR)
        if fd < 0 { return false }
        defer { close(fd) }
        let p = mmap(nil, bytes, PROT_READ | PROT_WRITE, MAP_SHARED, fd, 0)
        guard let base = p, base != MAP_FAILED else { return false }
        maps[id] = Mapped(base: base, bytes: bytes, w: w, h: h, stride: stride, count: count)
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
        let now = Date()
        let dt = now.timeIntervalSince(m.lastDraw)
        m.lastDraw = now
        Outbox.shared.send("vsync", ["id": id, "dx": dt, "text": String(m.current)])
    }
}
