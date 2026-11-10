// Gamepads for std.gfx (std.gfx.11, ADR 0985): GameController.framework is polled
// with each drawn frame, and a `pad` event goes to the program when a pad's state
// changes. `text` is `buttons,lx,ly,rx,ry,lt,rt`; `buttons` has the bits of
// `pad_bit` (a 1, b 2, x 4, y 8, lb 16, rb 32, back 64, start 128, l3 256, r3 512,
// up 1024, down 2048, left 4096, right 8192). Sticks are -1...1 with right and up
// positive, triggers 0...1.

import Foundation
import GameController

enum Pads {
    private static var last: [String] = []

    static func poll() {
        let pads = GCController.controllers().compactMap { $0.extendedGamepad }
        for (i, g) in pads.enumerated() {
            let bits: [(Bool, Int)] = [
                (g.buttonA.isPressed, 1), (g.buttonB.isPressed, 2), (g.buttonX.isPressed, 4), (g.buttonY.isPressed, 8),
                (g.leftShoulder.isPressed, 16), (g.rightShoulder.isPressed, 32),
                (g.buttonOptions?.isPressed ?? false, 64), (g.buttonMenu.isPressed, 128),
                (g.leftThumbstickButton?.isPressed ?? false, 256), (g.rightThumbstickButton?.isPressed ?? false, 512),
                (g.dpad.up.isPressed, 1024), (g.dpad.down.isPressed, 2048),
                (g.dpad.left.isPressed, 4096), (g.dpad.right.isPressed, 8192),
            ]
            let buttons = bits.reduce(0) { $0 | ($1.0 ? $1.1 : 0) }
            let state = String(format: "%d,%.3f,%.3f,%.3f,%.3f,%.3f,%.3f", buttons,
                               g.leftThumbstick.xAxis.value, g.leftThumbstick.yAxis.value,
                               g.rightThumbstick.xAxis.value, g.rightThumbstick.yAxis.value,
                               g.leftTrigger.value, g.rightTrigger.value)
            if i >= last.count {
                last.append(state)
            } else if last[i] == state {
                continue
            } else {
                last[i] = state
            }
            Outbox.shared.send("pad", ["id": String(i), "text": state])
        }
        // A pad that went away: say so once.
        while last.count > pads.count {
            let i = last.count - 1
            last.removeLast()
            Outbox.shared.send("pad", ["id": String(i), "text": ""])
        }
    }
}
