//! # Voracious sort
//!
//! Voracious sort is a [sorting algorithm](https://en.wikipedia.org/wiki/Sorting_algorithm), like
//! [Rust standard sort](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort)
//! or
//! [Rust unstable sort](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_unstable).
//! However it is a [radix sort](https://en.wikipedia.org/wiki/Radix_sort), it is a non comparative sort.
//! It is a **very fast sort** which compares very well against Rust standard
//! and unstable sorts or others state of the art sorting algorithms (see
//! runtimes below).
//!
//! Voracious sort can sort a
//! [`vector`](https://doc.rust-lang.org/stable/std/vec/)
//! or a
//! [`slice`](https://doc.rust-lang.org/stable/std/primitive.slice.html)
//! of:
//! - [`bool`](https://doc.rust-lang.org/stable/std/primitive.bool.html)
//! - [`char`](https://doc.rust-lang.org/stable/std/primitive.char.html)
//! - [`f32`](https://doc.rust-lang.org/stable/std/primitive.f32.html),
//! [`f64`](https://doc.rust-lang.org/stable/std/primitive.f64.html)
//! - [`i8`](https://doc.rust-lang.org/stable/std/primitive.i8.html),
//! [`i16`](https://doc.rust-lang.org/stable/std/primitive.i16.html),
//! [`i32`](https://doc.rust-lang.org/stable/std/primitive.i32.html),
//! [`i64`](https://doc.rust-lang.org/stable/std/primitive.i64.html),
//! [`i128`](https://doc.rust-lang.org/stable/std/primitive.i128.html)
//! - [`isize`](https://doc.rust-lang.org/stable/std/primitive.isize.html)
//! - [`u8`](https://doc.rust-lang.org/stable/std/primitive.u8.html),
//! [`u16`](https://doc.rust-lang.org/stable/std/primitive.u16.html),
//! [`u32`](https://doc.rust-lang.org/stable/std/primitive.u32.html),
//! [`u64`](https://doc.rust-lang.org/stable/std/primitive.u64.html),
//! [`u128`](https://doc.rust-lang.org/stable/std/primitive.u128.html)
//! - [`usize`](https://doc.rust-lang.org/stable/std/primitive.usize.html)
//! - [`struct`](https://doc.rust-lang.org/std/keyword.struct.html)
//!   - The struct must be mapped to a key. The key must be among the aforementioned types (bool, char, f32, etc...).
//!   - **Single thread** version: the struct must implement **`PartialOrd`**, **`PartialEq`**, **`Copy`** and **`Radixable`** traits.
//!   - **Multi thread** version: the struct must implement **`PartialOrd`**, **`PartialEq`**, **`Copy`**, **`Send`**, **`Sync`** and **`Radixable`** traits.
//!
//! Vocarious sort can only sort in ascending order. You can call the
//! [`reverse`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.reverse)
//! method if desired.
//!
//! Because of Rust Orphan Rule, we chose not to support tuple sorting. You
//! can use [struct](https://doc.rust-lang.org/std/keyword.struct.html) instead.
//!
//! ## Version
//!
//! Last version tested/used:
//! - Rustc: 1.46.0 stable
//! - Rustfmt: 1.4.18 stable
//! - Cargo: 1.46.0 stable
//! - Clippy: 0.0.212
//!
//! ## License
//!
//! See the license file.
//!
//! ## How to use it
//!
//! Add in `Cargo.toml` if you want only single thread version:
//! ```toml
//! [dependencies]
//! voracious_radix_sort = { version = "1.2.0" }
//! ```
//!
//! If you also want the multithread version:
//! ```toml
//! [dependencies]
//! voracious_radix_sort = { version = "1.2.0", features = ["voracious_multithread"] }
//! ```
//!
//! ### Environment variable
//!
//! To fully benefit from Voracious sort, it is better to add the environment
//! variable:
//!
//! ```toml
//! export RUSTFLAGS="-C target-cpu=native"
//! ```
//!
//! ### Methods
//!
//! When the Crate is imported, three methods are added to vectors and slices:
//! - `voracious_sort()` (single thread).
//! - `voracious_stable_sort()` (single thread).
//! - `voracious_mt_sort()` (multi thread). (with the "`voracious_multithread`" feature)
//!
//! ### Example
//!
//! ```ignore
//! use voracious_radix_sort::{RadixSort};
//!
//! let mut array = vec![2, 45, 8, 7, 9, 65, 8, 74, 1, 2, 56, 9, 7, 41];
//!
//! array.voracious_sort();
//!
//! assert_eq!(array, vec![1, 2, 2, 7, 7, 8, 8, 9, 9, 41, 45, 56, 65, 74]);
//!
//! let mut array = vec![2, 45, 8, 7, 9, 65, 8, 74, 1, 2, 56, 9, 7, 41];
//!
//! array.voracious_stable_sort();
//!
//! assert_eq!(array, vec![1, 2, 2, 7, 7, 8, 8, 9, 9, 41, 45, 56, 65, 74]);
//!
//! let mut array = vec![2, 45, 8, 7, 9, 65, 8, 74, 1, 2, 56, 9, 7, 41];
//!
//! // mt: Multithread sort.
//! // The argument is the number of threads for the threadpool.
//! array.voracious_mt_sort(4);
//!
//! assert_eq!(array, vec![1, 2, 2, 7, 7, 8, 8, 9, 9, 41, 45, 56, 65, 74]);
//! ```
//!
//! ### Implementing a custom `struct`
//!
//! Let's do it through an example.
//!
//! ```
//! use std::cmp::Ordering;
//!
//! // We need a struct.
//! // We want, for example, to sort these structs by the key: "value".
//! // This struct must implement the Copy and Clone traits, we can just derive them.
//! // For the multithread version the struct must implement de `Send` and `Sync` traits
//! // too, which are by default for primitive types.
//! #[derive(Copy, Clone, Debug)]
//! pub struct Custom {
//!     value: f32,
//!     other: usize,
//! }
//! impl PartialOrd for Custom {
//!     fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
//!         self.value.partial_cmp(&other.value)
//!     }
//! }
//! impl PartialEq for Custom {
//!     fn eq(&self, other: &Self) -> bool {
//!         self.value == other.value
//!     }
//! }
//! ```
//!
//! And then we have to implement the `Radixable` traits:
//! ```
//! # use std::cmp::Ordering;
//! use voracious_radix_sort::Radixable;
//! # #[derive(Copy, Clone, Debug)]
//! # pub struct Custom {
//! #     value: f32,
//! #     other: usize,
//! # }
//! # impl PartialOrd for Custom {
//! #     fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
//! #         self.value.partial_cmp(&other.value)
//! #     }
//! # }
//! # impl PartialEq for Custom {
//! #     fn eq(&self, other: &Self) -> bool {
//! #         self.value == other.value
//! #     }
//! # }
//!
//! impl Radixable<f32> for Custom {
//!     type Key = f32;
//!     #[inline]
//!     fn key(&self) -> Self::Key {
//!         self.value
//!     }
//! }
//! ```
//!
//! **See more implementation examples on our [GitHub](https://github.com/lakwet/voracious_sort)**
//!
//! When it is done, we can run a test:
//! ```
//! use voracious_radix_sort::RadixSort;
//! # use voracious_radix_sort::Radixable;
//! # use std::cmp::Ordering;
//! # #[derive(Copy, Clone, Debug)]
//! # pub struct Custom {
//! #     value: f32,
//! #     other: usize,
//! # }
//! # impl PartialOrd for Custom {
//! #     fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
//! #         self.value.partial_cmp(&other.value)
//! #     }
//! # }
//! # impl PartialEq for Custom {
//! #     fn eq(&self, other: &Self) -> bool {
//! #         self.value == other.value
//! #     }
//! # }
//! # impl Radixable<f32> for Custom {
//! #     type Key = f32;
//! #     #[inline]
//! #     fn key(&self) -> Self::Key {
//! #         self.value
//! #     }
//! # }
//!
//! let mut array = vec![
//!     Custom { value: 5.7, other: 29 },
//!     Custom { value: 2.7, other: 23 },
//!     Custom { value: 14.7, other: 17 },
//!     Custom { value: 4.7, other: 35 },
//! ];
//!
//! array.voracious_sort();
//!
//! assert_eq!(array, vec![
//!     Custom { value: 2.7, other: 23 },
//!     Custom { value: 4.7, other: 35 },
//!     Custom { value: 5.7, other: 29 },
//!     Custom { value: 14.7, other: 17 },
//! ]);
//!
//! let mut array = vec![
//!     Custom { value: 5.7, other: 29 },
//!     Custom { value: 2.7, other: 23 },
//!     Custom { value: 14.7, other: 17 },
//!     Custom { value: 4.7, other: 35 },
//! ];
//!
//! let mut array = vec![
//!     Custom { value: 5.7, other: 29 },
//!     Custom { value: 2.7, other: 23 },
//!     Custom { value: 14.7, other: 17 },
//!     Custom { value: 4.7, other: 35 },
//! ];
//!
//! let mut array = vec![
//!     Custom { value: 5.7, other: 29 },
//!     Custom { value: 2.7, other: 23 },
//!     Custom { value: 14.7, other: 17 },
//!     Custom { value: 4.7, other: 35 },
//! ];
//!
//! array.voracious_stable_sort();
//!
//! assert_eq!(array, vec![
//!     Custom { value: 2.7, other: 23 },
//!     Custom { value: 4.7, other: 35 },
//!     Custom { value: 5.7, other: 29 },
//!     Custom { value: 14.7, other: 17 },
//! ]);
//! ```
//!
//! ### Panics
//!
//! For [`f32`](https://doc.rust-lang.org/stable/std/primitive.f32.html) and
//! [`f64`](https://doc.rust-lang.org/stable/std/primitive.f64.html), if there is a
//! [`NaN`](https://doc.rust-lang.org/stable/std/f64/constant.NAN.html) value or an
//! [`INFINITY`](https://doc.rust-lang.org/std/f32/constant.INFINITY.html) or a
//! [`NEG_INFINITY`](https://doc.rust-lang.org/std/f32/constant.INFINITY.html)
//! in the [`vector`](https://doc.rust-lang.org/stable/std/vec/) or the
//! [`slice`](https://doc.rust-lang.org/stable/std/primitive.slice.html), the
//! behavior is not guaranteed.
//!
//! It might panic or not sort correctly the array.
//!
//! ## Dependencies
//!
//! - Rayon 1.5.0 (threadpool). This dependency is **optional**. If you use only the
//! single thread version, you don't need it. If you want to use the multithread
//! version, you will need it.
//!
//! ## Performances
//!
//! - First, please, read: [PROFILING.md](https://github.com/lakwet/voracious_sort/blob/master/PROFILING.md).
//! - These results are from the v1.0.0 version. It might vary a bit with v1.1.0.
//! - Performances can vary depending on the profile you are using.
//! - Please notice that dedicated sorts are faster than generic sorts.
//! - Tests have been done on an AMD Ryzen 9 3950x,  32GB DDR4 RAM, MB X570 TUF
//! Gaming.
//! - For more benchmarks, please visit our [GitHub](https://github.com/lakwet/voracious_sort).
//! - Times are in micro seconde.
//! - Since this crate outperforms all the other Rust sorting crates, only Rust
//! standard sorts and Rayon sorts are used in the benchmarks. Since Rust Unstable sort is
//! actually a PDQ sort, it can be considered as a gold standard.
//!
//! ### For **`u32`** (Distribution: Normal sd=2^20)
//!
//! | Array size | Voracious | Rust Unstable | | Array size | Voracious MT | Rayon Par Uns |
//! |---:|---:|---:|---:|---:|---:|---:|
//! | 500 | **9 us** | **10 us** | | 1_000_000 | **3_299 us** | **3_250 us** |
//! | 1_000 | **13 us** | **22 us** | | 5_000_000 | **9_819 us** | **16_662 us** |
//! | 10_000 | **75 us** | **209 us** | | 10_000_000 | **13_784 us** | **34_578 us** |
//! | 50_000 | **359 us** | **1_316 us** | | 20_000_000 | **21_277 us** | **69_020 us** |
//! | 100_000 | **717 us** | **2_293 us** | | 50_000_000 | **56_346 us** | **177_085 us** |
//! | 500_000 | **3_663 us** | **12_927 us** | | 100_000_000 | **119_500 us** | **366_164 us** |
//! | 1_000_000 | **6_596 us** | **24_879 us** | | 200_000_000 | **231_974 us** | **798_497 us** |
//! | 10_000_000 | **79_342 us** | **263_105 us** | | | | |
//!
//! ### For **`u64`** (Distribution: Normal sd=2^30)
//!
//! | Array size | Voracious | Rust Unstable | | Array size | Voracious MT | Rayon Par Uns |
//! |---:|---:|---:|---:|---:|---:|---:|
//! | 500 | **10 us** | **11 us** | | 1_000_000 | **3_407 us** | **3_375 us** |
//! | 1_000 | **15 us** | **23 us** | | 5_000_000 | **15_090 us** | **19_564 us** |
//! | 10_000 | **91 us** | **208 us** | | 10_000_000 | **22_679 us** | **40_165 us** |
//! | 50_000 | **434 us** | **1_140 us** | | 20_000_000 | **66_907 us** | **84_991 us** |
//! | 100_000 | **1_040 us** | **2_402 us** | | 50_000_000 | **118_142 us** | **241_001 us** |
//! | 500_000 | **4_830 us** | **13_067 us** | | 100_000_000 | **234_282 us** | **525_917 us** |
//! | 1_000_000 | **10_037 us** | **26_009 us** | | 200_000_000 | **511_266 us** | **1_159_379 us** |
//! | 10_000_000 | **111_603 us** | **296_762 us** | | | | |
//!
//! ### For **`f32`** (Distribution: Normal sd=2^20)
//!
//! | Array size | Voracious | Rust Unstable | | Array size | Voracious MT | Rayon Par Uns |
//! |---:|---:|---:|---:|---:|---:|---:|
//! | 500 | **17 us** | **18 us** | | 1_000_000 | **9_877 us** | **10_271 us** |
//! | 1_000 | **24 us** | **39 us** | | 5_000_000 | **16_135 us** | **53_146 us** |
//! | 10_000 | **117 us** | **412 us** | | 10_000_000 | **19_603 us** | **110_428 us** |
//! | 50_000 | **602 us** | **3_075 us** | | 20_000_000 | **40_954 us** | **220_620 us** |
//! | 100_000 | **1_118 us** | **6_617 us** | | 50_000_000 | **119_278 us** | **547_547 us** |
//! | 500_000 | **5_634 us** | **31_434 us** | | 100_000_000 | **192_523 us** | **1_117_997 us** |
//! | 1_000_000 | **10_668 us** | **64_040 us** | | 200_000_000 | **340_500 us** | **2_208_494 us** |
//! | 10_000_000 | **98_425 us** | **772_269 us** | | | | |
//!
//! ### For **`f64`** (Distribution: Normal sd=2^30)
//!
//! | Array size | Voracious | Rust Unstable | | Array size | Voracious MT | Rayon Par Uns |
//! |---:|---:|---:|---:|---:|---:|---:|
//! | 500 | **17 us** | **19 us** | | 1_000_000 | **6_784 us** | **10_588 us** |
//! | 1_000 | **35 us** | **40 us** | | 5_000_000 | **22_044 us** | **60_965 us** |
//! | 10_000 | **146 us** | **499 us** | | 10_000_000 | **34_392 us** | **124_281 us** |
//! | 50_000 | **805 us** | **2_603 us** | | 20_000_000 | **58_670 us** | **240_250 us** |
//! | 100_000 | **1_750 us** | **5_386 us** | | 50_000_000 | **168_876 us** | **618_027 us** |
//! | 500_000 | **7_584 us** | **30_667 us** | | 100_000_000 | **295_038 us** | **1_234_928 us** |
//! | 1_000_000 | **14_453 us** | **70_118 us** | | 200_000_000 | **608_247 us** | **2_523_838 us** |
//! | 10_000_000 | **168_004 us** | **868_874 us** | | | | |
//!
//! ### For **`i32`** (Distribution: Normal sd=2^20)
//!
//! | Array size | Voracious | Rust Unstable | | Array size | Voracious MT | Rayon Par Uns |
//! |---:|---:|---:|---:|---:|---:|---:|
//! | 500 | **11 us** | **11 us** | | 1_000_000 | **3_447 us** | **3_443 us** |
//! | 1_000 | **23 us** | **23 us** | | 5_000_000 | **11_331 us** | **17_511 us** |
//! | 10_000 | **134 us** | **212 us** | | 10_000_000 | **16_271 us** | **34_221 us** |
//! | 50_000 | **627 us** | **1_028 us** | | 20_000_000 | **34_267 us** | **70_053 us** |
//! | 100_000 | **1_182 us** | **2_415 us** | | 50_000_000 | **71_629 us** | **178_771 us** |
//! | 500_000 | **5_456 us** | **11_992 us** | | 100_000_000 | **147_718 us** | **390_487 us** |
//! | 1_000_000 | **11_765 us** | **26_072 us** | | 200_000_000 | **300_494 us** | **796_938 us** |
//! | 10_000_000 | **104_127 us** | **287_329 us** | | | | |
//!
//! ### For **`i64`** (Distribution: Normal sd=2^30)
//!
//! | Array size | Voracious | Rust Unstable | | Array size | Voracious MT | Rayon Par Uns |
//! |---:|---:|---:|---:|---:|---:|---:|
//! | 500 | **12 us** | **11 us** | | 1_000_000 | **3_487 us** | **3_724 us** |
//! | 1_000 | **23 us** | **23 us** | | 5_000_000 | **19_264 us** | **19_840 us** |
//! | 10_000 | **199 us** | **210 us** | | 10_000_000 | **45_942 us** | **42_359 us** |
//! | 50_000 | **1_021 us** | **1_067 us** | | 20_000_000 | **77_951 us** | **86_229 us** |
//! | 100_000 | **1_730 us** | **2_403 us** | | 50_000_000 | **176_850 us** | **241_632 us** |
//! | 500_000 | **8_853 us** | **13_072 us** | | 100_000_000 | **381_320 us** | **547_322 us** |
//! | 1_000_000 | **16_285 us** | **26_547 us** | | 200_000_000 | **819_088 us** | **1_154_061 us** |
//! | 10_000_000 | **170_758 us** | **301_531 us** | | | | |
//!
//! # For Developers
//!
//! ## Implementation details
//!
//! - [`bool`](https://doc.rust-lang.org/stable/std/primitive.bool.html)
//! (Counting sort with radix == 1),
//! - [`char`](https://doc.rust-lang.org/stable/std/primitive.char.html) (Behave
//!   like u32),
//! - [`f32`](https://doc.rust-lang.org/stable/std/primitive.f32.html),
//! [`f64`](https://doc.rust-lang.org/stable/std/primitive.f64.html) (See [link](http://stereopsis.com/radix.html)),
//! - [`i8`](https://doc.rust-lang.org/stable/std/primitive.i8.html),
//! [`i16`](https://doc.rust-lang.org/stable/std/primitive.i16.html),
//! [`i32`](https://doc.rust-lang.org/stable/std/primitive.i32.html),
//! [`i64`](https://doc.rust-lang.org/stable/std/primitive.i64.html),
//! [`i128`](https://doc.rust-lang.org/stable/std/primitive.i128.html)
//! [`isize`](https://doc.rust-lang.org/stable/std/primitive.isize.html) (First bit is flipped),
//! - [`u8`](https://doc.rust-lang.org/stable/std/primitive.u8.html),
//! [`u16`](https://doc.rust-lang.org/stable/std/primitive.u16.html),
//! [`u32`](https://doc.rust-lang.org/stable/std/primitive.u32.html),
//! [`u64`](https://doc.rust-lang.org/stable/std/primitive.u64.html),
//! [`u128`](https://doc.rust-lang.org/stable/std/primitive.u128.html),
//! [`usize`](https://doc.rust-lang.org/stable/std/primitive.usize.html) (Native implementation),
//! - [`struct`](https://doc.rust-lang.org/std/keyword.struct.html) (Mapped to a key among the aforementioned types).
//!
//! ## Using native functions
//!
//! As you can see, not only traits are exposed, but also native sorting functions.
//! That way you can do whatever you want as long as you know what you are doing.
//!
//! Using another value than 8 for the radix is your responsibility.
//!
//! **I can ensure you that sorting with the trait methods is correct (erk I hope ^_^). But if you play
//! with native functions, it is up to you not to do mischief.**
//!
//! Since [profiling](https://github.com/lakwet/voracious_sort/blob/master/PROFILING.md)
//! is not finished. You might need to work a bit more by doing your own profiling.
//! For this purpose, I highly recommend you to clone the github project and use
//! the provided benchmark.

