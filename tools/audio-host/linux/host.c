/* The std.audio host for Linux (spec std.audio.9-12, ADR 0949): ALSA, loaded at
 * run time with dlopen so no headers or link step are needed. One text line in,
 * one line out, as the macOS host: `play RATE CH B64` -> `done`,
 * `rec MS RATE CH` -> `data RATE CH B64`, and the stream requests
 * `open_stream RATE CH`, `write B64`, `clock`, `pause`, `resume`, `flush`,
 * `close_stream`. PipeWire and PulseAudio systems expose the "default" device. */
#define _GNU_SOURCE
#include <dlfcn.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef void snd_pcm_t;
typedef long snd_pcm_sframes_t;
typedef unsigned long snd_pcm_uframes_t;
static int (*p_open)(snd_pcm_t **, const char *, int, int);
static int (*p_set_params)(snd_pcm_t *, int, int, unsigned, unsigned, int, unsigned);
static snd_pcm_sframes_t (*p_writei)(snd_pcm_t *, const void *, snd_pcm_uframes_t);
static snd_pcm_sframes_t (*p_readi)(snd_pcm_t *, void *, snd_pcm_uframes_t);
static int (*p_drain)(snd_pcm_t *);
static int (*p_drop)(snd_pcm_t *);
static int (*p_prepare)(snd_pcm_t *);
static int (*p_close)(snd_pcm_t *);
static int (*p_recover)(snd_pcm_t *, int, int);
static int (*p_delay)(snd_pcm_t *, snd_pcm_sframes_t *);
static int (*p_pause)(snd_pcm_t *, int);

#define PLAYBACK 0
#define CAPTURE 1
#define FORMAT_S16_LE 2
#define ACCESS_RW_INTERLEAVED 3

static int load(void) {
    void *h = dlopen("libasound.so.2", RTLD_NOW);
    if (!h) return 0;
#define SYM(v, n) if (!(*(void **)&v = dlsym(h, n))) return 0
    SYM(p_open, "snd_pcm_open"); SYM(p_set_params, "snd_pcm_set_params");
    SYM(p_writei, "snd_pcm_writei"); SYM(p_readi, "snd_pcm_readi");
    SYM(p_drain, "snd_pcm_drain"); SYM(p_drop, "snd_pcm_drop");
    SYM(p_prepare, "snd_pcm_prepare"); SYM(p_close, "snd_pcm_close");
    SYM(p_recover, "snd_pcm_recover"); SYM(p_delay, "snd_pcm_delay");
    SYM(p_pause, "snd_pcm_pause");
    return 1;
}

