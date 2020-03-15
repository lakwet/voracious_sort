use rayon;

use std::sync::mpsc::channel;
use std::time::Instant;

use super::super::{RadixKey, Radixable};
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::algo::k_way_merge::k_way_merge_mt_with_buffer;
use super::comparative_sort::insertion_sort_try;
use super::lsd_sort::lsd_radixsort_body;
use super::ska_sort::ska_swap;
use super::msd_sort::{copy_by_histogram, msd_radixsort_rec};
use super::utils::{
    copy_nonoverlapping, only_one_bucket_filled,
    aggregate_histograms, swap_range, get_histogram, get_histogram_mt,
    prefix_sums, Params, perform_swaps_mt, perform_swaps,
    get_next_two_histograms,
};
use super::super::algo::verge_sort_heuristic::{
    explore_simple_forward, Orientation,
};

pub type CountryId = usize;
pub type RegionSize = usize;
pub type RegionStart = usize;

pub type RegionInfo = (CountryId, RegionStart, RegionSize);
pub type IncomingRegion = RegionInfo;
pub type OutgoingRegion = RegionInfo;

pub type Country = [Vec<RegionInfo>; 2];
pub type Countries = Vec<Country>;

pub type SwapSize = usize;
pub type SwapSource = usize;
pub type SwapDestination = usize;

// pub fn lsd_diversion<T: Radixable>(arr: &mut [T], p: Params) {
//     assert!(arr.len() <= 30_000);

//     if arr.len() <= 128 {
//         arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
//         return;
//     }

//     let size = arr.len();
//     let dummy = arr[0];
//     let mut buffer: Vec<T> = vec![arr[0]; arr.len()];

//     if p.max_level - p.level == 1 { // just do an lsd radixsort: 1 pass
//         let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
//         let histogram = get_histogram(arr, &p, mask, shift);
//         let (_, mut heads, _) = prefix_sums(&histogram);

//         copy_by_histogram(size, arr, &mut buffer, &mut heads, mask, shift);
//         copy_nonoverlapping(&mut buffer, arr, size);

//     } else { // do dlsd 2 passes
//         let histograms = get_next_two_histograms(arr, &p);

//         let pass_fst = only_one_bucket_filled(&histograms[1]);
//         let pass_snd = only_one_bucket_filled(&histograms[0]);

//         if pass_fst && pass_snd {

//         } else if pass_fst {
//             let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
//             let (_, mut heads, _) = prefix_sums(&histograms[0]);
//             copy_by_histogram(size, arr, &mut buffer,&mut heads, mask, shift);
//             copy_nonoverlapping(&mut buffer, arr, size);
//         } else if pass_snd {
//             let (mask, shift) = dummy.get_mask_and_shift_from_left(&p.new_level(p.level + 1));
//             let (_, mut heads, _) = prefix_sums(&histograms[1]);
//             copy_by_histogram(size, arr, &mut buffer, &mut heads, mask, shift);
//             copy_nonoverlapping(&mut buffer, arr, size);
//         } else {
//             let (mask, shift) = dummy.get_mask_and_shift_from_left(&p.new_level(p.level + 1));
//             let (_, mut heads, _) = prefix_sums(&histograms[1]);
//             copy_by_histogram(size, arr, &mut buffer, &mut heads, mask, shift);

//             let (mask, shift) = dummy.get_mask_and_shift_from_left(&p);
//             let (_, mut heads, _) = prefix_sums(&histograms[0]);
//             copy_by_histogram(size, &mut buffer, arr, &mut heads, mask, shift);
//         }

//         if p.max_level - p.level > 2 { // diversion if necessary
//             let new_params = p.new_level(0);
//             let unsorted_parts = insertion_sort_try(arr, &new_params);

