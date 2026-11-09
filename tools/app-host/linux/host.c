/* The generic GTK 3 host for std.app on Linux (ADR 0838). It draws whatever
 * view tree the AIL program sends on standard input and reports what the user
 * does on standard output, newline-delimited JSON with "v":1 on the first line
 * each way (spec std.app.7-9). It knows nothing about any one app, and speaks
 * the same protocol as tools/app-host/macos. Needs GTK 3 only (no other
 * packages): build with run.sh.
 */
#include <gtk/gtk.h>
#include <gdk/gdkkeysyms.h>
#include <fontconfig/fontconfig.h>
#include <math.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* ---- JSON (std.app.7-8): just enough to read a view line ---- */

typedef enum { J_NULL, J_BOOL, J_NUM, J_STR, J_ARR, J_OBJ } JKind;
typedef struct JVal JVal;
struct JVal {
    JKind kind;
    double num;
    char *str;       /* J_STR */
    int n;           /* J_ARR, J_OBJ: element count */
    char **keys;     /* J_OBJ */
    JVal **vals;     /* J_ARR, J_OBJ */
};

static void jfree(JVal *v) {
    if (!v) return;
    free(v->str);
    for (int i = 0; i < v->n; i++) {
        if (v->keys) free(v->keys[i]);
        jfree(v->vals[i]);
    }
    free(v->keys);
    free(v->vals);
    free(v);
}

static void jws(const char **p) { while (**p == ' ' || **p == '\t' || **p == '\r' || **p == '\n') (*p)++; }

static void put_utf8(GString *s, unsigned c) { g_string_append_unichar(s, c ? c : 0xfffd); }

static unsigned hex4(const char *p) {
    unsigned v = 0;
    for (int i = 0; i < 4; i++) {
        char c = p[i];
        v <<= 4;
        if (c >= '0' && c <= '9') v |= c - '0';
        else if (c >= 'a' && c <= 'f') v |= c - 'a' + 10;
        else if (c >= 'A' && c <= 'F') v |= c - 'A' + 10;
        else return 0xfffd;
    }
    return v;
}

static char *jstring(const char **p) {
    GString *s = g_string_new("");
    (*p)++; /* opening quote */
    while (**p && **p != '"') {
        if (**p == '\\') {
            (*p)++;
            switch (**p) {
            case 'n': g_string_append_c(s, '\n'); break;
            case 't': g_string_append_c(s, '\t'); break;
            case 'r': g_string_append_c(s, '\r'); break;
            case 'b': g_string_append_c(s, '\b'); break;
            case 'f': g_string_append_c(s, '\f'); break;
            case 'u': {
                unsigned c = hex4(*p + 1);
                *p += 4;
                if (c >= 0xd800 && c < 0xdc00 && (*p)[1] == '\\' && (*p)[2] == 'u') {
                    unsigned lo = hex4(*p + 3);
                    if (lo >= 0xdc00 && lo < 0xe000) {
                        c = 0x10000 + ((c - 0xd800) << 10) + (lo - 0xdc00);
                        *p += 6;
                    }
                }
                put_utf8(s, c);
                break;
            }
            case 0: goto done;
            default: g_string_append_c(s, **p); break;
            }
            (*p)++;
        } else {
            g_string_append_c(s, **p);
            (*p)++;
        }
    }
done:
    if (**p == '"') (*p)++;
    return g_string_free(s, FALSE);
}

static JVal *jparse(const char **p) {
    jws(p);
    JVal *v = calloc(1, sizeof *v);
    if (**p == '{') {
        v->kind = J_OBJ;
        (*p)++;
        jws(p);
        int cap = 0;
        while (**p && **p != '}') {
            jws(p);
            if (**p != '"') { jfree(v); return NULL; }
            char *k = jstring(p);
            jws(p);
            if (**p != ':') { free(k); jfree(v); return NULL; }
            (*p)++;
            JVal *c = jparse(p);
            if (!c) { free(k); jfree(v); return NULL; }
            if (v->n == cap) {
                cap = cap ? cap * 2 : 8;
                v->keys = realloc(v->keys, cap * sizeof *v->keys);
                v->vals = realloc(v->vals, cap * sizeof *v->vals);
            }
            v->keys[v->n] = k;
            v->vals[v->n++] = c;
            jws(p);
            if (**p == ',') (*p)++;
        }
        if (**p == '}') (*p)++;
    } else if (**p == '[') {
        v->kind = J_ARR;
        (*p)++;
        jws(p);
        int cap = 0;
        while (**p && **p != ']') {
            JVal *c = jparse(p);
            if (!c) { jfree(v); return NULL; }
            if (v->n == cap) {
                cap = cap ? cap * 2 : 8;
                v->vals = realloc(v->vals, cap * sizeof *v->vals);
            }
            v->vals[v->n++] = c;
            jws(p);
            if (**p == ',') (*p)++;
        }
        if (**p == ']') (*p)++;
    } else if (**p == '"') {
        v->kind = J_STR;
        v->str = jstring(p);
    } else if (!strncmp(*p, "true", 4)) {
        v->kind = J_BOOL; v->num = 1; *p += 4;
    } else if (!strncmp(*p, "false", 5)) {
        v->kind = J_BOOL; v->num = 0; *p += 5;
    } else if (!strncmp(*p, "null", 4)) {
        *p += 4;
    } else if (**p == '-' || (**p >= '0' && **p <= '9')) {
        char *end;
        v->kind = J_NUM;
        v->num = g_ascii_strtod(*p, &end);
        *p = end;
    } else {
        jfree(v);
        return NULL;
    }
    return v;
}

static JVal *jget(const JVal *o, const char *k) {
    if (!o || o->kind != J_OBJ) return NULL;
    for (int i = 0; i < o->n; i++) if (!strcmp(o->keys[i], k)) return o->vals[i];
    return NULL;
}
static double jnum(const JVal *o, const char *k) { JVal *v = jget(o, k); return v && (v->kind == J_NUM || v->kind == J_BOOL) ? v->num : 0; }
static const char *jstr(const JVal *o, const char *k) { JVal *v = jget(o, k); return v && v->kind == J_STR ? v->str : ""; }
static gboolean jflag(const JVal *o, const char *k) { return jnum(o, k) != 0; }

/* ---- Output: events to standard output, one JSON object per line (std.app.8) ---- */

static gboolean greeted = FALSE;

static void esc(GString *s, const char *t) {
    g_string_append_c(s, '"');
    for (const unsigned char *p = (const unsigned char *)t; *p; p++) {
        switch (*p) {
        case '"': g_string_append(s, "\\\""); break;
        case '\\': g_string_append(s, "\\\\"); break;
        case '\n': g_string_append(s, "\\n"); break;
        case '\r': g_string_append(s, "\\r"); break;
        case '\t': g_string_append(s, "\\t"); break;
        default:
            if (*p < 0x20) g_string_append_printf(s, "\\u%04x", *p);
            else g_string_append_c(s, *p);
        }
    }
    g_string_append_c(s, '"');
}

typedef struct { GString *s; } Ev;

static Ev ev_new(const char *kind) {
    Ev e = { g_string_new("{") };
    if (!greeted) { g_string_append(e.s, "\"v\":1,"); greeted = TRUE; }
    g_string_append(e.s, "\"kind\":");
    esc(e.s, kind);
    return e;
}
static void ev_str(Ev *e, const char *k, const char *v) { g_string_append_printf(e->s, ",\"%s\":", k); esc(e->s, v); }
static void ev_num(Ev *e, const char *k, double v) {
    char b[G_ASCII_DTOSTR_BUF_SIZE];
    g_ascii_formatd(b, sizeof b, "%.3f", v); /* never a decimal comma */
    g_string_append_printf(e->s, ",\"%s\":%s", k, b);
}
static void ev_bool(Ev *e, const char *k, gboolean v) { g_string_append_printf(e->s, ",\"%s\":%s", k, v ? "true" : "false"); }
static void ev_send(Ev *e) {
    g_string_append(e->s, "}\n");
    fwrite(e->s->str, 1, e->s->len, stdout);
    fflush(stdout);
    g_string_free(e->s, TRUE);
}
/* std.app.31: the primary monitor's size in points and its scale. */
static void ev_screen(Ev *e) {
    GdkDisplay *d = gdk_display_get_default();
    GdkMonitor *m = d ? gdk_display_get_primary_monitor(d) : NULL;
    if (!m && d && gdk_display_get_n_monitors(d) > 0) m = gdk_display_get_monitor(d, 0);
    if (!m) return;
    GdkRectangle g;
    gdk_monitor_get_geometry(m, &g);
    ev_num(e, "screen_w", g.width);
    ev_num(e, "screen_h", g.height);
    ev_num(e, "scale", gdk_monitor_get_scale_factor(m));
}
/* std.app.25, std.app.33: in edge mode the window has no title bar and the host's own controls sit
   at the top-right, so `start` and `resize` carry what they take. */
#define EDGE_CTL_W 138.0 /* three buttons of 46 points */
#define EDGE_CTL_H 32.0
static gboolean edge_mode = FALSE;
static void ev_insets(Ev *e) {
    if (!edge_mode) return;
    ev_num(e, "inset_left", 0);
    ev_num(e, "inset_top", EDGE_CTL_H);
    ev_num(e, "inset_right", EDGE_CTL_W);
}
static void send_simple(const char *kind) { Ev e = ev_new(kind); ev_send(&e); }
static void send_id(const char *kind, const char *id) { Ev e = ev_new(kind); ev_str(&e, "id", id); ev_send(&e); }

/* ---- The view tree (std.app.2-3) ---- */

