use super::super::sorts::regions_sort::{RegionsGraph, perform_swaps};

#[test]
fn test_regions_graph() {
    let h1 = vec![3, 5, 4, 4];
    let h2 = vec![2, 1, 6, 8];
    let h3 = vec![2, 9, 0, 3];
    let histograms = vec![h1, h2, h3];

    //               11 1111 11 1 122 222 2222233 3 33 333334444 444
    //  012 3456 7 8901 2345 67 8 901 234 5678901 2 34 567890123 456
    // |                    |                      |                | blocks
    //  000 1111 1 2222 3333 00 1 222 222 3333333 3 00 111111111 333
    // |   |    | |    |    |  | |   |   |       | |  |         |   | regions
    // |        |                    |           |                  | countries

    let mut g = RegionsGraph::new(4);
    g.build_regions_graph(&histograms);

    let mut check = RegionsGraph::new(4);
    check.add(0, 1, (4, 3));
    check.add(1, 2, (4, 8));
    check.add(1, 3, (4, 12));
    check.add(1, 0, (2, 16));
    check.add(1, 2, (3, 19));
    check.add(2, 3, (7, 25));
    check.add(3, 0, (2, 33));
    check.add(3, 1, (9, 35));
    // RegionsGraph { countries: [
    //     ([(1, 16, 2), (3, 33, 2)], [(1, 3, 4)]),
    //     ([(0, 3, 4), (3, 35, 9)], [(2, 8, 4), (3, 12, 4), (0, 16, 2), (2, 19, 3)]),
    //     ([(1, 8, 4), (1, 19, 3)], [(3, 25, 7)]),
    //     ([(1, 12, 4), (2, 25, 7)], [(0, 33, 2), (1, 35, 9)])
    // ] }

    assert_eq!(g, check);
}

#[test]
fn test_regions_graph_bis() {
    let h1 = vec![0, 1, 0, 2];
    let h2 = vec![0, 2, 1, 0];
    let h3 = vec![0, 1, 0, 0];
    let histograms = vec![h1, h2, h3];

    let mut g = RegionsGraph::new(4);
    g.build_regions_graph(&histograms);

    let mut check = RegionsGraph::new(4);
    check.add(1, 3, (2, 1));
    check.add(2, 1, (1, 4));
    check.add(3, 2, (1, 5));
    check.add(3, 1, (1, 6));
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([(2, 4, 1), (3, 6, 1)], [(3, 1, 2)]),
    //     ([(3, 5, 1)], [(1, 4, 1)]),
    //     ([(1, 1, 2)], [(2, 5, 1), (1, 6, 1)])
    // ] }

    assert_eq!(g, check);
}

#[test]
fn test_regions_graph_two_cycle_1() {
    let mut arr = vec![1, 3, 3, 1, 1, 2, 1];
    let h1 = vec![0, 1, 0, 2];
    let h2 = vec![0, 2, 1, 0];
    let h3 = vec![0, 1, 0, 0];
    let histograms = vec![h1, h2, h3];

    let mut g = RegionsGraph::new(4);
    g.build_regions_graph(&histograms);
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([(2, 4, 1), (3, 6, 1)], [(3, 1, 2)]),
    //     ([(3, 5, 1)], [(1, 4, 1)]),
    //     ([(1, 1, 2)], [(2, 5, 1), (1, 6, 1)])
    // ] }

    let swaps = g.two_cycle(1);
    perform_swaps(&mut arr, swaps);

    let mut check = RegionsGraph::new(4);
    check.add(1, 3, (1, 2));
    check.add(2, 1, (1, 4));
    check.add(3, 2, (1, 5));
    // RegionsGraph { countries: [
    //     [[], []],
    //     [[(2, 4, 1)], [(3, 2, 1)]],
    //     [[(3, 5, 1)], [(1, 4, 1)]],
    //     [[(1, 2, 1)], [(2, 5, 1)]],
    // ] }
    let check_arr = vec![1, 1, 3, 1, 1, 2, 3];

    assert_eq!(g, check);
    assert_eq!(arr, check_arr);
}

