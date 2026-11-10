//! Cache sizes read once at run time, for the cache-linked sizes of the
//! float sorts (leaf size, rollercoaster thresholds, staging threshold,
//! aliasing criterion).
//!
//! - L2: the smallest level-2 data or unified cache reported by the system
//!   (on hybrid processors the efficiency cores may have another size:
//!   the smallest is the safe choice). Snapdragon X Elite: 12 MiB per
//!   cluster of 4 cores; recent x86 cores: 1-2 MiB.
//! - Page: the virtual memory page size (4 KiB on x86 and most ARM
//!   systems, 16 KiB on Apple Silicon, 64 KiB on some ARM Linux).
//!
//! Sources: Windows `GetLogicalProcessorInformationEx` / `GetSystemInfo`,
//! Linux `/sys/devices/system/cpu/cpu0/cache` / `sysconf`, macOS
//! `sysctlbyname`. Elsewhere, or when the query fails: `DEFAULT_L2` and
//! `DEFAULT_PAGE`, conservative values.

use std::sync::OnceLock;

/// L2 size assumed when it cannot be read: small, as on many x86 cores.
const DEFAULT_L2: usize = 1 << 20;
/// Page size assumed when it cannot be read.
const DEFAULT_PAGE: usize = 4096;
/// Values read outside these bounds are ignored (bad report).
const L2_RANGE: (usize, usize) = (128 << 10, 256 << 20);
const PAGE_RANGE: (usize, usize) = (4096, 1 << 20);

#[derive(Clone, Copy, Debug)]
pub(crate) struct CacheInfo {
    /// L2 size in bytes.
    pub l2: usize,
    /// Page size in bytes.
    pub page: usize,
    /// Whether the L2 size was read from the system.
    pub l2_detected: bool,
}

static INFO: OnceLock<CacheInfo> = OnceLock::new();

/// Tests / benchmarks: L2 size override in bytes (0: detected).
#[cfg(test)]
pub(crate) static L2_OVERRIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Cache sizes of this machine (read on the first call).
pub(crate) fn cache_info() -> CacheInfo {
    #[allow(unused_mut)]
    let mut info = *INFO.get_or_init(detect);
    #[cfg(test)]
    {
        let o = L2_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed);
        if o != 0 {
            info.l2 = o;
        }
    }
    info
}

/// L2 size in bytes.
#[inline]
pub(crate) fn l2_bytes() -> usize {
    cache_info().l2
}

/// Page size in bytes.
#[inline]
pub(crate) fn page_bytes() -> usize {
    cache_info().page
}

fn detect() -> CacheInfo {
    let (l2, page) = os::query();
    let l2 = l2.filter(|&v| v >= L2_RANGE.0 && v <= L2_RANGE.1);
    let page = page.filter(|&v| v >= PAGE_RANGE.0 && v <= PAGE_RANGE.1 && v.is_power_of_two());
    CacheInfo { l2: l2.unwrap_or(DEFAULT_L2), page: page.unwrap_or(DEFAULT_PAGE), l2_detected: l2.is_some() }
}

#[cfg(windows)]
mod os {
    // Layouts from the Windows SDK (winnt.h, sysinfoapi.h).
    const RELATION_CACHE: u32 = 2;
    const CACHE_UNIFIED: u32 = 0;
    const CACHE_DATA: u32 = 2;

    #[repr(C)]
    struct SystemInfo {
        oem_id: u32,
        page_size: u32,
        minimum_application_address: *mut u8,
        maximum_application_address: *mut u8,
        active_processor_mask: usize,
        number_of_processors: u32,
        processor_type: u32,
        allocation_granularity: u32,
        processor_level: u16,
        processor_revision: u16,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GetLogicalProcessorInformationEx(relationship: u32, buffer: *mut u8, returned_length: *mut u32) -> i32;
        fn GetSystemInfo(info: *mut SystemInfo);
    }

    pub(super) fn query() -> (Option<usize>, Option<usize>) {
        (l2(), page())
    }

    fn page() -> Option<usize> {
        let mut info = std::mem::MaybeUninit::<SystemInfo>::zeroed();
        // SAFETY: GetSystemInfo fills the whole structure, whose layout
        // matches SYSTEM_INFO; it cannot fail.
        let info = unsafe {
            GetSystemInfo(info.as_mut_ptr());
            info.assume_init()
        };
        Some(info.page_size as usize)
    }

    fn l2() -> Option<usize> {
        let mut len: u32 = 0;
        // SAFETY: a null buffer with length 0 only asks for the needed size.
        unsafe { GetLogicalProcessorInformationEx(RELATION_CACHE, std::ptr::null_mut(), &mut len) };
        if len == 0 {
            return None;
        }
        // u64 storage: entries are 8-byte aligned.
        let mut buf = vec![0u64; (len as usize).div_ceil(8)];
        // SAFETY: the buffer holds at least `len` bytes.
        let ok = unsafe { GetLogicalProcessorInformationEx(RELATION_CACHE, buf.as_mut_ptr() as *mut u8, &mut len) };
        if ok == 0 {
            return None;
        }
        // SAFETY: `len <= 8 * buf.len()` bytes were written by the call.
        let bytes = unsafe { std::slice::from_raw_parts(buf.as_ptr() as *const u8, len as usize) };
        let u32_at = |o: usize| bytes.get(o..o + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
        let mut smallest: Option<usize> = None;
        let mut off = 0;
        // Entries: Relationship u32, Size u32, then CACHE_RELATIONSHIP:
        // Level u8, Associativity u8, LineSize u16, CacheSize u32, Type u32.
        while off + 20 <= bytes.len() {
            let (relationship, size) = (u32_at(off)?, u32_at(off + 4)? as usize);
            if size == 0 {
                break;
            }
            if relationship == RELATION_CACHE {
                let level = bytes[off + 8];
                let cache_size = u32_at(off + 12)? as usize;
                let kind = u32_at(off + 16)?;
                if level == 2 && (kind == CACHE_DATA || kind == CACHE_UNIFIED) {
                    smallest = Some(smallest.map_or(cache_size, |s| s.min(cache_size)));
                }
            }
            off += size;
        }
        smallest
    }
}

#[cfg(target_os = "linux")]
mod os {
    extern "C" {
        fn sysconf(name: i32) -> i64;
    }
    /// `_SC_PAGESIZE` (glibc and musl).
    const SC_PAGESIZE: i32 = 30;