typedef enum { O_STR, O_FILL, O_SHADE, O_STROKE, O_PIC, O_BITMAP, O_CLIP, O_UNCLIP, O_ROUND, O_POLY, O_LINE } OpKind;
typedef struct {
    OpKind kind;
    double x, y, w, h, x2, y2, size, width, alpha;
    double r, stroke;        /* O_ROUND (std.app.32) */
    double *pts; int npts;   /* O_POLY, O_LINE: x0, y0, x1, y1, ...; npts is the number of points */
    int color;
    char *text, *font;   /* O_STR: text, font; O_PIC: text is src; O_BITMAP: text is the id */
    gboolean bold, italic, underline;
} Op;

typedef enum { N_LABEL, N_BUTTON, N_FIELD, N_TOGGLE, N_COLUMN, N_ROW, N_SPACER, N_CANVAS } NodeKind;
typedef struct Node Node;
struct Node {
    NodeKind kind;
    char *id, *text, *hint;   /* text: label text, caption or field value */
    gboolean on;
    double w, h;
    Op *ops;
    int nops;
    Node **kids;
    int nkids;
    GtkWidget *widget;
};

static void node_free(Node *n) {
    if (!n) return;
    free(n->id); free(n->text); free(n->hint);
    for (int i = 0; i < n->nops; i++) { free(n->ops[i].text); free(n->ops[i].font); free(n->ops[i].pts); }
    free(n->ops);
    for (int i = 0; i < n->nkids; i++) node_free(n->kids[i]);
    free(n->kids);
    free(n);
}

/* Bitmaps are kept by id so a later view can omit the data (std.app.3). */
static gboolean debug = FALSE; /* AIL_HOST_DEBUG=FILE logs canvas sizes and clips to FILE (=1: stderr) */
static FILE *dbg;

static GHashTable *bitmaps; /* id -> cairo_surface_t* */
static GHashTable *reduced; /* "kind:src|WxH" -> cairo_surface_t*: an image already shrunk by halving (L1) */

/* Forget the shrunk copies of bitmap `id` (its pixels changed). */
static void reduced_forget(const char *id) {
    char *prefix = g_strdup_printf("b:%s|", id);
    GHashTableIter it;
    gpointer k;
    g_hash_table_iter_init(&it, reduced);
    while (g_hash_table_iter_next(&it, &k, NULL)) if (g_str_has_prefix(k, prefix)) g_hash_table_iter_remove(&it);
    g_free(prefix);
}

static void bitmap_store(const char *id, int w, int h, const char *b64) {
    if (!*b64 || w <= 0 || h <= 0) return;
    gsize len;
    guchar *data = g_base64_decode(b64, &len);
    if (len == (gsize)w * h * 4) {
        cairo_surface_t *s = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, w, h);
        cairo_surface_flush(s);
        guchar *dst = cairo_image_surface_get_data(s);
        int stride = cairo_image_surface_get_stride(s);
        for (int y = 0; y < h; y++) {
            guint32 *row = (guint32 *)(dst + y * stride);
            for (int x = 0; x < w; x++) {
                const guchar *p = data + (y * w + x) * 4;
                guint a = p[3];
                row[x] = (a << 24) | (((p[0] * a + 127) / 255) << 16) | (((p[1] * a + 127) / 255) << 8) | ((p[2] * a + 127) / 255);
            }
        }
        cairo_surface_mark_dirty(s);
        g_hash_table_replace(bitmaps, g_strdup(id), s);
        reduced_forget(id);
    }
    g_free(data);
}

/* Web fonts are kept by id: the bytes go to a temp file that fontconfig loads
   for this process, and `web:<id>` draws in the family name found in it (std.app.3). */
static GHashTable *webfonts; /* id -> family name (char*) */

static void webfont_store(const char *id, const char *b64) {
    if (!*b64) return;
    gsize len;
    guchar *data = g_base64_decode(b64, &len);
    if (len > 0) {
        char *path = g_strdup_printf("%s/ail-webfont-%d-%u.ttf", g_get_tmp_dir(), (int)getpid(), g_str_hash(id));
        if (g_file_set_contents(path, (const char *)data, (gssize)len, NULL)) {
            FcFontSet *set = FcFontSetCreate();
            FcFileScan(set, NULL, NULL, NULL, (const FcChar8 *)path, FcFalse);
            if (set->nfont > 0) {
                FcChar8 *fam = NULL;
                if (FcPatternGetString(set->fonts[0], FC_FAMILY, 0, &fam) == FcResultMatch && fam) {
                    FcConfigAppFontAddFile(NULL, (const FcChar8 *)path);
                    g_hash_table_replace(webfonts, g_strdup(id), g_strdup((const char *)fam));
                }
            }
            FcFontSetDestroy(set);
        }
        g_free(path);
    }
    g_free(data);
}

static Node *decode_node(const JVal *d) {
    Node *n = calloc(1, sizeof *n);
    n->id = strdup(""); n->text = strdup(""); n->hint = strdup("");
    const char *k = jstr(d, "kind");
    if (!strcmp(k, "Label")) { n->kind = N_LABEL; free(n->text); n->text = strdup(jstr(d, "text")); }
    else if (!strcmp(k, "Button")) { n->kind = N_BUTTON; free(n->id); n->id = strdup(jstr(d, "id")); free(n->text); n->text = strdup(jstr(d, "caption")); }
    else if (!strcmp(k, "Field")) {
        n->kind = N_FIELD;
        free(n->id); n->id = strdup(jstr(d, "id"));
        free(n->text); n->text = strdup(jstr(d, "value"));
        free(n->hint); n->hint = strdup(jstr(d, "hint"));
    } else if (!strcmp(k, "Toggle")) {
        n->kind = N_TOGGLE;
        free(n->id); n->id = strdup(jstr(d, "id"));
        free(n->text); n->text = strdup(jstr(d, "caption"));
        n->on = jflag(d, "on");
    } else if (!strcmp(k, "Column") || !strcmp(k, "Row")) {
        n->kind = k[0] == 'C' ? N_COLUMN : N_ROW;
        JVal *kids = jget(d, "kids");
        if (kids && kids->kind == J_ARR) {
            n->kids = calloc(kids->n ? kids->n : 1, sizeof *n->kids);
            for (int i = 0; i < kids->n; i++) n->kids[n->nkids++] = decode_node(kids->vals[i]);
        }
    } else if (!strcmp(k, "Spacer")) { n->kind = N_SPACER; }
    else if (!strcmp(k, "Canvas")) {
        n->kind = N_CANVAS;
        free(n->id); n->id = strdup(jstr(d, "id"));
        n->w = jnum(d, "w"); n->h = jnum(d, "h");
        JVal *ops = jget(d, "ops");
        if (ops && ops->kind == J_ARR) {
            n->ops = calloc(ops->n ? ops->n : 1, sizeof *n->ops);
            for (int i = 0; i < ops->n; i++) {
                const JVal *o = ops->vals[i];
                const char *ok = jstr(o, "kind");
                Op op = { 0 };
                op.text = strdup(""); op.font = strdup("");
                if (!strcmp(ok, "Str")) {
                    op.kind = O_STR; op.x = jnum(o, "x"); op.y = jnum(o, "y");
                    free(op.text); op.text = strdup(jstr(o, "text"));
                    free(op.font); op.font = strdup(jstr(o, "font"));
                    op.size = jnum(o, "size"); op.color = (int)jnum(o, "fg");
                    op.bold = jflag(o, "bold"); op.italic = jflag(o, "italic"); op.underline = jflag(o, "underline");
                } else if (!strcmp(ok, "Fill")) {
                    op.kind = O_FILL; op.x = jnum(o, "x"); op.y = jnum(o, "y"); op.w = jnum(o, "w"); op.h = jnum(o, "h"); op.color = (int)jnum(o, "color");
                } else if (!strcmp(ok, "Shade")) {
                    /* std.app.19: a translucent fill; alpha is clamped to 0..1. */
                    op.kind = O_SHADE; op.x = jnum(o, "x"); op.y = jnum(o, "y"); op.w = jnum(o, "w"); op.h = jnum(o, "h"); op.color = (int)jnum(o, "color");
                    op.alpha = CLAMP(jnum(o, "alpha"), 0.0, 1.0);
                } else if (!strcmp(ok, "Stroke")) {
                    op.kind = O_STROKE; op.x = jnum(o, "x1"); op.y = jnum(o, "y1"); op.x2 = jnum(o, "x2"); op.y2 = jnum(o, "y2");
                    op.color = (int)jnum(o, "color"); op.width = jnum(o, "width");
                } else if (!strcmp(ok, "Pic")) {
                    op.kind = O_PIC; op.x = jnum(o, "x"); op.y = jnum(o, "y"); op.w = jnum(o, "w"); op.h = jnum(o, "h");
                    free(op.text); op.text = strdup(jstr(o, "src"));
                } else if (!strcmp(ok, "Bitmap")) {
                    op.kind = O_BITMAP; op.x = jnum(o, "x"); op.y = jnum(o, "y"); op.w = jnum(o, "w"); op.h = jnum(o, "h");
                    free(op.text); op.text = strdup(jstr(o, "id"));
                    bitmap_store(op.text, (int)jnum(o, "pw"), (int)jnum(o, "ph"), jstr(o, "data"));
                } else if (!strcmp(ok, "Round")) {
                    op.kind = O_ROUND; op.x = jnum(o, "x"); op.y = jnum(o, "y"); op.w = jnum(o, "w"); op.h = jnum(o, "h");
                    op.r = jnum(o, "r"); op.color = (int)jnum(o, "color"); op.alpha = jnum(o, "alpha"); op.stroke = jnum(o, "stroke");
                } else if (!strcmp(ok, "Poly") || !strcmp(ok, "Line")) {
                    op.kind = ok[0] == 'P' ? O_POLY : O_LINE;
                    op.color = (int)jnum(o, "color"); op.alpha = jnum(o, "alpha"); op.width = jnum(o, "width");
                    JVal *pa = jget(o, "points");
                    if (pa && pa->kind == J_ARR) {
                        op.pts = calloc(pa->n ? pa->n : 1, sizeof *op.pts);
                        for (int k = 0; k < pa->n; k++) op.pts[k] = pa->vals[k]->kind == J_NUM ? pa->vals[k]->num : 0;
                        op.npts = pa->n / 2;
                    }
                } else if (!strcmp(ok, "Font")) {
                    webfont_store(jstr(o, "id"), jstr(o, "data"));
                    free(op.text); free(op.font);
                    continue;
                } else if (!strcmp(ok, "Clip")) {
                    op.kind = O_CLIP; op.x = jnum(o, "x"); op.y = jnum(o, "y"); op.w = jnum(o, "w"); op.h = jnum(o, "h");
                } else if (!strcmp(ok, "Unclip")) {
                    op.kind = O_UNCLIP;
                } else { free(op.text); free(op.font); continue; }
                n->ops[n->nops++] = op;
            }
        }
    } else { n->kind = N_LABEL; free(n->text); n->text = g_strdup_printf("unknown node %s", k); }
    return n;
}

