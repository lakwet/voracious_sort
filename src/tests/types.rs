use super::super::sorts::utils::Params;
use super::super::{Radixable, RadixableForContainer};

#[test]
fn test_types_compute_offset_u64() {
    let arr: Vec<u64> = vec![350];

    let (offset, _) = arr.compute_offset(8);
    assert_eq!(offset, 48);

    let (offset, _) = arr.compute_offset(2);
    assert_eq!(offset, 54);

    let (offset, _) = arr.compute_offset(3);
    assert_eq!(offset, 55);
}

#[test]
fn test_types_get_key() {
    let p = Params::new(6, 8, 0, 8); // level, radix, offset, max_level
    let number: u64 = 0b1010_0010_0100_1000_0000;
    //                       ^^^^ ^^^^
    let arr: Vec<u64> = vec![number];
    let (mask, shift) = arr.get_mask_and_shift(&p);

    assert_eq!(number.get_key(mask, shift), 0b0010_0100);
}

#[test]
fn test_types_mask_for_high_bits() {
    let p = Params::new(0, 8, 24, 3); // level, radix, offset, max_level
    let number: u64 = 0b1111_0001_0010_1011_1011_0101_0010_0100_1110_0101;
    //                  ^^^^ ^^^^ ^^^^ ^^^^ ^^^^ ^^^^
    let arr: Vec<u64> = vec![number];
    let default_mask = arr.get_default_mask(&p);

    let high_mask =
        number.mask_for_high_bits(default_mask, p.radix, p.offset, p.max_level);

    assert_eq!(
        high_mask,
        0b1111_1111_1111_1111_1111_1111_0000_0000_0000_0000u64
    );

    assert_eq!(
        high_mask & number,
        0b1111_0001_0010_1011_1011_0101_0000_0000_0000_0000u64,
    );

    let p = Params::new(0, 9, 24, 2); // level, radix, offset, max_level
    let number: u64 = 0b11_1111_0001_0010_1011_1011_0101_0010_0100_1110_0101;
    //                     ^^^^ ^^^^ ^^^^ ^^^^ ^^
    let arr: Vec<u64> = vec![number];
    let default_mask = arr.get_default_mask(&p);

    let high_mask =
        number.mask_for_high_bits(default_mask, p.radix, p.offset, p.max_level);

    assert_eq!(
        high_mask,
        0b00_1111_1111_1111_1111_1100_0000_0000_0000_0000_0000u64
    );

    assert_eq!(
        high_mask & number,
        0b00_1111_0001_0010_1011_1000_0000_0000_0000_0000_0000,
    );
}

#[test]
fn test_types_compute_max_level() {
    let arr: Vec<char> = vec!['a'];
    assert_eq!(arr.compute_max_level(0, 8), 3);
    assert_eq!(arr.compute_max_level(0, 7), 3);
    assert_eq!(arr.compute_max_level(0, 6), 4);
    assert_eq!(arr.compute_max_level(6, 8), 2);
    assert_eq!(arr.compute_max_level(4, 8), 3);

    let arr: Vec<u32> = vec![10];
    assert_eq!(arr.compute_max_level(0, 8), 4);
    assert_eq!(arr.compute_max_level(0, 7), 5);
    assert_eq!(arr.compute_max_level(0, 6), 6);
    assert_eq!(arr.compute_max_level(6, 8), 4);
    assert_eq!(arr.compute_max_level(8, 8), 3);
    assert_eq!(arr.compute_max_level(9, 8), 3);
}

#[test]
fn test_types_get_mask_and_shift_for_partial() {
    let p = Params::new(1, 7, 3, 2); // level, radix, offset, max_level
    let arr: Vec<u32> = vec![10];
    let (mask, shift) = arr.get_mask_and_shift_for_partial(&p);

    assert_eq!(mask, 0b0000_0000_0011_1111_1000_0000_0000_0000u32);
    assert_eq!(shift, 15);
}
