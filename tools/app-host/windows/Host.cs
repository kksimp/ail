// The generic Win32 host for std.app (ADR 0838): draws whatever view tree the
// AIL program sends (WinForms controls plus one GDI+ canvas control) and
// reports what the user does, over the same NDJSON protocol as
// tools/app-host/macos (std.app.7-8, 13). It knows nothing about any one app.
// Written in C# 5 so the csc.exe that ships with Windows builds it.

using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Text;
using System.Globalization;
using System.IO;
using System.Net;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;
using System.Windows.Forms;

// ---- JSON ----------------------------------------------------------------

static class Json
{
    public static object Parse(string s)
    {
        int i = 0;
        try { return Value(s, ref i); } catch (Exception) { return null; }
    }

    static void Ws(string s, ref int i) { while (i < s.Length && char.IsWhiteSpace(s[i])) i++; }

    static object Value(string s, ref int i)
    {
        Ws(s, ref i);
        char c = s[i];
        if (c == '{')
        {
            var d = new Dictionary<string, object>();
            i++; Ws(s, ref i);
            if (s[i] == '}') { i++; return d; }
            while (true)
            {
                Ws(s, ref i);
                string k = Str(s, ref i);
                Ws(s, ref i); i++; // ':'
                d[k] = Value(s, ref i);
                Ws(s, ref i);
                if (s[i++] == '}') return d;
            }
        }
        if (c == '[')
        {
            var l = new List<object>();
            i++; Ws(s, ref i);
            if (s[i] == ']') { i++; return l; }
            while (true)
            {
                l.Add(Value(s, ref i));
                Ws(s, ref i);
                if (s[i++] == ']') return l;
            }
        }
        if (c == '"') return Str(s, ref i);
        if (string.CompareOrdinal(s, i, "true", 0, 4) == 0) { i += 4; return true; }
        if (string.CompareOrdinal(s, i, "false", 0, 5) == 0) { i += 5; return false; }
        if (string.CompareOrdinal(s, i, "null", 0, 4) == 0) { i += 4; return null; }
        int st = i;
        while (i < s.Length && "+-0123456789.eE".IndexOf(s[i]) >= 0) i++;
        return double.Parse(s.Substring(st, i - st), CultureInfo.InvariantCulture);
    }

    static string Str(string s, ref int i)
    {
        var b = new StringBuilder();
        i++; // opening quote
        while (s[i] != '"')
        {
            char c = s[i++];
            if (c != '\\') { b.Append(c); continue; }
            c = s[i++];
            switch (c)
            {
                case 'n': b.Append('\n'); break;
                case 't': b.Append('\t'); break;
                case 'r': b.Append('\r'); break;
                case 'b': b.Append('\b'); break;
                case 'f': b.Append('\f'); break;
                case 'u': b.Append((char)Convert.ToInt32(s.Substring(i, 4), 16)); i += 4; break;
                default: b.Append(c); break;
            }
        }
        i++;
        return b.ToString();
    }

    public static string Quote(string s)
    {
        var b = new StringBuilder("\"");
        foreach (char c in s)
        {
            switch (c)
            {
                case '"': b.Append("\\\""); break;
                case '\\': b.Append("\\\\"); break;
                case '\n': b.Append("\\n"); break;
                case '\r': b.Append("\\r"); break;
                case '\t': b.Append("\\t"); break;
                default:
                    if (c < 0x20) b.Append("\\u").Append(((int)c).ToString("x4"));
                    else b.Append(c);
                    break;
            }
        }
        return b.Append('"').ToString();
    }

    public static string Text(object o) { return o as string ?? ""; }
    public static double Num(object o) { return o is double ? (double)o : 0; }
    public static bool Flag(object o) { return o is bool && (bool)o; }
    public static object Get(object d, string k)
    {
        var m = d as Dictionary<string, object>;
        object v;
        return m != null && m.TryGetValue(k, out v) ? v : null;
    }
}

// ---- Protocol ------------------------------------------------------------

/// std.app.31 (ADR 1017): the primary display's size in points and its scale.
static class ScreenInfo
{
    public static double W { get { return Screen.PrimaryScreen.Bounds.Width * 96.0 / Dpi; } }
    public static double H { get { return Screen.PrimaryScreen.Bounds.Height * 96.0 / Dpi; } }
    public static double Scale { get { return Dpi / 96.0; } }
    static double Dpi
    {
        get
        {
            using (Graphics g = Graphics.FromHwnd(IntPtr.Zero)) return g.DpiX;
        }
    }
}

/// Writes events to standard output, one JSON object per line (std.app.8).
static class Outbox
{
    static readonly object gate = new object();
    static Stream stdout = Console.OpenStandardOutput();
    static bool greeted;

    public static void Send(string kind, params object[] kv)
    {
        var b = new StringBuilder("{");
        var parts = new List<string>();
        parts.Add("\"kind\":" + Json.Quote(kind));
        for (int i = 0; i + 1 < kv.Length; i += 2)
        {
            object v = kv[i + 1];
            string t;
            if (v is string) t = Json.Quote((string)v);
            else if (v is bool) t = (bool)v ? "true" : "false";
            else t = Convert.ToDouble(v, CultureInfo.InvariantCulture).ToString("0.###", CultureInfo.InvariantCulture);
            parts.Add(Json.Quote((string)kv[i]) + ":" + t);
        }
        lock (gate)
        {
            if (!greeted) { parts.Add("\"v\":1"); greeted = true; }
            b.Append(string.Join(",", parts.ToArray())).Append("}\n");
            byte[] data = new UTF8Encoding(false).GetBytes(b.ToString());
            try { stdout.Write(data, 0, data.Length); stdout.Flush(); } catch (IOException) { }
        }
    }
}

// ---- View tree -----------------------------------------------------------

abstract class Op { }
class StrOp : Op { public double X, Y, Size; public string Text, Font; public int Fg; public bool Bold, Italic, Underline; }
class FillOp : Op { public double X, Y, W, H; public int Color; }
class ShadeOp : Op { public double X, Y, W, H, Alpha; public int Color; }
class StrokeOp : Op { public double X1, Y1, X2, Y2, Width; public int Color; }
class RoundOp : Op { public double X, Y, W, H, R, Alpha, Stroke; public int Color; }
class PolyOp : Op { public float[] Points; public double Alpha; public int Color; }
class LineOp : Op { public float[] Points; public double Width; public int Color; }
class PicOp : Op { public double X, Y, W, H; public string Src; }
class BitmapOp : Op { public double X, Y, W, H; public string Id; }
class ClipOp : Op { public double X, Y, W, H; }
class UnclipOp : Op { }

class Node
{
    public string Kind = "", Id = "", Text = "", Hint = "";
    public bool On;
    public double W, H;
    public List<Node> Kids = new List<Node>();
    public List<Op> Ops = new List<Op>();

    /// The tree's shape, ignoring what changes without a rebuild.
    public string Shape()
    {
        var b = new StringBuilder();
        Shape(b);
        return b.ToString();
    }

    void Shape(StringBuilder b)
    {
        b.Append('(').Append(Kind).Append(' ').Append(Id);
        if (Kind == "Button" || Kind == "Toggle") b.Append(' ').Append(Text);
        if (Kind == "Field") b.Append(' ').Append(Hint);
        if (Kind == "Canvas") b.Append(' ').Append(W).Append('x').Append(H);
        foreach (Node k in Kids) k.Shape(b);
        b.Append(')');
    }

    public static Node Decode(object any)
    {
        var n = new Node();
        n.Kind = Json.Text(Json.Get(any, "kind"));
        n.Id = Json.Text(Json.Get(any, "id"));
        switch (n.Kind)
        {
            case "Label": n.Text = Json.Text(Json.Get(any, "text")); break;
            case "Button": case "Toggle": n.Text = Json.Text(Json.Get(any, "caption")); n.On = Json.Flag(Json.Get(any, "on")); break;
            case "Field": n.Text = Json.Text(Json.Get(any, "value")); n.Hint = Json.Text(Json.Get(any, "hint")); break;
            case "Column": case "Row":
                var kids = Json.Get(any, "kids") as List<object>;
                if (kids != null) foreach (object k in kids) n.Kids.Add(Decode(k));
                break;
            case "Spacer": break;
            case "Canvas":
                n.W = Json.Num(Json.Get(any, "w")); n.H = Json.Num(Json.Get(any, "h"));
                var ops = Json.Get(any, "ops") as List<object>;
                if (ops != null) foreach (object o in ops) { Op op = DecodeOp(o); if (op != null) n.Ops.Add(op); }
                break;
            default: n.Kind = "Label"; n.Text = "unknown node"; break;
        }
        return n;
    }

    // The `points` array of a Poly or Line (std.app.32).
    static float[] Pts(object d)
    {
        var l = Json.Get(d, "points") as List<object>;
        if (l == null) return new float[0];
        var a = new float[l.Count];
        for (int i = 0; i < a.Length; i++) a[i] = (float)Json.Num(l[i]);
        return a;
    }

