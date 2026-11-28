/* gzip round trip with zlib (deflate + inflate at the default level); link with -lz. */
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <zlib.h>

int main(int argc, char **argv) {
    long n = argc > 1 ? atol(argv[1]) : 400000;
    const char *w[] = {"the", "of", "and", "to", "in", "fox", "jumps", "lazy", "dog", "river", "stone", "moss"};
    long seed = 7, cap = n * 7 + 16, len = 0;
    char *text = malloc(cap);
    for (long i = 0; i < n; i++) {
        seed = (seed * 3877 + 29573) % 139968;
        const char *s = w[seed % 12];
        if (i) text[len++] = ' ';
        size_t k = strlen(s);
        memcpy(text + len, s, k);
        len += k;
    }
    uLongf zc = compressBound(len);
    unsigned char *z = malloc(zc);
    z_stream d = {0};
    deflateInit2(&d, Z_DEFAULT_COMPRESSION, Z_DEFLATED, 31, 8, Z_DEFAULT_STRATEGY);
    d.next_in = (Bytef *)text; d.avail_in = len; d.next_out = z; d.avail_out = zc;
    deflate(&d, Z_FINISH);
    uLong zn = d.total_out;
    deflateEnd(&d);
    unsigned char *back = malloc(len + 1);
    z_stream s = {0};
    inflateInit2(&s, 31);
    s.next_in = z; s.avail_in = zn; s.next_out = back; s.avail_out = len + 1;
    inflate(&s, Z_FINISH);
    long bn = s.total_out;
    inflateEnd(&s);
    int ok = bn == len && memcmp(back, text, len) == 0;
    printf("bytes %ld, crc %lu, round trip %s\n", len, (unsigned long)crc32(crc32(0, Z_NULL, 0), (Bytef *)text, len), ok ? "true" : "false");
    return 0;
}
