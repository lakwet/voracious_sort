//! Key codecs: the map from stored bits to keys whose unsigned order is the
//! wanted order (`encode`), and back (`decode`), as functions generated
//! into each engine module (`BITS` in scope).

/// The map from the stored bits to keys whose unsigned order is the wanted
/// order (`encode`), and back (`decode`): floats (total order: flip every
/// bit of negative values, the sign bit of the others), unsigned integers
/// (identity), signed integers (flip the sign bit).
macro_rules! key_codec {
    (float, $uint:ty, $sint:ty) => {
        #[inline(always)]
        fn encode(bits: $uint) -> $uint {
            const SIGN: $uint = 1 << (BITS - 1);
            bits ^ ((((bits as $sint) >> (BITS - 1)) as $uint) | SIGN)
        }

        #[inline(always)]
        fn decode(key: $uint) -> $uint {
            const SIGN: $uint = 1 << (BITS - 1);
            key ^ ((key >> (BITS - 1)).wrapping_sub(1) | SIGN)
        }
    };
    (unsigned, $uint:ty, $sint:ty) => {
        #[inline(always)]
        fn encode(bits: $uint) -> $uint {
            bits
        }

        #[inline(always)]
        fn decode(key: $uint) -> $uint {
            key
        }
    };
    (signed, $uint:ty, $sint:ty) => {
        #[inline(always)]
        fn encode(bits: $uint) -> $uint {
            bits ^ (1 << (BITS - 1))
        }

        #[inline(always)]
        fn decode(key: $uint) -> $uint {
            key ^ (1 << (BITS - 1))
        }
    };
}
