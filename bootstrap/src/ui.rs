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
}

thread_local! {
    static SESSION: RefCell<Option<Session>> = const { RefCell::new(None) };
    static NEXT_ID: RefCell<i64> = const { RefCell::new(1) };
}

/// A request from the interpreter thread to the host.
enum Msg {
    Open { title: String, cols: usize, rows: usize, reply: Sender<Result<(), OpenError>> },
    Present { grid: Grid, cursor: (i64, i64) },
    Poll(Sender<Option<Event>>),
    Wait(Sender<Event>),
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
    SESSION.set(Some(Session { id, grid: Grid::new(cols, rows), cursor: (-1, -1), closed: false }));
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
    let event = if wait {
        let (reply, answer) = mpsc::channel();
        send(Msg::Wait(reply));
        answer.recv().unwrap_or(Event::Close)
    } else {
        let (reply, answer) = mpsc::channel();
        send(Msg::Poll(reply));
        answer.recv().unwrap_or(Some(Event::Close))?
    };
    with(id, |s| match &event {
        Event::Resize { cols, rows } => s.grid.resize(*cols, *rows),
        Event::Close => s.closed = true,
        Event::Key { .. } => {}
    });
    Some(event)
}

/// std.ui.8: close the window; later calls on it do nothing.
pub fn close(id: i64) {
    if with(id, |_| ()).is_some() {
        SESSION.set(None);
        send(Msg::Close);
    }
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
    let mut waiter: Option<Sender<Event>> = None;
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
                    None => Some(Event::Close),
                };
                let _ = reply.send(event);
            }
            Some(Msg::Wait(reply)) => {
                if let Some(w) = &mut window {
                    w.pump();
                }
                waiter = Some(reply);
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
                    let _ = reply.send(Event::Close);
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
        K::Minus => '-',
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

    use embedded_graphics::mono_font::{MonoFont, MonoTextStyle, iso_8859_1::FONT_10X20};
    use embedded_graphics::pixelcolor::BinaryColor;
    use embedded_graphics::prelude::*;
    use embedded_graphics::text::{Baseline, Text};
    use minifb::{Key, Window, WindowOptions};

    use super::{Cell, Event, Grid, OpenError, key_name};

    const FONT: MonoFont<'static> = FONT_10X20;
    /// One cell in pixels: the font's glyph size.
    const CW: usize = 10;
    const CH: usize = 20;

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
        cells: (usize, usize),
        closed: bool,
        pub events: VecDeque<Event>,
    }

    impl Backend {
        /// std.ui.1: `no_display` when there is nowhere to show a window.
        pub fn open(title: &str, cols: usize, rows: usize) -> Result<Backend, OpenError> {
            if cfg!(all(unix, not(target_os = "macos")))
                && std::env::var_os("DISPLAY").is_none_or(|d| d.is_empty())
            {
                return Err(("no_display", "no X11 display: DISPLAY is not set".to_string()));
            }
            let (w, h) = (cols * CW, rows * CH);
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
                cells: (cols, rows),
                closed: false,
                events: VecDeque::new(),
            };
            b.flush();
            Ok(b)
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
                self.events.push_back(Event::Close);
                return;
            }
            let (w, h) = self.window.get_size();
            if (w, h) != (self.w, self.h) && w > 0 && h > 0 {
                self.w = w;
                self.h = h;
                self.fb = vec![super::DEFAULT_BG; w * h];
                let cells = ((w / CW).max(1), (h / CH).max(1));
                if cells != self.cells {
                    self.cells = cells;
                    self.events.push_back(Event::Resize { cols: cells.0, rows: cells.1 });
                }
                if let Some((grid, cursor)) = self.shown.take() {
                    self.draw(grid, cursor, None);
                }
            }
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
                    self.events.push_back(Event::Key { key, ctrl: m.ctrl, alt: m.alt, shift: m.shift });
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
            let (x0, y0) = (col * CW, row * CH);
            if x0 >= self.w || y0 >= self.h {
                return;
            }
            let bitmap = self.glyphs.entry(cell.ch).or_insert_with(|| render(cell.ch));
            for y in 0..CH.min(self.h - y0) {
                let line = &mut self.fb[(y0 + y) * self.w + x0..][..CW.min(self.w - x0)];
                for (x, px) in line.iter_mut().enumerate() {
                    let on = bitmap[y * CW + x] || (caret && x < 2);
                    *px = if on { cell.fg } else { cell.bg };
                }
            }
        }
    }

    /// One glyph as `CW` by `CH` pixels, row by row.
    fn render(ch: char) -> Vec<bool> {
        struct Bits(Vec<bool>);
        impl OriginDimensions for Bits {
            fn size(&self) -> Size {
                Size::new(CW as u32, CH as u32)
            }
        }
        impl DrawTarget for Bits {
            type Color = BinaryColor;
            type Error = core::convert::Infallible;
            fn draw_iter<I: IntoIterator<Item = Pixel<BinaryColor>>>(&mut self, pixels: I) -> Result<(), Self::Error> {
                for Pixel(p, c) in pixels {
                    if (0..CW as i32).contains(&p.x) && (0..CH as i32).contains(&p.y) {
                        self.0[p.y as usize * CW + p.x as usize] = c.is_on();
                    }
                }
                Ok(())
            }
        }
        let mut bits = Bits(vec![false; CW * CH]);
        let mut buf = [0u8; 4];
        let style = MonoTextStyle::new(&FONT, BinaryColor::On);
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
}
