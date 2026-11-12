//! Generators shared by u8, i8, u16 and i16, and extra generators for char,
//! added to each type's own list. With 8 or 16 bits most of the large arrays
//! are full of duplicates, so these cover what the type's own lists miss:
//! very low cardinality, sorted and nearly sorted data with duplicates,
//! real data shapes (audio samples, sensor values, aligned values, sparse
//! data) and the worst cases of the detectors (late outlier, random tail).
//! The char generators follow real text: ASCII, accented Latin, Cyrillic,
//! CJK, emoji and a mix of them.

use rand::RngExt;
use rand_distr::{Distribution, Normal};
use rayon::prelude::*;

macro_rules! narrow_extra {
    ($t:ty, $name:ident) => {
        pub mod $name {
            use super::*;

            const BITS: u32 = <$t>::BITS;
            // Middle of the domain: 0 for signed types, 2^(bits-1) otherwise.
            const MID: $t = <$t>::MIN / 2 + <$t>::MAX / 2 + 1;

            fn random_vec(size: usize) -> Vec<$t> {
                (0..size).into_par_iter().map(|_| rand::rng().random::<$t>()).collect()
            }

            // Very low cardinality: 4 random values
            pub fn few4(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let values: Vec<$t> = (0..4).map(|_| rng.random::<$t>()).collect();
                (0..size).map(|_| values[rng.random_range(0..4)]).collect()
            }

            // Tiny range: 16 consecutive values starting at a random one
            pub fn range16(size: usize) -> Vec<$t> {
                let base = rand::rng().random_range(<$t>::MIN..=<$t>::MAX - 16);
                (0..size).into_par_iter().map(|_| base + rand::rng().random_range(0..16) as $t).collect()
            }

            // Random values already sorted (duplicates included)
            pub fn sorted(size: usize) -> Vec<$t> {
                let mut array = random_vec(size);
                array.sort_unstable();
                array
            }

            // Random values sorted in descending order
            pub fn sorted_desc(size: usize) -> Vec<$t> {
                let mut array = sorted(size);
                array.reverse();
                array
            }

            // Sorted, then 1 % of the positions swapped at random
            pub fn near_sorted(size: usize) -> Vec<$t> {
                let mut array = sorted(size);
                let mut rng = rand::rng();
                for _ in 0..size / 100 {
                    let (i, j) = (rng.random_range(0..size), rng.random_range(0..size));
                    array.swap(i, j);
                }
                array
            }

            // Detector worst case: sorted for 90 %, random tail
            pub fn sorted_random_tail(size: usize) -> Vec<$t> {
                let mut array = random_vec(size);
                array[..size * 9 / 10].sort_unstable();
                array
            }

            // Concatenation of 8 independently sorted random sources
            pub fn runs8(size: usize) -> Vec<$t> {
                let mut array = random_vec(size);
                for r in 0..8 {
                    array[r * size / 8..(r + 1) * size / 8].sort_unstable();
                }
                array
            }

            // Pipe organ: ascending half, then descending half
            pub fn pipe_organ(size: usize) -> Vec<$t> {
                let mut array = random_vec(size);
                let half = size / 2;
                array[..half].sort_unstable();
                array[half..].sort_unstable_by(|a, b| b.cmp(a));
                array
            }

            // Detector worst case: tiny range, one outlier late (at 95 %)
            pub fn late_outlier(size: usize) -> Vec<$t> {
                let mut array = range16(size);
                if size > 0 {
                    array[size * 95 / 100] = <$t>::MAX;
                }
                array
            }

            // Sparse data: 90 % zeros, 10 % random values
            pub fn sparse(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|_| {
                        let mut rng = rand::rng();
                        if rng.random_range(0..10) == 0 { rng.random::<$t>() } else { 0 }
                    })
                    .collect()
            }

            // Bimodal: half at the bottom of the domain, half at the top
            pub fn bimodal(size: usize) -> Vec<$t> {
                (0..size)
                    .into_par_iter()
                    .map(|i| {
                        let v = rand::rng().random_range(0..16) as $t;
                        if i % 2 == 0 { <$t>::MIN + v } else { <$t>::MAX - v }
                    })
                    .collect()
            }

            // Audio samples / pixels: normal around the middle of the
            // domain, standard deviation 1/32 of the domain, clamped.
            pub fn gaussian(size: usize) -> Vec<$t> {
                let sd = (1u64 << (BITS - 5)) as f64;
                let normal = Normal::new(MID as f64, sd).unwrap();
                (0..size)
                    .into_par_iter()
                    .map(|_| normal.sample(&mut rand::rng()).clamp(<$t>::MIN as f64, <$t>::MAX as f64) as $t)
                    .collect()
            }

            // Sensor values: 3/4 of the bits used (12 bits for 16-bit keys)
            pub fn sensor(size: usize) -> Vec<$t> {
                let top = 1u32 << (BITS * 3 / 4);
                (0..size).into_par_iter().map(|_| rand::rng().random_range(0..top) as $t).collect()
            }

            // Aligned values: the low half of the bits is zero
            pub fn aligned(size: usize) -> Vec<$t> {
                let mask = !(((1u32 << (BITS / 2)) - 1) as $t);
                (0..size).into_par_iter().map(|_| rand::rng().random::<$t>() & mask).collect()
            }

            // 60 % of the keys among 4 values, the rest random
            pub fn heavy4(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let values: Vec<$t> = (0..4).map(|_| rng.random::<$t>()).collect();
                (0..size)
                    .map(|_| if rng.random_range(0..10) < 6 { values[rng.random_range(0..4)] } else { rng.random::<$t>() })
                    .collect()
            }

            // Tight clusters: 16 random centres, each key within 8 of its centre
            pub fn clusters(size: usize) -> Vec<$t> {
                let mut rng = rand::rng();
                let centres: Vec<$t> = (0..16).map(|_| rng.random::<$t>()).collect();
                (0..size).map(|_| centres[rng.random_range(0..16)].wrapping_add(rng.random_range(0..8) as $t)).collect()
            }

            pub fn generators() -> Vec<(&'static dyn Fn(usize) -> Vec<$t>, &'static str)> {
                vec![
                    (&few4, "-- Few 4      :"),
                    (&heavy4, "-- Heavy 4    :"),
                    (&clusters, "-- Clusters   :"),
                    (&range16, "-- Range 16   :"),
                    (&sorted, "-- Sorted     :"),
                    (&sorted_desc, "-- Sorted desc:"),
                    (&near_sorted, "-- Near sorted:"),
                    (&sorted_random_tail, "-- Sort rnd tl:"),
                    (&runs8, "-- Runs 8     :"),
                    (&pipe_organ, "-- Pipe organ :"),
                    (&late_outlier, "-- Late outl. :"),
                    (&sparse, "-- Sparse 90% :"),
                    (&bimodal, "-- Bimodal    :"),
                    (&gaussian, "-- Gaussian   :"),
                    (&sensor, "-- Sensor 3/4 :"),
                    (&aligned, "-- Aligned    :"),
                ]
            }
        }
    };
}