/* ---- Fonts (std.app.5, std.app.13) ---- */

/* The families behind `sans`, `serif` and `mono`: a Pango family list, first match wins. `sans` is
 * the desktop's UI font unless that is DejaVu or a bitmap face (set up in main). */
static char *sans_family = NULL;
#define SANS_LIST "Inter, Noto Sans, Cantarell, Ubuntu, Liberation Sans, DejaVu Sans, sans-serif"
#define SERIF_LIST "Noto Serif, Liberation Serif, DejaVu Serif, serif"
#define MONO_LIST "JetBrains Mono, Noto Sans Mono, Liberation Mono, DejaVu Sans Mono, monospace"

static void fonts_init(void) {
    char *name = NULL;
    g_object_get(gtk_settings_get_default(), "gtk-font-name", &name, NULL);
    PangoFontDescription *d = name ? pango_font_description_from_string(name) : NULL;
    const char *f = d ? pango_font_description_get_family(d) : NULL;
    gboolean ok = f && *f && g_ascii_strncasecmp(f, "DejaVu", 6) != 0 && g_ascii_strcasecmp(f, "Sans") != 0 &&
                  g_ascii_strcasecmp(f, "fixed") != 0 && g_ascii_strcasecmp(f, "misc") != 0 && g_ascii_strcasecmp(f, "terminal") != 0 &&
                  !strchr(f, ',');
    sans_family = ok ? g_strdup_printf("%s, " SANS_LIST, f) : g_strdup(SANS_LIST);
    if (d) pango_font_description_free(d);
    g_free(name);
}

/* One set of font options for drawing and measuring, so both agree at fractional positions. */
static cairo_font_options_t *fopts = NULL;

static void ctx_setup(PangoContext *pc) {
    if (!fopts) {
        fopts = cairo_font_options_create();
        cairo_font_options_set_antialias(fopts, CAIRO_ANTIALIAS_GRAY);
        cairo_font_options_set_hint_style(fopts, CAIRO_HINT_STYLE_SLIGHT);
        cairo_font_options_set_hint_metrics(fopts, CAIRO_HINT_METRICS_OFF);
    }
    pango_cairo_context_set_font_options(pc, fopts);
#if PANGO_VERSION_CHECK(1, 44, 0)
    pango_context_set_round_glyph_positions(pc, FALSE);
#endif
}

static GHashTable *installed; /* lower-cased family name -> family name (std.app.28) */

static void load_installed(void) {
    if (installed) return;
    installed = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, g_free);
    PangoFontFamily **fams = NULL;
    int n = 0;
    pango_font_map_list_families(pango_cairo_font_map_get_default(), &fams, &n);
    for (int i = 0; i < n; i++) {
        const char *name = pango_font_family_get_name(fams[i]);
        g_hash_table_insert(installed, g_utf8_strdown(name, -1), g_strdup(name));
    }
    g_free(fams);
}

/* std.app.28: resolves a CSS `font-family` list to the first installed family, else the list's
   first generic (`serif`, `mono`, `sans`), else `sans`. The result is owned by the caller. */
static char *resolve_family(const char *list) {
    load_installed();
    const char *generic = NULL;
    char **parts = g_strsplit(list, ",", -1);
    for (int i = 0; parts[i]; i++) {
        char *f = g_strstrip(parts[i]);
        size_t n = strlen(f);
        if (n >= 2 && (f[0] == '"' || f[0] == '\'') && f[n - 1] == f[0]) { f[n - 1] = 0; f = g_strstrip(f + 1); }
        if (!*f) continue;
        char *low = g_utf8_strdown(f, -1);
        if (!strcmp(low, "serif")) { if (!generic) generic = "serif"; }
        else if (!strcmp(low, "monospace") || !strcmp(low, "ui-monospace") || !strcmp(low, "mono")) { if (!generic) generic = "mono"; }
        else if (!strcmp(low, "sans-serif") || !strcmp(low, "system-ui") || !strcmp(low, "ui-sans-serif") || !strcmp(low, "sans")) { if (!generic) generic = "sans"; }
        else {
            const char *name = g_hash_table_lookup(installed, low);
            if (name) { char *r = g_strdup(name); g_free(low); g_strfreev(parts); return r; }
        }
        g_free(low);
    }
    g_strfreev(parts);
    return g_strdup(generic ? generic : "sans");
}

/* The font description of `family` (std.app.3, std.app.28); `used`, when given, gets the host face
   or installed family chosen, owned by the caller. */
static PangoFontDescription *font_desc_used(const char *family, double size, gboolean bold, gboolean italic, char **used) {
    PangoFontDescription *d = pango_font_description_new();
    const char *web = !strncmp(family, "web:", 4) ? (const char *)g_hash_table_lookup(webfonts, family + 4) : NULL;
    char *chosen = NULL;
    if (web) chosen = g_strdup(family);
    else if (!strncmp(family, "web:", 4)) chosen = g_strdup("sans"); /* a web font not sent yet */
    else if (!strcmp(family, "sans") || !strcmp(family, "serif") || !strcmp(family, "mono")) chosen = g_strdup(family);
    else chosen = resolve_family(family);
    const char *pango = web ? web : !strcmp(chosen, "sans") ? sans_family : !strcmp(chosen, "serif") ? SERIF_LIST : !strcmp(chosen, "mono") ? MONO_LIST : chosen;
    pango_font_description_set_family(d, pango);
    if (used) *used = chosen; else g_free(chosen);
    pango_font_description_set_absolute_size(d, (size > 0 ? size : 13) * PANGO_SCALE);
    pango_font_description_set_weight(d, bold ? PANGO_WEIGHT_BOLD : PANGO_WEIGHT_NORMAL);
    pango_font_description_set_style(d, italic ? PANGO_STYLE_ITALIC : PANGO_STYLE_NORMAL);
    return d;
}

static PangoFontDescription *font_desc(const char *family, double size, gboolean bold, gboolean italic) {
    return font_desc_used(family, size, bold, italic, NULL);
}

/* The size of one character of `mono` at 13 points (std.app.5). */
static void cell_size(PangoContext *pc, double *w, double *h) {
    ctx_setup(pc);
    PangoFontDescription *d = font_desc("mono", 13, FALSE, FALSE);
    PangoFontMetrics *m = pango_context_get_metrics(pc, d, NULL);
    *w = pango_font_metrics_get_approximate_char_width(m) / (double)PANGO_SCALE;
    *h = ceil((pango_font_metrics_get_ascent(m) + pango_font_metrics_get_descent(m)) / (double)PANGO_SCALE);
    pango_font_metrics_unref(m);
    pango_font_description_free(d);
}

/* std.app.13: the advance width of each requested run, sent back as one `measured` event. */
static void answer_measure(const JVal *runs) {
    GtkWidget *w = gtk_label_new("");
    PangoContext *pc = gtk_widget_get_pango_context(w);
    ctx_setup(pc);
    GString *out = g_string_new("");
    GString *fonts = g_string_new("");
    for (int i = 0; runs && runs->kind == J_ARR && i < runs->n; i++) {
        const JVal *r = runs->vals[i];
        char *used = NULL;
        PangoFontDescription *d = font_desc_used(jstr(r, "font"), jnum(r, "size"), jflag(r, "bold"), jflag(r, "italic"), &used);
        if (i) g_string_append_c(fonts, ',');
        for (char *p = used; *p; p++) if (*p == ',') *p = ' ';
        g_string_append(fonts, used);
        g_free(used);
        PangoLayout *l = pango_layout_new(pc);
        pango_layout_set_font_description(l, d);
        pango_layout_set_text(l, jstr(r, "text"), -1);
        PangoRectangle ink, logical;
        pango_layout_get_extents(l, &ink, &logical);
        if (i) g_string_append_c(out, ',');
        char b[G_ASCII_DTOSTR_BUF_SIZE];
        g_string_append(out, g_ascii_formatd(b, sizeof b, "%.3f", logical.width / (double)PANGO_SCALE));
        g_object_unref(l);
        pango_font_description_free(d);
    }
    Ev e = ev_new("measured");
    ev_str(&e, "text", out->str);
    ev_str(&e, "fonts", fonts->str); /* std.app.28 */
    ev_send(&e);
    g_string_free(out, TRUE);
    g_string_free(fonts, TRUE);
    g_object_ref_sink(w);
    g_object_unref(w);
}

