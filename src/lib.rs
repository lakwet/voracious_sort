/// VORACIOUS: Variation Of Radixsort Axelle Cleverly Imagined Over Uncountable Sessions
pub mod algo;
#[cfg(test)]
pub mod generators;
mod sorts;
#[cfg(test)]
pub mod tests;
mod types;

pub use types::utils::offset_from_bits;
pub use types::{RadixSort, Radixable, RadixableForContainer};

pub use sorts::american_flag_sort::american_flag_sort;
pub use sorts::boolean_sort::boolean_sort;
pub use sorts::comparative_sort::insertion_sort;
pub use sorts::counting_sort::counting_sort;
pub use sorts::dlsd_sort::dlsd_radixsort;
pub use sorts::lsd_sort::lsd_radixsort;
pub use sorts::msd_sort::msd_radixsort;
pub use sorts::msd_string_sort::msd_string_radixsort;
pub use sorts::ska_sort::ska_sort;
pub use sorts::thiel_sort::thiel_radixsort;
pub use sorts::voracious_sort::voracious_sort;