    static Op DecodeOp(object d)
    {
        double X = Json.Num(Json.Get(d, "x")), Y = Json.Num(Json.Get(d, "y")), W = Json.Num(Json.Get(d, "w")), H = Json.Num(Json.Get(d, "h"));
        int color = (int)Json.Num(Json.Get(d, "color"));
        switch (Json.Text(Json.Get(d, "kind")))
        {
            case "Str":
                return new StrOp { X = X, Y = Y, Text = Json.Text(Json.Get(d, "text")), Font = Json.Text(Json.Get(d, "font")),
                    Size = Json.Num(Json.Get(d, "size")), Fg = (int)Json.Num(Json.Get(d, "fg")), Bold = Json.Flag(Json.Get(d, "bold")),
                    Italic = Json.Flag(Json.Get(d, "italic")), Underline = Json.Flag(Json.Get(d, "underline")) };
            case "Fill": return new FillOp { X = X, Y = Y, W = W, H = H, Color = color };
            case "Shade": // std.app.19: opacity 0..1, clamped
                return new ShadeOp { X = X, Y = Y, W = W, H = H, Color = color, Alpha = Math.Max(0, Math.Min(1, Json.Num(Json.Get(d, "alpha")))) };
            case "Stroke":
                return new StrokeOp { X1 = Json.Num(Json.Get(d, "x1")), Y1 = Json.Num(Json.Get(d, "y1")), X2 = Json.Num(Json.Get(d, "x2")),
                    Y2 = Json.Num(Json.Get(d, "y2")), Color = color, Width = Json.Num(Json.Get(d, "width")) };
            case "Round":
                return new RoundOp { X = X, Y = Y, W = W, H = H, R = Json.Num(Json.Get(d, "r")), Color = color,
                    Alpha = Json.Num(Json.Get(d, "alpha")), Stroke = Json.Num(Json.Get(d, "stroke")) };
            case "Poly": return new PolyOp { Points = Pts(d), Color = color, Alpha = Json.Num(Json.Get(d, "alpha")) };
            case "Line": return new LineOp { Points = Pts(d), Color = color, Width = Json.Num(Json.Get(d, "width")) };
            case "Pic": return new PicOp { X = X, Y = Y, W = W, H = H, Src = Json.Text(Json.Get(d, "src")) };
            case "Bitmap":
                string id = Json.Text(Json.Get(d, "id"));
                Bitmaps.Store(id, (int)Json.Num(Json.Get(d, "pw")), (int)Json.Num(Json.Get(d, "ph")), Json.Text(Json.Get(d, "data")));
                return new BitmapOp { X = X, Y = Y, W = W, H = H, Id = id };
            case "Font":
                Fonts.Store(Json.Text(Json.Get(d, "id")), Json.Text(Json.Get(d, "data")));
                return null;
            case "Clip": return new ClipOp { X = X, Y = Y, W = W, H = H };
            case "Unclip": return new UnclipOp();
        }
        return null;
    }
}

/// Images sent as `Bitmap` ops, kept by id so a later view can omit the data (std.app.3),
/// and pictures loaded from files or URLs (`Pic`).
static class Bitmaps
{
    static readonly Dictionary<string, Image> images = new Dictionary<string, Image>();
    static readonly HashSet<string> loading = new HashSet<string>();

    public static Image Get(string id) { lock (images) { Image i; return images.TryGetValue(id, out i) ? i : null; } }

    /// Decodes `R, G, B, A` bytes (non-premultiplied); empty data keeps what `id` has.
    public static void Store(string id, int w, int h, string b64)
    {
        if (b64.Length == 0 || w <= 0 || h <= 0) return;
        byte[] px;
        try { px = Convert.FromBase64String(b64); } catch (FormatException) { return; }
        if (px.Length != w * h * 4) return;
        for (int i = 0; i < px.Length; i += 4) { byte r = px[i]; px[i] = px[i + 2]; px[i + 2] = r; } // RGBA -> BGRA
        var bmp = new Bitmap(w, h, System.Drawing.Imaging.PixelFormat.Format32bppArgb);
        var data = bmp.LockBits(new Rectangle(0, 0, w, h), System.Drawing.Imaging.ImageLockMode.WriteOnly, bmp.PixelFormat);
        Marshal.Copy(px, 0, data.Scan0, px.Length);
        bmp.UnlockBits(data);
        lock (images) images[id] = bmp;
    }

    /// A picture by path or URL; a URL loads in the background and repaints `owner` when it arrives.
    public static Image Pic(string src, Control owner)
    {
        Image img = Get("pic:" + src);
        if (img != null) return img;
        lock (loading) { if (!loading.Add(src)) return null; }
        ThreadPool.QueueUserWorkItem(delegate
        {
            try
            {
                byte[] bytes = File.Exists(src) ? File.ReadAllBytes(src) : new WebClient().DownloadData(src);
                Image loaded = Image.FromStream(new MemoryStream(bytes));
                lock (images) images["pic:" + src] = loaded;
                owner.BeginInvoke((Action)owner.Invalidate);
            }
            catch (Exception) { }
        });
        return null;
    }
}

// ---- Canvas --------------------------------------------------------------

/// The window's scale: device pixels per point (dpi / 96). The canvas works in points.
static class Dpi
{
    public static double Scale = 1;
    public static bool Hover; // std.app.16: the last `view` carried "hover":true

    [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr ctx);
    [DllImport("user32.dll")] static extern bool SetProcessDPIAware();
    [DllImport("user32.dll")] static extern uint GetDpiForWindow(IntPtr hwnd);

    /// Per-monitor v2 where Windows has it (10 1703+), else system DPI awareness.
    public static void Aware()
    {
        try { if (SetProcessDpiAwarenessContext((IntPtr)(-4))) { Refresh(IntPtr.Zero); return; } }
        catch (EntryPointNotFoundException) { }
        catch (DllNotFoundException) { }
        try { SetProcessDPIAware(); } catch (Exception) { }
        Refresh(IntPtr.Zero);
    }

    /// Reads the scale for a window (or the screen when `hwnd` is zero).
    public static void Refresh(IntPtr hwnd)
    {
        double dpi = 0;
        try { if (hwnd != IntPtr.Zero) dpi = GetDpiForWindow(hwnd); }
        catch (EntryPointNotFoundException) { }
        catch (DllNotFoundException) { }
        if (dpi < 48)
        {
            using (var bmp = new Bitmap(1, 1))
            using (var g = Graphics.FromImage(bmp)) dpi = g.DpiX;
        }
        Scale = Math.Max(1, dpi / 96.0);
    }

    public static int Px(double points) { return (int)Math.Round(points * Scale); }
}

static class Fonts
{
    static readonly Dictionary<string, Font> cache = new Dictionary<string, Font>();
    public const float CellSize = 13;
    static string sans;
    static string mono;

    static bool Installed(string face)
    {
        using (var fonts = new InstalledFontCollection())
            foreach (FontFamily f in fonts.Families)
                if (string.Equals(f.Name, face, StringComparison.OrdinalIgnoreCase)) return true;
        return false;
    }

    /// The UI face (std.app.34): Segoe UI Variable Text when it is installed and the chosen renderer draws its
    /// bold and italic at the widths it measures (GDI+ did not for the variable font, ADR 1034), else Segoe UI.
    public static string Sans
    {
        get
        {
            if (sans == null)
            {
                sans = "Segoe UI";
                try { if (Installed("Segoe UI Variable Text") && TextMode.Agrees("Segoe UI Variable Text")) sans = "Segoe UI Variable Text"; }
                catch (Exception) { }
            }
            return sans;
        }
    }

    /// Cascadia Mono when it is installed, else Consolas.
    static string Mono
    {
        get
        {
            if (mono == null)
            {
                mono = "Consolas";
                try { if (Installed("Cascadia Mono")) mono = "Cascadia Mono"; } catch (Exception) { }
            }
            return mono;
        }
    }

    /// Web fonts sent as `Font` ops (std.app.3): the file bytes live in memory for the process,
    /// and `web:<id>` draws in the family found in them.
    static readonly PrivateFontCollection web = new PrivateFontCollection();
    static readonly Dictionary<string, FontFamily> webFamilies = new Dictionary<string, FontFamily>();

    public static void Store(string id, string b64)
    {
        if (b64.Length == 0) return;
        try
        {
            byte[] bytes = Convert.FromBase64String(b64);
            lock (web)
            {
                int before = web.Families.Length;
                IntPtr mem = Marshal.AllocCoTaskMem(bytes.Length);
                Marshal.Copy(bytes, 0, mem, bytes.Length);
                web.AddMemoryFont(mem, bytes.Length);
                FontFamily[] all = web.Families;
                if (all.Length > before) webFamilies[id] = all[all.Length - 1];
            }
        }
        catch (Exception) { }
    }

    static Dictionary<string, string> installed;

    /// The installed families, lower-cased, for std.app.28.
    static Dictionary<string, string> Installed()
    {
        if (installed == null)
        {
            var d = new Dictionary<string, string>();
            using (var all = new InstalledFontCollection())
                foreach (FontFamily ff in all.Families) d[ff.Name.ToLowerInvariant()] = ff.Name;
            installed = d;
        }
        return installed;
    }

    /// std.app.28: the host face (`mono`, `sans`, `serif`, `web:` id) or installed family a
    /// `font` names: a CSS `font-family` list resolves to its first installed family, else its
    /// first generic, else `sans`.
    public static string Resolve(string family)
    {
        if (family == "mono" || family == "sans" || family == "serif" || family.StartsWith("web:")) return family;
        string generic = null;
        foreach (string part in family.Split(','))
        {
            string f = part.Trim();
            if (f.Length >= 2 && (f[0] == '"' || f[0] == '\'') && f[f.Length - 1] == f[0]) f = f.Substring(1, f.Length - 2).Trim();
            if (f.Length == 0) continue;
            string low = f.ToLowerInvariant();
            if (low == "serif") { if (generic == null) generic = "serif"; }
            else if (low == "monospace" || low == "ui-monospace" || low == "mono") { if (generic == null) generic = "mono"; }
            else if (low == "sans-serif" || low == "system-ui" || low == "ui-sans-serif" || low == "sans") { if (generic == null) generic = "sans"; }
            else
            {
                string name;
                if (Installed().TryGetValue(low, out name)) return name;
            }
        }
        return generic ?? "sans";
    }

