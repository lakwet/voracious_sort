use super::super::algo::chi2::is_uniform;
use super::super::generators::signed_i64::helper_random_array_normale_10_i64;
use super::super::generators::unsigned_u8::helper_random_array_uniform_u8;
use super::super::sorts::utils::{get_histogram, Params};

#[test]
fn test_chi2_is_uniform() {
    // /!\ since it is a statistical test, it might fail, just rerun it.
    for _ in 0..10 {
        let sizes = vec![10_000, 50_000, 1_000_000, 10_000_000];
        for size in sizes.iter() {
            let p = Params::new(0, 8, 0, 1);
            let mut arr = helper_random_array_uniform_u8(*size);
            let (mask, shift) = (0xFFu8, 0);
            let histogram = get_histogram(&mut arr, &p, mask, shift);

            assert!(is_uniform(&histogram, p.radix_range));

            let p = Params::new(0, 8, 0, 8);
            let mut arr = helper_random_array_normale_10_i64(*size);
            let (mask, shift) = (0xFF00_0000_0000_0000u64, 56);
            let histogram = get_histogram(&mut arr, &p, mask, shift);

            assert!(!is_uniform(&histogram, p.radix_range));

            let p = Params::new(1, 8, 0, 8);
            let (mask, shift) = (0x00FF_0000_0000_0000u64, 48);
            let histogram = get_histogram(&mut arr, &p, mask, shift);

            assert!(!is_uniform(&histogram, p.radix_range));

            let p = Params::new(2, 8, 0, 8);
            let (mask, shift) = (0x0000_FF00_0000_0000u64, 40);
            let histogram = get_histogram(&mut arr, &p, mask, shift);

            assert!(!is_uniform(&histogram, p.radix_range));

            let p = Params::new(3, 8, 0, 8);
            let (mask, shift) = (0x0000_00FF_0000_0000u64, 32);
            let histogram = get_histogram(&mut arr, &p, mask, shift);

            assert!(!is_uniform(&histogram, p.radix_range));

            let p = Params::new(4, 8, 0, 8);
            let (mask, shift) = (0x0000_0000_FF00_0000u64, 24);
            let histogram = get_histogram(&mut arr, &p, mask, shift);

            assert!(!is_uniform(&histogram, p.radix_range));

            let p = Params::new(5, 8, 0, 8);
            let (mask, shift) = (0x0000_0000_00FF_0000u64, 16);
            let histogram = get_histogram(&mut arr, &p, mask, shift);

            assert!(!is_uniform(&histogram, p.radix_range));

            let p = Params::new(6, 8, 0, 8);
            let (mask, shift) = (0x0000_0000_0000_FF00u64, 8);
            let histogram = get_histogram(&mut arr, &p, mask, shift);

            assert!(!is_uniform(&histogram, p.radix_range));

            // Last level is most of the time uniform, do not perform
            // test on this last part
        }
    }
}
