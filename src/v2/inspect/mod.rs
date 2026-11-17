//! What is read on the keys before choosing how to sort them: already
//! sorted, reversed or constant (`presorted`), nearly sorted and its repair
//! (`near_sorted`).

pub(crate) mod near_sorted;
pub(crate) mod presorted;
