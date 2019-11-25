use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::{lsd_radixsort, lsd_radixsort_heu};
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::voracious_sort::voracious_sort_heu;
use super::utils::offset_from_bits;
use super::{RadixSort, Radixable, RadixableForContainer};

impl Radixable for (bool, bool) {
    type KeyType = u8;

    #[inline]
    fn get_key(&self, _mask: u8, _shift: usize) -> usize {
        match *self {
            (true, true) => 3,
            (true, false) => 2,
            (false, true) => 1,
            (false, false) => 0,
        }
    }
    #[inline]
    fn to_generic(&self, v: usize) -> (bool, bool) {
        match v {
            0 => (false, false),
            1 => (false, true),
            2 => (true, false),
            3 => (true, true),
            _ => panic!("[Types: (bool, bool)] Bad implementation."),
        }
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u8,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u8 {
        let bits = 8;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [(bool, bool)] {
    type T = (bool, bool);
    type KeyType = u8;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        2
    }
    #[inline]
    fn into_key_type(&self, (fst, snd): (bool, bool)) -> u8 {
        match (fst, snd) {
            (true, true) => 3,
            (true, false) => 2,
            (false, true) => 1,
            (false, false) => 0,
        }
    }
    #[inline]
    fn from_key_type(&self, v: u8) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u8 {
        v as u8
    }
}

impl RadixSort for [(bool, bool)] {
    fn voracious_sort(&mut self) {
        counting_sort(self, 2);
    }
    fn dlsd_sort(&mut self) {
        dlsd_radixsort(self, 8);
    }
}

impl Radixable for (bool, u8) {
    type KeyType = u16;

    #[inline]
    fn get_key(&self, _mask: u16, _shift: usize) -> usize {
        match *self {
            (true, i) => 0x100 | i as usize,
            (false, i) => i as usize,
        }
    }
    #[inline]
    fn to_generic(&self, v: usize) -> (bool, u8) {
        let number = v & 0xFF;
        if v & 0x100 == 0x100 {
            (true, number as u8)
        } else {
            (false, number as u8)
        }
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u16,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u16 {
        let bits = 16;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [(bool, u8)] {
    type T = (bool, u8);
    type KeyType = u16;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        9
    }
    #[inline]
    fn into_key_type(&self, (fst, snd): (bool, u8)) -> u16 {
        match (fst, snd) {
            (true, i) => 0x100 | u16::from(i),
            (false, i) => u16::from(i),
        }
    }
    #[inline]
    fn from_key_type(&self, v: u16) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u16 {
        v as u16
    }
}

impl RadixSort for [(bool, u8)] {
    fn voracious_sort(&mut self) {
        if self.len() <= 500 {
            msd_radixsort(self, 9);
        } else {
            counting_sort(self, 9);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 500 {
            msd_radixsort(self, 9);
        } else {
            dlsd_radixsort(self, 9);
        }
    }
}

impl Radixable for (bool, u16) {
    type KeyType = u32;

    #[inline]
    fn get_key(&self, mask: u32, shift: usize) -> usize {
        let key = match *self {
            (true, i) => 0x0001_0000 | u32::from(i),
            (false, i) => u32::from(i),
        };

        ((key & mask) >> shift) as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> (bool, u16) {
        let number = v & 0x0000_FFFF;
        if v & 0x0001_0000 == 0x0001_0000 {
            (true, number as u16)
        } else {
            (false, number as u16)
        }
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u32,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u32 {
        let bits = 32;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [(bool, u16)] {
    type T = (bool, u16);
    type KeyType = u32;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        17
    }
    #[inline]
    fn into_key_type(&self, (fst, snd): (bool, u16)) -> u32 {
        match (fst, snd) {
            (true, i) => 0x0001_0000 | u32::from(i),
            (false, i) => u32::from(i),
        }
    }
    #[inline]
    fn from_key_type(&self, v: u32) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u32 {
        v as u32
    }
}

impl RadixSort for [(bool, u16)] {
    fn voracious_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if self.len() <= 300_000 {
            lsd_radixsort(self, 9);
        } else {
            counting_sort(self, 17);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(self, 9);
        }
    }
}

impl Radixable for (u8, bool) {
    type KeyType = u16;

    #[inline]
    fn get_key(&self, _mask: u16, _shift: usize) -> usize {
        match *self {
            (i, true) => ((i as usize) << 1) | 0x01,
            (i, false) => (i as usize) << 1,
        }
    }
    #[inline]
    fn to_generic(&self, v: usize) -> (u8, bool) {
        let number = (v & 0x01FE) >> 1;
        if v & 0x01 == 0x01 {
            (number as u8, true)
        } else {
            (number as u8, false)
        }
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u16,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u16 {
        let bits = 16;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [(u8, bool)] {
    type T = (u8, bool);
    type KeyType = u16;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        9
    }
    #[inline]
    fn into_key_type(&self, (fst, snd): (u8, bool)) -> u16 {
        match (fst, snd) {
            (i, true) => (u16::from(i) << 1) | 0x01,
            (i, false) => u16::from(i) << 1,
        }
    }
    #[inline]
    fn from_key_type(&self, v: u16) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u16 {
        v as u16
    }
}

impl RadixSort for [(u8, bool)] {
    fn voracious_sort(&mut self) {
        if self.len() <= 500 {
            msd_radixsort(self, 9);
        } else {
            counting_sort(self, 9);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 500 {
            msd_radixsort(self, 9);
        } else {
            dlsd_radixsort(self, 9);
        }
    }
}

impl Radixable for (u8, u8) {
    type KeyType = u16;

    #[inline]
    fn get_key(&self, mask: u16, shift: usize) -> usize {
        ((((u16::from(self.0) << 8) | u16::from(self.1)) & mask) >> shift)
            as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> (u8, u8) {
        let fst = (v as u16 & 0xFF00) >> 8;
        let snd = v as u8;
        (fst as u8, snd)
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u16,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u16 {
        let bits = 16;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [(u8, u8)] {
    type T = (u8, u8);
    type KeyType = u16;

    #[inline]
    fn compute_offset(&self, _radix: usize) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        16
    }
    #[inline]
    fn into_key_type(&self, (fst, snd): (u8, u8)) -> u16 {
        (u16::from(fst) << 8) | u16::from(snd)
    }
    #[inline]
    fn from_key_type(&self, v: u16) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u16 {
        v as u16
    }
}

impl RadixSort for [(u8, u8)] {
    fn voracious_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if self.len() <= 300_000 {
            lsd_radixsort(self, 8);
        } else {
            counting_sort(self, 16);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(self, 8);
        }
    }
}

impl Radixable for (u32, u32) {
    type KeyType = u64;

    #[inline]
    fn get_key(&self, mask: u64, shift: usize) -> usize {
        ((((u64::from(self.0) << 32) | u64::from(self.1)) & mask) >> shift)
            as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> (u32, u32) {
        let fst = (v as u64 & 0xFFFF_FFFF_0000_0000) >> 32;
        let snd = v as u32;
        (fst as u32, snd)
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u64,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u64 {
        let bits = 64;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [(u32, u32)] {
    type T = (u32, u32);
    type KeyType = u64;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self
            .iter()
            .map(|elt| (u64::from(elt.0) << 32) | u64::from(elt.1))
            .max()
            .unwrap();
        offset_from_bits(buf, radix, 64, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        64
    }
    #[inline]
    fn into_key_type(&self, (fst, snd): (u32, u32)) -> u64 {
        (u64::from(fst) << 32) | u64::from(snd)
    }
    #[inline]
    fn from_key_type(&self, v: u64) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u64 {
        v as u64
    }
}

impl RadixSort for [(u32, u32)] {
    fn voracious_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if self.len() <= 8000 {
            msd_radixsort(self, 8);
        } else if self.len() <= 100_000 {
            lsd_radixsort_heu(self, 8, 200_000);
        } else {
            voracious_sort_heu(self, 8, 200_000);
        }
    }
    fn dlsd_sort(&mut self) {
        if self.len() <= 200 {
            self.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(self, 8);
        }
    }
}

impl Radixable for (u64, u64) {
    type KeyType = u128;

    #[inline]
    fn get_key(&self, mask: u128, shift: usize) -> usize {
        ((((u128::from(self.0) << 64) | u128::from(self.1)) & mask) >> shift)
            as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> (u64, u64) {
        let fst = (v as u128 & 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000) >> 64;
        let snd = v as u64;
        (fst as u64, snd)
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u128,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u128 {
        let bits = 128;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [(u64, u64)] {
    type T = (u64, u64);
    type KeyType = u128;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let buf = self
            .iter()
            .map(|elt| (u128::from(elt.0) << 64) | u128::from(elt.1))
            .max()
            .unwrap();
        offset_from_bits(buf, radix, 128, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        128
    }
    #[inline]
    fn into_key_type(&self, (fst, snd): (u64, u64)) -> u128 {
        (u128::from(fst) << 64) | u128::from(snd)
    }
    #[inline]
    fn from_key_type(&self, v: u128) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u128 {
        v as u128
    }
}

impl RadixSort for [(u64, u64)] {
    fn voracious_sort(&mut self) {
        voracious_sort_heu(self, 8, 200_000);
    }
    fn dlsd_sort(&mut self) {
        dlsd_radixsort(self, 8);
    }
}

impl Radixable for (i64, i64) {
    type KeyType = u128;

    #[inline]
    fn get_key(&self, mask: u128, shift: usize) -> usize {
        let flip = 0x8000_0000_0000_0000;
        ((((u128::from(self.0 as u64 ^ flip) << 64)
            | u128::from(self.1 as u64 ^ flip))
            & mask)
            >> shift) as usize
    }
    #[inline]
    fn to_generic(&self, v: usize) -> (i64, i64) {
        let flip = 0x8000_0000_0000_0000;
        let fst = (v as u128 & 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000) >> 64;
        let snd = v as u64;
        ((fst as u64 ^ flip) as i64, (snd as u64 ^ flip) as i64)
    }
    #[inline]
    fn mask_for_high_bits(
        &self,
        default_mask: u128,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> u128 {
        let bits = 128;
        let mut mask = 0;
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << (radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << (bits - offset - radix * (max_level - level));
            }
        }
        mask
    }
}

impl RadixableForContainer for [(i64, i64)] {
    type T = (i64, i64);
    type KeyType = u128;

    #[inline]
    fn compute_offset(&self, radix: usize) -> (usize, usize) {
        let flip = 0x8000_0000_0000_0000;
        let buf = self
            .iter()
            .map(|elt| {
                (u128::from(elt.0 as u64 ^ flip) << 64)
                    | u128::from(elt.1 as u64 ^ flip)
            })
            .max()
            .unwrap();
        offset_from_bits(buf, radix, 128, 0, 1)
    }
    #[inline]
    fn element_bit_size(&self) -> usize {
        128
    }
    #[inline]
    fn into_key_type(&self, (fst, snd): (i64, i64)) -> u128 {
        let flip = 0x8000_0000_0000_0000;
        (u128::from(fst as u64 ^ flip) << 64) | u128::from(snd as u64 ^ flip)
    }
    #[inline]
    fn from_key_type(&self, v: u128) -> usize {
        v as usize
    }
    #[inline]
    fn usize_into_key_type(&self, v: usize) -> u128 {
        v as u128
    }
}

impl RadixSort for [(i64, i64)] {
    fn voracious_sort(&mut self) {
        voracious_sort_heu(self, 8, 200_000);
    }
    fn dlsd_sort(&mut self) {
        dlsd_radixsort(self, 8);
    }
}
