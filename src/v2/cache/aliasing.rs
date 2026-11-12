//! Cache aliasing of scatter passes: write streams whose starts fall on the
//! same few cache-line positions of a page map to the same cache sets and
//! evict each other (integer-valued keys, permutations, saw teeth give bucket
//! sizes that are multiples of powers of two).

use super::page_bytes;

/// Cache line size assumed.
const LINE: usize = 64;
/// Most line positions tracked in a page.
const MAX_LINES: usize = 1024;
/// Smaller buckets share their lines instead of competing for sets.
const STREAM_MIN: usize = 1024;
/// Streams on one line position from which they evict each other (more
/// than the ways of an L1 set on many cores).
const CROWD_MIN: u32 = 8;

/// Whether the write streams of a scatter pass (start and length of each
/// bucket, in bytes) crowd on one line position of the page: more than
/// `CROWD_MIN` streams there, and more than 4 times the average.
pub(crate) fn crowded(streams: impl Iterator<Item = (usize, usize)>) -> bool {
    let lines = (page_bytes() / LINE).clamp(1, MAX_LINES);
    let period = lines * LINE;
    let mut per_line = [0u32; MAX_LINES];
    let mut counted = 0u32;
    for (start, len) in streams {
        if len >= STREAM_MIN {
            per_line[(start % period) / LINE] += 1;
            counted += 1;
        }
    }
    if counted == 0 {
        return false;
    }
    let busiest = *per_line[..lines].iter().max().unwrap();
    busiest > CROWD_MIN && busiest > 4 * counted.div_ceil(lines as u32)
}
