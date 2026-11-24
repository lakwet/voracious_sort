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
/// The same for the page size.
const PAGE_RANGE: (usize, usize) = (4096, 1 << 20);

#[derive(Clone, Copy, Debug)]
struct CacheSizes {
    l2: usize,
    page: usize,
}

/// The decisions taken from the cache size, one budget each, so that each
/// can follow the cache level that fits it on a given CPU (with a small L2
/// and a large L3, a larger size helps some decisions and hurts others).
/// Every budget is the L2 for now; the call sites keep their own fractions
/// of it.
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
#[cfg(test)]
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
    match b {
        // Measured on three machines (ARM, AMD, Intel): the 32-bit
        // rollercoaster threshold and the wide counting follow the largest
        // cache a core reaches.
        Budget::Roller32 | Budget::WideCount => largest_bytes(),
        _ => l2_bytes(),
    }
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

/// Largest cache a core reaches: the L2, or the L3 when one is reported
/// above it. For the decisions measured to follow it on three machines.
#[inline]
pub(crate) fn largest_bytes() -> usize {
    #[cfg(test)]
    {
        let forced = L2_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed);
        if forced != 0 {
            return forced;
        }
    }
    static LARGEST: OnceLock<usize> = OnceLock::new();
    *LARGEST.get_or_init(|| sizes().l2.max(os::levels()[2].unwrap_or(0)))
}

/// An older Intel core: Intel x86-64 with an L2 under 512 KiB (the client
/// cores of 2013-2019, Haswell to Comet Lake: 256 KiB). Their front end is
/// bound by uops (4 per cycle, indexed read-modify-writes split in two), not
/// by loads as Zen or the ARM cores measured: they get their own counting
/// paths. AMD and ARM cores with a small L2 keep the common ones. Tests
/// that force a small L2 (`L2_OVERRIDE`) take these paths on any CPU.
#[inline]
pub(crate) fn old_intel() -> bool {
    #[cfg(test)]
    {
        let forced = L2_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed);
        if forced != 0 {
            return forced < 512 << 10;
        }
    }
    intel() && l2_bytes() < 512 << 10
}

/// Whether the CPU is an Intel one (CPUID vendor string), read once.
fn intel() -> bool {
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        static INTEL: OnceLock<bool> = OnceLock::new();
        *INTEL.get_or_init(|| {
            // SAFETY: CPUID leaf 0 exists on every x86-64 CPU.
            #[allow(unused_unsafe)]
            let r = unsafe { std::arch::x86_64::__cpuid(0) };
            // "GenuineIntel" in EBX, EDX, ECX.
            r.ebx == 0x756e_6547 && r.edx == 0x4965_6e69 && r.ecx == 0x6c65_746e
        })
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    false
}

/// Benchmarks: L2 size forced (0: the detected one), to check on a machine
/// that the thresholds derived from it hold.
#[cfg(test)]
pub(crate) static L2_OVERRIDE: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);

/// The L2 size as the OS reports it (`None`: not read, the default is used).
#[cfg(all(test, feature = "bench"))]
pub(crate) fn reported_l2() -> Option<usize> { os::query().0 }

/// L1 data, L2 and L3 sizes as the OS reports them (smallest of each level;
/// for reports: the sorts read the L2 alone).
#[cfg(all(test, feature = "bench"))]
pub(crate) fn reported_levels() -> [Option<usize>; 3] { os::levels() }

/// Page size in bytes.
#[inline]
pub(crate) fn page_bytes() -> usize { sizes().page }

fn sizes() -> CacheSizes { *SIZES.get_or_init(detect) }

fn detect() -> CacheSizes {
    let (l2, page) = os::query();
    let l2 = l2.filter(|&v| v >= L2_RANGE.0 && v <= L2_RANGE.1);
    let page = page.filter(|&v| v >= PAGE_RANGE.0 && v <= PAGE_RANGE.1 && v.is_power_of_two());
    CacheSizes { l2: l2.unwrap_or(DEFAULT_L2), page: page.unwrap_or(DEFAULT_PAGE) }
}