/* ---- Canvas drawing and input (std.app.3-4) ---- */

static void set_color(cairo_t *cr, int c) {
    cairo_set_source_rgb(cr, ((c >> 16) & 0xff) / 255.0, ((c >> 8) & 0xff) / 255.0, (c & 0xff) / 255.0);
}

static GHashTable *pics; /* path -> GdkPixbuf* (or a NULL marker for a failed load) */

static void unref_pic(gpointer p) { if (p) g_object_unref(p); }

static GdkPixbuf *load_pic(const char *src) {
    if (g_hash_table_contains(pics, src)) return g_hash_table_lookup(pics, src);
    char *path = g_str_has_prefix(src, "file://") ? g_filename_from_uri(src, NULL, NULL) : g_strdup(src);
    GdkPixbuf *pb = path ? gdk_pixbuf_new_from_file(path, NULL) : NULL;
    g_free(path);
    g_hash_table_insert(pics, g_strdup(src), pb);
    return pb;
}

/* Images drawn at under half their size shimmer when one filter step samples sparsely, so they are
 * shrunk first by halving (a 2x2 average per step) down to between 1x and 2x the target and cached
 * per target size; the final step is CAIRO_FILTER_GOOD. Returns `src` when no reduction is needed. */
static cairo_surface_t *shrink(cairo_surface_t *src, const char *key, int tw, int th) {
    int cw = cairo_image_surface_get_width(src), ch = cairo_image_surface_get_height(src);
    if (tw < 1 || th < 1 || (cw < tw * 2 && ch < th * 2)) return src;
    char *k = g_strdup_printf("%s|%dx%d", key, tw, th);
    cairo_surface_t *hit = g_hash_table_lookup(reduced, k);
    if (hit) { g_free(k); return hit; }
    cairo_surface_t *cur = cairo_surface_reference(src);
    for (;;) {
        int nw = cw / 2 >= tw ? cw / 2 : cw, nh = ch / 2 >= th ? ch / 2 : ch;
        if (nw == cw && nh == ch) break;
        cairo_surface_t *next = cairo_image_surface_create(CAIRO_FORMAT_ARGB32, nw, nh);
        cairo_t *c = cairo_create(next);
        cairo_scale(c, (double)nw / cw, (double)nh / ch);
        cairo_set_source_surface(c, cur, 0, 0);
        cairo_pattern_set_filter(cairo_get_source(c), CAIRO_FILTER_BILINEAR);
        cairo_pattern_set_extend(cairo_get_source(c), CAIRO_EXTEND_PAD);
        cairo_paint(c);
        cairo_destroy(c);
        cairo_surface_destroy(cur);
        cur = next; cw = nw; ch = nh;
    }
    if (g_hash_table_size(reduced) > 256) g_hash_table_remove_all(reduced);
    g_hash_table_insert(reduced, k, cur);
    return cur;
}

static void placeholder(cairo_t *cr, const Op *op) {
    cairo_set_source_rgba(cr, 0.5, 0.5, 0.5, 0.25);
    cairo_rectangle(cr, op->x, op->y, op->w, op->h);
    cairo_fill(cr);
}

static const char *cid(GtkWidget *w);

static gboolean canvas_draw(GtkWidget *wd, cairo_t *cr, gpointer data) {
    Node *n = data;
    /* The canvas's own first fill is its background (so a dark app has no light flash); else the theme's. */
    gboolean painted = FALSE;
    for (int i = 0; i < n->nops && !painted; i++) {
        if (n->ops[i].kind != O_FILL) continue;
        set_color(cr, n->ops[i].color);
        cairo_paint(cr);
        painted = TRUE;
    }
    if (!painted) gtk_render_background(gtk_widget_get_style_context(wd), cr, 0, 0, gtk_widget_get_allocated_width(wd), gtk_widget_get_allocated_height(wd));
    PangoContext *pc = gtk_widget_get_pango_context(wd);
    ctx_setup(pc);
    int sf = gtk_widget_get_scale_factor(wd);
    int depth = 0;
    if (debug) {
        double cx1, cy1, cx2, cy2;
        cairo_clip_extents(cr, &cx1, &cy1, &cx2, &cy2);
        fprintf(dbg, "draw canvas %s alloc %dx%d ops %d clip %.0f,%.0f-%.0f,%.0f\n", cid(wd),
                gtk_widget_get_allocated_width(wd), gtk_widget_get_allocated_height(wd), n->nops, cx1, cy1, cx2, cy2);
    }
    for (int i = 0; i < n->nops; i++) {
        const Op *op = &n->ops[i];
        switch (op->kind) {
        case O_FILL:
            set_color(cr, op->color);
            cairo_rectangle(cr, op->x, op->y, op->w, op->h);
            cairo_fill(cr);
            break;
        case O_SHADE:
            cairo_set_source_rgba(cr, ((op->color >> 16) & 0xff) / 255.0, ((op->color >> 8) & 0xff) / 255.0, (op->color & 0xff) / 255.0, op->alpha);
            cairo_rectangle(cr, op->x, op->y, op->w, op->h);
            cairo_fill(cr);
            break;
        case O_STR: {
            PangoFontDescription *d = font_desc(op->font, op->size, op->bold, op->italic);
            PangoLayout *l = pango_layout_new(pc);
            pango_layout_set_font_description(l, d);
            pango_layout_set_text(l, op->text, -1);
            if (op->underline) {
                PangoAttrList *al = pango_attr_list_new();
                pango_attr_list_insert(al, pango_attr_underline_new(PANGO_UNDERLINE_SINGLE));
                pango_layout_set_attributes(l, al);
                pango_attr_list_unref(al);
            }
            set_color(cr, op->color);
            cairo_move_to(cr, op->x, op->y);
            pango_cairo_show_layout(cr, l);
            g_object_unref(l);
            pango_font_description_free(d);
            break;
        }
        case O_STROKE:
            cairo_move_to(cr, op->x, op->y);
            cairo_line_to(cr, op->x2, op->y2);
            cairo_set_line_width(cr, op->width > 0 ? op->width : 1);
            set_color(cr, op->color);
            cairo_stroke(cr);
            break;
        case O_ROUND: {
            /* std.app.32: radius at most half the shorter side; a stroke is centred on the edge. */
            double rr = op->r < 0 ? 0 : op->r, lim = (op->w < op->h ? op->w : op->h) / 2;
            if (rr > lim) rr = lim;
            double a = op->alpha < 0 ? 0 : op->alpha > 1 ? 1 : op->alpha;
            if (rr <= 0) cairo_rectangle(cr, op->x, op->y, op->w, op->h);
            else {
                cairo_new_sub_path(cr);
                cairo_arc(cr, op->x + op->w - rr, op->y + rr, rr, -G_PI / 2, 0);
                cairo_arc(cr, op->x + op->w - rr, op->y + op->h - rr, rr, 0, G_PI / 2);
                cairo_arc(cr, op->x + rr, op->y + op->h - rr, rr, G_PI / 2, G_PI);
                cairo_arc(cr, op->x + rr, op->y + rr, rr, G_PI, 3 * G_PI / 2);
                cairo_close_path(cr);
            }
            cairo_set_source_rgba(cr, ((op->color >> 16) & 0xff) / 255.0, ((op->color >> 8) & 0xff) / 255.0, (op->color & 0xff) / 255.0, a);
            if (op->stroke > 0) { cairo_set_line_width(cr, op->stroke); cairo_stroke(cr); }
            else cairo_fill(cr);
            break;
        }
        case O_POLY:
            if (op->npts >= 3) {
                double a = op->alpha < 0 ? 0 : op->alpha > 1 ? 1 : op->alpha;
                cairo_new_path(cr);
                cairo_move_to(cr, op->pts[0], op->pts[1]);
                for (int k = 1; k < op->npts; k++) cairo_line_to(cr, op->pts[k * 2], op->pts[k * 2 + 1]);
                cairo_close_path(cr);
                cairo_set_fill_rule(cr, CAIRO_FILL_RULE_WINDING);
                cairo_set_source_rgba(cr, ((op->color >> 16) & 0xff) / 255.0, ((op->color >> 8) & 0xff) / 255.0, (op->color & 0xff) / 255.0, a);
                cairo_fill(cr);
            }
            break;
        case O_LINE:
            if (op->npts >= 2) {
                cairo_new_path(cr);
                cairo_move_to(cr, op->pts[0], op->pts[1]);
                for (int k = 1; k < op->npts; k++) cairo_line_to(cr, op->pts[k * 2], op->pts[k * 2 + 1]);
                cairo_set_line_width(cr, op->width > 0 ? op->width : 1);
                cairo_set_line_cap(cr, CAIRO_LINE_CAP_ROUND);
                cairo_set_line_join(cr, CAIRO_LINE_JOIN_ROUND);
                set_color(cr, op->color);
                cairo_stroke(cr);
                cairo_set_line_cap(cr, CAIRO_LINE_CAP_BUTT);
                cairo_set_line_join(cr, CAIRO_LINE_JOIN_MITER);
            }
            break;
        case O_PIC: {
            GdkPixbuf *pb = load_pic(op->text);
            if (!pb || gdk_pixbuf_get_width(pb) <= 0 || gdk_pixbuf_get_height(pb) <= 0) { placeholder(cr, op); break; }
            int tw = (int)(op->w * sf + 0.5), th = (int)(op->h * sf + 0.5);
            cairo_surface_t *img;
            if (gdk_pixbuf_get_width(pb) >= tw * 2 || gdk_pixbuf_get_height(pb) >= th * 2) {
                char *key = g_strdup_printf("p:%s", op->text);
                cairo_surface_t *full = gdk_cairo_surface_create_from_pixbuf(pb, 1, NULL);
                img = shrink(full, key, tw, th);
                if (img != full) cairo_surface_reference(img);
                cairo_surface_destroy(full);
                g_free(key);
            } else {
                img = gdk_cairo_surface_create_from_pixbuf(pb, 1, NULL);
            }
            cairo_save(cr);
            cairo_translate(cr, op->x, op->y);
            cairo_scale(cr, op->w / cairo_image_surface_get_width(img), op->h / cairo_image_surface_get_height(img));
            cairo_set_source_surface(cr, img, 0, 0);
            cairo_pattern_set_filter(cairo_get_source(cr), CAIRO_FILTER_GOOD);
            cairo_paint(cr);
            cairo_restore(cr);
            cairo_surface_destroy(img);
            break;
        }
        case O_BITMAP: {
            cairo_surface_t *s = g_hash_table_lookup(bitmaps, op->text);
            if (!s) { placeholder(cr, op); break; }
            char *key = g_strdup_printf("b:%s", op->text);
            s = shrink(s, key, (int)(op->w * sf + 0.5), (int)(op->h * sf + 0.5));
            g_free(key);
            cairo_save(cr);
            cairo_translate(cr, op->x, op->y);
            cairo_scale(cr, op->w / cairo_image_surface_get_width(s), op->h / cairo_image_surface_get_height(s));
            cairo_set_source_surface(cr, s, 0, 0);
            cairo_pattern_set_filter(cairo_get_source(cr), CAIRO_FILTER_GOOD);
            cairo_paint(cr);
            cairo_restore(cr);
            break;
        }
        case O_CLIP:
            if (debug) fprintf(dbg, "  op %d clip %.0f,%.0f %.0fx%.0f\n", i, op->x, op->y, op->w, op->h);
            cairo_save(cr);
            depth++;
            cairo_rectangle(cr, op->x, op->y, op->w, op->h);
            cairo_clip(cr);
            break;
        case O_UNCLIP:
            if (depth > 0) { cairo_restore(cr); depth--; }
            break;
        }
    }
    while (depth-- > 0) cairo_restore(cr);
    return FALSE;
}

