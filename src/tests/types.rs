use super::super::sorts::utils::Params;
use super::super::Radixable;

#[test]
fn test_types_compute_offset_u64() {
    let mut arr: Vec<u64> = vec![350];
    let dummy = arr[0];

    let (offset, _) = dummy.compute_offset(&mut arr, 8);
    assert_eq!(offset, 48);

    let (offset, _) = dummy.compute_offset(&mut arr, 2);
    assert_eq!(offset, 54);

    let (offset, _) = dummy.compute_offset(&mut arr, 3);
    assert_eq!(offset, 55);
}

#[test]
fn test_types_get_key() {
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
    assert_eq!(arr[0].compute_max_level(0, 8), 3);
    assert_eq!(arr[0].compute_max_level(0, 7), 3);
    assert_eq!(arr[0].compute_max_level(0, 6), 4);
    assert_eq!(arr[0].compute_max_level(6, 8), 2);
    assert_eq!(arr[0].compute_max_level(4, 8), 3);

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
    let p = Params::new(1, 7, 3, 2); // level, radix, offset, max_level
    let arr: Vec<u32> = vec![10];
    let dummy = arr[0];
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);

    assert_eq!(mask, 0b0000_0000_0011_1111_1000_0000_0000_0000u32);
    assert_eq!(shift, 15);
}
