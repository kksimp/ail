// The std.audio host for Linux (spec std.audio.9-10, ADR 0949): plays and
// records through ALSA (libasound). One text line in, one line out, the same
// protocol as the macOS host: `play RATE CH BASE64` -> `done`,
// `rec MS RATE CH` -> `data RATE CH BASE64`, anything wrong -> `error TEXT`.
// Audio is little-endian 16-bit PCM, interleaved. The device is "default",
// or $AIL_ALSA_DEVICE (for example "null" on a machine with no sound card).
#include <alsa/asoundlib.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static const char *dev_name(void) {
    const char *d = getenv("AIL_ALSA_DEVICE");
    return d && *d ? d : "default";
}

static void fail(const char *what, int e) {
    printf("error %s: %s\n", what, snd_strerror(e));
    fflush(stdout);
}

static int b64val(int c) {
    if (c >= 'A' && c <= 'Z') return c - 'A';
    if (c >= 'a' && c <= 'z') return c - 'a' + 26;
    if (c >= '0' && c <= '9') return c - '0' + 52;
    if (c == '+' || c == '-') return 62;
    if (c == '/' || c == '_') return 63;
    return -1;
}

// Decodes base64 text; returns the byte count, or -1 on bad input.
static long b64decode(const char *s, uint8_t *out) {
    long n = 0;
    uint32_t acc = 0;
    int bits = 0;
    for (; *s && *s != '=' && *s != '\n' && *s != '\r'; s++) {
        int v = b64val((unsigned char)*s);
        if (v < 0) return -1;
        acc = (acc << 6) | (uint32_t)v;
        bits += 6;
        if (bits >= 8) {
            bits -= 8;
            out[n++] = (uint8_t)(acc >> bits);
        }
    }
    return n;
}

