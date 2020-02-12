//! # Voracious sort
//!
//! Voracious sort is a [sort algorithm](https://en.wikipedia.org/wiki/Sorting_algorithm), like
//! [Rust standard sort](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort)
//! or
//! [Rust unstable sort](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.sort_unstable).
//! However it is a [radix sort](https://en.wikipedia.org/wiki/Radix_sort), it is a non comparative sort.
//! It is a **very fast sort** and it compares very well against Rust standard and unstable sorts and other
//! state of the art sorting algorithms (see runtimes below).
//!
//! Voracious sort can sort a
//! [`vector`](https://doc.rust-lang.org/stable/std/vec/)
//! or a
//! [`slice`](https://doc.rust-lang.org/stable/std/primitive.slice.html)
//! of:
//! - [`bool`](https://doc.rust-lang.org/stable/std/primitive.bool.html) (Counting sort),
//! - [`char`](https://doc.rust-lang.org/stable/std/primitive.char.html) (Behave like u32),
//! - [`&str`](https://doc.rust-lang.org/std/primitive.str.html) (Dedicated sort),
//! - [`f32`](https://doc.rust-lang.org/stable/std/primitive.f32.html),
//! [`f64`](https://doc.rust-lang.org/stable/std/primitive.f64.html) (See [link](http://stereopsis.com/radix.html)),
//! - [`i8`](https://doc.rust-lang.org/stable/std/primitive.i8.html),
//! [`i16`](https://doc.rust-lang.org/stable/std/primitive.i16.html),
//! [`i32`](https://doc.rust-lang.org/stable/std/primitive.i32.html),
//! [`i64`](https://doc.rust-lang.org/stable/std/primitive.i64.html),
//! [`i128`](https://doc.rust-lang.org/stable/std/primitive.i128.html) (First bit is flipped),
//! - [`u8`](https://doc.rust-lang.org/stable/std/primitive.u8.html),
//! [`u16`](https://doc.rust-lang.org/stable/std/primitive.u16.html),
//! [`u32`](https://doc.rust-lang.org/stable/std/primitive.u32.html),
//! [`u64`](https://doc.rust-lang.org/stable/std/primitive.u64.html),
//! [`u128`](https://doc.rust-lang.org/stable/std/primitive.u128.html) (Native implementation),
//! - Some [`tuple`](https://doc.rust-lang.org/std/primitive.tuple.html), but
//! they do not have been all implemented (Mapped to a key).
//! - Custom [struct](https://doc.rust-lang.org/std/keyword.struct.html)
//! if a the struct implements
//! [`PartialOrd`](https://doc.rust-lang.org/std/cmp/trait.PartialOrd.html),
//! [`PartialEq`](https://doc.rust-lang.org/std/cmp/trait.PartialEq.html)
//! and [`Copy`](https://doc.rust-lang.org/std/marker/trait.Copy.html) (and thus, Clone trait too)
//! traits and `Radixable` trait (see below) (Mapped to a key).
//!
//! Vocarious sort can only sort in ascending order. You can call the
//! [`reverse`](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.reverse)
//! method if desired. Sorts from this crate are
//! [unstable](https://en.wikipedia.org/wiki/Sorting_algorithm#Stability) (Unstable sort,
//! not unstable Rust feature !).
//!
//! ## Version
//!
//! Last version tested/used:
//! - Rustc: 1.38.0 stable
//! - Rustfmt: 1.4.4 stable
//! - Cargo: 1.38.0 stable
//! - Clippy: 0.0.212
//!
//! ## License
//!
//! See the license file.
//!
//! ## How to use it
//!
//! Add in `Cargo.toml`:
//! ```toml
//! [dependencies]
//! voracious_radix_sort = "0.1.0"
//! ```
//!
//! Import the crate in your project:
//! ```no_run
//! extern crate voracious_radix_sort;
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
//! When the Crate is imported, two methods are added to vectors and slices:
//! - `voracious_sort()` (single thread).
//! - `dlsd_sort()` (single thread).
//!
//! ### Example
//!
//! ```
//! use voracious_radix_sort::*;
//!
//! let mut array = vec![2, 45, 8, 7, 9, 65, 8, 74, 1, 2, 56, 9, 7, 41];
//!
//! // General multipurpose sort, to be used by default.
//! array.voracious_sort();
//!
//! assert_eq!(array, vec![1, 2, 2, 7, 7, 8, 8, 9, 9, 41, 45, 56, 65, 74]);
//! ```
//!
//! ### Example (experimental)
//!
//! ```
//! use voracious_radix_sort::*;
//!
//! let mut array = vec![2, 45, 8, 7, 9, 65, 8, 74, 1, 2, 56, 9, 7, 41];
//!
//! // Experimental sort, faster than Voracious sort on
//! // uniformly distributed data. It uses statistical
//! // distribution hypothesis on the input.
//! array.dlsd_sort();
//!
//! assert_eq!(array, vec![1, 2, 2, 7, 7, 8, 8, 9, 9, 41, 45, 56, 65, 74]);
//! ```
//!
//! ### Implementing a custom `struct`
//!
//! Let's do it through an example.
//!
//! ```no_run
//! use std::cmp::Ordering;
//!
//! // We need a struct
//! // We want, for example, to sort these structs by the key = max - min
//! // This struct must implement the Copy and Clone traits, we can just derive them.
//! #[derive(Copy, Clone, Debug)]
//! pub struct Custom {
//!     min: u32,
//!     max: u32,
//! }
//!
//! // And the PartialOrd and the PartialEq traits
//! impl PartialOrd for Custom {
//!     fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
//!         (self.max - self.min).partial_cmp(&(other.max - other.min))
//!     }
//! }
//! impl PartialEq for Custom {
//!     fn eq(&self, other: &Self) -> bool {
//!         self.max - self.min == other.max - other.min
//!     }
//! }
//! ```
//!
//! And then we have to implement the `Radixable` trait:
//! ```no_run
//! # use std::cmp::Ordering;
//! use voracious_radix_sort::*;
//! # #[derive(Copy, Clone, Debug)]
//! # pub struct Custom {
//! #     min: u32,
//! #     max: u32,
//! # }
//! # impl PartialOrd for Custom {
//! #     fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
//! #         (self.max - self.min).partial_cmp(&(other.max - other.min))
//! #     }
//! # }
//! # impl PartialEq for Custom {
//! #     fn eq(&self, other: &Self) -> bool {
//! #         self.max - self.min == other.max - other.min
//! #     }
//! # }
//!
//! impl Radixable for Custom {
//!     // We have to map each struct to a key. The key must be amongst a type
//!     // for the native implementation.
//!     type KeyType = u32;
//!
//!     // This function is not mandatory
//!     // Unless you want to optimize something, there is no need to
//!     // implement it.
//!     #[inline]
//!     fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
//!         ((self.into_key_type() & mask) >> shift) as usize
//!     }
//!     // This function is mandatory.
//!     // You have to provide the transformation from the struct to the key.
//!     #[inline]
//!     fn into_key_type(&self) -> u32 {
//!         self.max - self.min
//!     }
//!     // This function is mandatory.
//!     // You have to fill the number of bit of the key.
//!     // For example, std::char::MAX == 0x10ffff, which is 21 bits.
//!     // Despite the keytype is u32 for char, we do fill 21 in this
//!     // function.
//!     // But in this exemple, the key lengh is 32 bits.
//!     #[inline]
//!     fn type_size(&self) -> usize {
//!         32
//!     }
//!     // This function is mandatory.
//!     // It is just a cast. It is usefull for other functions in this crate.
//!     #[inline]
//!     fn usize_to_keytype(&self, item: usize) -> u32 {
//!         item as u32
//!     }
//!     // This function is mandatory.
//!     // It is just a cast. It is usefull for other functions in this crate.
//!     #[inline]
//!     fn keytype_to_usize(&self, item: u32) -> usize {
//!         item as usize
//!     }
//!     // This function is mandatory.
//!     // We needed a zero somewhere with the correct type.
//!     #[inline]
//!     fn default_key(&self) -> Self::KeyType {
//!         0
//!     }
//!     // This function is mandatory.
//!     // We needed a one somewhere with the correct type.
//!     #[inline]
//!     fn one(&self) -> Self::KeyType {
//!         1
//!     }
//!     // This function is mandatory.
//!     // This is where we can choose what to do depending on the type, the
//!     // input size, the sort function etc...
//!     fn voracious_sort(&self, arr: &mut [Custom]) {
//!         lsd_radixsort(arr, 8);
//!     }
//!     // This function is mandatory.
//!     // This is where we can choose what to do depending on the type, the
//!     // input size, the sort function etc...
//!     fn dlsd_sort(&self, arr: &mut [Custom]) {
//!         dlsd_radixsort(arr, 8);
//!     }
//! }
//! ```
//!
//! **See more implementation examples in our [GitHub](https://github.com/fretlink/voracious_sort)**
//!
//! It is done, we can do a test:
//! ```
//! use voracious_radix_sort::*;
//! # use std::cmp::Ordering;
//! # #[derive(Copy, Clone, Debug)]
//! # pub struct Custom {
//! #     min: u32,
//! #     max: u32,
//! # }
//! # impl PartialOrd for Custom {
//! #     fn partial_cmp(&self, other: &Custom) -> Option<Ordering> {
//! #         (self.max - self.min).partial_cmp(&(other.max - other.min))
//! #     }
//! # }
//! # impl PartialEq for Custom {
//! #     fn eq(&self, other: &Self) -> bool {
//! #         self.max - self.min == other.max - other.min
//! #     }
//! # }
//! # impl Radixable for Custom {
//! #     type KeyType = u32;
//! #     #[inline]
//! #     fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
//! #         ((self.into_key_type() & mask) >> shift) as usize
//! #     }
//! #     #[inline]
//! #     fn into_key_type(&self) -> u32 {
//! #         self.max - self.min
//! #     }
//! #     #[inline]
//! #     fn type_size(&self) -> usize {
//! #         32
//! #     }
//! #     #[inline]
//! #     fn usize_to_keytype(&self, item: usize) -> u32 {
//! #         item as u32
//! #     }
//! #     #[inline]
//! #     fn keytype_to_usize(&self, item: u32) -> usize {
//! #         item as usize
//! #     }
//! #     #[inline]
//! #     fn default_key(&self) -> Self::KeyType {
//! #         0
//! #     }
//! #     #[inline]
//! #     fn one(&self) -> Self::KeyType {
//! #         1
//! #     }
//! #     fn voracious_sort(&self, arr: &mut [Custom]) {
//! #         lsd_radixsort(arr, 8);
//! #     }
//! #     fn dlsd_sort(&self, arr: &mut [Custom]) {
//! #         dlsd_radixsort(arr, 8);
//! #     }
//! # }
//!
//! let mut array = vec![
//!     Custom { min: 5, max: 29 },
//!     Custom { min: 2, max: 23 },
//!     Custom { min: 14, max: 17 },
//!     Custom { min: 4, max: 35 },
//! ];
//!
//! array.voracious_sort();
//!
//! // Caution: in this case, the function to transform the struct to the
//! // key is not injective, so depending on the input, it might have several
//! // correct results:
//! // Custom { min: 2, max: 4 } == Custom { min: 17, max: 19 }
//! assert_eq!(array, vec![
//!     Custom { min: 14, max: 17 },
//!     Custom { min: 2, max: 23 },
//!     Custom { min: 5, max: 29 },
//!     Custom { min: 4, max: 35 },
//! ]);
//! ```
//!
//! ### Panics
//!
//! For
//! [`f32`](https://doc.rust-lang.org/stable/std/primitive.f32.html)
//! and
//! [`f64`](https://doc.rust-lang.org/stable/std/primitive.f64.html)
//! , it `panics` if there is a
//! [`NaN`](https://doc.rust-lang.org/stable/std/f64/constant.NAN.html)
//! value in the
//! [`vector`](https://doc.rust-lang.org/stable/std/vec/)
//! or
//! the
//! [`slice`](https://doc.rust-lang.org/stable/std/primitive.slice.html).
//!
//! ## Dependencies
//!
//! There is no dependency.
//!
//! ## Performances
//!
//! - All tests have been done on a i5 7500 3.4GHz 6MB cache L3 with 40GB DDR4 RAM (November 2019)
//! with `RUSTFLAGS="-C target-cpu=native"`.
//! - Only one run has been done by test.
//! - For more runtimes, please visit our [GitHub](https://github.com/fretlink/voracious_sort).
//! - Times are in micro seconde.
//!
//! - *[RdxSort](https://crates.io/crates/rdxsort) version 0.3.0
//! - *[AfSort](https://crates.io/crates/afsort) version 0.3.1
//!
//! ### For **`u64`** 100_000_000 integers
//!
//! | Distribution | Voracious | DLSD | Rust Std | Rust Unstable | RdxSort* | AFSort* |
//! |-------------:|----------:|-----:|---------:|--------------:|---------:|--------:|
//! | Uniform | `1_969_984` | `1_393_206` | `10_124_103` | `3_372_091 `| `2_842_670` | `4_060_710` |
//! | Zipf | `352_684` | `1_315_713` | `5_584_313` | `331_330`| `5_587_266` | `2_155_202` |
//! | Normal (SD 10^6) | `1_863_536` | `3_008_335` | `9_675_911` | `2_454_208`| `3_874_521` | `4_482_161` |
//!
//! ### For **`f64`** 100_000_000 floats
//!
//! | Distribution | Voracious | DLSD | Rust Std | Rust Unstable | RdxSort* | AFSort* |
//! |-------------:|----------:|-----:|---------:|--------------:|---------:|--------:|
//! | Uniform | `2_358_032` | `2_873_768` | `14_247_551` | `7_108_842`| `4_548_991` | `N/A` |
//! | Zipf | `2_198_049` | `1_221_660` | `6_435_186` | `805_088`| `6_242_734` | `N/A` |
//! | Normal (SD 10^6) | `2_357_697` | `2_334_541` | `14_049_225` | `7_109_580`| `4_309_830` | `N/A` |
//!
//! ### For **`i64`** 100_000_000 integers
//!
//! | Distribution | Voracious | DLSD | Rust Std | Rust Unstable | RdxSort* | AFSort* |
//! |-------------:|----------:|-----:|---------:|--------------:|---------:|--------:|
//! | Uniform | `2_037_479` | `1_347_168` | `9_932_912` | `3_516_609`| `3_302_737` | `N/A` |
//! | Zipf | `401_947` | `1_287_534` | `5_499_072` | `320_038`| `5_807_618` | `N/A` |
//! | Normal (SD 10^6) | `1_856_729` | `3_039_194` | `9_821_670` | `2_602_098`| `4_225_584` | `N/A` |
//!
//! ### For **`char`** 100_000_000 chars
//!
//! | Distribution | Voracious | DLSD | Rust Std | Rust Unstable | RdxSort* | AFSort* |
//! |-------------:|----------:|-----:|---------:|--------------:|---------:|--------:|
//! | Uniform | `777_537` | `802_939` | `6_116_985` | `1_813_057`| `1_933_041` | `N/A` |
//! | All Equal | `47_914` | `47_929` | `47_488` | `41_212`| `2_229_338` | `N/A` |
//! | Small CharSet | `114_896` | `394_616` | `6_197_632` | `689_738`| `2_006_239` | `N/A` |
//! | Medium CharSet | `113_521` | `622_184` | `6_144_734` | `629_989`| `1_982_256` | `N/A` |
//! | Big CharSet | `867_986` | `857_884` | `6_169_368` | `727_749`| `2_029_007` | `N/A` |
//!
//! ### For **`&str`** 10_000_000 &str
//!
//! | Distribution | Voracious | DLSD | Rust Std | Rust Unstable | RdxSort* | AFSort* |
//! |-------------:|----------:|-----:|---------:|--------------:|---------:|--------:|
//! | Uniform (length: 20) | `2_526_030` | `N/A` | `3_360_045` | `5_585_305`| `N/A` | `3_096_866` |
//!
//! ### For **`(u32, u32)`** 100_000_000 tuples
//!
//! | Distribution | Voracious | DLSD | Rust Std | Rust Unstable | RdxSort* | AFSort* |
//! |-------------:|----------:|-----:|---------:|--------------:|---------:|--------:|
//! | Uniform | `2_306_188` | `1_605_143` | `12_913_487` | `4_780_460`| `3_147_126` | `N/A` |
//!
//! ### For **`(bool, bool)`** 100_000_000 tuples
//!
//! - For `(bool, bool)`, Voracious sort uses a counting sort.
//!
//! | Distribution | Voracious | DLSD | Rust Std | Rust Unstable | RdxSort* | AFSort* |
//! |-------------:|----------:|-----:|---------:|--------------:|---------:|--------:|
//! | Uniform | `118_116` | `394_691` | `5_223_943` | `846_968`| `456_879` | `N/A` |
//!
//! # For Developers and Researchers
//!
//! ## Logic
//!
//! - Voracious sort is a MSD radix sort. It is an improvement of the
//! [Ska sort](https://probablydance.com/2016/12/27/i-wrote-a-faster-sorting-algorithm/)
//! and it uses the [Verge sort pre-processing heuristic](https://github.com/Morwenn/vergesort).
//! Depending on the type and the input size, another sort might be choosen
//! (LSD sort, Counting sort, etc...).
//! The purpose is to implement a multithread radix sort (see
//! [Regions sort](https://github.com/omarobeya/parallel-inplace-radixsort) and
//! the [article](https://people.csail.mit.edu/jshun/RegionsSort.pdf)).
//!
//! - DLSD (Diverting LSD radix sort) is a simpler version of the
//! [DFR sort](https://github.com/ramou/dfr) with a different diversion and
//! a variable radix (see [article](https://users.encs.concordia.ca/~sthiel/DS/SEA2015_FastRadix.pdf)).
//!
//! - All sorts fallback on the
//! [PDQ sort](https://github.com/stjepang/pdqsort)
//! (Rust Unstable sort) for very small inputs.
//!
//! ## Futur work
//!
//! - Add multithread sort.
//! - Improve k-way-merge algorithm.
//! - Use more statistical hypothesis.
//! - Finish 2-arity tuple implementation.
//! - Add more generators (for tests).
//! - Replace the MSD sort for string by a Burstsort or Spreadsort implementation
//! or something else.

extern crate rayon;

pub mod algo;
#[cfg(test)]
pub mod generators;
mod sorts;
#[cfg(test)]
pub mod tests;
mod types;

pub use types::{RadixSort, Radixable};

pub use sorts::american_flag_sort::american_flag_sort;
pub use sorts::boolean_sort::boolean_sort;
pub use sorts::comparative_sort::insertion_sort;
pub use sorts::counting_sort::counting_sort;
pub use sorts::dlsd_sort::dlsd_radixsort;
pub use sorts::lsd_mt_sort::lsd_mt_radixsort;
pub use sorts::lsd_sort::lsd_radixsort;
pub use sorts::msd_sort::msd_radixsort;
pub use sorts::msd_string_sort::msd_string_radixsort;
pub use sorts::regions_sort::regions_sort;
pub use sorts::ska_sort::ska_sort;
pub use sorts::thiel_sort::thiel_radixsort;
pub use sorts::voracious_sort::voracious_sort;