    public static Font Get(string family0, double size, bool bold, bool italic, bool underline)
    {
        string family = Resolve(family0);
        float s = size > 0 ? (float)size : 13;
        string key = family + "|" + s + "|" + bold + italic + underline;
        Font f;
        if (!cache.TryGetValue(key, out f))
        {
            FontStyle st = (bold ? FontStyle.Bold : 0) | (italic ? FontStyle.Italic : 0) | (underline ? FontStyle.Underline : 0);
            FontFamily wf = null;
            if (family.StartsWith("web:")) lock (web) { webFamilies.TryGetValue(family.Substring(4), out wf); }
            if (wf != null)
            {
                FontStyle ws = wf.IsStyleAvailable(st) ? st : (FontStyle.Regular | (underline ? FontStyle.Underline : 0));
                f = new Font(wf, s, ws, GraphicsUnit.Pixel);
            }
            else
            {
                string name = family == "sans" || family.StartsWith("web:") ? Sans : family == "serif" ? "Georgia" : family == "mono" ? Mono : family;
                try { f = new Font(name, s, st, GraphicsUnit.Pixel); }
                catch (ArgumentException) { f = new Font(name, s, FontStyle.Regular | (underline ? FontStyle.Underline : 0), GraphicsUnit.Pixel); }
            }
            cache[key] = f;
        }
        return f;
    }

    public static Font Make(string face, float px, FontStyle st)
    {
        string key = "face|" + face + "|" + px + "|" + (int)st;
        Font f;
        if (!cache.TryGetValue(key, out f))
        {
            f = new Font(face, px, st, GraphicsUnit.Pixel);
            cache[key] = f;
        }
        return f;
    }

    /// The natural line height of `sans` at 13 points, whole points (std.app.34), like `cell_h` for `mono`.
    public static double SansLine()
    {
        Font f = Get("sans", CellSize, false, false, false);
        using (var bmp = new Bitmap(1, 1))
        using (var g = Graphics.FromImage(bmp))
            return Math.Ceiling(f.GetHeight(g));
    }
}

/// How text is drawn and measured (std.app.34). AIL_HOST_TEXT picks the variant while the choice is made by
/// capture: a = GDI+ ClearTypeGridFit (the first host), b = GDI+ AntiAliasGridFit, c = GDI+ AntiAlias with
/// TextContrast 0, d = GDI TextRenderer (what native Win32 programs get; whole-pixel positions).
static class TextMode
{
    public static readonly char Mode = Pick();

    static char Pick()
    {
        string v = Environment.GetEnvironmentVariable("AIL_HOST_TEXT");
        if (!string.IsNullOrEmpty(v) && "abcd".IndexOf(char.ToLowerInvariant(v[0])) >= 0) return char.ToLowerInvariant(v[0]);
        return 'a';
    }

    const TextFormatFlags Flags = TextFormatFlags.NoPadding | TextFormatFlags.NoPrefix | TextFormatFlags.PreserveGraphicsClipping |
                                  TextFormatFlags.Left | TextFormatFlags.Top | TextFormatFlags.SingleLine;

    public static void Apply(Graphics g)
    {
        switch (Mode)
        {
            case 'b': g.TextRenderingHint = TextRenderingHint.AntiAliasGridFit; break;
            case 'c': g.TextRenderingHint = TextRenderingHint.AntiAlias; g.TextContrast = 0; break;
            default: g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit; break;
        }
    }

    /// The advance width in points, trailing spaces counted.
    public static float Width(Graphics g, string text, Font f)
    {
        if (Mode == 'd')
            return TextRenderer.MeasureText(g, text, f, new Size(int.MaxValue, int.MaxValue), Flags).Width;
        var fmt = new StringFormat(StringFormat.GenericTypographic);
        fmt.FormatFlags |= StringFormatFlags.MeasureTrailingSpaces;
        return g.MeasureString(text, f, 100000, fmt).Width;
    }

    /// Top of the ascent line for a `Str` at y: the ascent-to-descent box is centred in the line box (std.app.34).
    public static float Lead(Font f)
    {
        FontFamily ff = f.FontFamily;
        FontStyle st = f.Style;
        float em = ff.GetEmHeight(st);
        if (em <= 0) return 0;
        float line = ff.GetLineSpacing(st), glyphs = ff.GetCellAscent(st) + ff.GetCellDescent(st);
        return Math.Max(0, (line - glyphs) / 2 * f.Size / em);
    }

    /// Draw `text` with its line box's top at (x, y) in the graphics' world space.
    public static void Draw(Graphics g, string text, Font f, Color c, float x, float y)
    {
        y += Lead(f);
        if (Mode != 'd')
        {
            using (var b = new SolidBrush(c))
                g.DrawString(text, f, b, x, y, StringFormat.GenericTypographic);
            return;
        }
        // TextRenderer ignores the world transform: draw in device space with a font of the device size.
        var p = new[] { new PointF(x, y) };
        g.TransformPoints(CoordinateSpace.Device, CoordinateSpace.World, p);
        float sc = g.Transform.Elements[0];
        Font df = Math.Abs(sc - 1) < 0.001f ? f : Fonts.Make(f.Name, f.Size * sc, f.Style);
        GraphicsState st = g.Save();
        g.ResetTransform();
        TextRenderer.DrawText(g, text, df, new Point((int)Math.Round(p[0].X), (int)Math.Round(p[0].Y)), c, Flags);
        g.Restore(st);
    }

    /// The width in px of the dark pixels of `text` drawn in `face`.
    static int Ink(string face, string text, FontStyle st, out float advance)
    {
        Font f = Fonts.Make(face, 13, st);
        using (var bmp = new Bitmap(300, 40))
        using (var g = Graphics.FromImage(bmp))
        {
            g.Clear(Color.White);
            Apply(g);
            advance = Width(g, text, f);
            Draw(g, text, f, Color.Black, 10, 8);
            int lo = int.MaxValue, hi = -1;
            for (int x = 0; x < bmp.Width; x++)
                for (int y = 0; y < bmp.Height; y++)
                    if (bmp.GetPixel(x, y).R < 200) { if (x < lo) lo = x; if (x > hi) hi = x; break; }
            return hi < lo ? 0 : hi - lo + 1;
        }
    }

    /// True when `face` is drawn at the widths it is measured at: how far the advance width exceeds the ink width
    /// (the side bearings) must be within 1 px for bold and italic of what it is for regular.
    public static bool Agrees(string face)
    {
        const string probe = "Hamburgefonstiv";
        float adv;
        int ink = Ink(face, probe, FontStyle.Regular, out adv);
        float regular = adv - ink;
        foreach (FontStyle st in new[] { FontStyle.Bold, FontStyle.Italic })
        {
            int w = Ink(face, probe, st, out adv);
            if (w == 0 || Math.Abs((adv - w) - regular) > 1) return false;
        }
        return ink > 0;
    }
}

// ---- Edge mode (std.app.25, std.app.33) ---------------------------------

/// What the host tells the app about its window controls while the app asks for edge mode:
/// three 46 x 32 point buttons at the top-right, so 138 points of room on the right.
static class EdgeMode
{
    public static bool On;
    public const double Right = 138, Top = 32;

    /// The `inset_*` members of `start` and `resize`; none outside edge mode (they read as 0).
    public static object[] Insets()
    {
        if (!On) return new object[0];
        return new object[] { "inset_left", 0.0, "inset_top", Top, "inset_right", Right };
    }

    public static object[] Cat(object[] a, object[] b)
    {
        var l = new List<object>(a);
        l.AddRange(b);
        return l.ToArray();
    }
}

/// Minimise, maximise or restore, and close, drawn in the Windows 11 style at the window's top-right
/// while the app owns the title bar (std.app.33). Glyphs come from Segoe Fluent Icons (Segoe MDL2
/// Assets before Windows 11), or are drawn as lines when neither is installed.
class CaptionButtons : Control
{
    readonly Form form;
    readonly Look look;
    readonly string iconFont;
    readonly float scale;
    readonly int bw, bh;
    int hot = -1, down = -1;

    static readonly char[] Glyphs = { '', '', '', '' }; // minimise, maximise, restore, close

    public CaptionButtons(Form f, Look l)
    {
        form = f;
        look = l;
        SetStyle(ControlStyles.UserPaint | ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer, true);
        TabStop = false;
        scale = (float)Dpi.Scale;
        bw = Dpi.Px(46);
        bh = Dpi.Px(32);
        Size = new Size(bw * 3, bh);
        Anchor = AnchorStyles.Top | AnchorStyles.Right;
        BackColor = l.Bg;
        iconFont = FindIconFont();
    }

    static string FindIconFont()
    {
        using (var fc = new InstalledFontCollection())
        {
            foreach (string want in new[] { "Segoe Fluent Icons", "Segoe MDL2 Assets" })
                foreach (FontFamily ff in fc.Families) if (ff.Name == want) return want;
        }
        return null;
    }

    int At(Point p) { return p.X < 0 || p.X >= Width || p.Y < 0 || p.Y >= Height ? -1 : p.X / bw; }

    protected override void OnMouseMove(MouseEventArgs e)
    {
        base.OnMouseMove(e);
        int i = At(e.Location);
        if (i != hot) { hot = i; Invalidate(); }
    }

