//! FRESH=1: every allocation of at least `BIG` bytes gets new pages from the
//! system (mmap / VirtualAlloc) and returns them on free, whatever the size
//! and the allocator's history. Without it the system allocator decides:
//! glibc recycles a freed buffer or not depending on its dynamic thresholds,
//! so a cold bench pays the page faults of a sort's scratch at some sizes
//! only (300k-900k u32 on the Ryzen). With it, every size pays them, as an
//! isolated sort in a fresh program does (glibc's initial threshold, 128 KiB).

use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicU8, Ordering};

const BIG: usize = 128 * 1024;
/// 0: not read yet, 1: off, 2: on. Read at the first big allocation (the
/// read itself only makes small ones), never changed after.
static ON: AtomicU8 = AtomicU8::new(0);

fn on() -> bool {
    let mut v = ON.load(Ordering::Relaxed);
    if v == 0 {
        v = if std::env::var("FRESH").is_ok_and(|v| v == "1") { 2 } else { 1 };
        ON.store(v, Ordering::Relaxed);
    }
    v == 2
}

/// Optional: reads `FRESH` now (it is read at the first big allocation
/// otherwise).
#[allow(dead_code)]
pub fn init() { on(); }

pub struct Fresh;

fn big(l: &Layout) -> bool { l.size() >= BIG && l.align() <= 4096 && on() }

#[cfg(windows)]
mod os {
    unsafe extern "system" {
        fn VirtualAlloc(a: *mut u8, n: usize, t: u32, p: u32) -> *mut u8;
        fn VirtualFree(a: *mut u8, n: usize, t: u32) -> i32;
    }
    pub unsafe fn get(n: usize) -> *mut u8 { unsafe { VirtualAlloc(std::ptr::null_mut(), n, 0x3000, 4) } }
    pub unsafe fn put(p: *mut u8, _n: usize) { unsafe { VirtualFree(p, 0, 0x8000) }; }
}

#[cfg(unix)]
mod os {
    unsafe extern "C" {
        fn mmap(a: *mut u8, n: usize, prot: i32, flags: i32, fd: i32, off: i64) -> *mut u8;
        fn munmap(a: *mut u8, n: usize) -> i32;
    }
    #[cfg(target_os = "linux")]
    const ANON: i32 = 0x20;
    #[cfg(not(target_os = "linux"))]
    const ANON: i32 = 0x1000;
    pub unsafe fn get(n: usize) -> *mut u8 {
        let p = unsafe { mmap(std::ptr::null_mut(), n, 3, 2 | ANON, -1, 0) };
        if p as isize == -1 { std::ptr::null_mut() } else { p }
    }
    pub unsafe fn put(p: *mut u8, n: usize) { unsafe { munmap(p, n) }; }
}

unsafe impl GlobalAlloc for Fresh {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        if big(&l) { unsafe { os::get(l.size()) } } else { unsafe { System.alloc(l) } }
    }
    unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
        // New pages are already zero.
        if big(&l) { unsafe { os::get(l.size()) } } else { unsafe { System.alloc_zeroed(l) } }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        if big(&l) { unsafe { os::put(p, l.size()) } } else { unsafe { System.dealloc(p, l) } }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let nl = unsafe { Layout::from_size_align_unchecked(n, l.align()) };
        if !big(&l) && !big(&nl) {
            return unsafe { System.realloc(p, l, n) };
        }
        let q = unsafe { self.alloc(nl) };
        if !q.is_null() {
            unsafe { std::ptr::copy_nonoverlapping(p, q, l.size().min(n)) };
            unsafe { self.dealloc(p, l) };
        }
        q
    }
}
