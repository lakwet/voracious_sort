use super::super::sorts::utils_mt::aggregate_histograms;

pub type CountryId = usize;
pub type RegionSize = usize;
pub type RegionStart = usize;

pub type RegionInfo = (CountryId, RegionStart, RegionSize);
pub type IncomingRegion = RegionInfo;
pub type OutgoingRegion = RegionInfo;

pub type Country = [Vec<RegionInfo>; 2];
pub type Countries = Vec<Country>;

pub type SwapSize = usize;
pub type SwapSource = (usize, usize);
pub type SwapDestination = (usize, usize);

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
        histograms: &[Vec<usize>],
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
    fn push_link(&mut self, ori: usize, dest: usize, start: usize, l: usize) {
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
        } else {
            // i_size < o_size
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

        (len, (destination, o_start), (origin, i_start))
    }

    pub fn two_cycle(
        &mut self,
        country: usize,
    ) -> Vec<(usize, (usize, usize), (usize, usize))> {
        if self.countries[country][0].is_empty()
            && self.countries[country][1].is_empty()
        {
            return Vec::new();
        }

        let mut i = 0;
        let mut j = 0;

        let mut incoming = self.countries[country][0].clone();
        let mut outgoing = self.countries[country][1].clone();

        incoming.sort_by_key(|(country, _, _)| *country);
        outgoing.sort_by_key(|(country, _, _)| *country);

        let mut swaps = Vec::new();

        loop {
            if incoming[i].0 == outgoing[j].0 {
                // if same country
                let (size, (cs, s), (cb, b)) = self.swap_regions(
                    country,
                    &mut incoming,
                    &mut outgoing,
                    i,
                    j,
                );
                swaps.push((size, (cs, s), (cb, b)));
            } else if incoming[i].0 < outgoing[j].0 {
                i += 1;
            } else {
                // incoming[i].0 > outgoing[j].0
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
    ) -> Vec<(usize, (usize, usize), (usize, usize))> {
        if self.countries[country][0].is_empty()
            && self.countries[country][1].is_empty()
        {
            return Vec::new();
        }

        let mut incoming = self.countries[country][0].clone();
        let mut outgoing = self.countries[country][1].clone();

        let mut i_len = incoming.len();
        let mut o_len = outgoing.len();

        let mut swaps = Vec::new();

        while i_len > 0 || o_len > 0 {
            let (size, (cs, s), (cb, b)) =
                self.swap_regions(country, &mut incoming, &mut outgoing, 0, 0);

            swaps.push((size, (cs, s), (cb, b)));

            i_len = incoming.len();
            o_len = outgoing.len();
            // assert!(!(i_len == 0 && o_len > 0) && !(i_len > 0 && o_len == 0));
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

fn swap_between_countries<T: Copy>(
    country_ori: &mut [T],
    (_, country_dest, g_offset_dest): &mut (usize, &mut [T], usize),
    (len, ori_offset, dest_offset): (usize, usize, usize),
    g_offset_ori: usize,
) {
    unsafe {
        let ori: *mut T =
            country_ori.get_unchecked_mut(ori_offset - g_offset_ori);
        let dest: *mut T =
            country_dest.get_unchecked_mut(dest_offset - *g_offset_dest);

        std::ptr::swap_nonoverlapping(ori, dest, len);
    }
}

pub fn swap_countries<T: Copy>(
    swaps: Vec<(usize, (usize, usize), (usize, usize))>,
    broker: &mut [T],
    countries: &mut Vec<(usize, &mut [T], usize)>,
    country_map: &[usize],
    bro_offset: usize,
) {
    for (len, (_, swap_index_bro), (inc_id, swap_index_dest)) in swaps.iter() {
        let pos_in_countries = country_map[*inc_id];
        swap_between_countries(
            broker,
            &mut countries[pos_in_countries],
            (*len, *swap_index_bro, *swap_index_dest),
            bro_offset,
        );
    }
}
