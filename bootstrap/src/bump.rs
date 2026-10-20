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
//! `ail run` and `ail test` run the program and keep the system allocator.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

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

pub struct FrontEnd {
    on: AtomicBool,
    lock: AtomicBool,
    state: UnsafeCell<State>,
}

// SAFETY: `state` is only touched while `lock` is held.
unsafe impl Sync for FrontEnd {}

impl FrontEnd {
    pub const fn new() -> FrontEnd {
        FrontEnd {
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
        if !self.on.load(Ordering::Relaxed) || !self.owns(ptr) {
            return unsafe { System.realloc(ptr, layout, new_size) };
        }
        let old = if layout.size() <= SMALL { (class(layout.size()) + 1) * 16 } else { layout.size().next_multiple_of(16) };
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
    const PROT_READ_WRITE: i32 = 3;
    const MAP_PRIVATE: i32 = 2;
    #[cfg(target_os = "linux")]
    const MAP_ANONYMOUS: i32 = 0x20;
    #[cfg(not(target_os = "linux"))]
    const MAP_ANONYMOUS: i32 = 0x1000; // macOS and the BSDs
    // SAFETY: a fresh private mapping; no existing memory is touched.
    let p = unsafe { mmap(std::ptr::null_mut(), REGION, PROT_READ_WRITE, MAP_PRIVATE | MAP_ANONYMOUS, -1, 0) };
    if p.is_null() || p as isize == -1 {
        return None;
    }
    // Transparent huge pages: one fault per 2 MiB rather than per 4 KiB.
    #[cfg(target_os = "linux")]
    unsafe {
        const MADV_HUGEPAGE: i32 = 14;
        madvise(p, REGION, MADV_HUGEPAGE);
    }
    Some((p as usize, p as usize + REGION))
}

#[cfg(not(unix))]
fn reserve() -> Option<(usize, usize)> {
    None
}