static void mods(Ev *e, guint state) {
    ev_bool(e, "ctrl", state & GDK_CONTROL_MASK);
    ev_bool(e, "alt", state & GDK_MOD1_MASK);
    ev_bool(e, "shift", state & GDK_SHIFT_MASK);
    ev_bool(e, "meta", state & (GDK_SUPER_MASK | GDK_META_MASK));
}

static const char *cid(GtkWidget *w) { return g_object_get_data(G_OBJECT(w), "ail-id"); }

/* A pointer event of `kind` for the button event `e` on canvas `w`. */
static void pointer_ev(const char *kind, GtkWidget *w, GdkEventButton *e) {
    Ev ev = ev_new(kind);
    mods(&ev, e->state);
    ev_str(&ev, "id", cid(w));
    ev_num(&ev, "x", e->x);
    ev_num(&ev, "y", e->y);
    ev_str(&ev, "text", e->button == 1 ? "left" : e->button == 3 ? "right" : "other");
    ev_num(&ev, "clicks", e->type == GDK_3BUTTON_PRESS ? 3 : e->type == GDK_2BUTTON_PRESS ? 2 : 1);
    ev_send(&ev);
}

/* std.app.27 (ADR 0983): `down` before `click`; GTK's implicit grab keeps the
   motion and the release on this canvas until the button is released. */
/* The last press, for the `drag` line (std.app.25): GTK moves a window from a press. */
static struct { guint button, time; gdouble x_root, y_root; gboolean valid; } last_press;

static gboolean canvas_press(GtkWidget *w, GdkEventButton *e, gpointer d) {
    (void)d;
    if (e->type != GDK_BUTTON_PRESS) return TRUE;
    last_press.button = e->button; last_press.time = e->time;
    last_press.x_root = e->x_root; last_press.y_root = e->y_root; last_press.valid = TRUE;
    gtk_widget_grab_focus(w);
    pointer_ev("down", w, e);
    pointer_ev("click", w, e);
    return TRUE;
}

static void mv_send(void);
static gboolean canvas_release(GtkWidget *w, GdkEventButton *e, gpointer d) {
    (void)d;
    mv_send(); /* a waiting move goes before the up */
    pointer_ev("up", w, e);
    return TRUE;
}

/* `move` (std.app.16, std.app.27): while a button is held, or while the app asked for hover, at most
 * once per 16 ms; a move that comes sooner waits for a timer so the last position is always sent. */
static gboolean hover_on = FALSE;
static char *mv_id = NULL;
static double mv_x, mv_y;
static guint mv_timer = 0;
static gint64 mv_last = 0; /* microseconds, monotonic */

static void mv_send(void) {
    if (mv_timer) { g_source_remove(mv_timer); mv_timer = 0; }
    if (!mv_id) return;
    Ev ev = ev_new("move");
    ev_str(&ev, "id", mv_id);
    ev_num(&ev, "x", mv_x);
    ev_num(&ev, "y", mv_y);
    ev_send(&ev);
    mv_last = g_get_monotonic_time();
    g_free(mv_id);
    mv_id = NULL;
}

static gboolean mv_fire(gpointer d) { (void)d; mv_timer = 0; mv_send(); return G_SOURCE_REMOVE; }

static gboolean canvas_drag(GtkWidget *w, GdkEventMotion *e, gpointer d) {
    (void)d;
    gboolean held = (e->state & (GDK_BUTTON1_MASK | GDK_BUTTON2_MASK | GDK_BUTTON3_MASK)) != 0;
    if (!held && !hover_on) return FALSE;
    g_free(mv_id);
    mv_id = g_strdup(cid(w));
    mv_x = e->x; mv_y = e->y;
    gint64 age = (g_get_monotonic_time() - mv_last) / 1000;
    if (age >= 16) mv_send();
    else if (!mv_timer) mv_timer = g_timeout_add((guint)(16 - age), mv_fire, NULL);
    return TRUE;
}

static gboolean canvas_leave(GtkWidget *w, GdkEventCrossing *e, gpointer d) {
    (void)d;
    if (!hover_on || e->detail == GDK_NOTIFY_INFERIOR) return FALSE;
    mv_send();
    send_id("leave", cid(w));
    return FALSE;
}

static gboolean canvas_scroll(GtkWidget *w, GdkEventScroll *e, gpointer d) {
    (void)d;
    /* Points, positive dy when the content should move up (scrolling down). */
    double dx = 0, dy = 0;
    switch (e->direction) {
    case GDK_SCROLL_SMOOTH: dx = e->delta_x * 16; dy = e->delta_y * 16; break;
    case GDK_SCROLL_UP: dy = -16; break;
    case GDK_SCROLL_DOWN: dy = 16; break;
    case GDK_SCROLL_LEFT: dx = -16; break;
    case GDK_SCROLL_RIGHT: dx = 16; break;
    }
    if (dx == 0 && dy == 0) return TRUE;
    Ev ev = ev_new("scroll");
    mods(&ev, e->state);
    ev_str(&ev, "id", cid(w));
    ev_num(&ev, "dx", dx);
    ev_num(&ev, "dy", dy);
    ev_send(&ev);
    return TRUE;
}

/* The key as std.ui names it (std.ui.7): a name for special keys, else the typed
 * character, unshifted while Control or Meta is held. */
static const char *key_name(guint kv) {
    switch (kv) {
    case GDK_KEY_Return: case GDK_KEY_KP_Enter: case GDK_KEY_ISO_Enter: return "enter";
    case GDK_KEY_Tab: case GDK_KEY_ISO_Left_Tab: return "tab";
    case GDK_KEY_BackSpace: return "backspace";
    case GDK_KEY_Delete: case GDK_KEY_KP_Delete: return "delete";
    case GDK_KEY_Escape: return "escape";
    case GDK_KEY_Up: case GDK_KEY_KP_Up: return "up";
    case GDK_KEY_Down: case GDK_KEY_KP_Down: return "down";
    case GDK_KEY_Left: case GDK_KEY_KP_Left: return "left";
    case GDK_KEY_Right: case GDK_KEY_KP_Right: return "right";
    case GDK_KEY_Home: case GDK_KEY_KP_Home: return "home";
    case GDK_KEY_End: case GDK_KEY_KP_End: return "end";
    case GDK_KEY_Page_Up: case GDK_KEY_KP_Page_Up: return "pageup";
    case GDK_KEY_Page_Down: case GDK_KEY_KP_Page_Down: return "pagedown";
    case GDK_KEY_F1: return "f1"; case GDK_KEY_F2: return "f2"; case GDK_KEY_F3: return "f3";
    case GDK_KEY_F4: return "f4"; case GDK_KEY_F5: return "f5"; case GDK_KEY_F6: return "f6";
    case GDK_KEY_F7: return "f7"; case GDK_KEY_F8: return "f8"; case GDK_KEY_F9: return "f9";
    case GDK_KEY_F10: return "f10"; case GDK_KEY_F11: return "f11"; case GDK_KEY_F12: return "f12";
    }
    return NULL;
}

