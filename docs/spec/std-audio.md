# Standard library: `std.audio` sound

Chapter key: `std`

> **Status:** Draft (ADR 0949). Part 1 (clips,
> mixing, WAV) is written in AIL in `std/audio/audio.ail` and tested by
> `tests/cases/std_audio.ail`. Part 2 (devices) is built for macOS.

**std.audio.1** — `use std.audio` imports a standard module written in AIL.
A `Clip` has `rate` (Hz), `channels` and `samples`: interleaved floats in
`-1.0..1.0`. Sound is plain data, so an agent can build, inspect and test it
without a device.

**std.audio.2** — `frames(c)` is `samples.len() / channels`; `duration_ms(c)`
is `frames * 1000 / rate`.

**std.audio.3** — `silence(rate, channels, ms)` and `tone(freq, ms, rate, amp)`
(mono sine) make clips.

**std.audio.4** — `gain(c, g)` scales; `peak(c)` is the largest absolute sample.

**std.audio.5** — `concat(a, b)` and `mix(a, b)` need equal rate and channels,
else `AudioError`. `mix` sums, clamps to `-1.0..1.0`, and lasts as long as the longer clip.

**std.audio.6** — `to_mono(c)` averages channels; `resample(c, rate)` is linear.

**std.audio.7** — `decode_wav(b bytes) Clip ! AudioError` reads RIFF/WAVE 16-bit
PCM; other chunks are skipped; 32-bit float is refused for now.
`encode_wav(c) bytes` writes 16-bit PCM, clamping.

**std.audio.8** — `to_pcm16(c) bytes` is the clip as raw little-endian 16-bit
PCM, interleaved, clamped; `from_pcm16(b, rate, channels) Clip` is the reverse.

**std.audio.11** — `decode_flac(b bytes) Clip ! AudioError` reads a FLAC file
of 8 to 24 bits per sample: constant, verbatim, fixed and LPC subframes,
rice residuals, and the stereo modes (left/side, side/right, mid/side). The
CRCs and the MD5 are not checked. Other data fails with `AudioError`. *(ADR 0974)*

## Devices

**std.audio.9** — `use std.audio.device` plays and records through a host
program that owns the platform's audio API, as `std.app` does (ADR 0838), so no
AIL code runs on an audio callback. `open() proc.Child ! DeviceError` (or `open_at(cmd)` with a given program) starts the
host (`AIL_AUDIO_HOST`, else the one built under `AIL_ROOT` by
`tools/audio-host/macos/run.sh`, `tools/audio-host/linux/run.sh` or
`tools/audio-host/windows/run.ps1`) and waits for its `ready` line;
`play(mut d, clip)` blocks until the clip has played; `close(move d)` stops the
host. `DeviceError` has `kind` (`no_host`, `start`, `version`, `host_exited`,
`device`, `protocol`) and `message`. The host speaks one text line per message:
`play RATE CH BASE64`, `rec MS RATE CH`, then `done`, `data RATE CH BASE64` or
`error TEXT`. Built: macOS (AVAudioEngine on CoreAudio), Linux (ALSA, `tools/audio-host/linux/host.c`; `AIL_ALSA_DEVICE` picks the device) and Windows (WASAPI in shared mode, with the engine's sample-rate and channel conversion, so any clip format plays). Where no host is built `open` fails with kind `no_host`
and a message naming this rule; it never crashes.

**std.audio.10** — `record(mut d, ms, rate, channels) Clip ! DeviceError` records
from the default input. The clip has the rate and channels the device gave,
which may differ from those asked for; `resample` and `to_mono` adapt it.

**Planned (no rule ID yet):** a PipeWire host; streaming output
(`open_output` with `write`) with a ring buffer in the host and under 20 ms
latency; mixing several plays in a task off the audio thread.

## Rejected alternatives

- Callbacks from the audio thread into AIL: break `ffi.decl.3` and risk GC pauses in real time.
- Linking SDL, miniaudio or PortAudio: custom tooling by default (ADR 0009).
- Integer sample clips: float keeps mixing and DSP simple; WAV encode converts.

## Examples

```ail rules=std.audio.1,std.audio.2,std.audio.3,std.audio.4,std.audio.5,std.audio.6,std.audio.7
use std.audio

fn main() !
    t := audio.tone(440.0, 100, 8000, 0.5)
    print(audio.frames(t))
    print(audio.duration_ms(t))
    print(audio.peak(t) > 0.49 and audio.peak(t) <= 0.5)
    s := audio.silence(8000, 1, 50)
    print(audio.frames(s))
    m := audio.mix(t, s)?
    print(audio.frames(m))
    print(audio.frames(audio.concat(t, s)?))
    print(audio.frames(audio.resample(t, 16000)))
    st := audio.Clip(rate: 8000, channels: 2, samples: [0.5, -0.5, 1.0, 0.0])
    print(audio.to_mono(st).samples)
    w := audio.encode_wav(t)
    print(w.len())
    back := audio.decode_wav(w)?
    print(back.rate)
    print(back.channels)
    print(audio.frames(back))
    d := audio.peak(back) - audio.peak(t)
    print(d < 0.001 and d > -0.001)
    bad := audio.decode_wav(b"nope") else err
        print("error: {err}")
        t
    print(audio.frames(bad))
    odd := audio.concat(t, st) else err
        print("error: {err}")
        t
    print(audio.frames(odd))
```

```ail norun rules=std.audio.8,std.audio.9,std.audio.10
use std.audio
use std.audio.device

fn main() !
    mut d := device.open()?
    device.play(mut d, audio.tone(440.0, 500, 44100, 0.3))?
    heard := device.record(mut d, 1000, 44100, 1)?
    print(audio.peak(heard))
    pcm := audio.to_pcm16(heard)
    print(audio.frames(audio.from_pcm16(pcm, heard.rate, heard.channels)) == audio.frames(heard))
    device.close(move d)
```

```ail rules=std.audio.8,std.audio.9
use std.audio
use std.audio.device

fn main() !
    t := audio.tone(440.0, 20, 8000, 0.5)
    print(audio.to_pcm16(t).len())
    c := device.open_at("/nonexistent/ail-audio-host") else err
        print(err.kind)
        return
    device.close(move c)
```

```ail rules=std.audio.11
use std.audio

fn main() !
    c := audio.decode_flac(b"RIFF") else err
        print("error: {err}")
        return
    print(audio.frames(c))
```

## Open for Fable

Vorbis and MP3 decoders (FLAC is done, ADR 0974); spatial audio; f32 sample storage for tokens and speed
once `List[f32]` is native-fast.
