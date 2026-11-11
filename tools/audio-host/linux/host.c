// The std.audio host for Linux (spec std.audio.9-10, ADR 0949): plays and
// records through ALSA (libasound). One text line in, one line out, the same
// protocol as the macOS host: `play RATE CH BASE64` -> `done`,
// `rec MS RATE CH` -> `data RATE CH BASE64`, anything wrong -> `error TEXT`.
// Streaming output (std.audio.12): `open_stream RATE CH` -> `ok`, `write BASE64`,
// `clock`, `pause`, `resume`, `flush` -> `played N buffered M`, `close_stream` -> `ok`.
// Audio is little-endian 16-bit PCM, interleaved. The device is "default",
// or $AIL_ALSA_DEVICE (for example "null" on a machine with no sound card).
// `--null` (or $AIL_AUDIO_NULL=1) makes streaming use a silent device whose
// clock runs in real time, so a test needs no sound card.
#include <alsa/asoundlib.h>
#include <pthread.h>
#include <time.h>
#include <unistd.h>
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

// ---- Streaming output (std.audio.12) ----
// A writer thread feeds ALSA from a queue of chunks. `played` is the frames the
// device has finished: frames handed to ALSA minus ALSA's own delay. With
// `--null` no device is opened and the clock is simulated in real time.
typedef struct chunk { struct chunk *next; int16_t *d; long frames; long off; } chunk;
static struct {
    pthread_mutex_t mu;
    pthread_cond_t cv;
    int open, null, paused, quit;
    snd_pcm_t *pcm;
    pthread_t th;
    unsigned rate, ch, dc;
    chunk *head, *tail;
    long total;    // frames accepted since the last flush
    long written;  // frames handed to ALSA since the last flush
    double nplayed;
    struct timespec last;
} S = { .mu = PTHREAD_MUTEX_INITIALIZER, .cv = PTHREAD_COND_INITIALIZER };

static int null_mode = 0;

static void free_queue(void) {
    while (S.head) { chunk *c = S.head; S.head = c->next; free(c->d); free(c); }
    S.tail = NULL;
}

static void *writer(void *arg) {
    (void)arg;
    pthread_mutex_lock(&S.mu);
    while (!S.quit) {
        if (!S.head || S.paused) { pthread_cond_wait(&S.cv, &S.mu); continue; }
        snd_pcm_sframes_t av = snd_pcm_avail_update(S.pcm);
        if (av < 0) { snd_pcm_recover(S.pcm, (int)av, 1); av = 0; }
        if (av == 0) {
            pthread_mutex_unlock(&S.mu);
            usleep(5000);
            pthread_mutex_lock(&S.mu);
            continue;
        }
        chunk *c = S.head;
        long n = c->frames - c->off;
        if (n > av) n = av;
        snd_pcm_sframes_t w = snd_pcm_writei(S.pcm, c->d + c->off * S.dc, (snd_pcm_uframes_t)n);
        if (w < 0) {
            if (snd_pcm_recover(S.pcm, (int)w, 1) < 0) {
                pthread_mutex_unlock(&S.mu);
                usleep(5000);
                pthread_mutex_lock(&S.mu);
            }
            continue;
        }
        c->off += w;
        S.written += w;
        if (c->off >= c->frames) {
            S.head = c->next;
            if (!S.head) S.tail = NULL;
            free(c->d);
            free(c);
        }
    }
    pthread_mutex_unlock(&S.mu);
    return NULL;
}

// Advances the silent clock to now. Caller holds S.mu.
static void null_tick(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    if (!S.paused) {
        double el = (double)(now.tv_sec - S.last.tv_sec) + (double)(now.tv_nsec - S.last.tv_nsec) / 1e9;
        S.nplayed += el * (double)S.rate;
    }
    S.last = now;
    if (S.nplayed > (double)S.total) S.nplayed = (double)S.total;
}

// Caller holds S.mu.
static void print_status(void) {
    long played;
    if (S.null) {
        null_tick();
        played = (long)S.nplayed;
    } else {
        snd_pcm_sframes_t d = 0;
        snd_pcm_state_t st = snd_pcm_state(S.pcm);
        if (st == SND_PCM_STATE_PREPARED) d = S.written;
        else if (st != SND_PCM_STATE_RUNNING && st != SND_PCM_STATE_PAUSED) d = 0;
        else if (snd_pcm_delay(S.pcm, &d) < 0 || d < 0) d = 0;
        played = S.written - (long)d;
        if (played < 0) played = 0;
        if (played > S.written) played = S.written;
    }
    printf("played %ld buffered %ld\n", played, S.total - played);
    fflush(stdout);
}