    pub(super) fn query() -> (Option<usize>, Option<usize>) {
        // SAFETY: sysconf has no preconditions.
        let page = unsafe { sysconf(SC_PAGESIZE) };
        (l2(), if page > 0 { Some(page as usize) } else { None })
    }

    /// Sizes like "1024K" or "12M".
    fn parse_size(s: &str) -> Option<usize> {
        let s = s.trim();
        let (num, mult) = match s.chars().last()? {
            'K' => (&s[..s.len() - 1], 1 << 10),
            'M' => (&s[..s.len() - 1], 1 << 20),
            'G' => (&s[..s.len() - 1], 1 << 30),
            _ => (s, 1),
        };
        num.parse::<usize>().ok().map(|v| v * mult)
    }

    fn l2() -> Option<usize> {
        let mut smallest: Option<usize> = None;
        let cpus = std::fs::read_dir("/sys/devices/system/cpu").ok()?;
        for cpu in cpus.flatten() {
            let name = cpu.file_name();
            let name = name.to_string_lossy();
            if !name.starts_with("cpu") || !name[3..].chars().all(|c| c.is_ascii_digit()) || name.len() == 3 {
                continue;
            }
            let Ok(indexes) = std::fs::read_dir(cpu.path().join("cache")) else { continue };
            for index in indexes.flatten() {
                let read = |f: &str| std::fs::read_to_string(index.path().join(f)).ok();
                let (Some(level), Some(kind), Some(size)) = (read("level"), read("type"), read("size")) else {
                    continue;
                };
                if level.trim() == "2" && matches!(kind.trim(), "Data" | "Unified") {
                    if let Some(v) = parse_size(&size) {
                        smallest = Some(smallest.map_or(v, |s| s.min(v)));
                    }
                }
            }
        }
        smallest
    }
}

#[cfg(target_os = "macos")]
mod os {
    extern "C" {
        fn sysctlbyname(name: *const u8, oldp: *mut u8, oldlenp: *mut usize, newp: *mut u8, newlen: usize) -> i32;
    }

    /// Integer sysctl value (`name` NUL-terminated).
    fn sysctl(name: &[u8]) -> Option<usize> {
        let mut v: u64 = 0;
        let mut len = std::mem::size_of::<u64>();
        // SAFETY: `name` is NUL-terminated, `v` holds `len` bytes; the call
        // writes at most `len` bytes and updates `len`.
        let r = unsafe { sysctlbyname(name.as_ptr(), &mut v as *mut u64 as *mut u8, &mut len, std::ptr::null_mut(), 0) };
        if r != 0 || (len != 4 && len != 8) {
            return None;
        }
        Some(if len == 4 { v as u32 as usize } else { v as usize })
    }

    pub(super) fn query() -> (Option<usize>, Option<usize>) {
        // Apple Silicon: one value per performance level, else the global one.
        let levels: Vec<usize> = [&b"hw.perflevel0.l2cachesize\0"[..], &b"hw.perflevel1.l2cachesize\0"[..]]
            .iter()
            .filter_map(|n| sysctl(n))
            .collect();
        let l2 = levels.into_iter().min().or_else(|| sysctl(b"hw.l2cachesize\0"));
        (l2, sysctl(b"hw.pagesize\0"))
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
mod os {
    pub(super) fn query() -> (Option<usize>, Option<usize>) {
        (None, None)
    }
}

/// Whether the starts of the non-empty buckets of a scatter pass (byte
/// offsets) crowd on a few cache-line positions inside a page: their write
/// streams then map to the same cache sets and evict each other (integer
/// valued or grid floats give bucket sizes that are multiples of powers of
/// two). Criterion: one of the page's line positions holds more than 16
/// bucket starts (more streams than an L1 set has ways) and more than 4
/// times the average. The page stands for the L1 way size (equal on x86
/// and Apple Silicon); lines of 64 bytes. Measured on ARM only.
pub(crate) fn crowded(byte_starts: impl Iterator<Item = usize>) -> bool {
    const LINE: usize = 64;
    const MAX_LINES: usize = 1024;
    let lines = (page_bytes() / LINE).clamp(1, MAX_LINES);
    let period = lines * LINE;
    let mut classes = [0u32; MAX_LINES];
    let mut nonempty = 0u32;
    for b in byte_starts {
        classes[(b % period) / LINE] += 1;
        nonempty += 1;
    }
    let max = *classes[..lines].iter().max().unwrap();
    max > 16 && max > 4 * nonempty.div_ceil(lines as u32)
}