#[test]
fn test_regions_graph_two_cycle_2() {
    let mut arr = vec![1, 3, 3, 1, 1, 2, 1];
    let h1 = vec![0, 1, 0, 2];
    let h2 = vec![0, 2, 1, 0];
    let h3 = vec![0, 1, 0, 0];
    let histograms = vec![h1, h2, h3];

    let mut g = RegionsGraph::new(4);
    g.build_regions_graph(&histograms);
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([(2, 4, 1), (3, 6, 1)], [(3, 1, 2)]),
    //     ([(3, 5, 1)], [(1, 4, 1)]),
    //     ([(1, 1, 2)], [(2, 5, 1), (1, 6, 1)])
    // ] }

    let swaps = g.two_cycle(3);
    perform_swaps(&mut arr, swaps);

    let mut check = RegionsGraph::new(4);
    check.add(1, 3, (1, 2));
    check.add(2, 1, (1, 4));
    check.add(3, 2, (1, 5));
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([(2, 4, 1)], [(3, 2, 1)]),
    //     ([(3, 5, 1)], [(1, 4, 1)]),
    //     ([(1, 2, 1)], [(2, 5, 1)])
    // ] }
    let check_arr = vec![1, 1, 3, 1, 1, 2, 3];

    assert_eq!(g, check);
    assert_eq!(arr, check_arr);
}

#[test]
fn test_regions_graph_two_cycle_3() {
    let mut arr = vec![1, 1, 3, 1, 1, 2, 1];
    let h1 = vec![0, 2, 0, 1];
    let h2 = vec![0, 2, 1, 0];
    let h3 = vec![0, 1, 0, 0];
    let histograms = vec![h1, h2, h3];

    let mut g = RegionsGraph::new(4);
    g.build_regions_graph(&histograms);
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([(3, 6, 1)], [(3, 2, 1)]),
    //     ([], []),
    //     ([(1, 2, 1)], [(1, 6, 1)])
    // ] }

    let swaps = g.two_cycle(1);
    perform_swaps(&mut arr, swaps);

    let check = RegionsGraph::new(4);
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([], []),
    //     ([], []),
    //     ([], []),
    // ] }
    let check_arr = vec![1, 1, 1, 1, 1, 2, 3];

    assert_eq!(g, check);
    assert_eq!(arr, check_arr);
}

#[test]
fn test_regions_graph_two_path_1() {
    let mut arr = vec![1, 3, 3, 1, 1, 2, 1];
    let h1 = vec![0, 1, 0, 2];
    let h2 = vec![0, 2, 1, 0];
    let h3 = vec![0, 1, 0, 0];
    let histograms = vec![h1, h2, h3];

    let mut g = RegionsGraph::new(4);
    g.build_regions_graph(&histograms);
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([(2, 4, 1), (3, 6, 1)], [(3, 1, 2)]),
    //     ([(3, 5, 1)], [(1, 4, 1)]),
    //     ([(1, 1, 2)], [(2, 5, 1), (1, 6, 1)])
    // ] }

    let swaps = g.two_path(2);
    perform_swaps(&mut arr, swaps);

    let mut check = RegionsGraph::new(4);
    check.add(1, 3, (2, 1));
    check.add(3, 1, (1, 6));
    check.add(3, 1, (1, 5));
    // RegionsGraph { countries: [
    // [[], []],
    // [[(3, 6, 1), (3, 5, 1)], [(3, 1, 2)]],
    // [[], []],
    // [[(1, 1, 2)], [(1, 6, 1), (1, 5, 1)]]
    // ] }
    let check_arr = vec![1, 3, 3, 1, 2, 1, 1];

    assert_eq!(g, check);
    assert_eq!(arr, check_arr);
}

#[test]
fn test_regions_graph_two_path_2() {
    let mut arr = vec![1, 3, 3, 1, 1, 2, 1];
    let h1 = vec![0, 1, 0, 2];
    let h2 = vec![0, 2, 1, 0];
    let h3 = vec![0, 1, 0, 0];
    let histograms = vec![h1, h2, h3];

    let mut g = RegionsGraph::new(4);
    g.build_regions_graph(&histograms);
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([(2, 4, 1), (3, 6, 1)], [(3, 1, 2)]),
    //     ([(3, 5, 1)], [(1, 4, 1)]),
    //     ([(1, 1, 2)], [(2, 5, 1), (1, 6, 1)])
    // ] }

    let swaps = g.two_path(1);
    perform_swaps(&mut arr, swaps);

    let mut check = RegionsGraph::new(4);
    check.add(2, 3, (1, 4));
    check.add(3, 2, (1, 5));
    // RegionsGraph { countries: [
    //     ([], []),
    //     ([], []),
    //     ([(3, 5, 1)], [(3, 4, 1)]),
    //     ([(2, 4, 1)], [(2, 5, 1)])
    // ] }
    let check_arr = vec![1, 1, 1, 1, 3, 2, 3];

    assert_eq!(g, check);
    assert_eq!(arr, check_arr);
}
