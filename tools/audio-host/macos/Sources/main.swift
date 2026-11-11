// The std.audio host for macOS (spec std.audio.9-10, ADR 0949): plays and
// records through AVAudioEngine, which sits on CoreAudio. One text line in,
// one line out: `play RATE CH BASE64` -> `done`, `rec MS RATE CH` ->
// `data RATE CH BASE64`; streaming (std.audio.12): `open_stream RATE CH` -> `ok`,
// `write BASE64`, `clock`, `pause`, `resume`, `flush` -> `played N buffered M`,
// `close_stream` -> `ok`. Audio is little-endian 16-bit PCM, interleaved.

import AVFoundation
import Foundation

setvbuf(stdout, nil, _IOLBF, 0)

func reply(_ s: String) { print(s) }

func makeBuffer(fmt: AVAudioFormat, ch: Int, data: Data) -> AVAudioPCMBuffer? {
    let frames = data.count / (2 * ch)
    guard frames > 0, let buf = AVAudioPCMBuffer(pcmFormat: fmt, frameCapacity: AVAudioFrameCount(frames)) else { return nil }
    buf.frameLength = AVAudioFrameCount(frames)
    data.withUnsafeBytes { (raw: UnsafeRawBufferPointer) in
        let s = raw.bindMemory(to: Int16.self)
        for c in 0..<ch {
            let dst = buf.floatChannelData![c]
            for i in 0..<frames { dst[i] = Float(Int16(littleEndian: s[i * ch + c])) / 32768.0 }
        }
    }
    return buf
}

func play(rate: Double, ch: Int, data: Data) {
    guard let fmt = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: rate, channels: AVAudioChannelCount(ch), interleaved: false) else {
        return reply("error bad format")
    }
    guard let buf = makeBuffer(fmt: fmt, ch: ch, data: data) else { return reply("done") }
    let engine = AVAudioEngine()
    let node = AVAudioPlayerNode()
    engine.attach(node)
    engine.connect(node, to: engine.mainMixerNode, format: fmt)
    do { try engine.start() } catch { return reply("error \(error.localizedDescription)") }
    let done = DispatchSemaphore(value: 0)
    node.scheduleBuffer(buf, at: nil, options: [], completionCallbackType: .dataPlayedBack) { _ in done.signal() }
    node.play()
    done.wait()
    node.stop()
    engine.stop()
    reply("done")
}

func record(ms: Int, rate: Double, ch: Int) {
    let engine = AVAudioEngine()
    let input = engine.inputNode
    let fmt = input.outputFormat(forBus: 0)
    guard fmt.sampleRate > 0, fmt.channelCount > 0 else { return reply("error no input device") }
    let n = Int(fmt.channelCount)
    var pcm = Data()
    let lock = NSLock()
    input.installTap(onBus: 0, bufferSize: 1024, format: fmt) { buf, _ in
        guard let cd = buf.floatChannelData else { return }
        var chunk = Data()
        for i in 0..<Int(buf.frameLength) {
            for c in 0..<n {
                let v = Int16(max(-1, min(1, cd[c][i])) * 32767).littleEndian
                withUnsafeBytes(of: v) { chunk.append(contentsOf: $0) }
            }
        }
        lock.lock(); pcm.append(chunk); lock.unlock()
    }
    do { try engine.start() } catch { return reply("error \(error.localizedDescription)") }
    Thread.sleep(forTimeInterval: Double(ms) / 1000.0)
    input.removeTap(onBus: 0)
    engine.stop()
    lock.lock(); let out = pcm; lock.unlock()
    reply("data \(Int(fmt.sampleRate)) \(n) \(out.base64EncodedString())")
}


/// A running output stream (spec std.audio.12): chunks queue on one player node;
/// `played` counts frames the device has finished, `scheduled` frames queued.
/// With `--null` (or AIL_AUDIO_NULL=1) there is no device: the clock runs in real
/// time, so a test needs no sound hardware.
let nullMode: Bool = {
    if CommandLine.arguments.contains("--null") { return true }
    if let v = ProcessInfo.processInfo.environment["AIL_AUDIO_NULL"], !v.isEmpty, v != "0" { return true }
    return false
}()

