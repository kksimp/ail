//! A front-end allocator for the commands that only read a program (ADR 0332).
//!
//! `ail check`, `ail api` and `ail effects` build the token stream, the
//! syntax tree and the checker's tables, print a result and exit. Profiling
//! `ail check tools/edit/edit.ail` showed a quarter of its instructions in
//! `malloc` and `free` (about 178,000 short-lived blocks) and a third of its
//! wall time in the kernel, faulting in fresh heap pages one 4 KiB page at a
//! time.
//!
//! In front-end mode, blocks come from one reserved region: a small block
//! (up to `SMALL` bytes) is reused from a free list of its 16-byte size
//! class, anything else takes the next bytes; freeing a large block does
//! nothing until the process exits. On Linux the region asks for transparent
//! huge pages, so the kernel faults it in 2 MiB at a time. When the region is
//! full, blocks come from the system allocator, and each block is freed by
//! the allocator that made it (the address says which).
//!
//! The mode is switched on once, at the start of `main`, and never off.
//!
//! `ail run` and `ail test` use the runtime mode (ADR 0737): small blocks
//! (up to `SMALL` bytes, 16-byte classes) come from per-thread free lists
//! that refill from per-thread chunks of one reserved region, so the usual
//! allocate-and-free of a short-lived value is a few instructions and takes
//! no lock. A block freed by another thread joins that thread's list; a
//! thread that exits hands its lists to a shared pool. Larger blocks, blocks
//! of alignment over 16, and anything after the region is full come from the
//! system allocator, which also frees any block outside the region.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// The region's size; a check of the 4,800-line editor allocates 34 MB in all.
const REGION: usize = 256 << 20;
const SMALL: usize = 1024;
const CLASSES: usize = SMALL / 16;

struct State {
    start: usize,
    next: usize,
    end: usize,
    free: [usize; CLASSES],
}

/// The runtime mode's region (virtual; the kernel commits pages as they are touched).
const RT_REGION: usize = 16 << 30;
/// What a thread takes from the region at a time.
const RT_CHUNK: usize = 256 << 10;

/// A thread's free lists and the chunk it carves new blocks from.
struct Cache {
    free: [usize; CLASSES],
    next: usize,
    end: usize,
}

impl Drop for Cache {
    fn drop(&mut self) {
        // Hand the blocks on to the other threads rather than lose them.
        let (free, next, end) = (self.free, self.next, self.end);
        RT.with_pool(|p| {
            for c in 0..CLASSES {
                let mut head = free[c];
                while head != 0 {
                    // SAFETY: a freed block holds the address of the next one.
                    let rest = unsafe { *(head as *const usize) };
                    unsafe { *(head as *mut usize) = p.free[c] };
                    p.free[c] = head;
                    head = rest;
                }
            }
            // The rest of its chunk, as blocks of the largest class.
            let mut at = next;
            while end - at >= SMALL && next != 0 {
                unsafe { *(at as *mut usize) = p.free[CLASSES - 1] };
                p.free[CLASSES - 1] = at;
                at += SMALL;
            }
        });
    }
}

thread_local! {
    static CACHE: UnsafeCell<Cache> = const { UnsafeCell::new(Cache { free: [0; CLASSES], next: 0, end: 0 }) };
}

/// The runtime mode's shared state: the region's cursor and the blocks that
/// exited threads left behind.
struct Pool {
    free: [usize; CLASSES],
}

struct Runtime {
    start: AtomicUsize,
    end: AtomicUsize,
    cursor: AtomicUsize,
    lock: AtomicBool,
    pool: UnsafeCell<Pool>,
}

// SAFETY: `pool` is only touched while `lock` is held.
unsafe impl Sync for Runtime {}

static RT: Runtime = Runtime {
    start: AtomicUsize::new(0),
    end: AtomicUsize::new(0),
    cursor: AtomicUsize::new(0),
    lock: AtomicBool::new(false),
    pool: UnsafeCell::new(Pool { free: [0; CLASSES] }),
};

impl Runtime {
    fn with_pool<R>(&self, f: impl FnOnce(&mut Pool) -> R) -> R {
        while self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            std::hint::spin_loop();
        }
        // SAFETY: the lock is held.
        let r = f(unsafe { &mut *self.pool.get() });
        self.lock.store(false, Ordering::Release);
        r
    }

    fn owns(&self, ptr: *mut u8) -> bool {
        let a = ptr as usize;
        a >= self.start.load(Ordering::Relaxed) && a < self.end.load(Ordering::Relaxed)
    }

    /// A fresh chunk of the region, or 0 when it is used up.
    fn chunk(&self) -> usize {
        let at = self.cursor.fetch_add(RT_CHUNK, Ordering::Relaxed);
        if at + RT_CHUNK <= self.end.load(Ordering::Relaxed) { at } else { 0 }
    }
}

