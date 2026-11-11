# 0993. std.audio.device: streaming output with a played-frames clock

- **Status:** Accepted
- **Date:** 2026-11-08
- **Decided by:** AI (delegated, ADR 0076)
- **Topic:** 17 — Standard library
- **Spec rules:** [std.audio.12](../spec/std-audio.md)
- **Amends:** [ADR 0949](0949-std-audio.md)

## Context

`play` takes a whole clip and blocks. Video players and games need to feed
audio continuously as it decodes, to know how much has actually played so
video frames can be timed to it, and to pause and seek.

## Decision

1. `stream_open(rate, channels)`, `stream_write(clip)`, `stream_status`,
   `stream_pause`, `stream_resume`, `stream_flush` and `stream_close` in
   `std.audio.device`, on the same host program and line protocol (`open_stream`,
   `write`, `clock`, `pause`, `resume`, `flush`, `close_stream`).
2. `StreamStatus { played, buffered }` is the answer to every call. `played` is
   the frames the device has finished, the clock; it is accurate to one written
   chunk. `flush` drops the queue and zeroes both counts, for a seek.
3. `stream_write` is non-blocking and the host's queue is unbounded. The caller
   keeps `buffered` near 200 ms by reading the status the write returns.
4. All three hosts implement the protocol: AVAudioEngine (macOS), ALSA with a
   writer thread (Linux; `played` is frames handed to ALSA minus `snd_pcm_delay`)
   and WASAPI with a writer thread (Windows; frames written minus the buffer's
   padding). Each accepts `--null` (or `AIL_AUDIO_NULL=1`): a silent device whose
   clock runs in real time, so headless tests need no sound card. `open_null()`
   in `std.audio.device` starts the host that way.
5. PCM stays base64 text: 48 kHz stereo 16-bit is about 256 KB/s, which a pipe
   carries easily. A binary channel would need a new transport for every host.

## Rejected alternatives

- A blocking write with a bounded queue: hides back-pressure and stalls the decoder thread.
- A sample-exact clock from the render callback: needs a real-time callback in the host; one chunk is enough for lip sync.
- Shared memory or a binary pipe: more host code for no measured need.
