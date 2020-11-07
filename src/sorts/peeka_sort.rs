use rayon::{ThreadPool, ThreadPoolBuilder};

use std::sync::mpsc::channel;

use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::regions_graph::{swap_countries, RegionsGraph};
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{RadixKey, RadixSort, Radixable};
use super::rollercoaster_sort::fallback;
use super::ska_sort::ska_swap;
use super::utils::{get_histogram, prefix_sums, Params};

const FALLBACK_THRESHOLD: usize = 128_000;

fn local_sorting<T, K>(
    arr: &mut [T],
    p: &Params,
    block_size: usize,
    pool: &ThreadPool,
) -> Vec<Vec<usize>>
where
    T: Radixable<K>,
    K: RadixKey,
{
    let dummy = arr[0];
    let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
    let mut histograms: Vec<Vec<usize>> = Vec::new();
    let mut receivers = Vec::new();

    pool.scope(|s| {
        let mut rest = arr;
        while !rest.is_empty() {
            let (mut fst, snd) = if block_size < rest.len() {
                rest.split_at_mut(block_size)
            } else {
                (rest, &mut [] as &mut [T])
            };
            rest = snd;

            let (sender, receiver) = channel();
            receivers.push(receiver);
            s.spawn(move |_| {
                let h = get_histogram(fst, p, mask, shift);
                let (_, mut heads, tails) = prefix_sums(&h);

                ska_swap(&mut fst, &mut heads, &tails, mask, shift);

                sender.send(h).unwrap();
            });
        }
    });

    for receiver in receivers.iter() {
        histograms.push(receiver.recv().unwrap());
    }

    histograms
}

fn peeka_sort_rec<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: Params,
    pool: &ThreadPool,
    block_size: usize,
) {
    if arr.len() <= FALLBACK_THRESHOLD {
        fallback(arr, p);
        return;
    }

    let dummy = arr[0];

    // Local Sorting Phase for each block
    let histograms = if arr.len() <= block_size {
        let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
        let histogram = get_histogram(arr, &p, mask, shift);
        let (_, mut heads, tails) = prefix_sums(&histogram);

        ska_swap(arr, &mut heads, &tails, mask, shift);

        vec![histogram]
    } else {
        local_sorting(arr, &p, block_size, pool)
    };

    // Graph Construction Phase
    let mut regions_graph = RegionsGraph::new(p.radix_range);
    let global_histogram = regions_graph.build_regions_graph(&histograms);

    // let sorted_countries = sort_countries(&global_histogram);
    let (p_sums, _, _) = prefix_sums(&global_histogram);

    // Global Sorting Phase and early recursion
    let mut countries = Vec::new();
    let mut rest = arr;
    let mut country_map = vec![0; p.radix_range];

    for country_id in 0..p.radix_range {
        let end = p_sums[country_id + 1] - p_sums[country_id];
        let (country, snd) = rest.split_at_mut(end);
        countries.push((country_id, country, p_sums[country_id]));
        rest = snd;
    }
    countries.sort_unstable_by(|(_, a, _), (_, b, _)| {
        a.len()
            .partial_cmp(&b.len())
            .expect("[Regions sort -> countries sorting] Bad implementation.")
    });
    countries.iter().enumerate().for_each(|(i, &(country_id, _, _))| {
        country_map[country_id] = i;
    });

    pool.scope(|s| {
        let mut smalls = Vec::new();
        for _ in 0..p.radix_range {
            let (bro_id, mut broker, bro_offset) = countries
                .pop()
                .expect("[Regions sort -> swapping] Bad implementation.");

            let swaps = regions_graph.two_cycle(bro_id);
            swap_countries(
                swaps,
                &mut broker,
                &mut countries,
                &country_map,
                bro_offset,
            );

            let swaps = regions_graph.two_path(bro_id);
            swap_countries(
                swaps,
                &mut broker,
                &mut countries,
                &country_map,
                bro_offset,
            );

            if p.level < p.max_level - 1 {
                if broker.len() > 3000 {
                    s.spawn(move |_| {
                        let new_params = p.new_level(p.level + 1);
                        peeka_sort_rec(
                            &mut broker,
                            new_params,
                            pool,
                            block_size,
                        );
                    });
                } else {
                    smalls.push(broker);
                }
            }
        }

        for mut small_array in smalls.into_iter() {
            fallback(&mut small_array, p.new_level(p.level + 1));
        }
    });
}

/// # Peek Regions sort (Peekasort)
///
/// This is an improvement of the
/// [Regions sort](https://github.com/omarobeya/parallel-inplace-radixsort) and
/// the [research article](https://people.csail.mit.edu/jshun/RegionsSort.pdf).
///
/// The Verge sort pre-processing heuristic has been added.
///
/// This sort is an inplace unstable radix sort.
///
/// For "small" arrays, this sort fallbacks on the single thread Voracious sort.
/// In the trait implementation, there is a first fallback on the Rayon
/// parallel quicksort.
pub fn peeka_sort<T, K>(
    arr: &mut [T],
    radix: usize,
    block_size: usize,
    thread_n: usize,
) where
    T: Radixable<K>,
    K: RadixKey,
{
    let size = arr.len();
    if size <= FALLBACK_THRESHOLD {
        arr.voracious_sort();
        return;
    }

    let pool = ThreadPoolBuilder::new().num_threads(thread_n).build().unwrap();

    let dummy = arr[0];
    let mut separators = verge_sort_preprocessing(arr, radix, &|array, rdx| {
        let (_offset, raw_offset) = dummy.compute_offset_mt(array, rdx);
        let max_level = dummy.compute_max_level(raw_offset, rdx);

        if max_level > 0 {
            let params = Params::new(0, rdx, raw_offset, max_level);

            peeka_sort_rec(array, params, &pool, block_size);
        }
    });

    k_way_merge(arr, &mut separators);
}