mod algo;
mod dedicated;
#[cfg(feature = "voracious_multithread")]
#[cfg(test)]
mod generators;
mod sorts;
#[cfg(feature = "voracious_multithread")]
#[cfg(test)]
mod tests;
mod traits;
mod types;

pub use traits::dispatcher::Dispatcher;
pub use traits::radix_key::RadixKey;
pub use traits::radixable::Radixable;
pub use traits::radixsort::RadixSort;

pub use sorts::american_flag_sort::american_flag_sort;
pub use sorts::boolean_sort::boolean_sort;
pub use sorts::comparative_sort::insertion_sort;
pub use sorts::counting_sort::counting_sort;
pub use sorts::dlsd_sort::dlsd_radixsort;
pub use sorts::lsd_sort::lsd_radixsort;
pub use sorts::lsd_stable_sort::lsd_stable_radixsort;
pub use sorts::msd_sort::msd_radixsort;
pub use sorts::msd_stable_sort::msd_stable_radixsort;
pub use sorts::rollercoaster_sort::rollercoaster_sort;
pub use sorts::ska_sort::ska_sort;
pub use sorts::thiel_sort::thiel_radixsort;
pub use sorts::voracious_sort::voracious_sort;

#[cfg(feature = "voracious_multithread")]
pub use sorts::peeka_sort::peeka_sort;

pub use dedicated::cs_u16::cs_u16;
pub use dedicated::lsd_f32::lsd_f32;
pub use dedicated::lsd_u32::lsd_u32;