    protected override void OnMouseLeave(EventArgs e)
    {
        base.OnMouseLeave(e);
        hot = -1;
        Invalidate();
    }

    protected override void OnMouseDown(MouseEventArgs e)
    {
        base.OnMouseDown(e);
        if (e.Button == MouseButtons.Left) { down = At(e.Location); Invalidate(); }
    }

    protected override void OnMouseUp(MouseEventArgs e)
    {
        base.OnMouseUp(e);
        int i = At(e.Location), was = down;
        down = -1;
        Invalidate();
        if (e.Button != MouseButtons.Left || i < 0 || i != was) return;
        if (i == 0) form.WindowState = FormWindowState.Minimized;
        else if (i == 1) form.WindowState = form.WindowState == FormWindowState.Maximized ? FormWindowState.Normal : FormWindowState.Maximized;
        else form.Close();
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        Graphics g = e.Graphics;
        g.Clear(look.Bg);
        bool max = form.WindowState == FormWindowState.Maximized;
        for (int i = 0; i < 3; i++)
        {
            var cell = new Rectangle(i * bw, 0, bw, bh);
            Color glyph = look.Fg;
            if (i == hot || i == down)
            {
                bool press = i == down && i == hot;
                if (i == 2)
                {
                    using (var b = new SolidBrush(Color.FromArgb(press ? 230 : 255, 0xC4, 0x2B, 0x1C))) g.FillRectangle(b, cell);
                    glyph = Color.White;
                }
                else using (var b = new SolidBrush(Color.FromArgb(press ? 15 : 26, look.Fg))) g.FillRectangle(b, cell);
            }
            DrawGlyph(g, cell, i == 1 && max ? 2 : i == 2 ? 3 : i, glyph);
        }
    }

    /// `kind`: 0 minimise, 1 maximise, 2 restore, 3 close.
    void DrawGlyph(Graphics g, Rectangle cell, int kind, Color c)
    {
        if (iconFont != null)
        {
            g.TextRenderingHint = TextRenderingHint.AntiAliasGridFit;
            using (var f = new Font(iconFont, 10f, FontStyle.Regular, GraphicsUnit.Point))
            using (var b = new SolidBrush(c))
            using (var fmt = new StringFormat { Alignment = StringAlignment.Center, LineAlignment = StringAlignment.Center })
                g.DrawString(Glyphs[kind].ToString(), f, b, cell, fmt);
            return;
        }
        // No icon font: lines, 10 points across, one device pixel wide.
        int s = (int)Math.Round(10 * scale) , cx = cell.X + cell.Width / 2, cy = cell.Y + cell.Height / 2;
        int x0 = cx - s / 2, y0 = cy - s / 2;
        using (var p = new Pen(c, 1f))
        {
            g.SmoothingMode = SmoothingMode.None;
            if (kind == 0) g.DrawLine(p, x0, cy, x0 + s, cy);
            else if (kind == 1) g.DrawRectangle(p, x0, y0, s, s);
            else if (kind == 2)
            {
                int d = (int)Math.Round(2 * scale);
                g.DrawRectangle(p, x0, y0 + d, s - d, s - d);
                g.DrawLines(p, new[] { new Point(x0 + d, y0 + d), new Point(x0 + d, y0), new Point(x0 + s, y0), new Point(x0 + s, y0 + s - d), new Point(x0 + s - d, y0 + s - d) });
            }
            else
            {
                g.SmoothingMode = SmoothingMode.AntiAlias;
                g.DrawLine(p, x0, y0, x0 + s, y0 + s);
                g.DrawLine(p, x0 + s, y0, x0, y0 + s);
            }
        }
    }
}

class CanvasView : Control
{
    public string CanvasId = "";
    public List<Op> Ops = new List<Op>();
    Size reported = Size.Empty;

    public CanvasView()
    {
        SetStyle(ControlStyles.AllPaintingInWmPaint | ControlStyles.UserPaint | ControlStyles.OptimizedDoubleBuffer |
                 ControlStyles.Selectable | ControlStyles.ResizeRedraw, true);
        TabStop = true;
        BackColor = Color.White; // replaced by the tree's first fill when the canvas is built
    }

    public static Color Rgb(int c) { return Color.FromArgb((c >> 16) & 0xff, (c >> 8) & 0xff, c & 0xff); }

    /// The monospace cell size a cell-based layout uses (std.app.5).
    public static void CellMetrics(out double w, out double h)
    {
        Font f = Fonts.Get("mono", Fonts.CellSize, false, false, false);
        using (var bmp = new Bitmap(1, 1))
        using (var g = Graphics.FromImage(bmp))
        {
            TextMode.Apply(g);
            w = TextMode.Width(g, "M", f);
            h = Math.Ceiling(f.GetHeight(g));
        }
    }

    /// Forgets the reported size so the next layout reports again (after a scale change).
    public void ReportAgain() { reported = Size.Empty; OnSizeChanged(EventArgs.Empty); }

    protected override void OnSizeChanged(EventArgs e)
    {
        base.OnSizeChanged(e);
        if (Size != reported && Width > 0 && Height > 0)
        {
            reported = Size;
            double cw, ch;
            CellMetrics(out cw, out ch);
            Outbox.Send("resize", EdgeMode.Cat(new object[] { "id", CanvasId, "dx", Width / Dpi.Scale, "dy", Height / Dpi.Scale, "cell_w", cw, "cell_h", ch, "screen_w", ScreenInfo.W, "screen_h", ScreenInfo.H, "scale", ScreenInfo.Scale, "sans_line", Fonts.SansLine() }, EdgeMode.Insets()));
        }
    }

    /// Fills the device-pixel rectangle a point rectangle covers: edges rounded, smoothing off, so
    /// neighbouring fills never leave a seam and never blur.
    static void FillSnapped(Graphics g, Brush b, double x, double y, double w, double h, double s)
    {
        int l = (int)Math.Round(x * s), t = (int)Math.Round(y * s);
        int r = (int)Math.Round((x + w) * s), bt = (int)Math.Round((y + h) * s);
        if (r <= l || bt <= t) return;
        g.ResetTransform(); // the clip is in device space already
        g.FillRectangle(b, l, t, r - l, bt - t);
        g.ScaleTransform((float)s, (float)s);
    }

    /// An axis-aligned stroke centred on the device pixels its width covers, so it stays crisp.
    static double SnapLine(double v, double width, double s)
    {
        double d = v * s;
        return (Math.Max(1, Math.Round(width * s)) % 2 == 1 ? Math.Floor(d) + 0.5 : Math.Round(d)) / s;
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        Graphics g = e.Graphics;
        g.Clear(BackColor);
        double s = Dpi.Scale;
        // Rectangles stay pixel-exact, slanted lines are anti-aliased; text per TextMode (std.app.34).
        TextMode.Apply(g);
        g.SmoothingMode = SmoothingMode.None;
        g.PixelOffsetMode = PixelOffsetMode.Half;
        g.InterpolationMode = InterpolationMode.HighQualityBicubic;
        g.ScaleTransform((float)s, (float)s);
        var saved = new Stack<GraphicsState>();
        foreach (Op op in Ops)
        {
            var fill = op as FillOp;
            if (fill != null)
            {
                using (var b = new SolidBrush(Rgb(fill.Color))) FillSnapped(g, b, fill.X, fill.Y, fill.W, fill.H, s);
                continue;
            }
            var shade = op as ShadeOp;
            if (shade != null)
            {
                Color c = Rgb(shade.Color);
                using (var b = new SolidBrush(Color.FromArgb((int)Math.Round(shade.Alpha * 255), c))) FillSnapped(g, b, shade.X, shade.Y, shade.W, shade.H, s);
                continue;
            }
            var str = op as StrOp;
            if (str != null)
            {
                TextMode.Draw(g, str.Text, Fonts.Get(str.Font, str.Size, str.Bold, str.Italic, str.Underline), Rgb(str.Fg),
                              (float)str.X, (float)str.Y);
                continue;
            }
            var stroke = op as StrokeOp;
            if (stroke != null)
            {
                double sw = stroke.Width > 0 ? stroke.Width : 1;
                double x1 = stroke.X1, y1 = stroke.Y1, x2 = stroke.X2, y2 = stroke.Y2;
                if (y1 == y2) { y1 = y2 = SnapLine(y1, sw, s); }
                else if (x1 == x2) { x1 = x2 = SnapLine(x1, sw, s); }
                using (var p = new Pen(Rgb(stroke.Color), (float)sw))
                {
                    g.SmoothingMode = SmoothingMode.AntiAlias;
                    g.DrawLine(p, (float)x1, (float)y1, (float)x2, (float)y2);
                    g.SmoothingMode = SmoothingMode.None;
                }
                continue;
            }
            // std.app.32: anti-aliased shapes, pixel corners at integer coordinates as in cairo and AppKit.
            var rnd = op as RoundOp;
            var poly = op as PolyOp;
            var line = op as LineOp;
            if (rnd != null || poly != null || line != null)
            {
                var smoothing = g.SmoothingMode;
                var offset = g.PixelOffsetMode;
                g.SmoothingMode = SmoothingMode.AntiAlias;
                g.PixelOffsetMode = PixelOffsetMode.None;
                if (rnd != null) DrawRound(g, rnd);
                else if (poly != null && poly.Points.Length >= 6)
                {
                    var pts = new PointF[poly.Points.Length / 2];
                    for (int i = 0; i < pts.Length; i++) pts[i] = new PointF(poly.Points[i * 2], poly.Points[i * 2 + 1]);
                    using (var b = new SolidBrush(Tint(poly.Color, poly.Alpha))) g.FillPolygon(b, pts, FillMode.Winding);
                }
                else if (line != null && line.Points.Length >= 4)
                {
                    var pts = new PointF[line.Points.Length / 2];
                    for (int i = 0; i < pts.Length; i++) pts[i] = new PointF(line.Points[i * 2], line.Points[i * 2 + 1]);
                    using (var p = new Pen(Rgb(line.Color), line.Width > 0 ? (float)line.Width : 1))
                    {
                        p.StartCap = LineCap.Round; p.EndCap = LineCap.Round; p.LineJoin = LineJoin.Round;
                        g.DrawLines(p, pts);
                    }
                }
                g.SmoothingMode = smoothing;
                g.PixelOffsetMode = offset;
                continue;
            }
            var pic = op as PicOp;
            var bm = op as BitmapOp;
            if (pic != null || bm != null)
            {
                double x = pic != null ? pic.X : bm.X, y = pic != null ? pic.Y : bm.Y;
                double w = pic != null ? pic.W : bm.W, h = pic != null ? pic.H : bm.H;
                Image img = pic != null ? Bitmaps.Pic(pic.Src, this) : Bitmaps.Get(bm.Id);
                if (img != null)
                {
                    // Pixel art blown up by a whole number of 4x or more stays sharp; everything else is smoothed.
                    double rx = w * s / img.Width, ry = h * s / img.Height;
                    bool crisp = rx >= 4 && ry >= 4 && Math.Abs(rx - Math.Round(rx)) < 0.001 && Math.Abs(ry - Math.Round(ry)) < 0.001;
                    g.InterpolationMode = crisp ? InterpolationMode.NearestNeighbor : InterpolationMode.HighQualityBicubic;
                    g.DrawImage(img, new RectangleF((float)x, (float)y, (float)w, (float)h));
                }
                else using (var b = new SolidBrush(Color.FromArgb(0xe8, 0xe8, 0xe8)))
                    FillSnapped(g, b, x, y, w, h, s);
                continue;
            }
            var clip = op as ClipOp;
            if (clip != null)
            {
                saved.Push(g.Save());
                g.SetClip(new RectangleF((float)clip.X, (float)clip.Y, (float)clip.W, (float)clip.H), CombineMode.Intersect);
                continue;
            }
            if (op is UnclipOp && saved.Count > 0) g.Restore(saved.Pop());
        }
    }

