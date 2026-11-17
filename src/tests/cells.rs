//! The cells shared by the decision trace and the sentinel bench: every
//! (type, size, distribution), with inputs seeded per cell (the same keys
//! at every run, whatever the cells kept). Records: `r<bytes><key>`
//! (`ru…`: the unstable sort).

use super::super::generators as g;
use super::super::generators::records::{keys_u32, keys_u64, rec, wrap, Gens, Rec};
use super::super::v2::sorts::by_key;
use super::super::v2::sorts::ravenous::ravenous_sort2;
use super::super::v2::sorts::devour::devour_sort2;

/// Sizes of every type; the 32- and 64-bit keys add `BIG`.
pub(super) const SIZES: [usize; 10] = [1_000, 10_000, 30_000, 50_000, 100_000, 200_000, 600_000, 1_000_000, 3_000_000, 10_000_000];
pub(super) const BIG: [usize; 2] = [30_000_000, 100_000_000];

/// What is done with each cell.
pub(super) trait Visit {
    /// Whether the cell (`key`: see `key`) is visited.
    fn wants(&self, key: &str) -> bool;
    /// `keys()`: the cell's input (seeded, the same at every call).
    fn cell<T: Copy + PartialOrd + Send + 'static>(&mut self, key: &str, n: usize, keys: &dyn Fn() -> Vec<T>, sort: fn(&mut [T]));
}

/// The key of a cell: `type size name`, single spaces.
pub(super) fn key(label: &str, n: usize, name: &str) -> String {
    format!("{} {} {}", label, n, name)
}

/// A key read from a line of a trace or a list (`... : path` dropped,
/// spaces collapsed).
pub(super) fn parse_key(line: &str) -> Option<String> {
    let k = line.split(" : ").next()?.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut it = k.splitn(3, ' ');
    let (_, n, _) = (it.next()?, it.next()?, it.next()?);
    n.parse::<usize>().ok()?;
    Some(k)
}

fn all<T>(mut a: Gens<T>, b: Gens<T>) -> Gens<T> {
    a.extend(b);
    a
}

/// A generator handed to the pool's thread (they are plain functions).
#[derive(Clone, Copy)]
struct Shared<T: 'static>(&'static dyn Fn(usize) -> Vec<T>);
// SAFETY: the generators capture nothing; the pool runs one at a time.
unsafe impl<T> Send for Shared<T> {}
unsafe impl<T> Sync for Shared<T> {}

/// The cells of one type. Inputs are drawn from a seed of the cell (FNV-1a
/// of its key) in a one-thread pool.
fn cells<T: Copy + PartialOrd + Send + 'static, V: Visit>(v: &mut V, pool: &rayon::ThreadPool, label: &str, gens: Gens<T>, sort: fn(&mut [T]), big: bool, max: usize) {
    let sizes = SIZES.iter().chain(if big { BIG.iter() } else { [].iter() });
    for &n in sizes.filter(|&&n| n <= max) {
        let mut seen: Vec<String> = Vec::new();
        for (gen, name) in gens.iter() {
            let mut name = name.trim_start_matches("-- ").trim_end_matches(':').split_whitespace().collect::<Vec<_>>().join(" ");
            // Same name twice in a type: numbered.
            let base = name.clone();
            let mut k = 2;
            while seen.contains(&name) {
                name = format!("{} #{}", base, k);
                k += 1;
            }
            seen.push(name.clone());
            let key = key(label, n, &name);
            if !v.wants(&key) {
                continue;
            }
            let seed = key.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3));
            let gen = Shared(*gen);
            let keys = move || {
                pool.install(move || {
                    g::seed(seed);
                    (gen.0)(n)
                })
            };
            v.cell(&key, n, &keys, sort);
        }
    }
}