narrow_extra!(u8, extra_u8);
narrow_extra!(i8, extra_i8);
narrow_extra!(u16, extra_u16);
narrow_extra!(i16, extra_i16);

pub mod extra_char {
    use super::*;

    fn chr(c: u32) -> char {
        std::char::from_u32(c).unwrap()
    }

    // Letters weighted roughly like English text, with spaces and punctuation
    const TEXT: &str = "eeeeeeeeeeeettttttttttaaaaaaaaoooooooiiiiiiinnnnnnnsssssshhhhhhrrrrrrdddddllllcccuuummwwffggyyppbbvkjxqzETAOINSHRDL0123456789                  ,,,..'-";

    fn text_from(alphabet: &[char], size: usize) -> Vec<char> {
        (0..size).into_par_iter().map(|_| alphabet[rand::rng().random_range(0..alphabet.len())]).collect()
    }

    // English-like ASCII text
    pub fn ascii_text(size: usize) -> Vec<char> {
        let alphabet: Vec<char> = TEXT.chars().collect();
        text_from(&alphabet, size)
    }

    // French-like text: ASCII text with about 4 % of accented letters
    pub fn latin_text(size: usize) -> Vec<char> {
        let mut alphabet: Vec<char> = TEXT.chars().collect();
        alphabet.extend("éééèàçêùôî".chars());
        text_from(&alphabet, size)
    }

    // Cyrillic letters (U+0410..U+044F) and spaces
    pub fn cyrillic_text(size: usize) -> Vec<char> {
        let mut alphabet: Vec<char> = (0x410..0x450).map(chr).collect();
        alphabet.extend(std::iter::repeat(' ').take(10));
        text_from(&alphabet, size)
    }

    // CJK text: 3000 ideographs of U+4E00..U+9FFF, Zipf-like frequencies
    pub fn cjk_text(size: usize) -> Vec<char> {
        let mut rng = rand::rng();
        let common: Vec<char> = (0..3000).map(|_| chr(rng.random_range(0x4E00..0xA000))).collect();
        (0..size)
            .into_par_iter()
            .map(|_| {
                let u: f64 = rand::rng().random();
                common[(3000f64.powf(u) as usize - 1).min(2999)]
            })
            .collect()
    }