    static Color Tint(int rgb, double alpha)
    {
        double a = Math.Min(Math.Max(alpha, 0), 1);
        Color c = Rgb(rgb);
        return Color.FromArgb((int)Math.Round(a * 255), c.R, c.G, c.B);
    }

    // A rounded rectangle, radius at most half the shorter side; a stroke is centred on the edge (std.app.32).
    static void DrawRound(Graphics g, RoundOp r)
    {
        float x = (float)r.X, y = (float)r.Y, w = (float)r.W, h = (float)r.H;
        float rr = (float)Math.Min(Math.Max(r.R, 0), Math.Min(r.W, r.H) / 2);
        float d = rr * 2;
        using (var path = new GraphicsPath())
        {
            if (rr <= 0) path.AddRectangle(new RectangleF(x, y, w, h));
            else
            {
                path.AddArc(x, y, d, d, 180, 90);
                path.AddArc(x + w - d, y, d, d, 270, 90);
                path.AddArc(x + w - d, y + h - d, d, d, 0, 90);
                path.AddArc(x, y + h - d, d, d, 90, 90);
                path.CloseFigure();
            }
            if (r.Stroke > 0)
                using (var p = new Pen(Tint(r.Color, r.Alpha), (float)r.Stroke)) g.DrawPath(p, path);
            else
                using (var b = new SolidBrush(Tint(r.Color, r.Alpha))) g.FillPath(b, path);
        }
    }

    protected override bool IsInputKey(Keys k) { return true; }

    object[] ModArgs(params object[] more)
    {
        var l = new List<object>(more);
        Keys m = Control.ModifierKeys;
        l.AddRange(new object[] { "ctrl", (m & Keys.Control) != 0, "alt", (m & Keys.Alt) != 0, "shift", (m & Keys.Shift) != 0,
                                  "meta", (Control.ModifierKeys & Keys.LWin) != 0 });
        return l.ToArray();
    }

    static string ButtonName(MouseButtons b)
    {
        return b == MouseButtons.Left ? "left" : b == MouseButtons.Right ? "right" : "other";
    }

    // std.app.27 (ADR 0983): `down` before `click`, `up` on release, `move` while held;
    // Capture keeps them on this canvas until the button is released.
    /// The canvas whose pressed button has not been released yet, and which button (for `BeginDrag`).
    public static CanvasView Held;
    MouseButtons heldButton;

    /// The window manager took over a press (a window drag from the app's strip) and swallowed the
    /// release, so say it to the app: its `down` needs an `up` (std.app.27).
    public void ReleaseHeld()
    {
        if (Held != this) return;
        Held = null;
        Capture = false;
        Point p = PointToClient(Cursor.Position);
        Outbox.Send("up", ModArgs("id", CanvasId, "x", p.X / Dpi.Scale, "y", p.Y / Dpi.Scale, "text", ButtonName(heldButton), "clicks", 0));
    }

    protected override void OnMouseDown(MouseEventArgs e)
    {
        base.OnMouseDown(e);
        Focus();
        Capture = true;
        Held = this;
        heldButton = e.Button;
        Outbox.Send("down", ModArgs("id", CanvasId, "x", e.X / Dpi.Scale, "y", e.Y / Dpi.Scale, "text", ButtonName(e.Button), "clicks", Math.Min(Math.Max(e.Clicks, 1), 3)));
        Outbox.Send("click", ModArgs("id", CanvasId, "x", e.X / Dpi.Scale, "y", e.Y / Dpi.Scale, "text", ButtonName(e.Button), "clicks", Math.Min(Math.Max(e.Clicks, 1), 3)));
    }

    protected override void OnMouseUp(MouseEventArgs e)
    {
        base.OnMouseUp(e);
        Capture = false;
        if (Held == this) Held = null;
        Outbox.Send("up", ModArgs("id", CanvasId, "x", e.X / Dpi.Scale, "y", e.Y / Dpi.Scale, "text", ButtonName(e.Button), "clicks", 0));
    }

    // `move` while a button is held (std.app.27) or while the app asked for hover (std.app.16):
    // at most one per 16 ms, carrying the last position.
    DateTime lastMove = DateTime.MinValue;
    Point pending;
    bool hasPending;
    System.Windows.Forms.Timer flush;

    void SendMove(Point p)
    {
        lastMove = DateTime.Now;
        hasPending = false;
        Outbox.Send("move", "id", CanvasId, "x", p.X / Dpi.Scale, "y", p.Y / Dpi.Scale);
    }

    protected override void OnMouseMove(MouseEventArgs e)
    {
        base.OnMouseMove(e);
        if (e.Button == MouseButtons.None && !Dpi.Hover) return;
        if ((DateTime.Now - lastMove).TotalMilliseconds >= 16) { SendMove(e.Location); return; }
        pending = e.Location;
        hasPending = true;
        if (flush == null)
        {
            flush = new System.Windows.Forms.Timer { Interval = 16 };
            flush.Tick += delegate { flush.Stop(); if (hasPending) SendMove(pending); };
        }
        if (!flush.Enabled) flush.Start();
    }

    protected override void OnMouseLeave(EventArgs e)
    {
        base.OnMouseLeave(e);
        hasPending = false;
        if (Dpi.Hover) Outbox.Send("leave", "id", CanvasId);
    }

    protected override void OnMouseWheel(MouseEventArgs e)
    {
        base.OnMouseWheel(e);
        // Positive when the content should move up (scrolling down); one notch is 48 px.
        Outbox.Send("scroll", ModArgs("id", CanvasId, "dx", 0.0, "dy", -e.Delta / 120.0 * 48));
    }

    static string KeyName(Keys k)
    {
        switch (k)
        {
            case Keys.Enter: return "enter";
            case Keys.Tab: return "tab";
            case Keys.Back: return "backspace";
            case Keys.Delete: return "delete";
            case Keys.Escape: return "escape";
            case Keys.Up: return "up";
            case Keys.Down: return "down";
            case Keys.Left: return "left";
            case Keys.Right: return "right";
            case Keys.Home: return "home";
            case Keys.End: return "end";
            case Keys.PageUp: return "pageup";
            case Keys.PageDown: return "pagedown";
        }
        if (k >= Keys.F1 && k <= Keys.F12) return "f" + (k - Keys.F1 + 1);
        return null;
    }

    protected override void OnKeyDown(KeyEventArgs e)
    {
        base.OnKeyDown(e);
        // std.app.23 (ADR 0978): Control-V brings the clipboard's text as `paste`.
        if (e.Control && !e.Alt && e.KeyCode == Keys.V && Clipboard.ContainsText())
        {
            e.SuppressKeyPress = true;
            e.Handled = true;
            Outbox.Send("paste", ModArgs("id", CanvasId, "text", Clipboard.GetText()));
            return;
        }
        string name = KeyName(e.KeyCode);
        bool chord = e.Control || e.Alt;
        if (name == null && chord)
        {
            // The unshifted character while Control is held (std.ui.7).
            Keys c = e.KeyCode;
            if (c >= Keys.A && c <= Keys.Z) name = ((char)('a' + (c - Keys.A))).ToString();
            else if (c >= Keys.D0 && c <= Keys.D9) name = ((char)('0' + (c - Keys.D0))).ToString();
        }
        if (name == null) return;
        e.SuppressKeyPress = true;
        e.Handled = true;
        Outbox.Send("key", ModArgs("id", CanvasId, "text", name));
    }

