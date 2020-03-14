use super::super::sorts::counting_sort::counting_sort;
use super::super::sorts::dlsd_sort::dlsd_radixsort;
use super::super::sorts::lsd_sort::{lsd_radixsort, lsd_radixsort_heu};
use super::super::sorts::msd_sort::msd_radixsort;
use super::super::sorts::voracious_sort::{voracious_sort, voracious_sort_heu};
use super::super::{RadixKey, Radixable};

impl Radixable for (bool, bool) {
    type KeyType = u8;

    #[inline] // overrided function
    fn extract(&self, _mask: u8, _shift: usize) -> usize {
        match *self {
            (true, true) => 3,
            (true, false) => 2,
            (false, true) => 1,
            (false, false) => 0,
        }
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> (bool, bool) {
        match value {
            0 => (false, false),
            1 => (false, true),
            2 => (true, false),
            3 => (true, true),
            _ => panic!("[Types: (bool, bool)] Bad implementation."),
        }
    }
    #[inline]
    fn into_key_type(&self) -> u8 {
        match *self {
            (true, true) => 3,
            (true, false) => 2,
            (false, true) => 1,
            (false, false) => 0,
        }
    }
    #[inline]
    fn type_size(&self) -> usize {
        2
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u8 {
        item as u8
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u8) -> usize {
        item as usize
    }
    #[inline] // overrided function
    fn compute_offset(
        &self,
        _arr: &mut [(bool, bool)],
        _radix: usize,
    ) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(bool, bool)]) {
        counting_sort(arr, 2);
    }
    fn dlsd_sort(&self, arr: &mut [(bool, bool)]) {
        dlsd_radixsort(arr, 8);
    }
}

impl Radixable for (bool, u8) {
    type KeyType = u16;

    #[inline] // overrided function
    fn extract(&self, _mask: u16, _shift: usize) -> usize {
        match *self {
            (true, i) => 0x100 | i as usize,
            (false, i) => i as usize,
        }
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> (bool, u8) {
        let number = value & 0xFF;
        if value & 0x100 == 0x100 {
            (true, number as u8)
        } else {
            (false, number as u8)
        }
    }
    #[inline]
    fn into_key_type(&self) -> u16 {
        match *self {
            (true, i) => 0x100 | u16::from(i),
            (false, i) => u16::from(i),
        }
    }
    #[inline]
    fn type_size(&self) -> usize {
        9
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u16 {
        item as u16
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u16) -> usize {
        item as usize
    }
    #[inline] // overrided function
    fn compute_offset(
        &self,
        _arr: &mut [(bool, u8)],
        _radix: usize,
    ) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(bool, u8)]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 9);
        } else {
            counting_sort(arr, 9);
        }
    }
    fn dlsd_sort(&self, arr: &mut [(bool, u8)]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 9);
        } else {
            dlsd_radixsort(arr, 9);
        }
    }
}

impl Radixable for (bool, u16) {
    type KeyType = u32;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> (bool, u16) {
        let number = value & 0x0000_FFFF;
        if value & 0x0001_0000 == 0x0001_0000 {
            (true, number as u16)
        } else {
            (false, number as u16)
        }
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        match *self {
            (true, i) => 0x0001_0000 | u32::from(i),
            (false, i) => u32::from(i),
        }
    }
    #[inline]
    fn type_size(&self) -> usize {
        17
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u32 {
        item as u32
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u32) -> usize {
        item as usize
    }
    #[inline] // overrided function
    fn compute_offset(
        &self,
        _arr: &mut [(bool, u16)],
        _radix: usize,
    ) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(bool, u16)]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 300_000 {
            lsd_radixsort(arr, 9);
        } else {
            counting_sort(arr, 17);
        }
    }
    fn dlsd_sort(&self, arr: &mut [(bool, u16)]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 9);
        }
    }
}

impl Radixable for (u8, bool) {
    type KeyType = u16;

    #[inline] // overrided function
    fn extract(&self, _mask: u16, _shift: usize) -> usize {
        match *self {
            (i, true) => ((i as usize) << 1) | 0x01,
            (i, false) => (i as usize) << 1,
        }
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> (u8, bool) {
        let number = (value & 0x01FE) >> 1;
        if value & 0x01 == 0x01 {
            (number as u8, true)
        } else {
            (number as u8, false)
        }
    }
    #[inline]
    fn into_key_type(&self) -> u16 {
        match *self {
            (i, true) => (u16::from(i) << 1) | 0x01,
            (i, false) => u16::from(i) << 1,
        }
    }
    #[inline]
    fn type_size(&self) -> usize {
        9
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u16 {
        item as u16
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u16) -> usize {
        item as usize
    }
    #[inline] // overrided function
    fn compute_offset(
        &self,
        _arr: &mut [(u8, bool)],
        _radix: usize,
    ) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(u8, bool)]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 9);
        } else {
            counting_sort(arr, 9);
        }
    }
    fn dlsd_sort(&self, arr: &mut [(u8, bool)]) {
        if arr.len() <= 500 {
            msd_radixsort(arr, 9);
        } else {
            dlsd_radixsort(arr, 9);
        }
    }
}

