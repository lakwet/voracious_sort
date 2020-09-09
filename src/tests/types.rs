use super::super::sorts::utils::Params;
use super::super::Radixable;

#[test]
fn test_types_compute_offset() {
    let mut arr: Vec<u64> = vec![350];
    let dummy = arr[0];

    // 350: 0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0010_1101

    let (offset, raw_offset) = dummy.compute_offset(&mut arr, 8);
    assert_eq!(offset, 48);
    assert_eq!(raw_offset, 55);

    let (offset, raw_offset) = dummy.compute_offset(&mut arr, 2);
    assert_eq!(offset, 54);
    assert_eq!(raw_offset, 55);

    let (offset, raw_offset) = dummy.compute_offset(&mut arr, 3);
    assert_eq!(offset, 55);
    assert_eq!(raw_offset, 55);
}

#[test]
fn test_types_compute_offset_mt() {
    let mut arr: Vec<u64> = vec![350];
    let dummy = arr[0];

    // 350: 0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0010_1101

    let (offset, raw_offset) = dummy.compute_offset_mt(&mut arr, 8);
    assert_eq!(offset, 48);
    assert_eq!(raw_offset, 55);

    let (offset, raw_offset) = dummy.compute_offset_mt(&mut arr, 2);
    assert_eq!(offset, 54);
    assert_eq!(raw_offset, 55);

    let (offset, raw_offset) = dummy.compute_offset_mt(&mut arr, 3);
    assert_eq!(offset, 55);
    assert_eq!(raw_offset, 55);
}

#[test]
fn test_types_extract() {
    let p = Params::new(6, 8, 0, 8); // level, radix, offset, max_level
    let number: u64 = 0b1010_0010_0100_1000_0000;
    //                       ^^^^ ^^^^
    let (mask, shift) = number.get_mask_and_shift(&p);

    assert_eq!(number.extract(mask, shift), 0b0010_0100);
}

#[test]
fn test_types_mask_for_high_bits() {
    let p = Params::new(0, 8, 24, 3); // level, radix, offset, max_level
    let number: u64 = 0b1111_0001_0010_1011_1011_0101_0010_0100_1110_0101;
    //                  ^^^^ ^^^^ ^^^^ ^^^^ ^^^^ ^^^^

    let high_mask = number.mask_for_high_bits(p.radix, p.offset, p.max_level);

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

    let high_mask = number.mask_for_high_bits(p.radix, p.offset, p.max_level);

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
    assert_eq!(arr[0].compute_max_level(0, 8), 4);
    assert_eq!(arr[0].compute_max_level(0, 7), 5);
    assert_eq!(arr[0].compute_max_level(0, 6), 6);
    assert_eq!(arr[0].compute_max_level(6, 8), 4);
    assert_eq!(arr[0].compute_max_level(9, 8), 3);

    let arr: Vec<u32> = vec![10];
    assert_eq!(arr[0].compute_max_level(0, 8), 4);
    assert_eq!(arr[0].compute_max_level(0, 7), 5);
    assert_eq!(arr[0].compute_max_level(0, 6), 6);
    assert_eq!(arr[0].compute_max_level(6, 8), 4);
    assert_eq!(arr[0].compute_max_level(8, 8), 3);
    assert_eq!(arr[0].compute_max_level(9, 8), 3);
}

#[test]
fn test_types_get_mask_and_shift_from_left() {
    /*
     * max_level is not used in get_mask_and_shift_from_left method.
     */
    let arr: Vec<u32> = vec![10];
    let dummy = arr[0];

    let p = Params::new(0, 7, 3, 2); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:    ^0       ^1       ^2       ^3      ^4
    // mask  :    ^ ^^^^ ^^
    // shift :             ^^ ^^^^ ^^^^ ^^^^ ^^^^ ^^^^ = 22
    assert_eq!(mask, 0b0001_1111_1100_0000_0000_0000_0000_0000u32);
    assert_eq!(shift, 22);

    let p = Params::new(1, 7, 3, 2); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:    ^0       ^1       ^2       ^3      ^4
    // mask  :             ^^ ^^^^ ^
    // shift :                      ^^^ ^^^^ ^^^^ ^^^^ = 15
    assert_eq!(mask, 0b0000_0000_0011_1111_1000_0000_0000_0000u32);
    assert_eq!(shift, 15);

    let p = Params::new(2, 7, 3, 2); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:    ^0       ^1       ^2       ^3      ^4
    // mask  :                      ^^^ ^^^^
    // shift :                               ^^^^ ^^^^ = 8
    assert_eq!(mask, 0b0000_0000_0000_0000_0111_1111_0000_0000u32);
    assert_eq!(shift, 8);

    let p = Params::new(3, 7, 3, 2); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:    ^0       ^1       ^2       ^3      ^4
    // mask  :                               ^^^^ ^^^
    // shift :                                       ^ = 1
    assert_eq!(mask, 0b0000_0000_0000_0000_0000_0000_1111_1110u32);
    assert_eq!(shift, 1);

    let p = Params::new(4, 7, 3, 2); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:    ^0       ^1       ^2       ^3      ^4
    // mask  :                                       ^
    // shift :                                         = 0
    assert_eq!(mask, 0b0000_0000_0000_0000_0000_0000_0000_0001u32);
    assert_eq!(shift, 0);
}

#[test]
fn test_types_get_mask_and_shift() {
    let arr: Vec<u32> = vec![10];
    let dummy = arr[0];

    let p = Params::new(3, 7, 3, 4); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:      ^0      ^1       ^2       ^3
    // mask  :                                ^^^ ^^^^
    // shift :                                         = 0
    assert_eq!(mask, 0b0000_0000_0000_0000_0000_0000_0111_1111u32);
    assert_eq!(shift, 0);

    let p = Params::new(2, 7, 3, 4); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:      ^0      ^1       ^2       ^3
    // mask  :                       ^^ ^^^^ ^
    // shift :                                ^^^ ^^^^ = 7
    assert_eq!(mask, 0b0000_0000_0000_0000_0011_1111_1000_0000u32);
    assert_eq!(shift, 7);

    let p = Params::new(1, 7, 3, 4); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:      ^0      ^1       ^2       ^3
    // mask  :              ^ ^^^^ ^^
    // shift :                       ^^ ^^^^ ^^^^ ^^^^ = 14
    assert_eq!(mask, 0b0000_0000_0001_1111_1100_0000_0000_0000u32);
    assert_eq!(shift, 14);

    let p = Params::new(0, 7, 3, 4); // level, radix, offset, max_level
    let (mask, shift) = dummy.get_mask_and_shift(&p);
    // 10    : 0000_0000_0000_0000_0000_0000_0000_1010
    // levels:      ^0      ^1       ^2       ^3
    // mask  :      ^^^^ ^^^
    // shift :              ^ ^^^^ ^^^^ ^^^^ ^^^^ ^^^^ = 21
    assert_eq!(mask, 0b0000_1111_1110_0000_0000_0000_0000_0000u32);
    assert_eq!(shift, 21);
}
