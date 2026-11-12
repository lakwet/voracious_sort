//! Cache and page sizes asked to the system (Windows, Linux, macOS);
//! `None` elsewhere or when the query fails.

#[cfg(windows)]
mod imp {
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

    pub(crate) fn query() -> (Option<usize>, Option<usize>) {
        (smallest(2), page())
    }

    pub(crate) fn levels() -> [Option<usize>; 3] {
        [smallest(1), smallest(2), smallest(3)]
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

    /// Smallest data or unified cache of `wanted` level.
    fn smallest(wanted: u8) -> Option<usize> {
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
                if level == wanted && (kind == CACHE_DATA || kind == CACHE_UNIFIED) {
                    smallest = Some(smallest.map_or(cache_size, |s| s.min(cache_size)));
                }
            }
            off += size;
        }
        smallest
    }
}

#[cfg(target_os = "linux")]
mod imp {
    extern "C" {
        fn sysconf(name: i32) -> i64;
    }
    /// `_SC_PAGESIZE` (glibc and musl).
    const SC_PAGESIZE: i32 = 30;

    pub(crate) fn query() -> (Option<usize>, Option<usize>) {
        // SAFETY: sysconf has no preconditions.
        let page = unsafe { sysconf(SC_PAGESIZE) };
        (smallest(2), if page > 0 { Some(page as usize) } else { None })
    }

    pub(crate) fn levels() -> [Option<usize>; 3] {
        [smallest(1), smallest(2), smallest(3)]
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

    /// Smallest data or unified cache of `wanted` level over the CPUs.
    fn smallest(wanted: u32) -> Option<usize> {
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
                if level.trim().parse::<u32>().ok() == Some(wanted) && matches!(kind.trim(), "Data" | "Unified") {
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
mod imp {
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

    pub(crate) fn query() -> (Option<usize>, Option<usize>) {
        // Apple Silicon: one value per performance level, else the global one.
        let levels: Vec<usize> = [&b"hw.perflevel0.l2cachesize\0"[..], &b"hw.perflevel1.l2cachesize\0"[..]]
            .iter()
            .filter_map(|n| sysctl(n))
            .collect();
        let l2 = levels.into_iter().min().or_else(|| sysctl(b"hw.l2cachesize\0"));
        (l2, sysctl(b"hw.pagesize\0"))
    }

    pub(crate) fn levels() -> [Option<usize>; 3] {
        [sysctl(b"hw.l1dcachesize\0"), query().0, sysctl(b"hw.l3cachesize\0")]
    }
}

#[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
mod imp {
    pub(crate) fn query() -> (Option<usize>, Option<usize>) {
        (None, None)
    }

    pub(crate) fn levels() -> [Option<usize>; 3] {
        [None, None, None]
    }
}

pub(super) use imp::{levels, query};
