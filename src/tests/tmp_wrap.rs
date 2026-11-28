use crate::v2::devour::devour_sort2;
use rand::RngExt;

#[test]
fn tmp_wrap_repro() {
    let mut rng = rand::rng();
    for _ in 0..20_000 {
        for &low in [1usize, 50, 99].iter() {
            let arr: Vec<i64> = (0..1000)
                .map(|_| {
                    let d = rng.random_range(0..300) as i64;
                    if rng.random_range(0..100) < low { i64::MIN.wrapping_add(d) } else { i64::MAX.wrapping_sub(d) }
                })
                .collect();
            let mut a = arr.clone();
            let mut e = arr.clone();
            e.sort_unstable();
            devour_sort2(&mut a);
            assert!(a == e);
            let mut u: Vec<u64> = arr.iter().map(|&x| x as u64).collect();
            let mut ue = u.clone();
            ue.sort_unstable();
            devour_sort2(&mut u);
            assert!(u == ue);
        }
    }
    let mut v = vec![i64::MIN, i64::MAX, 0, 5, i64::MIN, i64::MAX];
    v.extend(std::iter::repeat_n(7i64, 994));
    let mut e = v.clone();
    e.sort_unstable();
    devour_sort2(&mut v);
    assert!(v == e);
}
