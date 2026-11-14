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

/// The decisions taken from the cache size, one budget each, so that each
/// can follow the cache level that fits it on a given CPU (journal §68: on
/// the Core Ultra 9 185H, 2 MiB L2 and 24 MiB L3, a larger size helped
/// some decisions at 10M keys and hurt others). Every budget is the L2
/// for now; the call sites keep their own fractions of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Budget {
    /// 11-bit digits for 32-bit keys while keys and buffer fit (`engines/lsd`).
    Lsd11,
    /// 32-bit integers: rollercoaster instead of the fused LSD (`integers`).
    Roller32,
    /// 64-bit integers: rollercoaster instead of the DLSD (`integers`).
    Roller64,
    /// 128-bit integers: rollercoaster instead of the DLSD (`wide128`).
    Roller128,
    /// f32: rollercoaster instead of the fused LSD (`floats`).
    RollerF32,
    /// f64: rollercoaster instead of the DLSD (`floats`).
    RollerF64,
    /// DLSD on floats: from which size bucket starts are checked for aliasing.
    AliasFloat,
    /// DLSD on integers: the same.
    AliasInt,
    /// Counting sort: 4 counter lanes while the 4 tables fit (`detect/counting`).
    CountLanes,
    /// Wide range counted all the same while its counters fit (`detect/range`).
    WideCount,
    /// Two-run merge: two-ended through a scratch while it fits (`merge/two_way`).
    MergeTwoWay,
    /// Run merges: ping-pong rounds while array and buffer fit (`merge`).
    PingPong,
    /// Records: beyond this the moves dominate (LSD on small records, Verge).
    Records,
}

/// Every budget with its name (benchmarks).
pub(crate) const BUDGETS: [(Budget, &str); 13] = [
    (Budget::Lsd11, "lsd11"),
    (Budget::Roller32, "roller32"),
    (Budget::Roller64, "roller64"),
    (Budget::Roller128, "roller128"),
    (Budget::RollerF32, "rollerf32"),
    (Budget::RollerF64, "rollerf64"),
    (Budget::AliasFloat, "aliasfloat"),
    (Budget::AliasInt, "aliasint"),
    (Budget::CountLanes, "countlanes"),
    (Budget::WideCount, "widecount"),
    (Budget::MergeTwoWay, "mergetwoway"),
    (Budget::PingPong, "pingpong"),
    (Budget::Records, "records"),
];

/// Cache bytes a decision is taken from (see [`Budget`]).
#[inline]
pub(crate) fn budget(b: Budget) -> usize {
    #[cfg(test)]
    {
        let forced = BUDGET_OVERRIDE[b as usize].load(std::sync::atomic::Ordering::Relaxed);
        if forced != 0 {
            return forced;
        }
    }
    let _ = b;
    l2_bytes()
}

/// Benchmarks: one decision's budget forced (0: the default), the others
/// untouched, to find which cache level each decision should follow.
#[cfg(test)]
pub(crate) static BUDGET_OVERRIDE: [std::sync::atomic::AtomicUsize; 13] =
    [const { std::sync::atomic::AtomicUsize::new(0) }; 13];

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