/// Visits every cell up to `max` keys, of the types in `types` (all when
/// `None`).
pub(super) fn visit<V: Visit>(v: &mut V, types: Option<&[String]>, max: usize) {
    let pool = rayon::ThreadPoolBuilder::new().num_threads(1).build().unwrap();
    let wants = |t: &str| types.map_or(true, |l| l.iter().any(|x| x == t));
    macro_rules! ty {
        ($label:expr, $gens:expr, $sort:expr, $big:expr) => {
            if wants($label) {
                cells(v, &pool, $label, $gens, $sort, $big, max);
            }
        };
    }
    ty!("u32", all(g::unsigned_u32::generators_u32(), g::int_extra::extra_u32::generators()), |a| devour_sort2(a), true);
    ty!("i32", all(g::signed_i32::generators_i32(), g::int_extra::extra_i32::generators()), |a| devour_sort2(a), true);
    ty!("u64", all(g::unsigned_u64::generators_u64(), g::int_extra::extra_u64::generators()), |a| devour_sort2(a), true);
    ty!("i64", all(g::signed_i64::generators_i64(), g::int_extra::extra_i64::generators()), |a| devour_sort2(a), true);
    ty!("f32", all(g::float_32::generators_f32(), g::float_extra::extra_f32::generators()), |a| ravenous_sort2(a), true);
    ty!("f64", all(g::float_64::generators_f64(), g::float_extra::extra_f64::generators()), |a| ravenous_sort2(a), true);
    ty!("u16", all(g::unsigned_u16::generators_u16(), g::narrow_extra::extra_u16::generators()), |a| devour_sort2(a), false);
    ty!("i16", all(g::signed_i16::generators_i16(), g::narrow_extra::extra_i16::generators()), |a| devour_sort2(a), false);
    ty!("u8", all(g::unsigned_u8::generators_u8(), g::narrow_extra::extra_u8::generators()), |a| devour_sort2(a), false);
    ty!("i8", all(g::signed_i8::generators_i8(), g::narrow_extra::extra_i8::generators()), |a| devour_sort2(a), false);
    ty!("char", all(g::char::generators_char(), g::narrow_extra::extra_char::generators()), |a| devour_sort2(a), false);
    ty!("u128", all(g::unsigned_u128::generators_u128(), g::int128_extra::extra_u128::generators()), |a| devour_sort2(a), false);
    ty!("i128", all(g::signed_i128::generators_i128(), g::int128_extra::extra_i128::generators()), |a| devour_sort2(a), false);
    // Records sorted by key (stable, but the last), as in the CPU bench;
    // 128-byte records up to 1M keys.
    macro_rules! rec {
        ($label:expr, $k:ty, $w:expr, $keys:expr, $sort:path, $cap:expr) => {
            if wants($label) {
                cells(v, &pool, $label, wrap($keys, rec::<$k, $w>), |a: &mut [Rec<$k, $w>]| $sort(a, |x| x.key), false, max.min($cap));
            }
        };
    }
    rec!("r8u32", u32, 1, keys_u32(), by_key::devour_sort_by_key, usize::MAX);
    rec!("r16u64", u64, 2, keys_u64(), by_key::devour_sort_by_key, usize::MAX);
    rec!("r32u64", u64, 6, keys_u64(), by_key::devour_sort_by_key, usize::MAX);
    rec!("r128u32", u32, 31, keys_u32(), by_key::devour_sort_by_key, 1_000_000);
    rec!("ru8u32", u32, 1, keys_u32(), by_key::devour_sort_unstable_by_key, usize::MAX);
    rec!("ru16u64", u64, 2, keys_u64(), by_key::devour_sort_unstable_by_key, usize::MAX);
    rec!("ru128u32", u32, 31, keys_u32(), by_key::devour_sort_unstable_by_key, 1_000_000);
}

/// `TRACE_TYPES` / `SENT_TYPES`-style list from the environment.
pub(super) fn env_list(var: &str) -> Option<Vec<String>> {
    std::env::var(var).ok().map(|s| s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect())
}

/// A size from the environment (`10M`, `1G`, `30000` accepted).
pub(super) fn env_size(var: &str) -> Option<usize> {
    let s = std::env::var(var).ok()?.trim().to_uppercase().replace('_', "");
    let (num, mul) = match s.chars().last()? {
        'K' => (&s[..s.len() - 1], 1_000),
        'M' => (&s[..s.len() - 1], 1_000_000),
        'G' => (&s[..s.len() - 1], 1_000_000_000),
        _ => (&s[..], 1),
    };
    num.parse::<usize>().ok().map(|x| x * mul)
}
