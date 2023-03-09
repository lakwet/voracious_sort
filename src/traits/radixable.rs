use super::super::sorts::utils::{
    compute_max_level, compute_offset, get_full_histograms, Params,
};
#[cfg(feature = "voracious_multithread")]
use super::super::sorts::utils_mt::compute_offset_mt;
use super::super::{Dispatcher, RadixKey};

#[cfg(feature = "voracious_multithread")]
pub trait Radixable<K: RadixKey>: Copy + PartialOrd + Send + Sync {
    type Key: RadixKey + Dispatcher<Self, K>;
    fn key(&self) -> Self::Key;
    #[inline]
    fn extract(
        &self,
        mask: <<Self as Radixable<K>>::Key as RadixKey>::Key,
        shift: usize,
    ) -> usize {
        let s = self.usize_to_keytype(shift);
        self.keytype_to_usize((self.into_key_type() & mask) >> s)
    }
    #[inline]
    fn into_key_type(&self) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().into_keytype()
    }
    #[inline]
    fn type_size(&self) -> usize { self.key().type_size() }
    #[inline]
    fn usize_to_keytype(
        &self,
        item: usize,
    ) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().usize_to_keytype(item)
    }
    #[inline]
    fn keytype_to_usize(
        &self,
        item: <<Self as Radixable<K>>::Key as RadixKey>::Key,
    ) -> usize {
        self.key().keytype_to_usize(item)
    }
    #[inline]
    fn default_mask(
        &self,
        radix: usize,
    ) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        let mut mask: usize = 0;
        for _ in 0..radix {
            mask = (mask << 1) | 1;
        }
        self.usize_to_keytype(mask)
    }
    #[inline]
    fn get_mask_and_shift(
        &self,
        p: &Params,
    ) -> (<<Self as Radixable<K>>::Key as RadixKey>::Key, usize) {
        assert!(p.level < p.max_level);
        let mask = self.default_mask(p.radix);
        let shift: usize = p.radix * (p.max_level - p.level - 1);
        let mask = mask << self.usize_to_keytype(shift);

        (mask, shift)
    }
    #[inline]
    fn get_mask_and_shift_from_left(
        &self,
        p: &Params,
    ) -> (<<Self as Radixable<K>>::Key as RadixKey>::Key, usize) {
        let mask = self.default_mask(p.radix);
        let bits = self.type_size();
        let (mask, shift) = if bits < p.offset + p.radix * (p.level + 1) {
            let r_shift = p.offset + p.radix * (p.level + 1) - bits;
            (mask >> self.usize_to_keytype(r_shift), 0)
        } else {
            (mask, bits - p.offset - p.radix * (p.level + 1))
        };
        let mask = mask << self.usize_to_keytype(shift);

        (mask, shift)
    }
    // If counting sort is used, this method must be implemented and the
    // transformation from the type to the key must be bijective.
    #[inline]
    fn to_generic(&self, _value: usize) -> Self {
        panic!(
            "[Radiaxble -> to_generic] Counting Sort cannot be used on Struct."
        );
    }
    #[inline]
    fn compute_offset(&self, arr: &mut [Self], radix: usize) -> (usize, usize) {
        compute_offset(arr, radix)
    }
    #[cfg(feature = "voracious_multithread")]
    #[inline]
    fn compute_offset_mt(
        &self,
        arr: &mut [Self],
        radix: usize,
    ) -> (usize, usize) {
        compute_offset_mt(arr, radix)
    }
    #[inline]
    fn get_max_key(
        &self,
        arr: &mut [Self],
    ) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        arr.iter().map(|item| item.into_key_type()).max().unwrap()
    }
    #[inline]
    fn compute_max_level(&self, offset: usize, radix: usize) -> usize {
        compute_max_level(self.type_size(), offset, radix)
    }
    #[inline]
    fn default_key(&self) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().default_key()
    }
    #[inline]
    fn one(&self) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().one()
    }
    #[inline]
    fn is_i32(&self) -> bool { false }
    #[inline]
    fn is_i64(&self) -> bool { false }
    #[inline]
    fn is_i128(&self) -> bool { false }
    #[inline]
    fn mask_for_high_bits(
        &self,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        let bits = self.type_size();
        let default_mask = self.default_mask(radix);
        let mut mask = self.default_key();
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << self.usize_to_keytype(radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << self.usize_to_keytype(
                        bits - offset - radix * (max_level - level),
                    );
            }
        }
        mask
    }
    #[inline]
    fn get_full_histograms(
        &self,
        arr: &mut [Self],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        get_full_histograms(arr, p)
    }
    fn voracious_sort(&self, arr: &mut [Self]) {
        if arr.len() > 1 {
            let dummy_key = arr[0].key();
            Dispatcher::voracious_sort(&dummy_key, arr);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [Self]) {
        if arr.len() > 1 {
            let dummy_key = arr[0].key();
            Dispatcher::voracious_stable_sort(&dummy_key, arr);
        }
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        if arr.len() > 1 {
            let dummy_key = arr[0].key();
            Dispatcher::voracious_mt_sort(&dummy_key, arr, thread_n);
        }
    }
}