static gboolean canvas_key(GtkWidget *w, GdkEventKey *e, gpointer d) {
    (void)d;
    /* std.app.23 (ADR 0978): Control-V brings the clipboard's text as `paste`. */
    if ((e->state & GDK_CONTROL_MASK) && gdk_keyval_to_lower(e->keyval) == GDK_KEY_v) {
        gchar *s = gtk_clipboard_wait_for_text(gtk_clipboard_get(GDK_SELECTION_CLIPBOARD));
        if (s) {
            Ev ev = ev_new("paste");
            ev_str(&ev, "id", cid(w));
            ev_str(&ev, "text", s);
            ev_send(&ev);
            g_free(s);
            return TRUE;
        }
    }
    const char *name = key_name(e->keyval);
    char buf[8] = { 0 };
    if (!name) {
        guint kv = e->keyval;
        if (e->state & (GDK_CONTROL_MASK | GDK_SUPER_MASK | GDK_META_MASK)) kv = gdk_keyval_to_lower(kv);
        gunichar u = gdk_keyval_to_unicode(kv);
        if (!u) return FALSE; /* a bare modifier or an unnamed key */
        buf[g_unichar_to_utf8(u, buf)] = 0;
        name = buf;
    }
    Ev ev = ev_new("key");
    mods(&ev, e->state);
    ev_str(&ev, "id", cid(w));
    ev_str(&ev, "text", name);
    ev_send(&ev);
    return TRUE;
}

/* Last size reported per canvas id, so a rebuilt canvas does not repeat it. */
static GHashTable *reported; /* id -> double[2] */

static void canvas_alloc(GtkWidget *w, GdkRectangle *a, gpointer d) {
    (void)d;
    const char *id = cid(w);
    double *r = g_hash_table_lookup(reported, id);
    if (!r) {
        r = g_new0(double, 2);
        g_hash_table_insert(reported, g_strdup(id), r);
    }
    if (r[0] == a->width && r[1] == a->height) return;
    r[0] = a->width; r[1] = a->height;
    double cw, ch;
    cell_size(gtk_widget_get_pango_context(w), &cw, &ch);
    Ev ev = ev_new("resize");
    ev_str(&ev, "id", id);
    ev_num(&ev, "dx", a->width);
    ev_num(&ev, "dy", a->height);
    ev_num(&ev, "cell_w", cw);
    ev_num(&ev, "cell_h", ch);
    ev_screen(&ev);
    ev_insets(&ev);
    ev_send(&ev);
}

/* ---- Widgets for the tree ---- */

static GHashTable *field_text; /* what each field shows: the user's typing, until the program sends a new value */
static GHashTable *last_sent;
static gboolean suppress = FALSE;
static GtkWidget *window, *content = NULL;
static Node *current = NULL;
static GHashTable *by_id; /* id -> GtkWidget* for the focus */

/* The texts of the edits sent for a field since the program last answered with a value it chose
 * itself (as in the Windows host). A view that echoes an edit sent before the newest one is out of
 * date, since typing has gone on: it must not overwrite the box. One that echoes the newest edit is
 * in step. Any other value is the program's own choice and wins. */
static GHashTable *edits; /* id -> GPtrArray of char* */

static void edit_note(const char *id, const char *text) {
    GPtrArray *a = g_hash_table_lookup(edits, id);
    if (!a) {
        a = g_ptr_array_new_with_free_func(g_free);
        g_hash_table_insert(edits, g_strdup(id), a);
    }
    g_ptr_array_add(a, g_strdup(text));
}

/* TRUE when `text` is an echo of an edit, so the box keeps what it has. */
static gboolean edit_echo(const char *id, const char *text) {
    GPtrArray *a = g_hash_table_lookup(edits, id);
    if (!a || a->len == 0) return FALSE;
    for (int at = (int)a->len - 1; at >= 0; at--) {
        if (strcmp(g_ptr_array_index(a, at), text)) continue;
        if ((guint)at == a->len - 1) {
            g_ptr_array_set_size(a, 0);
            g_hash_table_insert(last_sent, g_strdup(id), g_strdup(text));
        } else {
            g_ptr_array_remove_range(a, 0, at + 1);
        }
        return TRUE;
    }
    g_ptr_array_set_size(a, 0);
    return FALSE;
}

static void sync_fields(const Node *n) {
    if (n->kind == N_FIELD && !edit_echo(n->id, n->text)) {
        const char *last = g_hash_table_lookup(last_sent, n->id);
        if (!last || strcmp(last, n->text)) {
            g_hash_table_insert(last_sent, g_strdup(n->id), g_strdup(n->text));
            g_hash_table_insert(field_text, g_strdup(n->id), g_strdup(n->text));
        }
    }
    for (int i = 0; i < n->nkids; i++) sync_fields(n->kids[i]);
}

static void on_button(GtkButton *b, gpointer d) { (void)d; send_id("press", cid(GTK_WIDGET(b))); }

static void on_changed(GtkEditable *e, gpointer d) {
    (void)d;
    if (suppress) return;
    const char *t = gtk_entry_get_text(GTK_ENTRY(e));
    g_hash_table_insert(field_text, g_strdup(cid(GTK_WIDGET(e))), g_strdup(t));
    edit_note(cid(GTK_WIDGET(e)), t);
    Ev ev = ev_new("edit");
    ev_str(&ev, "id", cid(GTK_WIDGET(e)));
    ev_str(&ev, "text", t);
    ev_send(&ev);
}

static void on_activate(GtkEntry *e, gpointer d) { (void)d; send_id("submit", cid(GTK_WIDGET(e))); }

static void on_toggled(GtkToggleButton *b, gpointer d) {
    (void)d;
    if (suppress) return;
    Ev ev = ev_new("toggle");
    ev_str(&ev, "id", cid(GTK_WIDGET(b)));
    ev_str(&ev, "text", gtk_toggle_button_get_active(b) ? "on" : "off");
    ev_send(&ev);
}

static void set_id(GtkWidget *w, const char *id) {
    g_object_set_data_full(G_OBJECT(w), "ail-id", g_strdup(id), g_free);
    g_hash_table_insert(by_id, g_strdup(id), w);
}

static void canvas_size(GtkWidget *w, const Node *n) {
    if (debug) fprintf(dbg, "canvas %s w %.0f h %.0f ops %d\n", n->id, n->w, n->h, n->nops);
    gtk_widget_set_size_request(w, n->w > 0 ? (int)n->w : -1, n->h > 0 ? (int)n->h : -1);
    gtk_widget_set_hexpand(w, n->w <= 0);
    gtk_widget_set_vexpand(w, n->h <= 0);
    gtk_widget_set_halign(w, n->w > 0 ? GTK_ALIGN_START : GTK_ALIGN_FILL);
    gtk_widget_set_valign(w, n->h > 0 ? GTK_ALIGN_START : GTK_ALIGN_FILL);
}

/* True when the tree has a canvas that fills the window: the app draws its
   own chrome, so the host adds no frame and no gaps (ADR 0888, as the
   Windows host does). */
static gboolean edge(const Node *n) {
    if (n->kind == N_CANVAS && (n->w <= 0 || n->h <= 0)) return TRUE;
    for (int i = 0; i < n->nkids; i++) if (edge(n->kids[i])) return TRUE;
    return FALSE;
}

static gboolean edge_tree = FALSE;

static GtkWidget *build(Node *n) {
    GtkWidget *w = NULL;
    switch (n->kind) {
    case N_LABEL:
        w = gtk_label_new(n->text);
        gtk_label_set_selectable(GTK_LABEL(w), TRUE);
        gtk_widget_set_halign(w, GTK_ALIGN_FILL);
        gtk_label_set_xalign(GTK_LABEL(w), 0);
        gtk_widget_set_hexpand(w, TRUE);
        break;
    case N_BUTTON:
        w = gtk_button_new_with_label(n->text);
        set_id(w, n->id);
        g_signal_connect(w, "clicked", G_CALLBACK(on_button), NULL);
        break;
    case N_FIELD: {
        w = gtk_entry_new();
        set_id(w, n->id);
        gtk_entry_set_placeholder_text(GTK_ENTRY(w), n->hint);
        const char *t = g_hash_table_lookup(field_text, n->id);
        suppress = TRUE;
        gtk_entry_set_text(GTK_ENTRY(w), t ? t : n->text);
        suppress = FALSE;
        gtk_widget_set_hexpand(w, TRUE);
        g_signal_connect(w, "changed", G_CALLBACK(on_changed), NULL);
        g_signal_connect(w, "activate", G_CALLBACK(on_activate), NULL);
        break;
    }
    case N_TOGGLE:
        w = gtk_check_button_new_with_label(n->text);
        set_id(w, n->id);
        suppress = TRUE;
        gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(w), n->on);
        suppress = FALSE;
        g_signal_connect(w, "toggled", G_CALLBACK(on_toggled), NULL);
        break;
    case N_COLUMN:
    case N_ROW:
        w = gtk_box_new(n->kind == N_COLUMN ? GTK_ORIENTATION_VERTICAL : GTK_ORIENTATION_HORIZONTAL, edge_tree ? 0 : 6);
        for (int i = 0; i < n->nkids; i++) {
            GtkWidget *k = build(n->kids[i]);
            gtk_box_pack_start(GTK_BOX(w), k, FALSE, TRUE, 0);
        }
        break;
    case N_SPACER:
        w = gtk_fixed_new();
        gtk_widget_set_hexpand(w, TRUE);
        gtk_widget_set_vexpand(w, TRUE);
        break;
    case N_CANVAS:
        w = gtk_drawing_area_new();
        set_id(w, n->id);
        gtk_widget_set_can_focus(w, TRUE);
        gtk_widget_add_events(w, GDK_BUTTON_PRESS_MASK | GDK_BUTTON_RELEASE_MASK | GDK_BUTTON_MOTION_MASK | GDK_POINTER_MOTION_MASK | GDK_LEAVE_NOTIFY_MASK | GDK_SCROLL_MASK | GDK_SMOOTH_SCROLL_MASK | GDK_KEY_PRESS_MASK);
        gtk_style_context_add_class(gtk_widget_get_style_context(w), "view");
        canvas_size(w, n);
        g_signal_connect(w, "draw", G_CALLBACK(canvas_draw), n);
        g_signal_connect(w, "button-press-event", G_CALLBACK(canvas_press), NULL);
        g_signal_connect(w, "button-release-event", G_CALLBACK(canvas_release), NULL);
        g_signal_connect(w, "motion-notify-event", G_CALLBACK(canvas_drag), NULL);
        g_signal_connect(w, "leave-notify-event", G_CALLBACK(canvas_leave), NULL);
        g_signal_connect(w, "scroll-event", G_CALLBACK(canvas_scroll), NULL);
        g_signal_connect(w, "key-press-event", G_CALLBACK(canvas_key), NULL);
        g_signal_connect(w, "size-allocate", G_CALLBACK(canvas_alloc), NULL);
        break;
    }
    n->widget = w;
    return w;
}