//             unsorted_parts.iter().for_each(|(start, end)| {
//                 // msd_radixsort_rec(&mut arr[*start..*end], p.new_level(p.level + 2));
//                 let mut part = &mut arr[*start..*end];
//                 match explore_simple_forward(&mut part) {
//                     Orientation::IsAsc => (),
//                     Orientation::IsDesc => {
//                         part.reverse();
//                     }
//                     Orientation::IsPlateau => (),
//                     Orientation::IsNone => {
//                         msd_radixsort_rec(&mut part, p.new_level(p.level + 2));
//                     }
//                 }
//             });
//         }
//     }
// }

pub fn sort_countries(histogram: &Vec<usize>) -> Vec<usize> {
    let mut enriched: Vec<(usize, usize)> =
        histogram.iter().enumerate().map(|(i, v)| (i, *v)).collect();

    enriched.sort_unstable_by_key(|(_, v)| *v);
    enriched.reverse();

    enriched.iter().map(|(i, _)| *i).collect()
}

#[derive(Clone, Debug, Default)]
pub struct RegionsGraph {
    countries: Countries,
}

impl RegionsGraph {
    pub fn new(size: usize) -> RegionsGraph {
        let mut countries = Countries::with_capacity(size);

        for _ in 0..size {
            countries.push([
                Vec::<IncomingRegion>::new(),
                Vec::<OutgoingRegion>::new(),
            ]);
        }

        RegionsGraph { countries }
    }

    pub fn add(
        &mut self,
        origin: usize,
        destination: usize,
        (weight, start): (usize, usize),
    ) {
        if weight > 0 {
            self.countries[origin][1].push((destination, start, weight));
            self.countries[destination][0].push((origin, start, weight));
        }
    }

    pub fn build_regions_graph(
        &mut self,
        histograms: &Vec<Vec<usize>>,
    ) -> Vec<usize> {
        let global_histogram = aggregate_histograms(&histograms);

        let mut position = 0;
        let mut country = 0;
        let mut country_bound = global_histogram[country];

        histograms.iter().for_each(|histo| {
            histo.iter().enumerate().for_each(|(radix, region_size)| {
                if radix != country {
                    if position + region_size >= country_bound {
                        let region_info = (country_bound - position, position);
                        self.add(country, radix, region_info);
                    } else {
                        self.add(country, radix, (*region_size, position));
                    }
                }

                while position + region_size >= country_bound {
                    let start_bound = country_bound;
                    country += 1;
                    if country >= global_histogram.len() {
                        break;
                    }
                    country_bound += global_histogram[country];

                    if radix != country {
                        if position + region_size >= country_bound {
                            let weight = country_bound - start_bound;
                            let region_info = (weight, start_bound);
                            self.add(country, radix, region_info);
                        } else {
                            let weight = position + region_size - start_bound;
                            let region_info = (weight, start_bound);
                            self.add(country, radix, region_info);
                        }
                    }
                }

                position += region_size;
            })
        });

        global_histogram
    }

    #[inline]
    fn push_link(&mut self, ori: usize, dest: usize, start: usize, l: usize,) {
        // for two path: no need to worry about the ordering
        self.countries[ori][1].push((dest, start, l));
        self.countries[dest][0].push((ori, start, l));
    }

    fn swap_regions(
        &mut self,
        country: usize,
        incoming: &mut Vec<IncomingRegion>,
        outgoing: &mut Vec<OutgoingRegion>,
        incoming_index: usize,
        outgoing_index: usize,
    ) -> (SwapSize, SwapSource, SwapDestination) {
        let (origin, i_start, i_size) = incoming.remove(incoming_index);
        let (destination, o_start, o_size) = outgoing.remove(outgoing_index);

        let p1 = self.countries[origin][1]
            .iter()
            .position(|&elt| elt.1 == i_start)
            .expect("[Regions graph -> swap regions] Bad implementation: p1.");
        let p2 = self.countries[destination][0]
            .iter()
            .position(|&elt| elt.1 == o_start)
            .expect("[Regions graph -> swap regions] Bad implementation: p2.");

        let len = if i_size == o_size {
            self.countries[origin][1].remove(p1);
            self.countries[destination][0].remove(p2);
            if origin != destination {
                self.push_link(origin, destination, i_start, o_size);
            }
            i_size
        } else if i_size > o_size {
            let new_start = i_start + o_size;
            let new_size = i_size - o_size;
            self.countries[destination][0].remove(p2);
            self.countries[origin][1][p1] = (country, new_start, new_size);
            if origin != destination {
                self.push_link(origin, destination, i_start, o_size);
            }
            incoming.insert(incoming_index, (origin, new_start, new_size));
            o_size
        } else { // i_size < o_size
            let new_start = o_start + i_size;
            let new_size = o_size - i_size;
            self.countries[origin][1].remove(p1);
            self.countries[destination][0][p2] = (country, new_start, new_size);
            if origin != destination {
                self.push_link(origin, destination, i_start, i_size);
            }
            outgoing.insert(outgoing_index, (destination, new_start, new_size));
            i_size
        };

        (len, o_start, i_start)
    }