#[cfg(not(feature = "voracious_multithread"))]
pub trait Radixable<K: RadixKey>: Copy + PartialOrd {
    type Key: RadixKey + Dispatcher<Self, K>;
    fn key(&self) -> Self::Key;
    #[inline]
    fn extract(
        &self,
        mask: <<Self as Radixable<K>>::Key as RadixKey>::Key,
        shift: usize,
    ) -> usize {
        let s = self.usize_to_keytype(shift);
        self.keytype_to_usize((self.into_key_type() & mask) >> s)
    }
    #[inline]
    fn into_key_type(&self) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().into_keytype()
    }
    #[inline]
    fn type_size(&self) -> usize { self.key().type_size() }
    #[inline]
    fn usize_to_keytype(
        &self,
        item: usize,
    ) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().usize_to_keytype(item)
    }
    #[inline]
    fn keytype_to_usize(
        &self,
        item: <<Self as Radixable<K>>::Key as RadixKey>::Key,
    ) -> usize {
        self.key().keytype_to_usize(item)
    }
    #[inline]
    fn default_mask(
        &self,
        radix: usize,
    ) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        let mut mask: usize = 0;
        for _ in 0..radix {
            mask = (mask << 1) | 1;
        }
        self.usize_to_keytype(mask)
    }
    #[inline]
    fn get_mask_and_shift(
        &self,
        p: &Params,
    ) -> (<<Self as Radixable<K>>::Key as RadixKey>::Key, usize) {
        assert!(p.level < p.max_level);
        let mask = self.default_mask(p.radix);
        let shift: usize = p.radix * (p.max_level - p.level - 1);
        let mask = mask << self.usize_to_keytype(shift);

        (mask, shift)
    }
    #[inline]
    fn get_mask_and_shift_from_left(
        &self,
        p: &Params,
    ) -> (<<Self as Radixable<K>>::Key as RadixKey>::Key, usize) {
        let mask = self.default_mask(p.radix);
        let bits = self.type_size();
        let (mask, shift) = if bits < p.offset + p.radix * (p.level + 1) {
            let r_shift = p.offset + p.radix * (p.level + 1) - bits;
            (mask >> self.usize_to_keytype(r_shift), 0)
        } else {
            (mask, bits - p.offset - p.radix * (p.level + 1))
        };
        let mask = mask << self.usize_to_keytype(shift);

        (mask, shift)
    }
    // If counting sort is used, this method must be implemented and the
    // transformation from the type to the key must be bijective.
    #[inline]
    fn to_generic(&self, _value: usize) -> Self {
        panic!(
            "[Radiaxble -> to_generic] Counting Sort cannot be used on Struct."
        );
    }
    #[inline]
    fn compute_offset(&self, arr: &mut [Self], radix: usize) -> (usize, usize) {
        compute_offset(arr, radix)
    }
    #[cfg(feature = "voracious_multithread")]
    #[inline]
    fn compute_offset_mt(
        &self,
        arr: &mut [Self],
        radix: usize,
    ) -> (usize, usize) {
        compute_offset_mt(arr, radix)
    }
    #[inline]
    fn get_max_key(
        &self,
        arr: &mut [Self],
    ) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        arr.iter().map(|item| item.into_key_type()).max().unwrap()
    }
    #[inline]
    fn compute_max_level(&self, offset: usize, radix: usize) -> usize {
        compute_max_level(self.type_size(), offset, radix)
    }
    #[inline]
    fn default_key(&self) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().default_key()
    }
    #[inline]
    fn one(&self) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().one()
    }
    #[inline]
    fn is_i32(&self) -> bool { false }
    #[inline]
    fn is_i64(&self) -> bool { false }
    #[inline]
    fn is_i128(&self) -> bool { false }
    #[inline]
    fn mask_for_high_bits(
        &self,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        let bits = self.type_size();
        let default_mask = self.default_mask(radix);
        let mut mask = self.default_key();
        if radix * max_level > bits - offset {
            for level in 0..max_level {
                mask |= default_mask << self.usize_to_keytype(radix * level);
            }
        } else {
            for level in 0..max_level {
                mask |= default_mask
                    << self.usize_to_keytype(
                        bits - offset - radix * (max_level - level),
                    );
            }
        }
        mask
    }
    #[inline]
    fn get_full_histograms(
        &self,
        arr: &mut [Self],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        get_full_histograms(arr, p)
    }
    fn voracious_sort(&self, arr: &mut [Self]) {
        if arr.len() > 1 {
            let dummy_key = arr[0].key();
            Dispatcher::voracious_sort(&dummy_key, arr);
        }
    }
    fn voracious_stable_sort(&self, arr: &mut [Self]) {
        if arr.len() > 1 {
            let dummy_key = arr[0].key();
            Dispatcher::voracious_stable_sort(&dummy_key, arr);
        }
    }
    #[cfg(feature = "voracious_multithread")]
    fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
        if arr.len() > 1 {
            let dummy_key = arr[0].key();
            Dispatcher::voracious_mt_sort(&dummy_key, arr, thread_n);
        }
    }
}