static void b64write(const uint8_t *p, size_t n) {
    static const char t[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    char buf[4096];
    size_t k = 0;
    for (size_t i = 0; i < n; i += 3) {
        uint32_t v = (uint32_t)p[i] << 16;
        if (i + 1 < n) v |= (uint32_t)p[i + 1] << 8;
        if (i + 2 < n) v |= p[i + 2];
        buf[k++] = t[v >> 18 & 63];
        buf[k++] = t[v >> 12 & 63];
        buf[k++] = i + 1 < n ? t[v >> 6 & 63] : '=';
        buf[k++] = i + 2 < n ? t[v & 63] : '=';
        if (k >= sizeof buf - 4) { fwrite(buf, 1, k, stdout); k = 0; }
    }
    fwrite(buf, 1, k, stdout);
}

// Opens the device with 16-bit interleaved samples; `*rate` and `*ch` are
// what was asked for and become what the device gave.
static snd_pcm_t *open_pcm(snd_pcm_stream_t dir, unsigned *rate, unsigned *ch, int *err_out) {
    snd_pcm_t *pcm = NULL;
    snd_pcm_hw_params_t *hw;
    int e = snd_pcm_open(&pcm, dev_name(), dir, 0);
    if (e < 0) { *err_out = e; return NULL; }
    snd_pcm_hw_params_alloca(&hw);
    snd_pcm_hw_params_any(pcm, hw);
    if ((e = snd_pcm_hw_params_set_access(pcm, hw, SND_PCM_ACCESS_RW_INTERLEAVED)) < 0 ||
        (e = snd_pcm_hw_params_set_format(pcm, hw, SND_PCM_FORMAT_S16_LE)) < 0 ||
        (e = snd_pcm_hw_params_set_rate_resample(pcm, hw, 1)) < 0 ||
        (e = snd_pcm_hw_params_set_channels_near(pcm, hw, ch)) < 0 ||
        (e = snd_pcm_hw_params_set_rate_near(pcm, hw, rate, NULL)) < 0) {
        snd_pcm_close(pcm);
        *err_out = e;
        return NULL;
    }
    unsigned lat = 100000;  // 100 ms buffer
    snd_pcm_hw_params_set_buffer_time_near(pcm, hw, &lat, NULL);
    if ((e = snd_pcm_hw_params(pcm, hw)) < 0) {
        snd_pcm_close(pcm);
        *err_out = e;
        return NULL;
    }
    return pcm;
}

// Writes `frames` frames of `ch`-channel audio; if the device has a different
// channel count, the clip is mixed to it (mono is repeated, extras dropped).
static void play(unsigned rate, unsigned ch, const uint8_t *data, long bytes) {
    long frames = bytes / (2 * (long)ch);
    if (frames <= 0) { puts("done"); fflush(stdout); return; }
    unsigned dr = rate, dc = ch;
    int e = 0;
    snd_pcm_t *pcm = open_pcm(SND_PCM_STREAM_PLAYBACK, &dr, &dc, &e);
    if (!pcm) { fail("open", e); return; }
    const int16_t *src = (const int16_t *)data;
    int16_t *mixed = NULL;
    if (dc != ch) {
        mixed = malloc((size_t)frames * dc * 2);
        if (!mixed) { snd_pcm_close(pcm); puts("error out of memory"); fflush(stdout); return; }
        for (long i = 0; i < frames; i++)
            for (unsigned c = 0; c < dc; c++)
                mixed[i * dc + c] = src[i * ch + (c < ch ? c : ch - 1)];
        src = mixed;
    }
    long done = 0;
    while (done < frames) {
        snd_pcm_sframes_t n = snd_pcm_writei(pcm, src + done * dc, (snd_pcm_uframes_t)(frames - done));
        if (n < 0) {
            n = snd_pcm_recover(pcm, (int)n, 1);
            if (n < 0) { fail("write", (int)n); free(mixed); snd_pcm_close(pcm); return; }
            continue;
        }
        done += n;
    }
    snd_pcm_drain(pcm);
    snd_pcm_close(pcm);
    free(mixed);
    puts("done");
    fflush(stdout);
}

static void record(long ms, unsigned rate, unsigned ch) {
    int e = 0;
    snd_pcm_t *pcm = open_pcm(SND_PCM_STREAM_CAPTURE, &rate, &ch, &e);
    if (!pcm) { fail("open", e); return; }
    long frames = (long)((double)rate * (double)ms / 1000.0);
    int16_t *buf = malloc((size_t)(frames > 0 ? frames : 1) * ch * 2);
    if (!buf) { snd_pcm_close(pcm); puts("error out of memory"); fflush(stdout); return; }
    long got = 0;
    snd_pcm_prepare(pcm);
    while (got < frames) {
        snd_pcm_sframes_t n = snd_pcm_readi(pcm, buf + got * ch, (snd_pcm_uframes_t)(frames - got));
        if (n < 0) {
            n = snd_pcm_recover(pcm, (int)n, 1);
            if (n < 0) { fail("read", (int)n); free(buf); snd_pcm_close(pcm); return; }
            continue;
        }
        got += n;
    }
    snd_pcm_close(pcm);
    printf("data %u %u ", rate, ch);
    b64write((const uint8_t *)buf, (size_t)got * ch * 2);
    putchar('\n');
    fflush(stdout);
    free(buf);
}

int main(void) {
    setvbuf(stdout, NULL, _IOLBF, 0);
    puts("ready");
    char *line = NULL;
    size_t cap = 0;
    while (getline(&line, &cap, stdin) >= 0) {
        char verb[16] = "";
        long a = 0, b = 0, c = 0;
        int off = 0;
        if (sscanf(line, "%15s", verb) < 1) { puts("error unknown request"); continue; }
        if (strcmp(verb, "play") == 0 && sscanf(line, "play %ld %ld %n", &a, &b, &off) >= 2 && a > 0 && b > 0) {
            const char *b64 = line + off;
            uint8_t *pcm = malloc(strlen(b64) / 4 * 3 + 3);
            long n = pcm ? b64decode(b64, pcm) : -1;
            if (n < 0) puts("error bad audio data");
            else play((unsigned)a, (unsigned)b, pcm, n);
            free(pcm);
        } else if (strcmp(verb, "rec") == 0 && sscanf(line, "rec %ld %ld %ld", &a, &b, &c) == 3 && b > 0 && c > 0) {
            record(a, (unsigned)b, (unsigned)c);
        } else {
            puts("error unknown request");
        }
    }
    free(line);
    return 0;
}
