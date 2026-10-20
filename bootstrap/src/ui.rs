//! `std.ui` (spec 17-std, std.ui.1-9, ADR 0177): a window showing a grid of
//! monospace text cells, with keyboard and resize events.
//!
//! The interpreter runs on its own thread (main.rs), but macOS lets only a
//! process's main thread own windows. So the main thread is the *host*: it
//! runs the interpreter thread and serves window requests until that thread
//! ends (`run_hosted`). A program that never calls `ui.open` costs one idle
//! channel. The interpreter thread keeps the cell grid itself, so `put` and
//! `clear` never cross threads; `present` sends a copy of the grid.
//!
//! The window and keyboard come from `minifb`, the glyphs from the
//! `embedded-graphics` 10x20 font (ADR 0177). Without a display, `open`
//! fails with `UiError(kind: "no_display")` and nothing else is touched, so
//! `./ail selftest` runs headless.

use std::cell::RefCell;
use std::sync::OnceLock;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::time::Duration;

/// The default colors of a fresh grid (std.ui.3): light grey on black.
pub const DEFAULT_FG: u32 = 0xD0D0D0;
pub const DEFAULT_BG: u32 = 0x000000;

/// The largest grid `ui.open` accepts in each direction (std.ui.1).
pub const MAX_CELLS: i64 = 1000;

/// One text cell: a character the font can draw, and its colors (0xRRGGBB).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Cell {
    pub ch: char,
    pub fg: u32,
    pub bg: u32,
}

impl Cell {
    const BLANK: Cell = Cell { ch: ' ', fg: DEFAULT_FG, bg: DEFAULT_BG };
}

/// std.ui.4: the character a cell shows: printable Latin-1 as is, anything
/// else (control characters, tabs, other scripts) as `?`.
pub fn glyph(c: char) -> char {
    if matches!(c as u32, 0x20..=0x7E | 0xA0..=0xFF) { c } else { '?' }
}

/// The cells of a window, row by row.
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Cell>,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Grid {
        Grid { cols, rows, cells: vec![Cell::BLANK; cols * rows] }
    }

    /// std.ui.3: every cell blank on `bg`.
    pub fn clear(&mut self, bg: i64) {
        let bg = color(bg);
        self.cells.fill(Cell { ch: ' ', fg: DEFAULT_FG, bg });
    }

    /// std.ui.4: `text` from (`col`, `row`) rightwards, one character per
    /// cell, clipped to the grid on every side.
    pub fn put(&mut self, col: i64, row: i64, text: &str, fg: i64, bg: i64) {
        let Ok(row) = usize::try_from(row) else { return };
        if row >= self.rows {
            return;
        }
        let (fg, bg) = (color(fg), color(bg));
        let mut c = col;
        for ch in text.chars() {
            if c >= self.cols as i64 {
                break;
            }
            if c >= 0 {
                self.cells[row * self.cols + c as usize] = Cell { ch: glyph(ch), fg, bg };
            }
            c += 1;
        }
    }

    /// std.ui.6: a new size keeps the cells that still fit; new cells are blank.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let mut next = Grid::new(cols, rows);
        for r in 0..rows.min(self.rows) {
            for c in 0..cols.min(self.cols) {
                next.cells[r * cols + c] = self.cells[r * self.cols + c];
            }
        }
        *self = next;
    }
}

/// Colors are the low 24 bits of an int, 0xRRGGBB (std.ui.3).
fn color(v: i64) -> u32 {
    (v & 0xFF_FFFF) as u32
}

/// A window event (std.ui.6), as the interpreter turns it into `ui.Event`.
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Key { key: String, ctrl: bool, alt: bool, shift: bool },
    Resize { cols: usize, rows: usize },
    Close,
    /// std.ui.10: `button` is `left`, `right` or `middle`; `kind` is
    /// `down`, `up` or `drag`.
    Mouse { col: i64, row: i64, button: &'static str, kind: &'static str },
    /// std.ui.11: lines to scroll, positive towards the end.
    Scroll { lines: i64 },
}

/// Ctrl, Alt and Shift as they were when an event happened (std.ui.14).
pub type Modifiers = [bool; 3];

/// An event and the modifiers held when it happened.
type Tagged = (Event, Modifiers);

/// The font sizes a window can use, as cell heights in pixels (std.ui.15).
pub const FONT_SIZES: &[i64] = &[10, 12, 13, 14, 15, 18, 20, 24, 26, 28, 30, 36, 40];

/// The font size a window opens with (std.ui.15).
pub const DEFAULT_FONT_SIZE: i64 = 20;