    protected override void OnKeyPress(KeyPressEventArgs e)
    {
        base.OnKeyPress(e);
        if (e.KeyChar < ' ' || e.KeyChar == 0x7f) return;
        e.Handled = true;
        Outbox.Send("key", ModArgs("id", CanvasId, "text", e.KeyChar.ToString()));
    }
}

// ---- Look ----------------------------------------------------------------

/// The window's colors, taken from the app's own first canvas fill (its bar or page
/// background) so native controls and the title bar match what the app draws.
class Look
{
    public Color Bg, Fg, FieldBg, FieldHot, Accent;
    public bool Dark;

    public static Look From(Node tree)
    {
        Color? bg = FirstFill(tree);
        return Over(bg.HasValue ? bg.Value : Color.FromArgb(0xf3, 0xf3, 0xf3));
    }

    /// The look of controls sitting on a surface of colour `c`.
    public static Look Over(Color c)
    {
        bool dark = c.GetBrightness() < 0.5f;
        var l = new Look { Bg = c, Dark = dark };
        l.Fg = dark ? Color.FromArgb(0xf2, 0xf2, 0xf7) : Color.FromArgb(0x1f, 0x1f, 0x1f);
        l.FieldBg = Mix(c, dark ? Color.White : Color.FromArgb(0xff, 0xff, 0xff), dark ? 0.10f : 1f);
        l.FieldHot = Mix(c, dark ? Color.White : Color.Black, dark ? 0.18f : 0.08f);
        l.Accent = dark ? Color.FromArgb(0x0a, 0x84, 0xff) : Color.FromArgb(0x00, 0x67, 0xc0);
        return l;
    }

    public string Key() { return Bg.ToArgb().ToString("x8"); }

    static Color Mix(Color a, Color b, float t)
    {
        return Color.FromArgb((int)(a.R + (b.R - a.R) * t), (int)(a.G + (b.G - a.G) * t), (int)(a.B + (b.B - a.B) * t));
    }

    public static Color? FirstFill(Node n)
    {
        if (n.Kind == "Canvas")
        {
            foreach (Op op in n.Ops) { var f = op as FillOp; if (f != null) return CanvasView.Rgb(f.Color); }
            return null;
        }
        foreach (Node k in n.Kids) { Color? c = FirstFill(k); if (c.HasValue) return c; }
        return null;
    }

    /// True when the tree has a canvas that fills the window, which then needs no frame.
    public static bool Edge(Node n)
    {
        if (n.Kind == "Canvas" && (n.W <= 0 || n.H <= 0)) return true;
        foreach (Node k in n.Kids) if (Edge(k)) return true;
        return false;
    }
}

/// A text box set in a rounded, borderless pill that lights up with the accent while focused.
class FieldBox : Panel
{
    public readonly TextBox Box = new TextBox();
    readonly Look look;
    bool hot;

    public FieldBox(Look l, string fontFamily)
    {
        look = l;
        SetStyle(ControlStyles.UserPaint | ControlStyles.AllPaintingInWmPaint | ControlStyles.OptimizedDoubleBuffer | ControlStyles.ResizeRedraw, true);
        Height = Dpi.Px(32);
        Margin = new Padding(Dpi.Px(4), 0, Dpi.Px(8), 0);
        Anchor = AnchorStyles.Left | AnchorStyles.Right;
        Box.BorderStyle = BorderStyle.None;
        Box.Font = new Font(fontFamily, (float)(14 * Dpi.Scale), FontStyle.Regular, GraphicsUnit.Pixel);
        Box.BackColor = l.FieldBg;
        Box.ForeColor = l.Fg;
        Box.Anchor = AnchorStyles.Left | AnchorStyles.Right;
        Box.Left = Dpi.Px(12);
        Box.Width = Width - Dpi.Px(24);
        Box.Top = (Height - Box.Height) / 2;
        Box.Enter += delegate { Invalidate(); };
        Box.Leave += delegate { Invalidate(); };
        Controls.Add(Box);
        MouseEnter += delegate { hot = true; Invalidate(); };
        MouseLeave += delegate { hot = false; Invalidate(); };
        Click += delegate { Box.Focus(); };
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        Graphics g = e.Graphics;
        g.Clear(look.Bg);
        g.SmoothingMode = SmoothingMode.AntiAlias;
        Color fill = Box.Focused || !hot ? look.FieldBg : look.FieldHot;
        Box.BackColor = fill;
        var r = new RectangleF(0.5f, 0.5f, Width - 1, Height - 1);
        using (var path = Round(r, Height / 2f))
        {
            using (var b = new SolidBrush(fill)) g.FillPath(b, path);
            if (Box.Focused) using (var p = new Pen(look.Accent, 1.5f)) g.DrawPath(p, path);
            else if (!look.Dark) using (var p = new Pen(Color.FromArgb(0xd0, 0xd0, 0xd0), 1f)) g.DrawPath(p, path);
        }
    }

    static GraphicsPath Round(RectangleF r, float d)
    {
        var p = new GraphicsPath();
        p.AddArc(r.X, r.Y, d, d, 180, 90);
        p.AddArc(r.Right - d, r.Y, d, d, 270, 90);
        p.AddArc(r.Right - d, r.Bottom - d, d, d, 0, 90);
        p.AddArc(r.X, r.Bottom - d, d, d, 90, 90);
        p.CloseFigure();
        return p;
    }
}

// ---- Window --------------------------------------------------------------

class HostForm : Form
{
    string shape = "";
    Node current;
    bool started;
    bool quiet; // set while the host itself changes a control
    readonly Dictionary<string, string> lastSent = new Dictionary<string, string>();
    // The texts of the edits sent for a field since the program last answered with a value it chose itself.
    readonly Dictionary<string, List<string>> edits = new Dictionary<string, List<string>>();
    readonly Dictionary<string, Control> byId = new Dictionary<string, Control>();
    readonly List<Control> leaves = new List<Control>();
    Look look = Look.From(new Node());
    CaptionButtons caption; // only in edge mode (std.app.25, std.app.33)

    const int WM_NCCALCSIZE = 0x0083, WM_NCHITTEST = 0x0084, WM_NCLBUTTONDOWN = 0x00A1;
    const int HTCLIENT = 1, HTCAPTION = 2, HTTOP = 12, HTTOPLEFT = 13, HTTOPRIGHT = 14;
    const int SM_CYSIZEFRAME = 33, SM_CXPADDEDBORDER = 92;
    const int GRIP = 6; // the top edge that still resizes while the app owns the title bar
    const uint SWP_NOSIZE = 0x1, SWP_NOMOVE = 0x2, SWP_NOZORDER = 0x4, SWP_NOACTIVATE = 0x10, SWP_FRAMECHANGED = 0x20;

    [DllImport("user32.dll")] static extern int GetSystemMetrics(int index);
    [DllImport("user32.dll")] static extern bool ReleaseCapture();
    [DllImport("user32.dll")] static extern IntPtr SendMessage(IntPtr hwnd, int msg, IntPtr w, IntPtr l);
    [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr hwnd, IntPtr after, int x, int y, int w, int h, uint flags);

    [DllImport("dwmapi.dll")]
    static extern int DwmSetWindowAttribute(IntPtr hwnd, int attr, ref int val, int size);

    public HostForm()
    {
        Text = "AIL";
        ClientSize = new Size(Dpi.Px(1000), Dpi.Px(720));
        MinimumSize = new Size(Dpi.Px(340), Dpi.Px(240));
        Font = new Font(Fonts.Sans, (float)(14 * Dpi.Scale), FontStyle.Regular, GraphicsUnit.Pixel);
        SetIcon();
        StartPosition = FormStartPosition.CenterScreen;
        Padding = new Padding(Dpi.Px(8));
        BackColor = look.Bg;
        ForeColor = look.Fg;
    }

    /// AIL_APP_ICON names a PNG for the window and the taskbar.
    void SetIcon()
    {
        string path = Environment.GetEnvironmentVariable("AIL_APP_ICON");
        if (string.IsNullOrEmpty(path) || !File.Exists(path)) return;
        try { using (var bmp = new Bitmap(path)) Icon = Icon.FromHandle(bmp.GetHicon()); }
        catch (Exception) { }
    }

    protected override void OnHandleCreated(EventArgs e)
    {
        base.OnHandleCreated(e);
        double before = Dpi.Scale;
        Dpi.Refresh(Handle);
        if (Math.Abs(Dpi.Scale - before) > 0.001)
        {
            // The window is on a monitor with another scale than the primary one.
            ClientSize = new Size(Dpi.Px(1000), Dpi.Px(720));
            MinimumSize = new Size(Dpi.Px(340), Dpi.Px(240));
            Font = new Font(Fonts.Sans, (float)(14 * Dpi.Scale), FontStyle.Regular, GraphicsUnit.Pixel);
        }
        Chrome();
    }

