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
class StrokeOp : Op { public double X1, Y1, X2, Y2, Width; public int Color; }
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
            case "Stroke":
                return new StrokeOp { X1 = Json.Num(Json.Get(d, "x1")), Y1 = Json.Num(Json.Get(d, "y1")), X2 = Json.Num(Json.Get(d, "x2")),
                    Y2 = Json.Num(Json.Get(d, "y2")), Color = color, Width = Json.Num(Json.Get(d, "width")) };
            case "Pic": return new PicOp { X = X, Y = Y, W = W, H = H, Src = Json.Text(Json.Get(d, "src")) };
            case "Bitmap":
                string id = Json.Text(Json.Get(d, "id"));
                Bitmaps.Store(id, (int)Json.Num(Json.Get(d, "pw")), (int)Json.Num(Json.Get(d, "ph")), Json.Text(Json.Get(d, "data")));
                return new BitmapOp { X = X, Y = Y, W = W, H = H, Id = id };
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

static class Fonts
{
    static readonly Dictionary<string, Font> cache = new Dictionary<string, Font>();
    public const float CellSize = 13;

    public static Font Get(string family, double size, bool bold, bool italic, bool underline)
    {
        float s = size > 0 ? (float)size : 13;
        string key = family + "|" + s + "|" + bold + italic + underline;
        Font f;
        if (!cache.TryGetValue(key, out f))
        {
            FontStyle st = (bold ? FontStyle.Bold : 0) | (italic ? FontStyle.Italic : 0) | (underline ? FontStyle.Underline : 0);
            f = new Font(family == "sans" ? "Segoe UI" : "Consolas", s, st, GraphicsUnit.Pixel);
            cache[key] = f;
        }
        return f;
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
        BackColor = Color.White;
    }

    public static Color Rgb(int c) { return Color.FromArgb((c >> 16) & 0xff, (c >> 8) & 0xff, c & 0xff); }

    /// The monospace cell size a cell-based layout uses (std.app.5).
    public static void CellMetrics(out double w, out double h)
    {
        Font f = Fonts.Get("mono", Fonts.CellSize, false, false, false);
        using (var bmp = new Bitmap(1, 1))
        using (var g = Graphics.FromImage(bmp))
        {
            w = g.MeasureString("M", f, 10000, StringFormat.GenericTypographic).Width;
            h = Math.Ceiling(f.GetHeight(g));
        }
    }

