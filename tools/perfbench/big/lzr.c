/* C twin of lzr.ail: LZ77 + adaptive binary range coder. */
#include <stdio.h>
#include <stdlib.h>

typedef long long ll;
static ll low = 0, rng = 4294967295LL, cache = 0, cache_size = 1;
static ll *outb; static ll nout = 0;
static ll probs[2400];

static void shift_low(void) {
    if (low < 4278190080LL || low >= 4294967296LL) {
        ll carry = low >> 32, temp = cache;
        for (;;) {
            outb[nout++] = (temp + carry) & 255;
            temp = 255;
            if (--cache_size == 0) break;
        }
        cache = (low >> 24) & 255;
    }
    cache_size++;
    low = (low & 16777215) << 8;
}
static void ebit(ll idx, ll b) {
    ll p = probs[idx], bound = (rng >> 11) * p;
    if (b == 0) { rng = bound; probs[idx] = p + ((2048 - p) >> 5); }
    else { low += bound; rng -= bound; probs[idx] = p - (p >> 5); }
    while (rng < 16777216) { rng <<= 8; shift_low(); }
}
static void edirect(ll v, ll nbits) {
    for (ll i = nbits - 1; i >= 0; i--) {
        rng >>= 1;
        if ((v >> i) & 1) low += rng;
        while (rng < 16777216) { rng <<= 8; shift_low(); }
    }
}
static void etree(ll base, ll nbits, ll v) {
    ll m = 1;
    for (ll i = nbits - 1; i >= 0; i--) { ll b = (v >> i) & 1; ebit(base + m, b); m = m * 2 + b; }
}

static ll code, pos; static ll *comp;
static ll dnext(void) { return comp[pos++]; }
static ll dbit(ll idx) {
    ll p = probs[idx], bound = (rng >> 11) * p, r = 0;
    if (code < bound) { rng = bound; probs[idx] = p + ((2048 - p) >> 5); }
    else { code -= bound; rng -= bound; probs[idx] = p - (p >> 5); r = 1; }
    if (rng < 16777216) { rng <<= 8; code = (code << 8) | dnext(); }
    return r;
}
static ll ddirect(ll nbits) {
    ll v = 0;
    for (ll i = 0; i < nbits; i++) {
        rng >>= 1;
        ll b = 0;
        if (code >= rng) { code -= rng; b = 1; }
        v = (v << 1) | b;
        if (rng < 16777216) { rng <<= 8; code = (code << 8) | dnext(); }
    }
    return v;
}
static ll dtree(ll base, ll nbits) {
    ll m = 1, top = 1;
    for (ll i = 0; i < nbits; i++) { m = m * 2 + dbit(base + m); top *= 2; }
    return m - top;
}

static ll hash3(ll a, ll b, ll c) { return (((a * 65536 + b * 256 + c) * 2654435761LL) & 4294967295LL) >> 12; }
static ll *head, *prev, *data;
static void insert_at(ll p, ll n) {
    if (p + 2 < n) { ll h = hash3(data[p], data[p + 1], data[p + 2]); prev[p] = head[h]; head[h] = p; }
}
static ll find(ll i, ll n) {
    ll best_len = 0, best_dist = 0;
    if (i + 2 < n) {
        ll cand = head[hash3(data[i], data[i + 1], data[i + 2])], chain = 0;
        while (cand >= 0 && i - cand <= 1048576 && chain < 32) {
            ll l = 0;
            while (l < 66 && i + l < n && data[cand + l] == data[i + l]) l++;
            if (l > best_len) { best_len = l; best_dist = i - cand; }
            cand = prev[cand];
            chain++;
        }
    }
    if (best_len < 3) return 0;
    return best_len * 2097152 + best_dist;
}
static ll dist_slot(ll d) {
    if (d < 4) return d;
    ll n = 0, t = d;
    while (t > 1) { t >>= 1; n++; }
    return 2 * n + ((d >> (n - 1)) & 1);
}
static ll min_ll(ll a, ll b) { return a < b ? a : b; }

int main(int argc, char **argv) {
    ll n = argc > 1 ? atoll(argv[1]) : 12000000;
    data = malloc((n + 16) * sizeof *data);
    ll len = 0, seed = 7;
    while (len < n) {
        seed = (seed * 3877 + 29573) % 139968;
        ll k = seed % 32768;
        for (ll j = 0; j < 3 + k % 7; j++) data[len++] = 97 + (k * 7 + j * 3 + k / 64) % 26;
        data[len++] = 32;
    }
    n = len;
    head = malloc(1048576 * sizeof *head); prev = malloc(n * sizeof *prev);
    for (ll i = 0; i < 1048576; i++) head[i] = -1;
    for (ll i = 0; i < n; i++) prev[i] = -1;
    outb = malloc((n + 1024) * sizeof *outb);
    for (int i = 0; i < 2400; i++) probs[i] = 1024;
    ll ins = 0, st = 0, i = 0;
    while (i < n) {
        while (ins < i) { insert_at(ins, n); ins++; }
        ll m = find(i, n);
        if (m > 0 && i + 1 < n) {
            while (ins < i + 1) { insert_at(ins, n); ins++; }
            if (find(i + 1, n) / 2097152 > m / 2097152 + 1) m = 0;
        }
        if (m > 0) {
            ll l = m / 2097152 - 3, d = m % 2097152 - 1;
            ebit(st, 1);
            etree(2, 6, l);
            ll slot = dist_slot(d);
            etree(66 + min_ll(l, 3) * 64, 6, slot);
            if (slot >= 4) { ll nb = (slot >> 1) - 1; edirect(d - ((2 + (slot & 1)) << nb), nb); }
            st = 1;
            i += l + 3;
        } else {
            ll ctx = i > 0 ? data[i - 1] >> 5 : 0;
            ebit(st, 0);
            etree(322 + ctx * 256, 8, data[i]);
            st = 0;
            i += 1;
        }
    }
    for (int k = 0; k < 5; k++) shift_low();
    ll clen = nout;
    /* decode */
    comp = outb; pos = 1; code = 0; rng = 4294967295LL;
    for (int k = 0; k < 4; k++) code = (code << 8) | dnext();
    for (int k = 0; k < 2400; k++) probs[k] = 1024;
    ll *back = malloc((n + 80) * sizeof *back), nb2 = 0;
    st = 0;
    while (nb2 < n) {
        if (dbit(st) == 0) {
            ll ctx = nb2 > 0 ? back[nb2 - 1] >> 5 : 0;
            back[nb2++] = dtree(322 + ctx * 256, 8);
            st = 0;
        } else {
            ll l = dtree(2, 6);
            ll slot = dtree(66 + min_ll(l, 3) * 64, 6);
            ll dist = slot;
            if (slot >= 4) { ll nb = (slot >> 1) - 1; dist = ((2 + (slot & 1)) << nb) + ddirect(nb); }
            ll start = nb2 - (dist + 1);
            for (ll k = 0; k < l + 3; k++) back[nb2++] = back[start + k];
            st = 1;
        }
    }
    ll cks = 0;
    for (ll k = 0; k < clen; k++) cks = (cks * 31 + outb[k]) % 1000000007;
    int ok = nb2 == n;
    for (ll k = 0; ok && k < n; k++) ok = back[k] == data[k];
    printf("input %lld\ncompressed %lld\npermille %lld\nchecksum %lld\nroundtrip %s\n", n, clen, clen * 1000 / n, cks, ok ? "true" : "false");
    return 0;
}