    // WM_DPICHANGED: take the suggested rectangle, rebuild at the new scale, report the new sizes.
    protected override void WndProc(ref Message m)
    {
        if (m.Msg == 0x02E0)
        {
            Dpi.Refresh(Handle);
            var r = (RECT)Marshal.PtrToStructure(m.LParam, typeof(RECT));
            SetBounds(r.L, r.T, r.R - r.L, r.B - r.T);
            MinimumSize = new Size(Dpi.Px(340), Dpi.Px(240));
            Font = new Font(Fonts.Sans, (float)(14 * Dpi.Scale), FontStyle.Regular, GraphicsUnit.Pixel);
            if (current != null)
            {
                shape = ""; // rebuild the controls at the new scale
                ShowView(Text, "", current, Dpi.Hover, EdgeMode.On);
            }
            foreach (Control c in byId.Values)
            {
                var cv = c as CanvasView;
                if (cv != null) { cv.ReportAgain(); cv.Invalidate(); }
            }
            return;
        }
        EdgeProc(ref m);
    }

    [StructLayout(LayoutKind.Sequential)]
    struct RECT { public int L, T, R, B; }

    /// Dark or light title bar in the window's own color (Windows 10 1809+ and 11; ignored elsewhere).
    void Chrome()
    {
        if (!IsHandleCreated) return;
        try
        {
            int dark = look.Dark ? 1 : 0;
            if (DwmSetWindowAttribute(Handle, 20, ref dark, 4) != 0) DwmSetWindowAttribute(Handle, 19, ref dark, 4);
            int cap = look.Bg.R | (look.Bg.G << 8) | (look.Bg.B << 16);
            int txt = look.Fg.R | (look.Fg.G << 8) | (look.Fg.B << 16);
            DwmSetWindowAttribute(Handle, 35, ref cap, 4);  // DWMWA_CAPTION_COLOR
            DwmSetWindowAttribute(Handle, 34, ref cap, 4);  // DWMWA_BORDER_COLOR
            DwmSetWindowAttribute(Handle, 36, ref txt, 4);  // DWMWA_TEXT_COLOR
        }
        catch (DllNotFoundException) { }
        catch (EntryPointNotFoundException) { }
    }

    /// Edge mode keeps the frame (shadow, snapping, resize edges, rounded corners) but gives the client
    /// area the title bar's room: WM_NCCALCSIZE leaves the top at the window's top.
    void EdgeProc(ref Message m)
    {
        if (EdgeMode.On && m.Msg == WM_NCCALCSIZE && m.WParam != IntPtr.Zero)
        {
            int top = Marshal.ReadInt32(m.LParam, 4); // rgrc[0].top: left, top, right, bottom
            base.WndProc(ref m);
            int inset = WindowState == FormWindowState.Maximized ? GetSystemMetrics(SM_CYSIZEFRAME) + GetSystemMetrics(SM_CXPADDEDBORDER) : 0;
            Marshal.WriteInt32(m.LParam, 4, top + inset);
            return;
        }
        base.WndProc(ref m);
        if (EdgeMode.On && m.Msg == WM_NCHITTEST && (int)m.Result == HTCLIENT && WindowState == FormWindowState.Normal)
        {
            long l = m.LParam.ToInt64();
            Point p = PointToClient(new Point((short)(l & 0xffff), (short)((l >> 16) & 0xffff)));
            if (p.Y >= 0 && p.Y < GRIP)
            {
                int right = ClientSize.Width - (caption != null ? caption.Width : 0);
                if (p.X < GRIP) m.Result = (IntPtr)HTTOPLEFT;
                else if (p.X >= ClientSize.Width - GRIP) m.Result = (IntPtr)HTTOPRIGHT;
                else if (p.X < right) m.Result = (IntPtr)HTTOP; // the buttons keep their top rows
            }
        }
    }

    protected override void OnResize(EventArgs e)
    {
        base.OnResize(e);
        if (caption != null) caption.Invalidate(); // maximise or restore glyph
    }

    void SetEdge(bool on)
    {
        if (EdgeMode.On == on) return;
        EdgeMode.On = on;
        // Make Windows ask for the client area again, with or without the title bar.
        if (IsHandleCreated)
            SetWindowPos(Handle, IntPtr.Zero, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED);
    }

    /// `{"t":"drag"}` (std.app.25): the press in the app's strip moves the window, as a title bar would.
    public void BeginDrag()
    {
        CanvasView held = CanvasView.Held;
        if (held != null && (Control.MouseButtons & MouseButtons.Left) != 0)
        {
            ReleaseCapture();
            SendMessage(Handle, WM_NCLBUTTONDOWN, (IntPtr)HTCAPTION, IntPtr.Zero);
        }
        if (held != null) held.ReleaseHeld();
    }

    /// `{"t":"zoom"}`: maximise, or restore.
    public void Zoom()
    {
        WindowState = WindowState == FormWindowState.Maximized ? FormWindowState.Normal : FormWindowState.Maximized;
    }

    protected override void OnFormClosing(FormClosingEventArgs e)
    {
        base.OnFormClosing(e);
        Outbox.Send("close");
    }

    // std.app.30 (ADR 1008): "dark" or "light" from the apps theme setting, once after `start`
    // and on every change.
    string appearance = "";

    static string SystemAppearance()
    {
        try
        {
            object v = Microsoft.Win32.Registry.GetValue(
                @"HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize", "AppsUseLightTheme", 1);
            if (v is int && (int)v == 0) return "dark";
        }
        catch (Exception) { }
        return "light";
    }

    void SendAppearance()
    {
        string now = SystemAppearance();
        if (now == appearance) return;
        appearance = now;
        Outbox.Send("appearance", "text", now);
    }

    public void ShowView(string title, string focus, Node tree, bool hover, bool edge)
    {
        Dpi.Hover = hover;
        Text = title;
        SetEdge(edge && Look.Edge(tree));
        Look next = Look.From(tree);
        string s = tree.Shape() + "|" + next.Key() + (EdgeMode.On ? "|edge" : "");
        string wantFocus = focus;
        // The first line to the program is `start`, ahead of any `resize` the layout causes.
        if (!started)
        {
            started = true;
            Outbox.Send("start", EdgeMode.Cat(new object[] { "screen_w", ScreenInfo.W, "screen_h", ScreenInfo.H, "scale", ScreenInfo.Scale, "sans_line", Fonts.SansLine() }, EdgeMode.Insets()));
            SendAppearance();
            Microsoft.Win32.SystemEvents.UserPreferenceChanged += delegate { SendAppearance(); };
        }
        if (s != shape)
        {
            string keep = ActiveControl != null && ActiveControl.Tag is string ? (string)ActiveControl.Tag : "";
            if (wantFocus.Length == 0) wantFocus = keep;
            SuspendLayout();
            look = next;
            BackColor = look.Bg;
            ForeColor = look.Fg;
            Padding = Look.Edge(tree) ? new Padding(0) : new Padding(Dpi.Px(8));
            Chrome();
            var old = new Control[Controls.Count];
            Controls.CopyTo(old, 0);
            Controls.Clear();
            foreach (Control o in old) o.Dispose();
            byId.Clear();
            leaves.Clear();
            Control root = Build(tree, true);
            root.Dock = DockStyle.Fill;
            Controls.Add(root);
            caption = null;
            if (EdgeMode.On)
            {
                caption = new CaptionButtons(this, look);
                caption.Location = new Point(ClientSize.Width - caption.Width, 0);
                Controls.Add(caption);
                caption.BringToFront();
            }
            ResumeLayout(true);
            shape = s;
        }
        current = tree;
        int idx = 0;
        quiet = true;
        Apply(tree, ref idx);
        quiet = false;
        if (wantFocus.Length > 0)
        {
            Control c;
            if (byId.TryGetValue(wantFocus, out c) && !c.Focused) c.Focus();
        }
    }

    /// Pushes the new values into the controls of an unchanged shape, in tree order.
    void Apply(Node n, ref int idx)
    {
        switch (n.Kind)
        {
            case "Label": leaves[idx++].Text = n.Text; break;
            case "Field":
                var tb = (TextBox)leaves[idx++];
                string last;
                // A view that echoes an edit sent before the newest one is out of date: typing has gone on
                // since, so it must not overwrite the box. One that echoes the newest edit is in step. Any
                // other value is the program's own choice and wins.
                List<string> es;
                if (edits.TryGetValue(n.Id, out es) && es.Count > 0)
                {
                    int at = es.LastIndexOf(n.Text);
                    if (at >= 0)
                    {
                        if (at == es.Count - 1) { es.Clear(); lastSent[n.Id] = n.Text; }
                        else es.RemoveRange(0, at + 1);
                        break;
                    }
                    es.Clear();
                }
                // The user's typing stands until the program sends a different value than last time.
                if (!lastSent.TryGetValue(n.Id, out last) || last != n.Text)
                {
                    lastSent[n.Id] = n.Text;
                    if (tb.Text != n.Text) { tb.Text = n.Text; tb.SelectionStart = tb.Text.Length; }
                }
                break;
            case "Toggle": ((CheckBox)leaves[idx++]).Checked = n.On; break;
            case "Button": idx++; break;
            case "Canvas":
                var cv = (CanvasView)leaves[idx++];
                cv.Ops = n.Ops;
                cv.Invalidate();
                break;
            case "Spacer": break;
            default: foreach (Node k in n.Kids) Apply(k, ref idx); break;
        }
    }

    /// True when a node wants the leftover space along an axis.
    static bool Grows(Node n, bool vertical)
    {
        if (n.Kind == "Spacer") return true;
        if (n.Kind == "Canvas") return vertical ? n.H <= 0 : n.W <= 0;
        if (n.Kind == "Field") return !vertical;
        foreach (Node k in n.Kids)
            if (Grows(k, vertical)) return true;
        return false;
    }