impl Radixable for (u8, u8) {
    type KeyType = u16;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> (u8, u8) {
        let fst = (value as u16 & 0xFF00) >> 8;
        let snd = value as u8;
        (fst as u8, snd)
    }
    #[inline]
    fn into_key_type(&self) -> u16 {
        (u16::from(self.0) << 8) | u16::from(self.1)
    }
    #[inline]
    fn type_size(&self) -> usize {
        16
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u16 {
        item as u16
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u16) -> usize {
        item as usize
    }
    #[inline] // overrided function
    fn compute_offset(
        &self,
        _arr: &mut [(u8, u8)],
        _radix: usize,
    ) -> (usize, usize) {
        (0, 0)
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(u8, u8)]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 300_000 {
            lsd_radixsort(arr, 8);
        } else {
            counting_sort(arr, 16);
        }
    }
    fn dlsd_sort(&self, arr: &mut [(u8, u8)]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}

impl Radixable for (u32, u32) {
    type KeyType = u64;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> (u32, u32) {
        let fst = (value as u64 & 0xFFFF_FFFF_0000_0000) >> 32;
        let snd = value as u32;
        (fst as u32, snd)
    }
    #[inline]
    fn into_key_type(&self) -> u64 {
        (u64::from(self.0) << 32) | u64::from(self.1)
    }
    #[inline]
    fn type_size(&self) -> usize {
        64
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u64 {
        item as u64
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u64) -> usize {
        item as usize
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(u32, u32)]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else if arr.len() <= 8000 {
            msd_radixsort(arr, 8);
        } else if arr.len() <= 100_000 {
            lsd_radixsort_heu(arr, 8, 200_000);
        } else {
            voracious_sort_heu(arr, 8, 200_000);
        }
    }
    fn dlsd_sort(&self, arr: &mut [(u32, u32)]) {
        if arr.len() <= 200 {
            arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}

impl Radixable for (u64, u64) {
    type KeyType = u128;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> (u64, u64) {
        let fst =
            (value as u128 & 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000) >> 64;
        let snd = value as u64;
        (fst as u64, snd)
    }
    #[inline]
    fn into_key_type(&self) -> u128 {
        (u128::from(self.0) << 64) | u128::from(self.1)
    }
    #[inline]
    fn type_size(&self) -> usize {
        128
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u128 {
        item as u128
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u128) -> usize {
        item as usize
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(u64, u64)]) {
        voracious_sort_heu(arr, 8, 200_000);
    }
    fn dlsd_sort(&self, arr: &mut [(u64, u64)]) {
        dlsd_radixsort(arr, 8);
    }
}

impl Radixable for (i64, i64) {
    type KeyType = u128;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline] // overrided function
    fn to_generic(&self, value: usize) -> (i64, i64) {
        let flip = 0x8000_0000_0000_0000;
        let fst =
            (value as u128 & 0xFFFF_FFFF_FFFF_FFFF_0000_0000_0000_0000) >> 64;
        let snd = value as u64;
        ((fst as u64 ^ flip) as i64, (snd as u64 ^ flip) as i64)
    }
    #[inline]
    fn into_key_type(&self) -> u128 {
        let flip = 0x8000_0000_0000_0000;
        (u128::from(self.0 as u64 ^ flip) << 64)
            | u128::from(self.1 as u64 ^ flip)
    }
    #[inline]
    fn type_size(&self) -> usize {
        128
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u128 {
        item as u128
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u128) -> usize {
        item as usize
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(i64, i64)]) {
        voracious_sort_heu(arr, 8, 200_000);
    }
    fn dlsd_sort(&self, arr: &mut [(i64, i64)]) {
        dlsd_radixsort(arr, 8);
    }
}

impl Radixable for (usize, f64) {
    type KeyType = u64;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u64 {
        unsafe {
            let submask = 0x8000_0000_0000_0000;
            let casted = std::mem::transmute::<f64, u64>(self.1);

            if casted & submask == submask {
                casted ^ 0xFFFF_FFFF_FFFF_FFFF
            } else {
                casted ^ submask
            }
        }
    }
    #[inline]
    fn type_size(&self) -> usize {
        64
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u64 {
        item as u64
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u64) -> usize {
        item as usize
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(usize, f64)]) {
        if arr.len() <= 500 {
            arr.sort_unstable_by(|a, b| (a.1).partial_cmp(&(b.1)).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn dlsd_sort(&self, arr: &mut [(usize, f64)]) {
        if arr.len() <= 500 {
            voracious_sort(arr, 8);
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}

impl Radixable for (usize, f32) {
    type KeyType = u32;

    #[inline]
    fn extract(&self, mask: Self::KeyType, shift: usize) -> usize {
        ((self.into_key_type() & mask) >> shift) as usize
    }
    #[inline]
    fn into_key_type(&self) -> u32 {
        unsafe {
            let submask = 0x8000_0000;
            let casted = std::mem::transmute::<f32, u32>(self.1);

            if casted & submask == submask {
                casted ^ 0xFFFF_FFFF
            } else {
                casted ^ submask
            }
        }
    }
    #[inline]
    fn type_size(&self) -> usize {
        32
    }
    #[inline(always)]
    fn usize_to_keytype(&self, item: usize) -> u32 {
        item as u32
    }
    #[inline(always)]
    fn keytype_to_usize(&self, item: u32) -> usize {
        item as usize
    }
    #[inline]
    fn default_key(&self) -> Self::KeyType {
        0
    }
    #[inline]
    fn one(&self) -> Self::KeyType {
        1
    }
    fn voracious_sort(&self, arr: &mut [(usize, f32)]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| (a.1).partial_cmp(&(b.1)).unwrap());
        } else {
            lsd_radixsort(arr, 8);
        }
    }
    fn dlsd_sort(&self, arr: &mut [(usize, f32)]) {
        if arr.len() <= 300 {
            arr.sort_unstable_by(|a, b| (a.1).partial_cmp(&(b.1)).unwrap());
        } else {
            dlsd_radixsort(arr, 8);
        }
    }
}
