//! Histogram counters: `u32` up to `u32::MAX` keys (half the cache
//! footprint), `usize` beyond.

/// Histogram counter: `u32` for arrays up to `u32::MAX` elements.
pub(crate) trait Counter: Copy + std::ops::AddAssign {
    /// Constants of the counter type.
    const ZERO: Self;
    const ONE: Self;
    const FOUR: Self;
    /// The count as an index or a length.
    fn get(self) -> usize;
    /// A count (callers keep it within the counter type).
    fn from_usize(v: usize) -> Self;
}

impl Counter for u32 {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const FOUR: Self = 4;
    #[inline(always)]
    fn get(self) -> usize {
        self as usize
    }
    #[inline(always)]
    fn from_usize(v: usize) -> Self {
        v as u32
    }
}

impl Counter for usize {
    const ZERO: Self = 0;
    const ONE: Self = 1;
    const FOUR: Self = 4;
    #[inline(always)]
    fn get(self) -> usize {
        self
    }
    #[inline(always)]
    fn from_usize(v: usize) -> Self {
        v
    }
}