/// A block of class `c` for the calling thread; 0 where the region is used up
/// (the caller then takes one from the system allocator).
fn rt_alloc(c: usize) -> usize {
    CACHE
        .try_with(|cell| {
            // SAFETY: the cache belongs to this thread; no call below re-enters it.
            let k = unsafe { &mut *cell.get() };
            let head = k.free[c];
            if head != 0 {
                // SAFETY: a freed block holds the address of the next one.
                k.free[c] = unsafe { *(head as *const usize) };
                return head;
            }
            let size = (c + 1) * 16;
            if k.end - k.next >= size {
                let at = k.next;
                k.next += size;
                return at;
            }
            let from_pool = RT.with_pool(|p| {
                let head = p.free[c];
                if head != 0 {
                    p.free[c] = unsafe { *(head as *const usize) };
                }
                head
            });
            if from_pool != 0 {
                return from_pool;
            }
            // The rest of the old chunk is lost; it is less than one block.
            let chunk = RT.chunk();
            if chunk == 0 {
                return 0;
            }
            k.next = chunk + size;
            k.end = chunk + RT_CHUNK;
            chunk
        })
        .unwrap_or(0)
}

fn rt_free(ptr: usize, c: usize) {
    let local = CACHE.try_with(|cell| {
        // SAFETY: as in `rt_alloc`.
        let k = unsafe { &mut *cell.get() };
        unsafe { *(ptr as *mut usize) = k.free[c] };
        k.free[c] = ptr;
    });
    if local.is_err() {
        // The thread's cache is gone (it is exiting): the shared pool takes it.
        RT.with_pool(|p| {
            unsafe { *(ptr as *mut usize) = p.free[c] };
            p.free[c] = ptr;
        });
    }
}

pub struct FrontEnd {
    runtime: AtomicBool,
    on: AtomicBool,
    lock: AtomicBool,
    state: UnsafeCell<State>,
}

// SAFETY: `state` is only touched while `lock` is held.
unsafe impl Sync for FrontEnd {}

impl FrontEnd {
    pub const fn new() -> FrontEnd {
        FrontEnd {
            runtime: AtomicBool::new(false),
            on: AtomicBool::new(false),
            lock: AtomicBool::new(false),
            state: UnsafeCell::new(State { start: 0, next: 0, end: 0, free: [0; CLASSES] }),
        }
    }

    /// Switch to front-end allocation for the rest of the process. Does
    /// nothing where the region can't be reserved.
    pub fn enable(&self) {
        let Some((start, end)) = reserve() else { return };
        self.with_state(|s| {
            s.start = start;
            s.next = start;
            s.end = end;
        });
        self.on.store(true, Ordering::SeqCst);
    }

    /// Switch to the runtime mode for the rest of the process (`ail run`,
    /// `ail test`). Does nothing where the region can't be reserved.
    pub fn enable_runtime(&self) {
        if self.on.load(Ordering::Relaxed) {
            return;
        }
        let Some((start, end)) = reserve_sized(RT_REGION) else { return };
        RT.start.store(start, Ordering::SeqCst);
        RT.end.store(end, Ordering::SeqCst);
        RT.cursor.store(start, Ordering::SeqCst);
        self.runtime.store(true, Ordering::SeqCst);
    }

    fn with_state<R>(&self, f: impl FnOnce(&mut State) -> R) -> R {
        while self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed).is_err() {
            std::hint::spin_loop();
        }
        // SAFETY: the lock is held.
        let r = f(unsafe { &mut *self.state.get() });
        self.lock.store(false, Ordering::Release);
        r
    }

    fn owns(&self, ptr: *mut u8) -> bool {
        // `start` and `end` never change once `on` is set.
        let s = unsafe { &*self.state.get() };
        (s.start..s.end).contains(&(ptr as usize))
    }
}

/// The size class of a small block: its size rounded up to 16 bytes.
fn class(size: usize) -> usize {
    size.max(1).div_ceil(16) - 1
}

impl Default for FrontEnd {
    fn default() -> FrontEnd {
        FrontEnd::new()
    }
}