/// std.ui.15: the available size nearest `px`, the smaller on a tie.
pub fn nearest_font_size(px: i64) -> i64 {
    *FONT_SIZES.iter().min_by_key(|s| ((**s - px).abs(), **s)).expect("sizes")
}

/// Why `ui.open` failed: `UiError.kind` and `UiError.message` (std.ui.1).
pub type OpenError = (&'static str, String);

// ---------------------------------------------------------------------------
// The interpreter thread's side.

/// The open window, as the interpreter thread sees it.
struct Session {
    id: i64,
    grid: Grid,
    cursor: (i64, i64),
    /// Set once the window has closed (std.ui.8).
    closed: bool,
    /// The modifiers of the last event returned (std.ui.14).
    mods: Modifiers,
    font_size: i64,
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
    static NEXT_ID: RefCell<i64> = const { RefCell::new(1) };
}

/// A request from the interpreter thread to the host.
enum Msg {
    Open { title: String, cols: usize, rows: usize, reply: Sender<Result<(), OpenError>> },
    Present { grid: Grid, cursor: (i64, i64) },
    Title(String),
    Font(i64),
    Poll(Sender<Option<Tagged>>),
    Wait(Sender<Tagged>),
    Close,
    /// The interpreter thread has ended.
    Done,
}

static HOST: OnceLock<Sender<Msg>> = OnceLock::new();

fn send(msg: Msg) -> bool {
    HOST.get().is_some_and(|h| h.send(msg).is_ok())
}

/// std.ui.1: open the window; its handle on success. At most one window is
/// open at a time.
pub fn open(title: &str, cols: i64, rows: i64) -> Result<i64, OpenError> {
    if !(1..=MAX_CELLS).contains(&cols) || !(1..=MAX_CELLS).contains(&rows) {
        return Err(("other", format!("a window is 1 to {MAX_CELLS} cells each way, not {cols} by {rows}")));
    }
    if SESSION.with_borrow(|s| s.as_ref().is_some_and(|s| !s.closed)) {
        return Err(("other", "a window is already open; close it first".to_string()));
    }
    if HOST.get().is_none() {
        return Err(("no_display", "this program runs without a window host".to_string()));
    }
    let (cols, rows) = (cols as usize, rows as usize);
    let (reply, answer) = mpsc::channel();
    send(Msg::Open { title: title.to_string(), cols, rows, reply });
    answer.recv().unwrap_or_else(|_| Err(("other", "the window host has stopped".to_string())))?;
    let id = NEXT_ID.with_borrow_mut(|n| {
        *n += 1;
        *n - 1
    });
    SESSION.set(Some(Session { id, grid: Grid::new(cols, rows), cursor: (-1, -1), closed: false, mods: [false; 3], font_size: DEFAULT_FONT_SIZE }));
    Ok(id)
}

/// Run `f` on the session of window `id`, if it is the current one.
fn with<R>(id: i64, f: impl FnOnce(&mut Session) -> R) -> Option<R> {
    SESSION.with_borrow_mut(|s| s.as_mut().filter(|s| s.id == id).map(f))
}

/// std.ui.2: the size in cells; 0 by 0 for a window that is not open.
pub fn size(id: i64) -> (i64, i64) {
    with(id, |s| (s.grid.cols as i64, s.grid.rows as i64)).unwrap_or((0, 0))
}

pub fn clear(id: i64, bg: i64) {
    with(id, |s| s.grid.clear(bg));
}

pub fn put(id: i64, col: i64, row: i64, text: &str, fg: i64, bg: i64) {
    with(id, |s| s.grid.put(col, row, text, fg, bg));
}

/// std.ui.5: where the caret is drawn; off the grid hides it.
pub fn cursor(id: i64, col: i64, row: i64) {
    with(id, |s| s.cursor = (col, row));
}

/// std.ui.5: show the grid. Asynchronous: the host draws it in order.
pub fn present(id: i64) {
    if let Some(Some((grid, cursor))) = with(id, |s| (!s.closed).then(|| (s.grid.clone(), s.cursor))) {
        send(Msg::Present { grid, cursor });
    }
}

/// std.ui.6-8: the next event, waiting for one when `wait`. A closed window,
/// or one that isn't open, gives `Close`.
pub fn next_event(id: i64, wait: bool) -> Option<Event> {
    let live = with(id, |s| !s.closed).unwrap_or(false);
    if !live {
        return Some(Event::Close);
    }
    let closed = (Event::Close, [false; 3]);
    let (event, mods) = if wait {
        let (reply, answer) = mpsc::channel();
        send(Msg::Wait(reply));
        answer.recv().unwrap_or(closed)
    } else {
        let (reply, answer) = mpsc::channel();
        send(Msg::Poll(reply));
        answer.recv().unwrap_or(Some(closed))?
    };
    with(id, |s| {
        s.mods = mods;
        match &event {
            Event::Resize { cols, rows } => s.grid.resize(*cols, *rows),
            Event::Close => s.closed = true,
            _ => {}
        }
    });
    Some(event)
}

/// std.ui.14: Ctrl, Alt and Shift as they were at the last event returned.
pub fn modifiers(id: i64) -> Modifiers {
    with(id, |s| s.mods).unwrap_or([false; 3])
}

/// std.ui.15: change the font size. The window keeps its pixel size, so the
/// grid follows with a `Resize` when the number of cells changes.
pub fn set_font_size(id: i64, px: i64) {
    let size = nearest_font_size(px);
    let changed = with(id, |s| {
        let changed = !s.closed && s.font_size != size;
        s.font_size = size;
        changed
    });
    if changed == Some(true) {
        send(Msg::Font(size));
    }
}

/// std.ui.15: the font size, as a cell height in pixels.
pub fn font_size(id: i64) -> i64 {
    with(id, |s| s.font_size).unwrap_or(DEFAULT_FONT_SIZE)
}

/// std.ui.13: the window's title.
pub fn set_title(id: i64, title: &str) {
    if with(id, |s| !s.closed).unwrap_or(false) {
        send(Msg::Title(title.to_string()));
    }
}

/// std.ui.8: close the window; later calls on it do nothing.
pub fn close(id: i64) {
    if with(id, |_| ()).is_some() {
        SESSION.set(None);
        send(Msg::Close);
    }
}

// ---------------------------------------------------------------------------
// The clipboard (std.ui.12): the system's, through its command-line tools,
// or this program's own when none of them works or the program runs without
// a window host (`ail test`), so tests never touch the user's clipboard.

thread_local! {
    static CLIPBOARD: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// The commands that write and read the system clipboard, in order of preference.
fn clipboard_tools() -> (Vec<(&'static str, Vec<&'static str>)>, Vec<(&'static str, Vec<&'static str>)>) {
    if cfg!(target_os = "macos") {
        (vec![("pbcopy", vec![])], vec![("pbpaste", vec![])])
    } else if cfg!(windows) {
        (
            vec![("powershell", vec!["-NoProfile", "-Command", "$input | Set-Clipboard"])],
            vec![("powershell", vec!["-NoProfile", "-Command", "Get-Clipboard -Raw"])],
        )
    } else {
        let mut set = Vec::new();
        let mut get = Vec::new();
        if std::env::var_os("WAYLAND_DISPLAY").is_some() {
            set.push(("wl-copy", vec![]));
            get.push(("wl-paste", vec!["--no-newline"]));
        }
        if std::env::var_os("DISPLAY").is_some() {
            set.push(("xclip", vec!["-selection", "clipboard"]));
            set.push(("xsel", vec!["--clipboard", "--input"]));
            get.push(("xclip", vec!["-selection", "clipboard", "-o"]));
            get.push(("xsel", vec!["--clipboard", "--output"]));
        }
        (set, get)
    }
}

/// std.ui.12: put `text` on the clipboard.
pub fn clipboard_set(text: &str) {
    use std::io::Write as _;
    use std::process::{Command, Stdio};
    CLIPBOARD.set(Some(text.to_string()));
    if HOST.get().is_none() {
        return;
    }
    for (cmd, args) in clipboard_tools().0 {
        let child = Command::new(cmd)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        let Ok(mut child) = child else { continue };
        let wrote = child.stdin.take().is_some_and(|mut i| i.write_all(text.as_bytes()).is_ok());
        if wrote && child.wait().is_ok_and(|s| s.success()) {
            return;
        }
    }
}

/// std.ui.12: the clipboard's text; `None` when it holds none.
pub fn clipboard_get() -> Option<String> {
    use std::process::{Command, Stdio};
    let tools = if HOST.get().is_some() { clipboard_tools().1 } else { Vec::new() };
    for (cmd, args) in tools {
        let out = Command::new(cmd).args(&args).stdin(Stdio::null()).stderr(Stdio::null()).output();
        if let Ok(out) = out
            && out.status.success()
        {
            let text = String::from_utf8_lossy(&out.stdout).into_owned();
            return (!text.is_empty()).then_some(text);
        }
    }
    CLIPBOARD.with_borrow(|c| c.clone().filter(|t| !t.is_empty()))
}

// ---------------------------------------------------------------------------
// The host: the main thread.

/// Run `f` on a new thread with `stack` bytes of stack while this thread
/// serves its window requests, and return what `f` returns (ADR 0177).
pub fn run_hosted<T: Send + 'static>(stack: usize, f: impl FnOnce() -> T + Send + 'static) -> std::thread::Result<T> {
    let (tx, rx) = mpsc::channel();
    let hosting = HOST.set(tx.clone()).is_ok();
    struct Done(Sender<Msg>);
    impl Drop for Done {
        fn drop(&mut self) {
            let _ = self.0.send(Msg::Done);
        }
    }
    let handle = std::thread::Builder::new()
        .stack_size(stack)
        .spawn(move || {
            let _done = Done(tx);
            f()
        })
        .expect("spawn interpreter thread");
    if hosting {
        serve(&rx);
    }
    handle.join()
}

/// How often the host pumps an open window's events while it waits.
const TICK: Duration = Duration::from_millis(8);

fn serve(rx: &Receiver<Msg>) {
    let mut window: Option<backend::Backend> = None;
    let mut waiter: Option<Sender<Tagged>> = None;
    loop {
        let msg = if window.is_some() {
            match rx.recv_timeout(TICK) {
                Ok(m) => Some(m),
                Err(RecvTimeoutError::Timeout) => None,
                Err(RecvTimeoutError::Disconnected) => return,
            }
        } else {
            match rx.recv() {
                Ok(m) => Some(m),
                Err(_) => return,
            }
        };
        match msg {
            None => {
                if let Some(w) = &mut window {
                    w.pump();
                }
            }
            Some(Msg::Done) => return,
            Some(Msg::Open { title, cols, rows, reply }) => {
                let result = match backend::Backend::open(&title, cols, rows) {
                    Ok(w) => {
                        window = Some(w);
                        Ok(())
                    }
                    Err(e) => Err(e),
                };
                let _ = reply.send(result);
            }
            Some(Msg::Present { grid, cursor }) => {
                if let Some(w) = &mut window {
                    w.present(grid, cursor);
                }
            }
            Some(Msg::Poll(reply)) => {
                let event = match &mut window {
                    Some(w) => {
                        w.pump();
                        w.events.pop_front()
                    }
                    None => Some((Event::Close, [false; 3])),
                };
                let _ = reply.send(event);
            }
            Some(Msg::Wait(reply)) => {
                if let Some(w) = &mut window {
                    w.pump();
                }
                waiter = Some(reply);
            }
            Some(Msg::Font(size)) => {
                if let Some(w) = &mut window {
                    w.set_font(size);
                }
            }
            Some(Msg::Title(t)) => {
                if let Some(w) = &mut window {
                    w.set_title(&t);
                }
            }
            Some(Msg::Close) => window = None,
        }
        if let Some(reply) = waiter.take() {
            match &mut window {
                Some(w) => match w.events.pop_front() {
                    Some(e) => {
                        let _ = reply.send(e);
                    }
                    None => waiter = Some(reply),
                },
                None => {
                    let _ = reply.send((Event::Close, [false; 3]));
                }
            }
        }
    }
}

/// std.ui.7: the name of a key that isn't text, or, with Ctrl held (`chord`),
/// the key's unshifted character: Ctrl-S is `"s"` with `ctrl` and Ctrl-. is
/// `"."`. Text keys without Ctrl come from the platform's character input
/// instead, so layouts and Shift work. Modifier keys alone give nothing.
pub fn key_name(key: minifb::Key, chord: bool) -> Option<String> {
    use minifb::Key as K;
    let named = match key {
        K::Enter | K::NumPadEnter => "enter",
        K::Tab => "tab",
        K::Backspace => "backspace",
        K::Delete => "delete",
        K::Escape => "escape",
        K::Up => "up",
        K::Down => "down",
        K::Left => "left",
        K::Right => "right",
        K::Home => "home",
        K::End => "end",
        K::PageUp => "pageup",
        K::PageDown => "pagedown",
        K::F1 => "f1",
        K::F2 => "f2",
        K::F3 => "f3",
        K::F4 => "f4",
        K::F5 => "f5",
        K::F6 => "f6",
        K::F7 => "f7",
        K::F8 => "f8",
        K::F9 => "f9",
        K::F10 => "f10",
        K::F11 => "f11",
        K::F12 => "f12",
        _ if chord => return chord_char(key).map(String::from),
        _ => return None,
    };
    Some(named.to_string())
}

/// The unshifted character of a text key on a US layout, for Ctrl chords.
fn chord_char(key: minifb::Key) -> Option<char> {
    use minifb::Key as K;
    let k = key as u32;
    Some(match key {
        K::Key0 | K::Key1 | K::Key2 | K::Key3 | K::Key4 | K::Key5 | K::Key6 | K::Key7 | K::Key8 | K::Key9 => {
            char::from(b'0' + (k - K::Key0 as u32) as u8)
        }
        _ if (K::A as u32..=K::Z as u32).contains(&k) => char::from(b'a' + (k - K::A as u32) as u8),
        K::Space => ' ',
        K::Apostrophe => '\'',
        K::Backquote => '`',
        K::Backslash => '\\',
        K::Comma => ',',
        K::Equal => '=',
        K::LeftBracket => '[',
        K::Minus | K::NumPadMinus => '-',
        K::NumPadPlus => '+',
        K::Period => '.',
        K::RightBracket => ']',
        K::Semicolon => ';',
        K::Slash => '/',
        _ => return None,
    })
}

mod backend {
    //! The window itself: minifb on the host thread (std.ui.1, std.ui.5-7).

    use std::cell::RefCell;
    use std::collections::{HashMap, VecDeque};
    use std::rc::Rc;

    use embedded_graphics::mono_font::{MonoFont, MonoTextStyle, iso_8859_1 as fonts};
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::prelude::*;
    use embedded_graphics::text::{Baseline, Text};
    use minifb::{Key, Window, WindowOptions};

    use super::{Cell, Event, Grid, OpenError, key_name};

    /// std.ui.15: the font for a cell height, and how many times to scale it.
    fn font(size: i64) -> (&'static MonoFont<'static>, usize) {
        match size {
            10 => (&fonts::FONT_6X10, 1),
            12 => (&fonts::FONT_6X12, 1),
            13 => (&fonts::FONT_7X13, 1),
            14 => (&fonts::FONT_7X14, 1),
            15 => (&fonts::FONT_9X15, 1),
            18 => (&fonts::FONT_9X18, 1),
            24 => (&fonts::FONT_6X12, 2),
            26 => (&fonts::FONT_7X13, 2),
            28 => (&fonts::FONT_7X14, 2),
            30 => (&fonts::FONT_9X15, 2),
            36 => (&fonts::FONT_9X18, 2),
            40 => (&fonts::FONT_10X20, 2),
            _ => (&fonts::FONT_10X20, 1),
        }
    }

    /// Keyboard input in the order it happened, from minifb's callback. A
    /// press and release between two pumps still counts (std.ui.7).
    enum Input {
        Char(u32),
        Key(Key, bool),
    }

    struct Typed(Rc<RefCell<Vec<Input>>>);

    impl minifb::InputCallback for Typed {
        fn add_char(&mut self, c: u32) {
            self.0.borrow_mut().push(Input::Char(c));
        }
        fn set_key_state(&mut self, key: Key, down: bool) {
            self.0.borrow_mut().push(Input::Key(key, down));
        }
    }

    /// The modifier keys held: Ctrl, Alt, Shift.
    #[derive(Default)]
    struct Mods {
        ctrl: bool,
        alt: bool,
        shift: bool,
    }

    impl Mods {
        /// Track a modifier key; false for any other key.
        fn track(&mut self, key: Key, down: bool) -> bool {
            match key {
                Key::LeftCtrl | Key::RightCtrl => self.ctrl = down,
                // On macOS, Command is the shortcut key: it reports as Ctrl.
                Key::LeftSuper | Key::RightSuper if cfg!(target_os = "macos") => self.ctrl = down,
                Key::LeftAlt | Key::RightAlt => self.alt = down,
                Key::LeftShift | Key::RightShift => self.shift = down,
                _ => return false,
            }
            true
        }
    }

    pub struct Backend {
        window: Window,
        typed: Rc<RefCell<Vec<Input>>>,
        mods: Mods,
        /// The pixels, `w` by `h`, 0x00RRGGBB.
        fb: Vec<u32>,
        w: usize,
        h: usize,
        /// The grid last presented, and its caret; `None` before the first.
        shown: Option<(Grid, (i64, i64))>,
        glyphs: HashMap<char, Vec<bool>>,
        /// The font, its scale, and one cell in pixels (std.ui.15).
        font: &'static MonoFont<'static>,
        scale: usize,
        cw: usize,
        ch: usize,
        cells: (usize, usize),
        closed: bool,
        /// Each mouse button's state at the last pump, and the cell the
        /// mouse was over (std.ui.10).
        buttons: [bool; 3],
        mouse: (i64, i64),
        last_drag: (i64, i64),
        /// Scrolling not yet reported as whole lines (std.ui.11).
        scroll: f32,
        pub events: VecDeque<(Event, [bool; 3])>,
    }

    impl Backend {
        /// std.ui.1: `no_display` when there is nowhere to show a window.
        pub fn open(title: &str, cols: usize, rows: usize) -> Result<Backend, OpenError> {
            if cfg!(all(unix, not(target_os = "macos")))
                && std::env::var_os("DISPLAY").is_none_or(|d| d.is_empty())
            {
                return Err(("no_display", "no X11 display: DISPLAY is not set".to_string()));
            }
            let (font, scale) = font(super::DEFAULT_FONT_SIZE);
            let (cw, ch) = (font.character_size.width as usize * scale, font.character_size.height as usize * scale);
            let (w, h) = (cols * cw, rows * ch);
            let options = WindowOptions {
                resize: true,
                scale_mode: minifb::ScaleMode::UpperLeft,
                ..WindowOptions::default()
            };
            let made = std::panic::catch_unwind(|| Window::new(title, w, h, options));
            let mut window = match made {
                Ok(Ok(w)) => w,
                Ok(Err(e)) => return Err(("no_display", format!("can't open a window: {e}"))),
                Err(_) => return Err(("no_display", "can't open a window".to_string())),
            };
            window.set_target_fps(0);
            let typed = Rc::new(RefCell::new(Vec::new()));
            window.set_input_callback(Box::new(Typed(typed.clone())));
            let mut b = Backend {
                window,
                typed,
                mods: Mods::default(),
                fb: vec![super::DEFAULT_BG; w * h],
                w,
                h,
                shown: None,
                glyphs: HashMap::new(),
                font,
                scale,
                cw,
                ch,
                cells: (cols, rows),
                closed: false,
                buttons: [false; 3],
                mouse: (-1, -1),
                last_drag: (-1, -1),
                scroll: 0.0,
                events: VecDeque::new(),
            };
            b.flush();
            Ok(b)
        }

        /// Queue `e` with the modifiers held now (std.ui.14).
        fn queue(&mut self, e: Event) {
            let m = &self.mods;
            self.events.push_back((e, [m.ctrl, m.alt, m.shift]));
        }

        /// std.ui.15: switch fonts, keep the window's pixel size, and redraw.
        pub fn set_font(&mut self, size: i64) {
            let (font, scale) = font(size);
            self.font = font;
            self.scale = scale;
            self.cw = font.character_size.width as usize * scale;
            self.ch = font.character_size.height as usize * scale;
            self.glyphs.clear();
            self.relayout();
        }

        /// Fit the cells to the window, queue a `Resize` if their number
        /// changed, and redraw what was shown (std.ui.7).
        fn relayout(&mut self) {
            self.fb.resize(self.w * self.h, super::DEFAULT_BG);
            self.fb.fill(super::DEFAULT_BG);
            let cells = ((self.w / self.cw).max(1), (self.h / self.ch).max(1));
            if cells != self.cells {
                self.cells = cells;
                self.queue(Event::Resize { cols: cells.0, rows: cells.1 });
            }
            if let Some((grid, cursor)) = self.shown.take() {
                self.draw(grid, cursor, None);
            }
            self.flush();
        }

        pub fn set_title(&mut self, title: &str) {
            self.window.set_title(title);
        }

        /// std.ui.10-11: mouse buttons, movement while a button is held, and
        /// the wheel, in cells.
        fn pump_mouse(&mut self) {
            use minifb::{MouseButton, MouseMode};
            if let Some((x, y)) = self.window.get_mouse_pos(MouseMode::Discard) {
                self.mouse = ((x.max(0.0) as usize / self.cw) as i64, (y.max(0.0) as usize / self.ch) as i64);
            }
            let (col, row) = self.mouse;
            let moved = self.mouse != self.last_drag;
            for (i, (b, name)) in
                [(MouseButton::Left, "left"), (MouseButton::Right, "right"), (MouseButton::Middle, "middle")]
                    .into_iter()
                    .enumerate()
            {
                let down = self.window.get_mouse_down(b);
                let kind = match (self.buttons[i], down) {
                    (false, true) => "down",
                    (true, false) => "up",
                    (true, true) if moved => "drag",
                    _ => continue,
                };
                self.buttons[i] = down;
                if col >= 0 {
                    self.queue(Event::Mouse { col, row, button: name, kind });
                }
            }
            self.last_drag = self.mouse;
            if let Some((_, dy)) = self.window.get_scroll_wheel() {
                // About three lines per notch: minifb reports a notch as 1.0
                // on X11 and macOS and as 12.0 on Windows.
                let per_line = if cfg!(windows) { 4.0 } else { 1.0 / 3.0 };
                self.scroll -= dy / per_line;
                let lines = self.scroll.trunc();
                if lines != 0.0 {
                    self.scroll -= lines;
                    self.queue(Event::Scroll { lines: lines as i64 });
                }
            }
        }

        fn flush(&mut self) {
            let _ = self.window.update_with_buffer(&self.fb, self.w, self.h);
        }

        /// Process the window's input, queueing events (std.ui.6-7).
        pub fn pump(&mut self) {
            if self.closed {
                return;
            }
            self.flush();
            if !self.window.is_open() {
                self.closed = true;
                self.queue(Event::Close);
                return;
            }
            let (w, h) = self.window.get_size();
            if (w, h) != (self.w, self.h) && w > 0 && h > 0 {
                self.w = w;
                self.h = h;
                self.relayout();
            }
            self.pump_mouse();
            let typed = std::mem::take(&mut *self.typed.borrow_mut());
            for input in typed {
                let m = &mut self.mods;
                let key = match input {
                    Input::Key(k, down) => {
                        if m.track(k, down) || !down {
                            continue;
                        }
                        key_name(k, m.ctrl)
                    }
                    // Text comes as characters, except in Ctrl chords.
                    Input::Char(c) => char::from_u32(c).filter(|c| !m.ctrl && *c >= ' ' && *c != '\u{7f}').map(String::from),
                };
                if let Some(key) = key {
                    let (ctrl, alt, shift) = (m.ctrl, m.alt, m.shift);
                    self.queue(Event::Key { key, ctrl, alt, shift });
                }
            }
        }

        /// std.ui.5: draw the cells that changed since the last present.
        pub fn present(&mut self, grid: Grid, cursor: (i64, i64)) {
            let before = self.shown.take();
            self.draw(grid, cursor, before);
            self.flush();
        }

        fn draw(&mut self, grid: Grid, cursor: (i64, i64), before: Option<(Grid, (i64, i64))>) {
            let at = |(c, r): (i64, i64)| -> Option<(usize, usize)> {
                let (c, r) = (usize::try_from(c).ok()?, usize::try_from(r).ok()?);
                (c < grid.cols && r < grid.rows).then_some((c, r))
            };
            let caret = at(cursor);
            let old = before.as_ref().filter(|(g, _)| g.cols == grid.cols && g.rows == grid.rows);
            let old_caret = old.and_then(|(_, c)| at(*c));
            if old.is_none() {
                // A full redraw: clear what a larger grid or font left behind.
                self.fb.fill(super::DEFAULT_BG);
            }
            for r in 0..grid.rows {
                for c in 0..grid.cols {
                    let cell = grid.cells[r * grid.cols + c];
                    let here = Some((c, r));
                    let same = old.is_some_and(|(g, _)| g.cells[r * grid.cols + c] == cell)
                        && here != caret
                        && here != old_caret;
                    if !same {
                        self.cell(c, r, cell, here == caret);
                    }
                }
            }
            self.shown = Some((grid, cursor));
        }

        fn cell(&mut self, col: usize, row: usize, cell: Cell, caret: bool) {
            let (cw, ch, scale) = (self.cw, self.ch, self.scale);
            let (x0, y0) = (col * cw, row * ch);
            if x0 >= self.w || y0 >= self.h {
                return;
            }
            let font = self.font;
            let bitmap = self.glyphs.entry(cell.ch).or_insert_with(|| render(font, cell.ch));
            let caret_color = contrast(cell.bg);
            let gw = cw / scale;
            for y in 0..ch.min(self.h - y0) {
                let line = &mut self.fb[(y0 + y) * self.w + x0..][..cw.min(self.w - x0)];
                for (x, px) in line.iter_mut().enumerate() {
                    *px = if caret && x < 2 * scale {
                        caret_color
                    } else if bitmap[y / scale * gw + x / scale] {
                        cell.fg
                    } else {
                        cell.bg
                    };
                }
            }
        }
    }

    /// The caret's color on `bg`: white on dark colors, black on light ones,
    /// so it shows whatever the cell's colors are (std.ui.5).
    fn contrast(bg: u32) -> u32 {
        let (r, g, b) = (bg >> 16 & 0xFF, bg >> 8 & 0xFF, bg & 0xFF);
        if r * 299 + g * 587 + b * 114 < 128_000 { 0xFF_FFFF } else { 0 }
    }

    /// One glyph at the font's own size, row by row.
    fn render(font: &MonoFont<'static>, ch: char) -> Vec<bool> {
        let (w, h) = (font.character_size.width as i32, font.character_size.height as i32);
        struct Bits(Vec<bool>, i32, i32);
        impl OriginDimensions for Bits {
            fn size(&self) -> Size {
                Size::new(self.1 as u32, self.2 as u32)
            }
        }
        impl DrawTarget for Bits {
            type Color = BinaryColor;
            type Error = core::convert::Infallible;
            fn draw_iter<I: IntoIterator<Item = Pixel<BinaryColor>>>(&mut self, pixels: I) -> Result<(), Self::Error> {
                for Pixel(p, c) in pixels {
                    if (0..self.1).contains(&p.x) && (0..self.2).contains(&p.y) {
                        self.0[(p.y * self.1 + p.x) as usize] = c.is_on();
                    }
                }
                Ok(())
            }
        }
        let mut bits = Bits(vec![false; (w * h) as usize], w, h);
        let mut buf = [0u8; 4];
        let style = MonoTextStyle::new(font, BinaryColor::On);
        let _ = Text::with_baseline(ch.encode_utf8(&mut buf), Point::zero(), style, Baseline::Top).draw(&mut bits);
        bits.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_clips_on_every_side() {
        let mut g = Grid::new(4, 2);
        g.put(-2, 0, "abcdef", 0x112233, 0x445566);
        let row0: String = g.cells[..4].iter().map(|c| c.ch).collect();
        assert_eq!(row0, "cdef");
        g.put(2, 1, "xyz", 1, 2);
        let row1: String = g.cells[4..].iter().map(|c| c.ch).collect();
        assert_eq!(row1, "  xy");
        assert_eq!(g.cells[6], Cell { ch: 'x', fg: 1, bg: 2 });
        let before = g.clone();
        g.put(0, 2, "zz", 0, 0);
        g.put(0, -1, "zz", 0, 0);
        g.put(4, 0, "zz", 0, 0);
        assert_eq!(g, before);
    }

    #[test]
    fn put_draws_unknown_glyphs_as_question_marks() {
        let mut g = Grid::new(5, 1);
        g.put(0, 0, "é\tλ😀a", -1, 0x1_000000);
        let row: String = g.cells.iter().map(|c| c.ch).collect();
        assert_eq!(row, "é???a");
        assert_eq!((g.cells[0].fg, g.cells[0].bg), (0xFF_FFFF, 0));
    }

    #[test]
    fn resize_keeps_what_fits() {
        let mut g = Grid::new(3, 2);
        g.put(0, 0, "abc", 0, 0);
        g.put(0, 1, "def", 0, 0);
        g.resize(2, 3);
        let all: String = g.cells.iter().map(|c| c.ch).collect();
        assert_eq!(all, "abde  ");
    }

    #[test]
    fn font_sizes_round_to_the_nearest() {
        assert_eq!(nearest_font_size(20), 20);
        assert_eq!(nearest_font_size(16), 15);
        assert_eq!(nearest_font_size(19), 18);
        assert_eq!(nearest_font_size(1), 10);
        assert_eq!(nearest_font_size(99), 40);
    }

    #[test]
    fn key_names() {
        use minifb::Key as K;
        let name = |k, chord| key_name(k, chord);
        assert_eq!(name(K::Enter, false).as_deref(), Some("enter"));
        assert_eq!(name(K::NumPadEnter, false).as_deref(), Some("enter"));
        assert_eq!(name(K::PageDown, false).as_deref(), Some("pagedown"));
        assert_eq!(name(K::F12, true).as_deref(), Some("f12"));
        assert_eq!(name(K::Left, false).as_deref(), Some("left"));
        // Text keys are typed characters unless Ctrl is held.
        assert_eq!(name(K::S, false), None);
        assert_eq!(name(K::S, true).as_deref(), Some("s"));
        assert_eq!(name(K::Key7, true).as_deref(), Some("7"));
        assert_eq!(name(K::Period, true).as_deref(), Some("."));
        assert_eq!(name(K::Space, true).as_deref(), Some(" "));
        assert_eq!(name(K::LeftShift, true), None);
        assert_eq!(name(K::LeftCtrl, true), None);
    }

    #[test]
    fn without_a_host_open_reports_no_display() {
        let e = open("t", 10, 5).unwrap_err();
        assert_eq!(e.0, "no_display");
        assert_eq!(open("t", 0, 5).unwrap_err().0, "other");
        assert_eq!(size(1), (0, 0));
        assert_eq!(next_event(1, true), Some(Event::Close));
    }

    #[test]
    fn without_a_host_the_clipboard_is_the_programs_own() {
        assert_eq!(clipboard_get(), None);
        clipboard_set("");
        assert_eq!(clipboard_get(), None);
        clipboard_set("a\nb");
        assert_eq!(clipboard_get().as_deref(), Some("a\nb"));
    }
}