static void stream_close(void) {
    if (!S.open) return;
    pthread_mutex_lock(&S.mu);
    S.quit = 1;
    pthread_cond_signal(&S.cv);
    pthread_mutex_unlock(&S.mu);
    if (!S.null) pthread_join(S.th, NULL);
    pthread_mutex_lock(&S.mu);
    free_queue();
    pthread_mutex_unlock(&S.mu);
    if (S.pcm) { snd_pcm_drop(S.pcm); snd_pcm_close(S.pcm); S.pcm = NULL; }
    S.open = 0;
}

static void stream_open(unsigned rate, unsigned ch) {
    stream_close();
    S.null = null_mode;
    S.paused = S.quit = 0;
    S.total = S.written = 0;
    S.nplayed = 0;
    S.rate = rate;
    S.ch = ch;
    S.dc = ch;
    S.head = S.tail = NULL;
    clock_gettime(CLOCK_MONOTONIC, &S.last);
    if (!S.null) {
        unsigned dr = rate, dc = ch;
        int e = 0;
        S.pcm = open_pcm(SND_PCM_STREAM_PLAYBACK, &dr, &dc, &e);
        if (!S.pcm) { fail("open", e); return; }
        S.dc = dc;
        if (pthread_create(&S.th, NULL, writer, NULL) != 0) {
            snd_pcm_close(S.pcm);
            S.pcm = NULL;
            puts("error cannot start the audio thread");
            fflush(stdout);
            return;
        }
    }
    S.open = 1;
    puts("ok");
    fflush(stdout);
}

static int stream_write(const char *b64) {
    uint8_t *raw = malloc(strlen(b64) / 4 * 3 + 3);
    long n = raw ? b64decode(b64, raw) : -1;
    if (n < 0) { free(raw); puts("error bad audio data"); fflush(stdout); return 0; }
    long frames = n / (2 * (long)S.ch);
    if (frames > 0) {
        const int16_t *src = (const int16_t *)raw;
        int16_t *d = malloc((size_t)frames * S.dc * 2);
        chunk *c = malloc(sizeof *c);
        if (!d || !c) { free(d); free(c); free(raw); puts("error out of memory"); fflush(stdout); return 0; }
        for (long i = 0; i < frames; i++)
            for (unsigned k = 0; k < S.dc; k++)
                d[i * S.dc + k] = src[i * S.ch + (k < S.ch ? k : S.ch - 1)];
        c->next = NULL; c->d = d; c->frames = frames; c->off = 0;
        pthread_mutex_lock(&S.mu);
        if (S.tail) S.tail->next = c; else S.head = c;
        S.tail = c;
        S.total += frames;
        pthread_cond_signal(&S.cv);
        pthread_mutex_unlock(&S.mu);
    }
    free(raw);
    return 1;
}

static void stream_cmd(const char *verb) {
    pthread_mutex_lock(&S.mu);
    if (S.null) null_tick();
    if (strcmp(verb, "pause") == 0) {
        if (!S.paused) {
            if (!S.null) snd_pcm_pause(S.pcm, 1);  // ignored if the device cannot pause
            S.paused = 1;
        }
    } else if (strcmp(verb, "resume") == 0) {
        if (S.paused) {
            if (!S.null) snd_pcm_pause(S.pcm, 0);
            S.paused = 0;
            pthread_cond_signal(&S.cv);
        }
    } else if (strcmp(verb, "flush") == 0) {
        free_queue();
        S.total = S.written = 0;
        S.nplayed = 0;
        if (!S.null) { snd_pcm_drop(S.pcm); snd_pcm_prepare(S.pcm); }
    }
    print_status();
    pthread_mutex_unlock(&S.mu);
}

int main(int argc, char **argv) {
    const char *nv = getenv("AIL_AUDIO_NULL");
    null_mode = nv && *nv && strcmp(nv, "0") != 0;
    for (int i = 1; i < argc; i++) if (strcmp(argv[i], "--null") == 0) null_mode = 1;

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
        } else if (strcmp(verb, "open_stream") == 0 && sscanf(line, "open_stream %ld %ld", &a, &b) == 2 && a > 0 && b > 0) {
            stream_open((unsigned)a, (unsigned)b);
        } else if (strcmp(verb, "close_stream") == 0 && S.open) {
            stream_close();
            puts("ok");
        } else if (S.open && strcmp(verb, "write") == 0) {
            sscanf(line, "write %n", &off);
            if (stream_write(line + off)) stream_cmd("clock");
        } else if (S.open && (strcmp(verb, "clock") == 0 || strcmp(verb, "pause") == 0 ||
                              strcmp(verb, "resume") == 0 || strcmp(verb, "flush") == 0)) {
            stream_cmd(verb);
        } else if (strncmp(verb, "close_stream", 12) == 0 || strcmp(verb, "write") == 0 ||
                   strcmp(verb, "clock") == 0 || strcmp(verb, "pause") == 0 ||
                   strcmp(verb, "resume") == 0 || strcmp(verb, "flush") == 0) {
            puts("error no stream open");
        } else {
            puts("error unknown request");
        }
    }
    stream_close();
    free(line);
    return 0;
}
