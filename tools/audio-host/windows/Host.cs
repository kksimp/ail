// The std.audio host for Windows (spec std.audio.9-10, ADR 0949): plays and
// records through WASAPI in shared mode. One text line in, one line out:
// `play RATE CH BASE64` -> `done`, `rec MS RATE CH` -> `data RATE CH BASE64`,
// the same protocol as the macOS host. Streaming output (std.audio.12):
// `open_stream RATE CH` -> `ok`, `write BASE64`, `clock`, `pause`, `resume`,
// `flush` -> `played N buffered M`, `close_stream` -> `ok`; `--null` (or
// $AIL_AUDIO_NULL=1) streams to a silent device with a real-time clock, so a
// test needs no sound card. Audio is little-endian 16-bit PCM,
// interleaved. The stream is opened as 16-bit PCM with the engine's
// converter (AUTOCONVERTPCM) so the clip's own rate and channel count work
// on any device. C# 5 only: the .NET Framework's csc.exe builds it.
using System;
using System.Collections.Generic;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading;

static class Com
{
    public static readonly Guid CLSID_Enumerator = new Guid("BCDE0395-E52F-467C-8E3D-C4579291692E");
    public static Guid IID_AudioClient = new Guid("1CB9AD4C-DBFA-4c32-B178-C2F568A703B2");
    public static Guid IID_RenderClient = new Guid("F294ACFC-3146-4483-A7BF-ADDCA7C260E2");
    public static Guid IID_CaptureClient = new Guid("C8ADBD64-E71E-48a0-A4DE-185C395CD317");
    public const uint CLSCTX_ALL = 23;
    public const int EDataFlowRender = 0, EDataFlowCapture = 1, ERoleConsole = 0;
    public const uint FLAGS_CONVERT = 0x80000000 | 0x08000000; // AUTOCONVERTPCM | SRC_DEFAULT_QUALITY
}