    pub fn two_cycle(
        &mut self,
        country: usize,
    ) -> Vec<(usize, usize, usize)> {
        if self.countries[country][0].len() == 0
            && self.countries[country][1].len() == 0 {
            return Vec::new();
        }

        let mut i = 0;
        let mut j = 0;

        let mut incoming = self.countries[country][0].clone();
        let mut outgoing = self.countries[country][1].clone();

        incoming.sort_unstable_by_key(|(country, _, _)| *country);
        outgoing.sort_unstable_by_key(|(country, _, _)| *country);

        let mut swaps = Vec::new();

        loop {
            if incoming[i].0 == outgoing[j].0 {
                // if same country
                let (size, s, b) = self.swap_regions(
                    country,
                    &mut incoming,
                    &mut outgoing,
                    i,
                    j,
                );
                swaps.push((size, s, b));
            } else if incoming[i].0 < outgoing[j].0 {
                i += 1;
            } else { // incoming[i].0 > outgoing[j].0
                j += 1;
            }

            if i >= incoming.len() || j >= outgoing.len() {
                break;
            }
        }

        self.countries[country][0] = incoming;
        self.countries[country][1] = outgoing;

        swaps
    }

    pub fn two_path(
        &mut self,
        country: usize,
    ) -> Vec<(usize, usize, usize)> {
        if self.countries[country][0].len() == 0
            && self.countries[country][1].len() == 0 {
            return Vec::new();
        }

        let mut incoming = self.countries[country][0].clone();
        let mut outgoing = self.countries[country][1].clone();

        let mut i_len = incoming.len();
        let mut o_len = outgoing.len();

        let mut swaps = Vec::new();

        while i_len > 0 || o_len > 0 {
            let (size, s, b) =
                self.swap_regions(country, &mut incoming, &mut outgoing, 0, 0);

            swaps.push((size, s, b));

            i_len = incoming.len();
            o_len = outgoing.len();
            assert!(!(i_len == 0 && o_len > 0) && !(i_len > 0 && o_len == 0));
        }

        self.countries[country][0] = incoming;
        self.countries[country][1] = outgoing;

        swaps
    }
}

impl PartialEq for RegionsGraph {
    fn eq(&self, other: &RegionsGraph) -> bool {
        self.countries == other.countries
    }
}

impl Eq for RegionsGraph {}