/* The same widgets can show `b` when it has `a`'s shape: the same kinds, in the same places. */
static gboolean same_shape(const Node *a, const Node *b) {
    if (a->kind != b->kind || a->nkids != b->nkids) return FALSE;
    if ((a->kind == N_CANVAS || a->kind == N_BUTTON || a->kind == N_TOGGLE || a->kind == N_FIELD) && strcmp(a->id, b->id)) return FALSE;
    for (int i = 0; i < a->nkids; i++) if (!same_shape(a->kids[i], b->kids[i])) return FALSE;
    return TRUE;
}

/* Moves `a`'s widgets to `b` and brings them up to date. A canvas's draw callback reads
 * its node, so it is pointed at `b` (the old tree is freed afterwards). */
static void update_in_place(Node *a, Node *b) {
    GtkWidget *w = a->widget;
    b->widget = w;
    suppress = TRUE;
    switch (b->kind) {
    case N_LABEL:
        if (strcmp(a->text, b->text)) gtk_label_set_text(GTK_LABEL(w), b->text);
        break;
    case N_BUTTON:
        if (strcmp(a->text, b->text)) gtk_button_set_label(GTK_BUTTON(w), b->text);
        break;
    case N_FIELD: {
        const char *want = g_hash_table_lookup(field_text, b->id);
        if (!want) want = b->text;
        if (strcmp(gtk_entry_get_text(GTK_ENTRY(w)), want)) gtk_entry_set_text(GTK_ENTRY(w), want);
        if (strcmp(a->hint, b->hint)) gtk_entry_set_placeholder_text(GTK_ENTRY(w), b->hint);
        break;
    }
    case N_TOGGLE:
        if (strcmp(a->text, b->text)) gtk_button_set_label(GTK_BUTTON(w), b->text);
        gtk_toggle_button_set_active(GTK_TOGGLE_BUTTON(w), b->on);
        break;
    case N_CANVAS:
        g_signal_handlers_disconnect_by_func(w, G_CALLBACK(canvas_draw), a);
        g_signal_connect(w, "draw", G_CALLBACK(canvas_draw), b);
        canvas_size(w, b);
        gtk_widget_queue_draw(w);
        break;
    default: break;
    }
    suppress = FALSE;
    for (int i = 0; i < b->nkids; i++) update_in_place(a->kids[i], b->kids[i]);
}

static char *last_focus = NULL;

/* The colour of the first Fill in the tree, if any. */
static gboolean first_fill(const Node *n, int *c) {
    if (n->kind == N_CANVAS) {
        for (int i = 0; i < n->nops; i++) if (n->ops[i].kind == O_FILL) { *c = n->ops[i].color; return TRUE; }
    }
    for (int i = 0; i < n->nkids; i++) if (first_fill(n->kids[i], c)) return TRUE;
    return FALSE;
}

/* A dark first fill (luma under 50 %) puts GTK in its dark theme, so the window frame matches. */
static void follow_dark(const Node *tree) {
    static int dark = -1;
    int c;
    if (!first_fill(tree, &c)) return;
    double luma = (0.2126 * ((c >> 16) & 0xff) + 0.7152 * ((c >> 8) & 0xff) + 0.0722 * (c & 0xff)) / 255.0;
    int want = luma < 0.5;
    if (want == dark) return;
    dark = want;
    g_object_set(gtk_settings_get_default(), "gtk-application-prefer-dark-theme", want ? TRUE : FALSE, NULL);
}

static GtkWidget *overlay, *ctl_box, *grips[3];

/* std.app.25: edge mode drops the title bar and shows the host's controls and resize edges. */
static void set_edge(gboolean on) {
    if (edge_mode == on) return;
    edge_mode = on;
    gtk_window_set_decorated(GTK_WINDOW(window), !on);
    gtk_widget_set_visible(ctl_box, on);
    for (int i = 0; i < 3; i++) gtk_widget_set_visible(grips[i], on);
    g_hash_table_remove_all(reported); /* the next size-allocate sends `resize` with the insets */
}

static void on_minimize(GtkButton *b, gpointer d) { (void)b; (void)d; gtk_window_iconify(GTK_WINDOW(window)); }
static void toggle_max(void) {
    if (gtk_window_is_maximized(GTK_WINDOW(window))) gtk_window_unmaximize(GTK_WINDOW(window));
    else gtk_window_maximize(GTK_WINDOW(window));
}
static void on_maximize(GtkButton *b, gpointer d) { (void)b; (void)d; toggle_max(); }
static void on_close(GtkButton *b, gpointer d) { (void)b; (void)d; gtk_window_close(GTK_WINDOW(window)); }

static gboolean on_grip(GtkWidget *w, GdkEventButton *e, gpointer edge) {
    (void)w;
    if (e->type != GDK_BUTTON_PRESS || e->button != 1 || gtk_window_is_maximized(GTK_WINDOW(window))) return FALSE;
    gtk_window_begin_resize_drag(GTK_WINDOW(window), (GdkWindowEdge)GPOINTER_TO_INT(edge), e->button, (gint)e->x_root, (gint)e->y_root, e->time);
    return TRUE;
}

/* The overlay holds the app's content, the controls at the top-right (46 by 32 points each) and
   thin edges that resize an undecorated window. Both stay hidden outside edge mode. */
static void build_edge_chrome(void) {
    overlay = gtk_overlay_new();
    gtk_container_add(GTK_CONTAINER(window), overlay);
    ctl_box = gtk_box_new(GTK_ORIENTATION_HORIZONTAL, 0);
    const char *icons[3] = { "window-minimize-symbolic", "window-maximize-symbolic", "window-close-symbolic" };
    GCallback cbs[3] = { G_CALLBACK(on_minimize), G_CALLBACK(on_maximize), G_CALLBACK(on_close) };
    for (int i = 0; i < 3; i++) {
        GtkWidget *b = gtk_button_new_from_icon_name(icons[i], GTK_ICON_SIZE_MENU);
        gtk_button_set_relief(GTK_BUTTON(b), GTK_RELIEF_NONE);
        gtk_widget_set_size_request(b, (int)(EDGE_CTL_W / 3), (int)EDGE_CTL_H);
        gtk_widget_set_can_focus(b, FALSE);
        g_signal_connect(b, "clicked", cbs[i], NULL);
        gtk_box_pack_start(GTK_BOX(ctl_box), b, FALSE, FALSE, 0);
    }
    gtk_widget_set_halign(ctl_box, GTK_ALIGN_END);
    gtk_widget_set_valign(ctl_box, GTK_ALIGN_START);
    gtk_widget_set_no_show_all(ctl_box, TRUE);
    gtk_overlay_add_overlay(GTK_OVERLAY(overlay), ctl_box);
    /* left, right and bottom edges, 5 points wide */
    GtkAlign ha[3] = { GTK_ALIGN_START, GTK_ALIGN_END, GTK_ALIGN_FILL };
    GtkAlign va[3] = { GTK_ALIGN_FILL, GTK_ALIGN_FILL, GTK_ALIGN_END };
    GdkWindowEdge ed[3] = { GDK_WINDOW_EDGE_WEST, GDK_WINDOW_EDGE_EAST, GDK_WINDOW_EDGE_SOUTH };
    for (int i = 0; i < 3; i++) {
        GtkWidget *g = gtk_event_box_new();
        gtk_event_box_set_visible_window(GTK_EVENT_BOX(g), FALSE);
        gtk_widget_add_events(g, GDK_BUTTON_PRESS_MASK);
        gtk_widget_set_size_request(g, i < 2 ? 5 : -1, i < 2 ? -1 : 5);
        gtk_widget_set_halign(g, ha[i]);
        gtk_widget_set_valign(g, va[i]);
        gtk_widget_set_no_show_all(g, TRUE);
        g_signal_connect(g, "button-press-event", G_CALLBACK(on_grip), GINT_TO_POINTER(ed[i]));
        gtk_overlay_add_overlay(GTK_OVERLAY(overlay), g);
        grips[i] = g;
    }
    gtk_widget_show_all(overlay);
}