    protected override void OnSizeChanged(EventArgs e)
    {
        base.OnSizeChanged(e);
        if (Size != reported && Width > 0 && Height > 0)
        {
            reported = Size;
            double cw, ch;
            CellMetrics(out cw, out ch);
            Outbox.Send("resize", "id", CanvasId, "dx", Width, "dy", Height, "cell_w", cw, "cell_h", ch);
        }
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        Graphics g = e.Graphics;
        g.Clear(Color.White);
        g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;
        g.SmoothingMode = SmoothingMode.None;
        g.InterpolationMode = InterpolationMode.HighQualityBilinear;
        var saved = new Stack<GraphicsState>();
        foreach (Op op in Ops)
        {
            var fill = op as FillOp;
            if (fill != null)
            {
                using (var b = new SolidBrush(Rgb(fill.Color)))
                    g.FillRectangle(b, (float)fill.X, (float)fill.Y, (float)fill.W, (float)fill.H);
                continue;
            }
            var str = op as StrOp;
            if (str != null)
            {
                using (var b = new SolidBrush(Rgb(str.Fg)))
                    g.DrawString(str.Text, Fonts.Get(str.Font, str.Size, str.Bold, str.Italic, str.Underline), b,
                                 (float)str.X, (float)str.Y, StringFormat.GenericTypographic);
                continue;
            }
            var stroke = op as StrokeOp;
            if (stroke != null)
            {
                using (var p = new Pen(Rgb(stroke.Color), stroke.Width > 0 ? (float)stroke.Width : 1))
                    g.DrawLine(p, (float)stroke.X1, (float)stroke.Y1, (float)stroke.X2, (float)stroke.Y2);
                continue;
            }
            var pic = op as PicOp;
            var bm = op as BitmapOp;
            if (pic != null || bm != null)
            {
                double x = pic != null ? pic.X : bm.X, y = pic != null ? pic.Y : bm.Y;
                double w = pic != null ? pic.W : bm.W, h = pic != null ? pic.H : bm.H;
                Image img = pic != null ? Bitmaps.Pic(pic.Src, this) : Bitmaps.Get(bm.Id);
                if (img != null) g.DrawImage(img, new RectangleF((float)x, (float)y, (float)w, (float)h));
                else using (var b = new SolidBrush(Color.FromArgb(0xe8, 0xe8, 0xe8)))
                    g.FillRectangle(b, (float)x, (float)y, (float)w, (float)h);
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

    protected override bool IsInputKey(Keys k) { return true; }

    object[] ModArgs(params object[] more)
    {
        var l = new List<object>(more);
        Keys m = Control.ModifierKeys;
        l.AddRange(new object[] { "ctrl", (m & Keys.Control) != 0, "alt", (m & Keys.Alt) != 0, "shift", (m & Keys.Shift) != 0,
                                  "meta", (Control.ModifierKeys & Keys.LWin) != 0 });
        return l.ToArray();
    }

    protected override void OnMouseDown(MouseEventArgs e)
    {
        base.OnMouseDown(e);
        Focus();
        string button = e.Button == MouseButtons.Left ? "left" : e.Button == MouseButtons.Right ? "right" : "other";
        Outbox.Send("click", ModArgs("id", CanvasId, "x", e.X, "y", e.Y, "text", button));
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

// ---- Window --------------------------------------------------------------

class HostForm : Form
{
    string shape = "";
    Node current;
    bool started;
    bool quiet; // set while the host itself changes a control
    readonly Dictionary<string, string> lastSent = new Dictionary<string, string>();
    readonly Dictionary<string, Control> byId = new Dictionary<string, Control>();
    readonly List<Control> leaves = new List<Control>();

    public HostForm()
    {
        Text = "AIL";
        ClientSize = new Size(1000, 720);
        MinimumSize = new Size(340, 240);
        Font = SystemFonts.MessageBoxFont;
        StartPosition = FormStartPosition.CenterScreen;
        Padding = new Padding(8);
    }

    protected override void OnFormClosing(FormClosingEventArgs e)
    {
        base.OnFormClosing(e);
        Outbox.Send("close");
    }

    public void ShowView(string title, string focus, Node tree)
    {
        Text = title;
        string s = tree.Shape();
        string wantFocus = focus;
        if (s != shape)
        {
            string keep = ActiveControl != null && ActiveControl.Tag is string ? (string)ActiveControl.Tag : "";
            if (wantFocus.Length == 0) wantFocus = keep;
            SuspendLayout();
            Controls.Clear();
            byId.Clear();
            leaves.Clear();
            Control root = Build(tree);
            root.Dock = DockStyle.Fill;
            Controls.Add(root);
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
        if (!started)
        {
            started = true;
            Outbox.Send("start");
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

    Control Build(Node n)
    {
        switch (n.Kind)
        {
            case "Label":
                var l = new Label { Text = n.Text, AutoSize = true, Margin = new Padding(3) };
                leaves.Add(l);
                return l;
            case "Button":
                var b = new Button { Text = n.Text, AutoSize = true, Tag = n.Id, Margin = new Padding(3) };
                string bid = n.Id;
                b.Click += delegate { Outbox.Send("press", "id", bid); };
                byId[n.Id] = b;
                leaves.Add(b);
                return b;
            case "Field":
                var t = new TextBox { Tag = n.Id, Dock = DockStyle.Fill, Margin = new Padding(3), MinimumSize = new Size(120, 0) };
                string fid = n.Id;
                t.TextChanged += delegate { if (!quiet) { lastSent[fid] = null; Outbox.Send("edit", "id", fid, "text", t.Text); } };
                t.KeyDown += delegate (object s, KeyEventArgs e)
                {
                    if (e.KeyCode == Keys.Enter) { e.SuppressKeyPress = true; Outbox.Send("submit", "id", fid); }
                };
                SetHint(t, n.Hint);
                byId[n.Id] = t;
                leaves.Add(t);
                return t;
            case "Toggle":
                var c = new CheckBox { Text = n.Text, AutoSize = true, Checked = n.On, Tag = n.Id, Margin = new Padding(3) };
                string tid = n.Id;
                c.CheckedChanged += delegate { if (!quiet) Outbox.Send("toggle", "id", tid, "text", c.Checked ? "on" : "off"); };
                byId[n.Id] = c;
                leaves.Add(c);
                return c;
            case "Canvas":
                var cv = new CanvasView { CanvasId = n.Id, Tag = n.Id, Ops = n.Ops, Margin = new Padding(0) };
                if (n.W > 0) cv.Width = (int)n.W;
                if (n.H > 0) cv.Height = (int)n.H;
                cv.Dock = n.W > 0 && n.H > 0 ? DockStyle.None : DockStyle.Fill;
                byId[n.Id] = cv;
                leaves.Add(cv);
                return cv;
            case "Spacer":
                return new Panel { Margin = new Padding(0), Size = new Size(1, 1), Dock = DockStyle.Fill };
            default:
                return BuildBox(n);
        }
    }

    Control BuildBox(Node n)
    {
        bool vertical = n.Kind == "Column";
        var t = new TableLayoutPanel { Margin = new Padding(0), Padding = new Padding(0), AutoSize = true, Dock = DockStyle.Fill };
        t.GrowStyle = TableLayoutPanelGrowStyle.FixedSize;
        int count = n.Kids.Count;
        if (vertical) { t.ColumnCount = 1; t.RowCount = count; t.ColumnStyles.Add(new ColumnStyle(SizeType.Percent, 100)); }
        else { t.RowCount = 1; t.ColumnCount = count; t.RowStyles.Add(new RowStyle(SizeType.Percent, 100)); }
        int grow = 0;
        foreach (Node k in n.Kids) if (Grows(k, vertical)) grow++;
        for (int i = 0; i < count; i++)
        {
            Node k = n.Kids[i];
            Control c = Build(k);
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
    [DllImport("user32.dll")] static extern bool SetProcessDPIAware();

    /// std.app.13: the advance width of each requested run, in the font a `Str` op of the
    /// same family, size and weight is drawn in, answered as one `measured` event.
    static void AnswerMeasure(List<object> runs)
    {
        var widths = new List<string>();
        using (var bmp = new Bitmap(1, 1))
        using (var g = Graphics.FromImage(bmp))
        {
            g.TextRenderingHint = TextRenderingHint.ClearTypeGridFit;
            foreach (object r in runs)
            {
                Font f = Fonts.Get(Json.Text(Json.Get(r, "font")), Json.Num(Json.Get(r, "size")),
                                   Json.Flag(Json.Get(r, "bold")), Json.Flag(Json.Get(r, "italic")), false);
                float w = g.MeasureString(Json.Text(Json.Get(r, "text")), f, 100000, StringFormat.GenericTypographic).Width;
                widths.Add(w.ToString("0.000", CultureInfo.InvariantCulture));
            }
        }
        Outbox.Send("measured", "text", string.Join(",", widths.ToArray()));
    }

    [STAThread]
    static void Main()
    {
        try { SetProcessDPIAware(); } catch (Exception) { }
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
                Node tree = Node.Decode(Json.Get(obj, "tree"));
                form.BeginInvoke((Action)delegate { form.ShowView(title.Length > 0 ? title : "AIL", focus, tree); });
            }
            form.BeginInvoke((Action)delegate { Application.Exit(); });
        });
        reader.IsBackground = true;
        reader.Start();
        Application.Run(form);
    }
}