    Control Build(Node n, bool root = false, bool parentVertical = true)
    {
        switch (n.Kind)
        {
            case "Label":
                var l = new Label { Text = n.Text, AutoSize = true, Margin = new Padding(8, 6, 8, 6), ForeColor = look.Fg, BackColor = look.Bg };
                leaves.Add(l);
                return l;
            case "Button":
                var b = new Button
                {
                    Text = n.Text, AutoSize = true, AutoSizeMode = AutoSizeMode.GrowAndShrink, Tag = n.Id, Margin = new Padding(4, 3, 4, 3),
                    Padding = new Padding(10, 3, 10, 3), FlatStyle = FlatStyle.Flat, UseVisualStyleBackColor = false,
                    BackColor = look.FieldBg, ForeColor = look.Fg, Cursor = Cursors.Hand
                };
                b.FlatAppearance.BorderSize = look.Dark ? 0 : 1;
                b.FlatAppearance.BorderColor = Color.FromArgb(0xd0, 0xd0, 0xd0);
                b.FlatAppearance.MouseOverBackColor = look.FieldHot;
                b.FlatAppearance.MouseDownBackColor = look.Accent;
                string bid = n.Id;
                b.Click += delegate { Outbox.Send("press", "id", bid); };
                byId[n.Id] = b;
                leaves.Add(b);
                return b;
            case "Field":
                var box = new FieldBox(look, Fonts.Sans) { MinimumSize = new Size(120, 0) };
                var t = box.Box;
                t.Tag = n.Id;
                string fid = n.Id;
                t.TextChanged += delegate
                {
                    if (quiet) return;
                    lastSent[fid] = null;
                    List<string> es;
                    if (!edits.TryGetValue(fid, out es)) { es = new List<string>(); edits[fid] = es; }
                    if (es.Count >= 256) es.RemoveAt(0);
                    es.Add(t.Text);
                    Outbox.Send("edit", "id", fid, "text", t.Text);
                };
                t.KeyDown += delegate (object s, KeyEventArgs e)
                {
                    if (e.KeyCode == Keys.Enter) { e.SuppressKeyPress = true; Outbox.Send("submit", "id", fid); }
                };
                SetHint(t, n.Hint);
                byId[n.Id] = t;
                leaves.Add(t);
                return box;
            case "Toggle":
                var c = new CheckBox { Text = n.Text, AutoSize = true, Checked = n.On, Tag = n.Id, Margin = new Padding(8, 6, 8, 6), ForeColor = look.Fg, BackColor = look.Bg };
                string tid = n.Id;
                c.CheckedChanged += delegate { if (!quiet) Outbox.Send("toggle", "id", tid, "text", c.Checked ? "on" : "off"); };
                byId[n.Id] = c;
                leaves.Add(c);
                return c;
            case "Canvas":
                var cv = new CanvasView { CanvasId = n.Id, Tag = n.Id, Ops = n.Ops, Margin = new Padding(0) };
                Color? first = Look.FirstFill(n);
                cv.BackColor = first.HasValue ? first.Value : look.Bg;
                if (n.W > 0) cv.Width = Dpi.Px(n.W);
                if (n.H > 0) cv.Height = Dpi.Px(n.H);
                cv.Dock = n.W > 0 && n.H > 0 ? DockStyle.None : DockStyle.Fill;
                byId[n.Id] = cv;
                leaves.Add(cv);
                return cv;
            case "Spacer":
                return new Panel { Margin = new Padding(0), Size = new Size(1, 1), Dock = DockStyle.Fill };
            default:
                return BuildBox(n, root, parentVertical);
        }
    }

    Control BuildBox(Node n, bool root, bool parentVertical)
    {
        // A box whose canvases are painted in one colour is a surface of that colour (a toolbar under a darker tab strip).
        Look outer = look;
        Color? surface = root ? null : Look.FirstFill(n);
        if (surface.HasValue) look = Look.Over(surface.Value);
        try { return BuildBoxIn(n, root, parentVertical); }
        finally { look = outer; }
    }

    Control BuildBoxIn(Node n, bool root, bool parentVertical)
    {
        bool vertical = n.Kind == "Column";
        // The window's own box fills it; a nested box takes its content's size unless it grows.
        bool auto = !root && !Grows(n, parentVertical);
        var t = new TableLayoutPanel { Margin = new Padding(0), Padding = new Padding(0), AutoSize = auto, Dock = DockStyle.Fill, BackColor = look.Bg };
        t.GrowStyle = TableLayoutPanelGrowStyle.FixedSize;
        int count = n.Kids.Count;
        if (vertical) { t.ColumnCount = 1; t.RowCount = count; t.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); }
        else { t.RowCount = 1; t.ColumnCount = count; t.RowStyles.Add(new RowStyle(SizeType.Percent, 100)); }
        int grow = 0;
        foreach (Node k in n.Kids) if (Grows(k, vertical)) grow++;
        for (int i = 0; i < count; i++)
        {
            Node k = n.Kids[i];
            Control c = Build(k, false, vertical);
            bool g = Grows(k, vertical);
            SizeType st = g ? SizeType.Percent : SizeType.AutoSize;
            float amount = g ? 100f / grow : 0;
            if (vertical) t.RowStyles.Add(new RowStyle(st, amount)); else t.ColumnStyles.Add(new ColumnStyle(st, amount));
            if (vertical && !(c is CanvasView && c.Dock == DockStyle.None) && !(c is Button) && !(c is CheckBox))
                c.Dock = c.Dock == DockStyle.Fill ? DockStyle.Fill : DockStyle.Top;
            t.Controls.Add(c, vertical ? 0 : i, vertical ? i : 0);
        }
        return t;
    }

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    static extern IntPtr SendMessage(IntPtr h, int msg, IntPtr w, string l);

    static void SetHint(TextBox t, string hint)
    {
        if (hint.Length == 0) return;
        EventHandler apply = delegate { SendMessage(t.Handle, 0x1501, (IntPtr)1, hint); }; // EM_SETCUEBANNER
        if (t.IsHandleCreated) apply(null, null);
        else t.HandleCreated += apply;
    }
}

// ---- Entry ---------------------------------------------------------------

static class Program
{
    /// std.app.13: the advance width of each requested run, in the font a `Str` op of the
    /// same family, size and weight is drawn in, answered as one `measured` event.
    static void AnswerMeasure(List<object> runs)
    {
        var widths = new List<string>();
        var fonts = new List<string>();
        using (var bmp = new Bitmap(1, 1))
        using (var g = Graphics.FromImage(bmp))
        {
            TextMode.Apply(g);
            // A run that ends in a space (the text before a bold or linked word) must count it.
            foreach (object r in runs)
            {
                string font = Json.Text(Json.Get(r, "font"));
                Font f = Fonts.Get(font, Json.Num(Json.Get(r, "size")),
                                   Json.Flag(Json.Get(r, "bold")), Json.Flag(Json.Get(r, "italic")), false);
                float w = TextMode.Width(g, Json.Text(Json.Get(r, "text")), f);
                widths.Add(w.ToString("0.000", CultureInfo.InvariantCulture));
                fonts.Add(Fonts.Resolve(font).Replace(',', ' '));
            }
        }
        // std.app.28: the family each run was measured in.
        Outbox.Send("measured", "text", string.Join(",", widths.ToArray()), "fonts", string.Join(",", fonts.ToArray()));
    }

    [STAThread]
    static void Main()
    {
        Dpi.Aware();
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        var form = new HostForm();
        var reader = new Thread(delegate ()
        {
            var input = new StreamReader(Console.OpenStandardInput(), new UTF8Encoding(false));
            bool first = true;
            string line;
            while ((line = input.ReadLine()) != null)
            {
                object obj = Json.Parse(line);
                string kind = Json.Text(Json.Get(obj, "t"));
                if (kind == "pick")
                {
                    // std.app.22: no native chooser on this host yet; say so rather than hang.
                    Outbox.Send("pick_error", "id", Json.Text(Json.Get(obj, "id")), "text", "this host has no native file chooser yet");
                    continue;
                }
                if (kind == "clipboard")
                {
                    // std.app.26 (ADR 0982): the app puts text on the system clipboard (an STA call, so on the form's thread).
                    string t = Json.Text(Json.Get(obj, "text"));
                    form.BeginInvoke((Action)delegate { try { if (t.Length > 0) Clipboard.SetText(t); else Clipboard.Clear(); } catch (Exception) { } });
                    continue;
                }
                if (kind == "drag") { form.BeginInvoke((Action)form.BeginDrag); continue; }
                if (kind == "zoom") { form.BeginInvoke((Action)form.Zoom); continue; }
                if (kind == "measure") { AnswerMeasure(Json.Get(obj, "runs") as List<object> ?? new List<object>()); continue; }
                if (kind != "view") continue;
                if (first)
                {
                    first = false;
                    if (Json.Num(Json.Get(obj, "v")) != 1)
                    {
                        Console.Error.WriteLine("AIL App Host: the program speaks another protocol version");
                        break;
                    }
                }
                string title = Json.Text(Json.Get(obj, "title"));
                string focus = Json.Text(Json.Get(obj, "focus"));
                bool hover = Json.Flag(Json.Get(obj, "hover"));
                bool edge = Json.Flag(Json.Get(obj, "edge"));
                Node tree = Node.Decode(Json.Get(obj, "tree"));
                form.BeginInvoke((Action)delegate { form.ShowView(title.Length > 0 ? title : "AIL", focus, tree, hover, edge); });
            }
            form.BeginInvoke((Action)delegate { Application.Exit(); });
        });
        reader.IsBackground = true;
        reader.Start();
        Application.Run(form);
    }
}
