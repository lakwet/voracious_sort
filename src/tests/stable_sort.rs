use std::cmp::Ordering;

use super::super::{
    lsd_stable_radixsort, msd_stable_radixsort, RadixSort, Radixable,
};

#[derive(Clone, Copy, Debug)]
struct Record {
    key: u32,
    position: usize,
}

impl PartialEq for Record {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl PartialOrd for Record {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.key.partial_cmp(&other.key)
    }
}

impl Radixable<u32> for Record {
    type Key = u32;

    fn key(&self) -> u32 {
        self.key
    }
}

fn assert_stable(keys: Vec<u32>, sort: impl FnOnce(&mut Vec<Record>)) {
    let mut records: Vec<_> = keys
        .into_iter()
        .enumerate()
        .map(|(position, key)| Record { key, position })
        .collect();
    let mut expected = records.clone();
    expected.sort_by_key(|record| record.key);
    sort(&mut records);

    // Record equality compares keys only; compare payloads explicitly so that
    // reordering equal keys cannot pass a stability check.
    for (index, (actual, expected)) in records.iter().zip(&expected).enumerate()
    {
        assert_eq!(
            (actual.key, actual.position),
            (expected.key, expected.position),
            "record at output index {}",
            index,
        );
    }
}

#[test]
fn test_sort_lsd_desc() {
    assert_stable((0..512).map(|i| (511 - i) / 4).collect(), |records| {
        lsd_stable_radixsort(records, 8);
    });
}

#[test]
fn test_sort_msd_desc() {
    assert_stable((0..512).map(|i| (511 - i) / 4).collect(), |records| {
        msd_stable_radixsort(records, 8);
    });
}

#[test]
fn test_sort_vec_stable() {
    assert_stable((0..128).map(|i| (i * 37) % 11).collect(), |records| {
        records.voracious_stable_sort();
    });
}

#[test]
fn test_sort_turning_points() {
    // Short ascent before a long descent, and long descent before a short
    // ascent: both put the preprocessor's sample inside an equal-key plateau.
    let peak = (0..512)
        .map(|i| match i {
            0..=35 => (i % 2) * 1000,
            36..=51 => 100 + (i - 36) / 2,
            52..=59 => 108,
            60..=299 => 108 - (i - 60) / 4,
            _ => 200 + (i - 300) / 4,
        })
        .collect::<Vec<u32>>();
    let valley = (0..512)
        .map(|i| match i {
            0..=51 => 100 - i / 4,
            52..=59 => 87,
            60..=71 => 88 + (i - 60) / 4,
            _ => (i - 72) / 4,
        })
        .collect::<Vec<u32>>();
    for keys in &[peak, valley] {
        assert_stable(keys.clone(), |records| lsd_stable_radixsort(records, 8));
        assert_stable(keys.clone(), |records| msd_stable_radixsort(records, 8));
    }
}

#[test]
fn test_sort_patterns() {
    for &len in &[0, 1, 127, 128, 129, 257, 1024, 4097] {
        let patterns: Vec<Vec<u32>> = vec![
            vec![7; len],
            (0..len).map(|i| (i / 4) as u32).collect(),
            (0..len).map(|i| ((len - i) / 4) as u32).collect(),
            (0..len).map(|i| (i.min(len - 1 - i) / 4) as u32).collect(),
            (0..len).map(|i| (i.max(len - 1 - i) / 4) as u32).collect(),
            (0..len).map(|i| ((i % 137) / 4) as u32).collect(),
            (0..len).map(|i| ((i * 37) % 31) as u32 * 0x01010101).collect(),
        ];
        for keys in patterns {
            for &radix in &[4, 8, 11] {
                assert_stable(keys.clone(), |records| {
                    lsd_stable_radixsort(records, radix);
                });
                assert_stable(keys.clone(), |records| {
                    msd_stable_radixsort(records, radix);
                });
            }
            assert_stable(keys.clone(), |records| {
                records.as_mut_slice().voracious_stable_sort();
            });
            assert_stable(keys, |records| records.voracious_stable_sort());
        }
    }
}