pub fn local_sorting<T, K>(
    arr: &mut [T],
    p: &Params,
    block_size: usize,
    pool: &rayon::ThreadPool,
    _thread_n: usize,
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
        while rest.len() > 0 {
            let (mut fst, snd) = if block_size < rest.len() {
                rest.split_at_mut(block_size)
            } else {
                (rest, &mut [] as &mut [T])
            };
            rest = snd;

            let (sender, receiver) = channel();
            receivers.push(receiver);
            s.spawn(move|_| {
                let h = if fst.len() >= 200_000 {
                    get_histogram_mt(fst, p, mask, shift, pool, 8)
                } else {
                    get_histogram(fst, p, mask, shift)
                };
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

fn regions_sort_rec<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: Params,
    block_size: usize,
    pool: &rayon::ThreadPool,
    thread_n: usize,
    pass: bool,
) {
    if arr.len() <= 30_000 {
        // lsd_diversion(arr, p);
        let diff_level = p.max_level - p.level;
        if 1 < diff_level && diff_level < 5 && arr.len() >= 3_000 {
            let new_offset = ((p.offset / p.radix) * p.radix) + (p.level * p.radix);
            lsd_radixsort_body(arr, Params::new(0, p.radix, new_offset, p.max_level - p.level));
        } else {
            msd_radixsort_rec(arr, p);
        }
        // msd_radixsort_rec(arr, p);
        return;
    }

    let start = Instant::now();
    // Local Sorting Phase for each block
    let histograms = local_sorting(arr, &p, block_size, pool, thread_n);
    if pass {
        println!("local sorting time: {}us", start.elapsed().as_micros() as u64);
    }

    let start = Instant::now();
    // Graph Construction Phase
    let mut regions_graph = RegionsGraph::new(p.radix_range);
    let global_histogram = regions_graph.build_regions_graph(&histograms);

    // let sorted_countries = sort_countries(&global_histogram);
    let (p_sums, _, _) = prefix_sums(&global_histogram);
    if pass {
        println!("middle: {}us", start.elapsed().as_micros() as u64);
    }


    let start = Instant::now();
    let mut parts = Vec::new();
    // Global Sorting Phase
    let mut rest = arr;
    // let mut offset = 0;
    // pool.scope(|s| {
    for country_id in 0..p.radix_range {
        // let swaps = regions_graph.two_cycle(country);
        // perform_swaps(rest, swaps, offset);
        // let swaps = regions_graph.two_path(country);
        // perform_swaps(rest, swaps, offset);
        let end = p_sums[country_id + 1] - p_sums[country_id];
        let (part, snd) = rest.split_at_mut(end);
        parts.push((country_id, part, p_sums[country_id]));
        rest = snd;
        // offset += end;
    }
    if pass {
        println!("aggregate: {}us", start.elapsed().as_micros() as u64);
    }

    let start = Instant::now();
    pool.scope(|s| {
        parts.sort_unstable_by(|(_, a, _), (_, b, _)| {
            a.len().partial_cmp(&b.len()).unwrap()
        });
        let mut country_map = vec![0; parts.len()];
        parts.iter().enumerate().for_each(|(i, &(country_id, _, _))| {
            country_map[country_id] = i;
        });
        // parts.reverse();
        while let Some((country_id, part, country_offset)) = parts.pop() {
        // for part in parts.into_iter() {
            let swaps = regions_graph.two_cycle(country_id);
            perform_swaps(rest, swaps, 0);
            let swaps = regions_graph.two_path(country_id);
            perform_swaps(rest, swaps, 0);

            if p.level < p.max_level - 1 && part.len() > 1 {
                s.spawn(move|_| {
                    let p2 = p.new_level(p.level + 1);
                    regions_sort_rec(part, p2, block_size, pool, thread_n, false);
                });
            }
        }
    });
    if pass {
        println!("rec: {}us", start.elapsed().as_micros() as u64);
    }
}

pub fn regions_sort<T, K>(arr: &mut [T], radix: usize, block_size: usize, thread_n: usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    let size = arr.len();
    if size <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let dummy = arr[0];
    let (offset, _) = dummy.compute_offset(arr, radix);
    let max_level = dummy.compute_max_level(offset, radix);

    if max_level == 0 {
        return;
    }

    let params = Params::new(0, radix, offset, max_level);

    if size <= 30_000 {
        msd_radixsort_rec(arr, params);
    } else {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(thread_n)
            .build()
            .unwrap();

        let mut separators = verge_sort_preprocessing(arr, radix, &|arr, _radix| {
            regions_sort_rec(arr, params, block_size, &pool, thread_n, false)
        });
        k_way_merge_mt_with_buffer(arr, &mut separators, thread_n);
    }
}