unsafe impl GlobalAlloc for FrontEnd {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if self.runtime.load(Ordering::Relaxed) {
            if layout.size() <= SMALL && layout.align() <= 16 {
                let got = rt_alloc(class(layout.size()));
                if got != 0 {
                    return got as *mut u8;
                }
            }
            return unsafe { System.alloc(layout) };
        }
        if !self.on.load(Ordering::Relaxed) || layout.align() > 16 {
            return unsafe { System.alloc(layout) };
        }
        let small = layout.size() <= SMALL;
        let got = self.with_state(|s| {
            let size = if small { (class(layout.size()) + 1) * 16 } else { layout.size().next_multiple_of(16) };
            if small {
                let c = class(layout.size());
                let head = s.free[c];
                if head != 0 {
                    // SAFETY: a freed block holds the address of the next one.
                    s.free[c] = unsafe { *(head as *const usize) };
                    return head;
                }
            }
            if s.end - s.next < size {
                return 0;
            }
            let at = s.next;
            s.next += size;
            at
        });
        if got == 0 { unsafe { System.alloc(layout) } } else { got as *mut u8 }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.runtime.load(Ordering::Relaxed) {
            if RT.owns(ptr) {
                rt_free(ptr as usize, class(layout.size()));
                return;
            }
            return unsafe { System.dealloc(ptr, layout) };
        }
        if !self.on.load(Ordering::Relaxed) || !self.owns(ptr) {
            return unsafe { System.dealloc(ptr, layout) };
        }
        if layout.size() <= SMALL {
            let c = class(layout.size());
            self.with_state(|s| {
                // SAFETY: the block is at least 16 bytes and 16-aligned.
                unsafe { *(ptr as *mut usize) = s.free[c] };
                s.free[c] = ptr as usize;
            });
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if self.runtime.load(Ordering::Relaxed) {
            if !RT.owns(ptr) {
                return unsafe { System.realloc(ptr, layout, new_size) };
            }
            // A block of ours: it already has room, or it moves.
            if new_size <= (class(layout.size()) + 1) * 16 {
                return ptr;
            }
            let new = unsafe { self.alloc(Layout::from_size_align_unchecked(new_size, layout.align())) };
            if !new.is_null() {
                unsafe {
                    std::ptr::copy_nonoverlapping(ptr, new, layout.size().min(new_size));
                    self.dealloc(ptr, layout);
                }
            }
            return new;
        }
        if !self.on.load(Ordering::Relaxed) || !self.owns(ptr) {
            return unsafe { System.realloc(ptr, layout, new_size) };
        }
        let old =
            if layout.size() <= SMALL { (class(layout.size()) + 1) * 16 } else { layout.size().next_multiple_of(16) };
        // The block already has room; freeing it later by its new size
        // returns it to a class no larger than it is.
        if new_size <= old {
            return ptr;
        }
        // A large block at the end of the region grows in place.
        if layout.size() > SMALL {
            let grown = self.with_state(|s| {
                let want = new_size.next_multiple_of(16);
                if s.next == ptr as usize + old && want >= old && s.end - (ptr as usize) >= want {
                    s.next = ptr as usize + want;
                    true
                } else {
                    false
                }
            });
            if grown {
                return ptr;
            }
        }
        let new = unsafe { self.alloc(Layout::from_size_align_unchecked(new_size, layout.align())) };
        if !new.is_null() {
            unsafe {
                std::ptr::copy_nonoverlapping(ptr, new, layout.size().min(new_size));
                self.dealloc(ptr, layout);
            }
        }
        new
    }
}

#[cfg(unix)]
unsafe extern "C" {
    fn mmap(addr: *mut u8, len: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut u8;
    #[cfg(target_os = "linux")]
    fn madvise(addr: *mut u8, len: usize, advice: i32) -> i32;
}

/// Reserve the region: anonymous memory the kernel commits as it's touched.
#[cfg(unix)]
fn reserve() -> Option<(usize, usize)> {
    reserve_sized(REGION)
}

#[cfg(unix)]
fn reserve_sized(len: usize) -> Option<(usize, usize)> {
    const PROT_READ_WRITE: i32 = 3;
    const MAP_PRIVATE: i32 = 2;
    #[cfg(target_os = "linux")]
    const MAP_ANONYMOUS: i32 = 0x20;
    #[cfg(not(target_os = "linux"))]
    const MAP_ANONYMOUS: i32 = 0x1000; // macOS and the BSDs
    // SAFETY: a fresh private mapping; no existing memory is touched.
    #[cfg(target_os = "linux")]
    const NORESERVE: i32 = 0x4000;
    #[cfg(not(target_os = "linux"))]
    const NORESERVE: i32 = 0x40;
    let p = unsafe { mmap(std::ptr::null_mut(), len, PROT_READ_WRITE, MAP_PRIVATE | MAP_ANONYMOUS | NORESERVE, -1, 0) };
    if p.is_null() || p as isize == -1 {
        return None;
    }
    // Transparent huge pages: one fault per 2 MiB rather than per 4 KiB.
    #[cfg(target_os = "linux")]
    unsafe {
        const MADV_HUGEPAGE: i32 = 14;
        madvise(p, len, MADV_HUGEPAGE);
    }
    Some((p as usize, p as usize + len))
}

#[cfg(not(unix))]
fn reserve() -> Option<(usize, usize)> {
    None
}

#[cfg(not(unix))]
fn reserve_sized(_len: usize) -> Option<(usize, usize)> {
    None
}