[ComImport, Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
class MMDeviceEnumeratorObject { }

[ComImport, Guid("A95664D2-9614-4F35-A746-DE8DB63617E6"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDeviceEnumerator
{
    [PreserveSig] int EnumAudioEndpoints(int dataFlow, uint stateMask, out IntPtr devices);
    [PreserveSig] int GetDefaultAudioEndpoint(int dataFlow, int role, out IMMDevice device);
}

[ComImport, Guid("D666063F-1587-4E43-81F1-B948E807363F"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IMMDevice
{
    [PreserveSig] int Activate(ref Guid iid, uint clsCtx, IntPtr activationParams, [MarshalAs(UnmanagedType.IUnknown)] out object iface);
}

[ComImport, Guid("1CB9AD4C-DBFA-4c32-B178-C2F568A703B2"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IAudioClient
{
    [PreserveSig] int Initialize(int shareMode, uint streamFlags, long bufferDuration, long periodicity, IntPtr format, IntPtr sessionGuid);
    [PreserveSig] int GetBufferSize(out uint frames);
    [PreserveSig] int GetStreamLatency(out long latency);
    [PreserveSig] int GetCurrentPadding(out uint frames);
    [PreserveSig] int IsFormatSupported(int shareMode, IntPtr format, out IntPtr closest);
    [PreserveSig] int GetMixFormat(out IntPtr format);
    [PreserveSig] int GetDevicePeriod(out long defaultPeriod, out long minimumPeriod);
    [PreserveSig] int Start();
    [PreserveSig] int Stop();
    [PreserveSig] int Reset();
    [PreserveSig] int SetEventHandle(IntPtr eventHandle);
    [PreserveSig] int GetService(ref Guid iid, [MarshalAs(UnmanagedType.IUnknown)] out object service);
}

[ComImport, Guid("F294ACFC-3146-4483-A7BF-ADDCA7C260E2"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IAudioRenderClient
{
    [PreserveSig] int GetBuffer(uint frames, out IntPtr data);
    [PreserveSig] int ReleaseBuffer(uint frames, uint flags);
}

[ComImport, Guid("C8ADBD64-E71E-48a0-A4DE-185C395CD317"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
interface IAudioCaptureClient
{
    [PreserveSig] int GetBuffer(out IntPtr data, out uint frames, out uint flags, out ulong devicePosition, out ulong qpcPosition);
    [PreserveSig] int ReleaseBuffer(uint frames);
    [PreserveSig] int GetNextPacketSize(out uint frames);
}

class HostError : Exception
{
    public HostError(string m) : base(m) { }
}

// A running output stream (spec std.audio.12): a writer thread feeds the
// WASAPI buffer from a queue; `played` is the frames written minus the
// frames still in the device's buffer. With `--null` no device is opened
// and the clock runs in real time.
static class Out
{
    class Chunk { public byte[] Data; public int Off; }

    static readonly object mu = new object();
    static IAudioClient ac;
    static IAudioRenderClient rc;
    static uint size;
    static int block, rate;
    static bool isNull, paused, quit, started, open;
    static Queue<Chunk> q = new Queue<Chunk>();
    static long total, written;
    static double nplayed;
    static DateTime last;
    static Thread th;
    static string failure;

    public static bool Open { get { return open; } }

    public static void Start(int r, int ch, bool silent)
    {
        Close();
        lock (mu)
        {
            isNull = silent; rate = r; block = ch * 2;
            paused = quit = started = false;
            q.Clear(); total = written = 0; nplayed = 0; failure = null;
            last = DateTime.UtcNow;
        }
        if (!silent)
        {
            ac = Host.OpenClient(Com.EDataFlowRender, r, ch);
            object o;
            Host.Check(ac.GetService(ref Com.IID_RenderClient, out o), "render service");
            rc = (IAudioRenderClient)o;
            Host.Check(ac.GetBufferSize(out size), "buffer size");
            th = new Thread(Writer);
            th.IsBackground = true;
            th.Start();
        }
        open = true;
    }

    static void Writer()
    {
        lock (mu)
        {
            while (!quit)
            {
                try
                {
                    if (q.Count == 0 || paused) { Monitor.Wait(mu, 50); continue; }
                    uint pad;
                    Host.Check(ac.GetCurrentPadding(out pad), "padding");
                    int room = (int)(size - pad);
                    if (room <= 0) { Monitor.Wait(mu, 5); continue; }
                    Chunk c = q.Peek();
                    int n = Math.Min(room, (c.Data.Length - c.Off) / block);
                    IntPtr dst;
                    Host.Check(rc.GetBuffer((uint)n, out dst), "get buffer");
                    Marshal.Copy(c.Data, c.Off, dst, n * block);
                    Host.Check(rc.ReleaseBuffer((uint)n, 0), "release buffer");
                    c.Off += n * block;
                    written += n;
                    if (c.Off >= c.Data.Length) q.Dequeue();
                    if (!started) { Host.Check(ac.Start(), "start"); started = true; }
                }
                catch (HostError e) { failure = e.Message; return; }
            }
        }
    }

    // Advances the silent clock to now. Caller holds mu.
    static void Tick()
    {
        DateTime now = DateTime.UtcNow;
        if (!paused) nplayed += (now - last).TotalSeconds * rate;
        last = now;
        if (nplayed > total) nplayed = total;
    }

    public static string Status()
    {
        lock (mu)
        {
            if (failure != null) return "error " + failure;
            long played;
            if (isNull)
            {
                Tick();
                played = (long)nplayed;
            }
            else
            {
                uint pad;
                Host.Check(ac.GetCurrentPadding(out pad), "padding");
                played = written - pad;
                if (played < 0) played = 0;
                if (played > written) played = written;
            }
            return "played " + played + " buffered " + (total - played);
        }
    }

    public static void Write(byte[] data)
    {
        lock (mu)
        {
            int frames = data.Length / block;
            if (frames > 0)
            {
                byte[] d = new byte[frames * block];
                Array.Copy(data, d, d.Length);
                Chunk c = new Chunk(); c.Data = d;
                q.Enqueue(c);
                total += frames;
                Monitor.PulseAll(mu);
            }
        }
    }

    public static void Pause()
    {
        lock (mu)
        {
            if (paused) return;
            if (isNull) Tick();
            if (!isNull && started) ac.Stop();
            paused = true;
        }
    }

    public static void Resume()
    {
        lock (mu)
        {
            if (!paused) return;
            if (isNull) Tick();
            if (!isNull && started) ac.Start();
            paused = false;
            Monitor.PulseAll(mu);
        }
    }

    public static void Flush()
    {
        lock (mu)
        {
            q.Clear(); total = written = 0; nplayed = 0;
            if (!isNull && started) { ac.Stop(); ac.Reset(); started = false; }
        }
    }

    public static void Close()
    {
        if (!open) return;
        lock (mu) { quit = true; Monitor.PulseAll(mu); }
        if (th != null) { th.Join(); th = null; }
        if (ac != null) { ac.Stop(); Marshal.ReleaseComObject(ac); ac = null; rc = null; }
        lock (mu) { q.Clear(); }
        open = false;
    }
}

static class Host
{
    static TextWriter outp;

    static void Reply(string s) { outp.WriteLine(s); outp.Flush(); }

    public static void Check(int hr, string what)
    {
        if (hr < 0)
            throw new HostError(what + " failed (0x" + hr.ToString("X8") + ")" + (hr == unchecked((int)0x80070490) ? ": no device" : ""));
    }

    // A WAVEFORMATEX for 16-bit PCM, in unmanaged memory (the caller frees it).
    static IntPtr Format(int rate, int ch)
    {
        IntPtr p = Marshal.AllocHGlobal(18);
        int block = ch * 2;
        Marshal.WriteInt16(p, 0, 1);                 // WAVE_FORMAT_PCM
        Marshal.WriteInt16(p, 2, (short)ch);
        Marshal.WriteInt32(p, 4, rate);
        Marshal.WriteInt32(p, 8, rate * block);
        Marshal.WriteInt16(p, 12, (short)block);
        Marshal.WriteInt16(p, 14, 16);
        Marshal.WriteInt16(p, 16, 0);
        return p;
    }

    // The default endpoint's audio client, started in shared mode with a one-second buffer.
    public static IAudioClient OpenClient(int flow, int rate, int ch)
    {
        IMMDeviceEnumerator en = (IMMDeviceEnumerator)new MMDeviceEnumeratorObject();
        IMMDevice dev;
        Check(en.GetDefaultAudioEndpoint(flow, Com.ERoleConsole, out dev), flow == Com.EDataFlowRender ? "no output device" : "no input device");
        object o;
        Check(dev.Activate(ref Com.IID_AudioClient, Com.CLSCTX_ALL, IntPtr.Zero, out o), "activate");
        IAudioClient ac = (IAudioClient)o;
        IntPtr fmt = Format(rate, ch);
        try { Check(ac.Initialize(0, Com.FLAGS_CONVERT, 10000000L, 0, fmt, IntPtr.Zero), "initialize"); }
        finally { Marshal.FreeHGlobal(fmt); }
        return ac;
    }

    static void Play(int rate, int ch, byte[] data)
    {
        int block = ch * 2;
        int total = data.Length / block;
        if (total == 0) { Reply("done"); return; }
        IAudioClient ac = OpenClient(Com.EDataFlowRender, rate, ch);
        object o;
        Check(ac.GetService(ref Com.IID_RenderClient, out o), "render service");
        IAudioRenderClient rc = (IAudioRenderClient)o;
        uint size;
        Check(ac.GetBufferSize(out size), "buffer size");
        int pos = 0;
        bool started = false;
        while (pos < total)
        {
            uint pad;
            Check(ac.GetCurrentPadding(out pad), "padding");
            int n = Math.Min((int)(size - pad), total - pos);
            if (n > 0)
            {
                IntPtr dst;
                Check(rc.GetBuffer((uint)n, out dst), "get buffer");
                Marshal.Copy(data, pos * block, dst, n * block);
                Check(rc.ReleaseBuffer((uint)n, 0), "release buffer");
                pos += n;
            }
            if (!started) { Check(ac.Start(), "start"); started = true; }
            if (pos < total) Thread.Sleep(5);
        }
        // Everything is queued: wait for the device to play it out.
        uint left;
        do
        {
            Thread.Sleep(5);
            Check(ac.GetCurrentPadding(out left), "padding");
        } while (left > 0);
        ac.Stop();
        Reply("done");
    }

    static void Record(int ms, int rate, int ch)
    {
        IAudioClient ac = OpenClient(Com.EDataFlowCapture, rate, ch);
        object o;
        Check(ac.GetService(ref Com.IID_CaptureClient, out o), "capture service");
        IAudioCaptureClient cc = (IAudioCaptureClient)o;
        int block = ch * 2;
        long want = (long)ms * rate / 1000;
        MemoryStream pcm = new MemoryStream();
        long got = 0;
        Check(ac.Start(), "start");
        DateTime end = DateTime.UtcNow.AddMilliseconds(ms + 2000);
        while (got < want && DateTime.UtcNow < end)
        {
            uint n;
            Check(cc.GetNextPacketSize(out n), "packet size");
            if (n == 0) { Thread.Sleep(5); continue; }
            IntPtr src; uint frames, flags; ulong dpos, qpc;
            Check(cc.GetBuffer(out src, out frames, out flags, out dpos, out qpc), "get buffer");
            int take = (int)Math.Min((long)frames, want - got);
            byte[] chunk = new byte[take * block];
            if ((flags & 2) == 0) Marshal.Copy(src, chunk, 0, chunk.Length); // 2 = silent
            pcm.Write(chunk, 0, chunk.Length);
            got += take;
            Check(cc.ReleaseBuffer(frames), "release buffer");
        }
        ac.Stop();
        Reply("data " + rate + " " + ch + " " + Convert.ToBase64String(pcm.ToArray()));
    }

    static int Int(string s, int d)
    {
        int v;
        return int.TryParse(s, out v) ? v : d;
    }

    static int Main(string[] args)
    {
        string nv = Environment.GetEnvironmentVariable("AIL_AUDIO_NULL");
        bool silent = !string.IsNullOrEmpty(nv) && nv != "0";
        foreach (string a in args) if (a == "--null") silent = true;
        outp = new StreamWriter(Console.OpenStandardOutput(), new UTF8Encoding(false));
        TextReader inp = new StreamReader(Console.OpenStandardInput(), Encoding.ASCII, false, 1 << 20);
        Reply("ready");
        string line;
        while ((line = inp.ReadLine()) != null)
        {
            string[] p = line.Split(new char[] { ' ' }, StringSplitOptions.RemoveEmptyEntries);
            try
            {
                if (p.Length >= 3 && p[0] == "play")
                {
                    byte[] data = p.Length > 3 ? Convert.FromBase64String(p[3]) : new byte[0];
                    Play(Int(p[1], 44100), Math.Max(1, Int(p[2], 1)), data);
                }
                else if (p.Length >= 4 && p[0] == "rec")
                    Record(Int(p[1], 0), Int(p[2], 44100), Math.Max(1, Int(p[3], 1)));
                else if (p.Length >= 3 && p[0] == "open_stream")
                {
                    Out.Start(Int(p[1], 44100), Math.Max(1, Int(p[2], 1)), silent);
                    Reply("ok");
                }
                else if (p.Length >= 1 && p[0] == "close_stream" && Out.Open) { Out.Close(); Reply("ok"); }
                else if (p.Length >= 1 && Out.Open && p[0] == "write")
                {
                    Out.Write(p.Length > 1 ? Convert.FromBase64String(p[1]) : new byte[0]);
                    Reply(Out.Status());
                }
                else if (p.Length >= 1 && Out.Open && p[0] == "clock") Reply(Out.Status());
                else if (p.Length >= 1 && Out.Open && p[0] == "pause") { Out.Pause(); Reply(Out.Status()); }
                else if (p.Length >= 1 && Out.Open && p[0] == "resume") { Out.Resume(); Reply(Out.Status()); }
                else if (p.Length >= 1 && Out.Open && p[0] == "flush") { Out.Flush(); Reply(Out.Status()); }
                else if (p.Length >= 1 && (p[0] == "close_stream" || p[0] == "write" || p[0] == "clock" || p[0] == "pause" || p[0] == "resume" || p[0] == "flush"))
                    Reply("error no stream open");
                else
                    Reply("error unknown request");
            }
            catch (HostError e) { Reply("error " + e.Message); }
            catch (COMException e) { Reply("error " + e.Message.Replace('\n', ' ').Replace('\r', ' ')); }
            catch (FormatException) { Reply("error bad audio data"); }
        }
        Out.Close();
        return 0;
    }
}