static const char B64[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

static uint8_t *b64_decode(const char *s, size_t *n) {
    size_t len = strlen(s);
    uint8_t *out = malloc(len / 4 * 3 + 3);
    size_t o = 0; unsigned acc = 0; int bits = 0;
    for (size_t i = 0; i < len; i++) {
        const char *p = strchr(B64, s[i]);
        if (s[i] == '=' || !p || !s[i]) continue;
        acc = (acc << 6) | (unsigned)(p - B64); bits += 6;
        if (bits >= 8) { bits -= 8; out[o++] = (acc >> bits) & 0xff; }
    }
    *n = o;
    return out;
}

static void b64_print(const uint8_t *d, size_t n) {
    for (size_t i = 0; i < n; i += 3) {
        unsigned v = d[i] << 16;
        if (i + 1 < n) v |= d[i + 1] << 8;
        if (i + 2 < n) v |= d[i + 2];
        putchar(B64[(v >> 18) & 63]); putchar(B64[(v >> 12) & 63]);
        putchar(i + 1 < n ? B64[(v >> 6) & 63] : '=');
        putchar(i + 2 < n ? B64[v & 63] : '=');
    }
}

static int open_pcm(snd_pcm_t **h, int dir, unsigned rate, unsigned ch, unsigned latency_us) {
    if (p_open(h, "default", dir, 0) < 0) return 0;
    if (p_set_params(*h, FORMAT_S16_LE, ACCESS_RW_INTERLEAVED, ch, rate, 1, latency_us) < 0) {
        p_close(*h); return 0;
    }
    return 1;
}

static void write_all(snd_pcm_t *h, const uint8_t *d, size_t frames, unsigned ch) {
    while (frames > 0) {
        snd_pcm_sframes_t n = p_writei(h, d, frames);
        if (n < 0) { if (p_recover(h, (int)n, 1) < 0) return; continue; }
        d += (size_t)n * 2 * ch; frames -= (size_t)n;
    }
}

static snd_pcm_t *stream; static unsigned s_ch; static long written; static int paused;

static void status(void) {
    snd_pcm_sframes_t delay = 0;
    if (stream && p_delay(stream, &delay) < 0) delay = 0;
    if (delay < 0) delay = 0;
    if (delay > written) delay = written;
    printf("played %ld buffered %ld\n", written - (long)delay, (long)delay);
}

int main(void) {
    setvbuf(stdout, NULL, _IOLBF, 0);
    int ok = load();
    puts("ready");
    char *line = NULL; size_t cap = 0;
    while (getline(&line, &cap, stdin) > 0) {
        size_t L = strlen(line);
        while (L && (line[L - 1] == '\n' || line[L - 1] == '\r')) line[--L] = 0;
        char cmd[32] = ""; sscanf(line, "%31s", cmd);
        if (!strcmp(cmd, "play") || !strcmp(cmd, "rec") || !strcmp(cmd, "open_stream")) {
            if (!ok) { puts("error std.audio.9: libasound.so.2 is not installed"); continue; }
        }
        if (!strcmp(cmd, "play")) {
            unsigned rate, ch; char *b = malloc(L + 1); b[0] = 0;
            if (sscanf(line, "play %u %u %s", &rate, &ch, b) < 2 || ch < 1) { puts("error bad request"); free(b); continue; }
            size_t n; uint8_t *d = b64_decode(b, &n);
            snd_pcm_t *h;
            if (!open_pcm(&h, PLAYBACK, rate, ch, 100000)) puts("error no output device");
            else { write_all(h, d, n / (2 * ch), ch); p_drain(h); p_close(h); puts("done"); }
            free(d); free(b);
        } else if (!strcmp(cmd, "rec")) {
            unsigned ms, rate, ch;
            if (sscanf(line, "rec %u %u %u", &ms, &rate, &ch) != 3 || ch < 1) { puts("error bad request"); continue; }
            snd_pcm_t *h;
            if (!open_pcm(&h, CAPTURE, rate, ch, 100000)) { puts("error no input device"); continue; }
            size_t frames = (size_t)rate * ms / 1000, got = 0;
            uint8_t *buf = malloc(frames * 2 * ch + 2);
            while (got < frames) {
                snd_pcm_sframes_t n = p_readi(h, buf + got * 2 * ch, frames - got);
                if (n < 0) { if (p_recover(h, (int)n, 1) < 0) break; continue; }
                got += (size_t)n;
            }
            p_close(h);
            printf("data %u %u ", rate, ch); b64_print(buf, got * 2 * ch); putchar('\n');
            free(buf);
        } else if (!strcmp(cmd, "open_stream")) {
            unsigned rate, ch;
            if (sscanf(line, "open_stream %u %u", &rate, &ch) != 2 || ch < 1) { puts("error bad request"); continue; }
            if (stream) { p_drop(stream); p_close(stream); stream = NULL; }
            if (!open_pcm(&stream, PLAYBACK, rate, ch, 200000)) { stream = NULL; puts("error no output device"); continue; }
            s_ch = ch; written = 0; paused = 0; puts("ok");
        } else if (stream && !strcmp(cmd, "write")) {
            char *b = malloc(L + 1); b[0] = 0; sscanf(line, "write %s", b);
            size_t n; uint8_t *d = b64_decode(b, &n);
            write_all(stream, d, n / (2 * s_ch), s_ch); written += (long)(n / (2 * s_ch));
            free(d); free(b); status();
        } else if (stream && !strcmp(cmd, "clock")) { status();
        } else if (stream && !strcmp(cmd, "pause")) { if (!paused && p_pause(stream, 1) >= 0) paused = 1; status();
        } else if (stream && !strcmp(cmd, "resume")) { if (paused && p_pause(stream, 0) >= 0) paused = 0; status();
        } else if (stream && !strcmp(cmd, "flush")) { p_drop(stream); p_prepare(stream); written = 0; paused = 0; status();
        } else if (stream && !strcmp(cmd, "close_stream")) { p_drain(stream); p_close(stream); stream = NULL; puts("ok");
        } else puts("error unknown request");
    }
    return 0;
}