final class OutStream {
    let engine = AVAudioEngine()
    let node = AVAudioPlayerNode()
    let fmt: AVAudioFormat
    let ch: Int
    let lock = NSLock()
    var scheduled = 0
    var played = 0
    var generation = 0
    let silent = nullMode
    let rate: Double
    var paused = false
    var nplayed = 0.0
    var last = Date()

    init?(rate: Double, ch: Int) {
        guard let f = AVAudioFormat(commonFormat: .pcmFormatFloat32, sampleRate: rate, channels: AVAudioChannelCount(ch), interleaved: false) else { return nil }
        fmt = f
        self.ch = ch
        self.rate = rate
        if silent { return }
        engine.attach(node)
        engine.connect(node, to: engine.mainMixerNode, format: f)
        do { try engine.start() } catch { return nil }
        node.play()
    }

    func write(_ data: Data) {
        guard let buf = makeBuffer(fmt: fmt, ch: ch, data: data) else { return }
        let frames = Int(buf.frameLength)
        lock.lock(); scheduled += frames; let gen = generation; lock.unlock()
        if silent { return }
        node.scheduleBuffer(buf, at: nil, options: [], completionCallbackType: .dataPlayedBack) { [self] _ in
            lock.lock()
            if gen == generation { played += frames }
            lock.unlock()
        }
    }

    /// Drops everything queued and zeroes the clock (for a seek).
    func flush() {
        if !silent { node.stop() }
        lock.lock(); generation += 1; scheduled = 0; played = 0; nplayed = 0; lock.unlock()
        if !silent && !paused { node.play() }
    }

    /// Advances the silent clock to now. Caller holds `lock`.
    func tick() {
        let now = Date()
        if !paused { nplayed += now.timeIntervalSince(last) * rate }
        last = now
        nplayed = min(nplayed, Double(scheduled))
    }

    func pause() { lock.lock(); if silent { tick() }; paused = true; lock.unlock(); if !silent { node.pause() } }
    func resume() { lock.lock(); if silent { tick() }; paused = false; lock.unlock(); if !silent { node.play() } }

    func status() -> String {
        lock.lock(); defer { lock.unlock() }
        if silent {
            tick()
            played = Int(nplayed)
        }
        return "played \(played) buffered \(scheduled - played)"
    }

    func close() { if !silent { node.stop(); engine.stop() } }
}

var stream: OutStream? = nil

reply("ready")
while let line = readLine() {
    let p = line.split(separator: " ", omittingEmptySubsequences: true).map(String.init)
    switch p.first ?? "" {
    case "play" where p.count >= 3:
        let data = p.count > 3 ? (Data(base64Encoded: p[3]) ?? Data()) : Data()
        play(rate: Double(p[1]) ?? 44100, ch: max(1, Int(p[2]) ?? 1), data: data)
    case "rec" where p.count >= 4:
        record(ms: Int(p[1]) ?? 0, rate: Double(p[2]) ?? 44100, ch: Int(p[3]) ?? 1)
    case "open_stream" where p.count >= 3:
        stream?.close()
        stream = OutStream(rate: Double(p[1]) ?? 44100, ch: max(1, Int(p[2]) ?? 1))
        reply(stream == nil ? "error bad format" : "ok")
    case "write" where stream != nil:
        if p.count > 1 { stream!.write(Data(base64Encoded: p[1]) ?? Data()) }
        reply(stream!.status())
    case "clock" where stream != nil:
        reply(stream!.status())
    case "pause" where stream != nil:
        stream!.pause()
        reply(stream!.status())
    case "resume" where stream != nil:
        stream!.resume()
        reply(stream!.status())
    case "flush" where stream != nil:
        stream!.flush()
        reply(stream!.status())
    case "close_stream" where stream != nil:
        stream!.close()
        stream = nil
        reply("ok")
    case "write", "clock", "pause", "resume", "flush", "close_stream":
        reply("error no stream open")
    default:
        reply("error unknown request")
    }
}