    // Emoji (U+1F300..U+1FAFF)
    pub fn emoji(size: usize) -> Vec<char> {
        (0..size).into_par_iter().map(|_| chr(rand::rng().random_range(0x1F300..0x1FB00))).collect()
    }

    // Web page: 85 % ASCII text, 10 % accented Latin, 3 % CJK, 2 % emoji
    pub fn mixed_web(size: usize) -> Vec<char> {
        let ascii: Vec<char> = TEXT.chars().collect();
        let accents: Vec<char> = (0xC0..0x180).filter(|&c| c != 0xD7 && c != 0xF7).map(chr).collect();
        (0..size)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::rng();
                match rng.random_range(0..100) {
                    0..=84 => ascii[rng.random_range(0..ascii.len())],
                    85..=94 => accents[rng.random_range(0..accents.len())],
                    95..=97 => chr(rng.random_range(0x4E00..0xA000)),
                    _ => chr(rng.random_range(0x1F300..0x1FB00)),
                }
            })
            .collect()
    }

    // Numbers in a CSV file: digits, separators, sign and decimal point
    pub fn digits(size: usize) -> Vec<char> {
        let alphabet: Vec<char> = "01234567890123456789,,,.-\n".chars().collect();
        text_from(&alphabet, size)
    }

    // Sorted ASCII text
    pub fn sorted_text(size: usize) -> Vec<char> {
        let mut array = ascii_text(size);
        array.sort_unstable();
        array
    }

    // Detector worst case: ASCII text, one emoji late (at 95 %)
    pub fn ascii_late_emoji(size: usize) -> Vec<char> {
        let mut array = ascii_text(size);
        if size > 0 {
            array[size * 95 / 100] = '\u{1F600}';
        }
        array
    }

    // Basic Multilingual Plane, uniform (surrogates excluded)
    pub fn bmp(size: usize) -> Vec<char> {
        (0..size)
            .into_par_iter()
            .map(|_| {
                let c = rand::rng().random_range(0..0xF800u32);
                chr(if c < 0xD800 { c } else { c + 0x800 })
            })
            .collect()
    }

    // ASCII text sorted in descending order
    pub fn sorted_text_desc(size: usize) -> Vec<char> {
        let mut array = sorted_text(size);
        array.reverse();
        array
    }

    // Sorted text with 1 % of the characters swapped at random
    pub fn near_sorted_text(size: usize) -> Vec<char> {
        let mut array = sorted_text(size);
        let mut rng = rand::rng();
        for _ in 0..size / 100 {
            let (i, j) = (rng.random_range(0..size), rng.random_range(0..size));
            array.swap(i, j);
        }
        array
    }

    // 8 sorted chunks of text (merged sorted word lists)
    pub fn text_runs8(size: usize) -> Vec<char> {
        let mut array = ascii_text(size);
        for r in 0..8 {
            array[r * size / 8..(r + 1) * size / 8].sort_unstable();
        }
        array
    }

    // Padded columns: 90 % spaces, the rest text
    pub fn padded(size: usize) -> Vec<char> {
        let alphabet: Vec<char> = TEXT.chars().collect();
        (0..size)
            .into_par_iter()
            .map(|_| {
                let mut rng = rand::rng();
                if rng.random_range(0..10) == 0 { alphabet[rng.random_range(0..alphabet.len())] } else { ' ' }
            })
            .collect()
    }

    // Any code point, uniform (surrogates excluded)
    pub fn all_planes(size: usize) -> Vec<char> {
        (0..size)
            .into_par_iter()
            .map(|_| {
                let c = rand::rng().random_range(0..0x10F800u32);
                chr(if c < 0xD800 { c } else { c + 0x800 })
            })
            .collect()
    }

    pub fn generators() -> Vec<(&'static dyn Fn(usize) -> Vec<char>, &'static str)> {
        vec![
            (&sorted_text_desc, "-- Sorted desc:"),
            (&near_sorted_text, "-- Near sorted:"),
            (&text_runs8, "-- Runs 8     :"),
            (&padded, "-- Padded 90% :"),
            (&all_planes, "-- All planes :"),
            (&ascii_text, "-- ASCII text :"),
            (&latin_text, "-- Latin text :"),
            (&cyrillic_text, "-- Cyrillic   :"),
            (&cjk_text, "-- CJK text   :"),
            (&emoji, "-- Emoji      :"),
            (&mixed_web, "-- Mixed web  :"),
            (&digits, "-- Digits CSV :"),
            (&sorted_text, "-- Sorted text:"),
            (&ascii_late_emoji, "-- Late emoji :"),
            (&bmp, "-- BMP        :"),
        ]
    }
}

