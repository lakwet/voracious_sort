//! Cache sizes, read once at run time: the thresholds of the sorts are
//! derived from them (L2: smallest level-2 data or unified cache reported;
//! page: virtual memory page size). Defaults when they cannot be read.

mod aliasing;
mod os;

pub(crate) use aliasing::crowded;

use std::sync::OnceLock;

/// L2 size assumed when it cannot be read: small, as on many x86 cores.
const DEFAULT_L2: usize = 1 << 20;
/// Page size assumed when it cannot be read.
const DEFAULT_PAGE: usize = 4096;
/// Values read outside these bounds are ignored (bad report).
const L2_RANGE: (usize, usize) = (128 << 10, 256 << 20);
const PAGE_RANGE: (usize, usize) = (4096, 1 << 20);

#[derive(Clone, Copy, Debug)]
struct CacheSizes {
    l2: usize,
    page: usize,
}

static SIZES: OnceLock<CacheSizes> = OnceLock::new();

/// L2 size in bytes.
#[inline]
pub(crate) fn l2_bytes() -> usize {
    #[cfg(test)]
    {
        let forced = L2_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed);
        if forced != 0 {
            return forced;
        }
    }
    sizes().l2
}

/// Benchmarks: L2 size forced (0: the detected one), to check on a machine
/// that the thresholds derived from it hold.
#[cfg(test)]
pub(crate) static L2_OVERRIDE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// The L2 size as the OS reports it (`None`: not read, the default is used).
pub(crate) fn reported_l2() -> Option<usize> {
    os::query().0
}

/// L1 data, L2 and L3 sizes as the OS reports them (smallest of each level;
/// for reports: the sorts read the L2 alone).
pub(crate) fn reported_levels() -> [Option<usize>; 3] {
    os::levels()
}

/// Page size in bytes.
#[inline]
pub(crate) fn page_bytes() -> usize {
    sizes().page
}

fn sizes() -> CacheSizes {
    *SIZES.get_or_init(detect)
}

fn detect() -> CacheSizes {
    let (l2, page) = os::query();
    let l2 = l2.filter(|&v| v >= L2_RANGE.0 && v <= L2_RANGE.1);
    let page = page.filter(|&v| v >= PAGE_RANGE.0 && v <= PAGE_RANGE.1 && v.is_power_of_two());
    CacheSizes { l2: l2.unwrap_or(DEFAULT_L2), page: page.unwrap_or(DEFAULT_PAGE) }
}
