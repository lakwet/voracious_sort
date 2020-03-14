use super::super::RadixKey;
use super::super::sorts::utils::{
    compute_max_level, compute_offset, get_full_histograms_fast, Params,
};

pub trait Radixable<K: RadixKey>: Sized + Copy + PartialOrd + Send + Sync {
    type Key: RadixKey;
    fn key(&self) -> Self::Key;
    #[inline] // default implementation, might be override
    fn extract(&self, mask: <<Self as Radixable<K>>::Key as RadixKey>::Key, shift: usize) -> usize {
        let s = self.usize_to_keytype(shift);
        self.keytype_to_usize((self.into_key_type() & mask) >> s)
    }
    fn into_key_type(&self) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().into_keytype()
    }
    fn type_size(&self) -> usize { self.key().type_size() }
    fn usize_to_keytype(&self, item: usize) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().usize_to_keytype(item)
    }
    fn keytype_to_usize(&self, item: <<Self as Radixable<K>>::Key as RadixKey>::Key) -> usize {
        self.key().keytype_to_usize(item)
    }
    #[inline]
    fn default_mask(&self, radix: usize) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        let mut mask: usize = 0;
        for _ in 0..radix {
            mask = (mask << 1) | 1;
        }
        self.usize_to_keytype(mask)
    }
    #[inline]
    fn get_mask_and_shift(&self, p: &Params) -> (<<Self as Radixable<K>>::Key as RadixKey>::Key, usize) {
        let mask = self.default_mask(p.radix);
        let shift: usize = p.radix * (p.max_level - p.level - 1);
        let mask = mask << self.usize_to_keytype(shift);

        (mask, shift)
    }
    #[inline] // default implementation, might be override
    fn get_mask_and_shift_from_left(
        &self,
        p: &Params,
    ) -> (<<Self as Radixable<K>>::Key as RadixKey>::Key, usize) {
        let mask = self.default_mask(p.radix);
        let bits = self.type_size();
        let shift = if bits < p.offset + p.radix * (p.level + 1) {
            0
        } else {
            bits - p.offset - p.radix * (p.level + 1)
        };
        let mask = mask << self.usize_to_keytype(shift);

        (mask, shift)
    }
    // If counting sort is used, this method must be implemented and the
    // transformation from the type to the key must be bijective.
    #[inline] // default implementation, might be override
    fn to_generic(&self, _value: usize) -> Self {
        panic!("[Radiaxble -> to_generic] Counting Sort cannot be used on Struct.");
    }
    #[inline] // default implementation, might be override
    fn compute_offset(&self, arr: &mut [Self], radix: usize) -> (usize, usize) {
        compute_offset(arr, radix)
    }
    #[inline]
    fn get_max_key(&self, arr: &mut [Self]) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        arr.iter().map(|item| item.into_key_type()).max().unwrap()
    }
    #[inline] // default implementation, might be override
    fn compute_max_level(&self, offset: usize, radix: usize) -> usize {
        compute_max_level(self.type_size(), offset, radix)
    }
    fn default_key(&self) -> <<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().default_key()
    }
    fn one(&self) -><<Self as Radixable<K>>::Key as RadixKey>::Key {
        self.key().one()
    }
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
    fn get_full_histograms(
        &self,
        arr: &mut [Self],
        p: &Params,
    ) -> Vec<Vec<usize>> {
        get_full_histograms_fast(arr, p)
    }
    fn voracious_sort(&self, arr: &mut [Self]);
    fn dlsd_sort(&self, arr: &mut [Self]);
}