static void show_view(const char *title, const char *focus, Node *tree, gboolean want_edge) {
    gtk_window_set_title(GTK_WINDOW(window), title);
    set_edge(want_edge && edge(tree));
    sync_fields(tree);
    follow_dark(tree);
    if (current && same_shape(current, tree)) {
        update_in_place(current, tree);
        node_free(current);
    } else {
        /* Keep the keyboard focus (and the cursor in a field) across a rebuild. */
        GtkWidget *fw = gtk_window_get_focus(GTK_WINDOW(window));
        char *fid = fw && cid(fw) ? g_strdup(cid(fw)) : NULL;
        int pos = fw && GTK_IS_EDITABLE(fw) ? gtk_editable_get_position(GTK_EDITABLE(fw)) : 0;
        if (content) gtk_widget_destroy(content);
        g_hash_table_remove_all(by_id);
        node_free(current);
        edge_tree = edge(tree);
        gtk_container_set_border_width(GTK_CONTAINER(window), edge_tree ? 0 : 8);
        content = build(tree);
        gtk_container_add(GTK_CONTAINER(overlay), content);
        gtk_widget_show_all(content);
        if (fid) {
            GtkWidget *w = g_hash_table_lookup(by_id, fid);
            if (w) {
                gtk_widget_grab_focus(w);
                if (GTK_IS_EDITABLE(w)) gtk_editable_set_position(GTK_EDITABLE(w), pos);
            }
            g_free(fid);
        }
    }
    current = tree;
    if (strcmp(focus, last_focus ? last_focus : "")) {
        g_free(last_focus);
        last_focus = g_strdup(focus);
        GtkWidget *w = *focus ? g_hash_table_lookup(by_id, focus) : NULL;
        if (w) gtk_widget_grab_focus(w);
    }
}

/* ---- System appearance (std.app.30, ADR 1008) ---- */

static char last_appearance[8] = "";

/* "dark" when GNOME's color-scheme says so, else when GTK's theme is dark. */
static const char *appearance_now(void) {
    GSettingsSchemaSource *src = g_settings_schema_source_get_default();
    GSettingsSchema *sc = src ? g_settings_schema_source_lookup(src, "org.gnome.desktop.interface", TRUE) : NULL;
    if (sc) {
        const char *dark = NULL;
        if (g_settings_schema_has_key(sc, "color-scheme")) {
            GSettings *st = g_settings_new("org.gnome.desktop.interface");
            gchar *v = g_settings_get_string(st, "color-scheme");
            if (!strcmp(v, "prefer-dark")) dark = "dark";
            else if (!strcmp(v, "prefer-light")) dark = "light";
            g_free(v);
            g_object_unref(st);
        }
        g_settings_schema_unref(sc);
        if (dark) return dark;
    }
    gboolean prefer = FALSE;
    gchar *theme = NULL;
    g_object_get(gtk_settings_get_default(), "gtk-application-prefer-dark-theme", &prefer, "gtk-theme-name", &theme, NULL);
    gboolean dark = prefer || (theme && strstr(theme, "dark") != NULL) || (theme && strstr(theme, "Dark") != NULL);
    g_free(theme);
    return dark ? "dark" : "light";
}

static void send_appearance(void) {
    const char *now = appearance_now();
    if (!strcmp(now, last_appearance)) return;
    snprintf(last_appearance, sizeof last_appearance, "%s", now);
    Ev e = ev_new("appearance");
    ev_str(&e, "text", now);
    ev_send(&e);
}

static void on_appearance_change(GObject *o, GParamSpec *p, gpointer d) {
    (void)o; (void)p; (void)d;
    send_appearance();
}

static void watch_appearance(void) {
    GtkSettings *gs = gtk_settings_get_default();
    g_signal_connect(gs, "notify::gtk-theme-name", G_CALLBACK(on_appearance_change), NULL);
    g_signal_connect(gs, "notify::gtk-application-prefer-dark-theme", G_CALLBACK(on_appearance_change), NULL);
    GSettingsSchemaSource *src = g_settings_schema_source_get_default();
    GSettingsSchema *sc = src ? g_settings_schema_source_lookup(src, "org.gnome.desktop.interface", TRUE) : NULL;
    if (sc) {
        if (g_settings_schema_has_key(sc, "color-scheme")) {
            GSettings *st = g_settings_new("org.gnome.desktop.interface");
            g_signal_connect(st, "changed::color-scheme", G_CALLBACK(on_appearance_change), NULL);
        }
        g_settings_schema_unref(sc);
    }
}

/* ---- Reading views from standard input (std.app.7) ---- */

static gboolean first_view = TRUE, started = FALSE;

static void handle_line(const char *line) {
    const char *p = line;
    JVal *obj = jparse(&p);
    if (!obj || obj->kind != J_OBJ) { jfree(obj); return; }
    const char *t = jstr(obj, "t");
    if (!strcmp(t, "measure")) {
        answer_measure(jget(obj, "runs"));
    } else if (!strcmp(t, "clipboard")) {
        /* std.app.26 (ADR 0982): the app puts text on the system clipboard. */
        gtk_clipboard_set_text(gtk_clipboard_get(GDK_SELECTION_CLIPBOARD), jstr(obj, "text"), -1);
    } else if (!strcmp(t, "drag") || !strcmp(t, "zoom")) {
        /* std.app.25: the app's own strip moves the window (a press began it) or toggles maximise. */
        if (edge_mode && !strcmp(t, "zoom")) toggle_max();
        else if (edge_mode && last_press.valid) {
            last_press.valid = FALSE;
            gtk_window_begin_move_drag(GTK_WINDOW(window), (gint)last_press.button, (gint)last_press.x_root, (gint)last_press.y_root, last_press.time);
        }
    } else if (!strcmp(t, "pick")) {
        /* std.app.22: no native chooser on this host yet; say so rather than hang. */
        Ev e = ev_new("pick_error");
        ev_str(&e, "id", jstr(obj, "id"));
        ev_str(&e, "text", "this host has no native file chooser yet");
        ev_send(&e);
    } else if (!strcmp(t, "view")) {
        if (first_view) {
            first_view = FALSE;
            if ((int)jnum(obj, "v") != 1) {
                fprintf(stderr, "AIL App Host: the program speaks another protocol version\n");
                jfree(obj);
                gtk_main_quit();
                return;
            }
        }
        JVal *title = jget(obj, "title");
        hover_on = jflag(obj, "hover");
        Node *tree = decode_node(jget(obj, "tree"));
        show_view(title && title->kind == J_STR ? title->str : "AIL", jstr(obj, "focus"), tree, jflag(obj, "edge"));
        if (!started) {
            started = TRUE;
            { Ev se = ev_new("start"); ev_screen(&se); ev_insets(&se); ev_send(&se); }
            send_appearance();
            watch_appearance();
        }
    }
    jfree(obj);
}

static gboolean on_stdin(GIOChannel *ch, GIOCondition cond, gpointer d) {
    (void)d; (void)cond;
    for (;;) {
        gchar *line = NULL;
        gsize len;
        GIOStatus st = g_io_channel_read_line(ch, &line, &len, NULL, NULL);
        if (st == G_IO_STATUS_NORMAL) {
            handle_line(line);
            g_free(line);
        } else if (st == G_IO_STATUS_AGAIN) {
            return TRUE;
        } else {
            g_free(line);
            gtk_main_quit();
            return FALSE;
        }
    }
}

static gboolean on_delete(GtkWidget *w, GdkEvent *e, gpointer d) {
    (void)w; (void)e; (void)d;
    send_simple("close");
    return FALSE;
}

int main(int argc, char **argv) {
    gtk_init(&argc, &argv);
    webfonts = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, g_free);
    bitmaps = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, (GDestroyNotify)cairo_surface_destroy);
    reduced = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, (GDestroyNotify)cairo_surface_destroy);
    fonts_init();
    pics = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, unref_pic);
    reported = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, g_free);
    field_text = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, g_free);
    last_sent = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, g_free);
    by_id = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, NULL);
    edits = g_hash_table_new_full(g_str_hash, g_str_equal, g_free, (GDestroyNotify)g_ptr_array_unref);
    const char *dv = g_getenv("AIL_HOST_DEBUG");
    debug = dv && *dv;
    dbg = debug && strcmp(dv, "1") ? fopen(dv, "w") : stderr;
    if (!dbg) dbg = stderr;
    if (debug) setvbuf(dbg, NULL, _IOLBF, 0);

    window = gtk_window_new(GTK_WINDOW_TOPLEVEL);
    gtk_window_set_title(GTK_WINDOW(window), "AIL");
    gtk_window_set_default_size(GTK_WINDOW(window), 1000, 720);
    const char *icon = g_getenv("AIL_APP_ICON"); /* a PNG; run.sh sets it for Garlic */
    if (icon && *icon) gtk_window_set_icon_from_file(GTK_WINDOW(window), icon, NULL);
    GdkGeometry g = { .min_width = 320, .min_height = 200 };
    gtk_window_set_geometry_hints(GTK_WINDOW(window), NULL, &g, GDK_HINT_MIN_SIZE);
    gtk_container_set_border_width(GTK_CONTAINER(window), 8);
    g_signal_connect(window, "delete-event", G_CALLBACK(on_delete), NULL);
    g_signal_connect(window, "destroy", G_CALLBACK(gtk_main_quit), NULL);
    build_edge_chrome();
    gtk_widget_show(window);

    GIOChannel *in = g_io_channel_unix_new(0);
    g_io_channel_set_encoding(in, NULL, NULL);
    g_io_channel_set_flags(in, G_IO_FLAG_NONBLOCK, NULL);
    g_io_add_watch(in, G_IO_IN | G_IO_HUP | G_IO_ERR, on_stdin, NULL);
    gtk_main();
    return 0;
}
