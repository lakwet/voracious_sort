# Profiling

What is "profiling" ? Unlike the comparative sorts (most of them), radix sorts
might need constants. Because an universal perfect sort does not exist, I chose
to use the best sort I have with respect to the given input.

So we have to take into account the type and the size of the input and the
radix sort algorithm, this latter might needs differents constants.

For example (because it is always clearer with an example):

A snippet of the implementation of the `Radixable` trait for the `f64` type.

```Rust
fn voracious_sort(&self, arr: &mut [f64]) {
    if arr.len() <= 300 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap())
    } else if arr.len() < 800 {
        dlsd_radixsort(arr, 8);
    } else {
        rollercoaster_sort(arr, 8);
    }
}
fn voracious_mt_sort(&self, arr: &mut [Self], thread_n: usize) {
    if arr.len() < 800_000 {
        arr.par_sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
    } else {
        let chunk_size = if arr.len() < 1_000_000 {
            100_000
        } else if arr.len() < 5_000_000 {
            200_000
        } else if arr.len() < 20_000_000 {
            500_000
        } else if arr.len() < 500_000_000 {
            400_000
        } else {
            500_000
        };
        peeka_sort(arr, 8, chunk_size, thread_n);
    }
}
```

As you can see, depending on the input size and the type, I will choose to use
a sort or another. The chosen sort might need differents constants such as the
`radix` size or the `chunk_size` or the number of `thread`.

Doing this takes a lot of time and is valid only for the computer on which I am
doing the profiling.

The default profiling in the `voracious_radix_sort` crate is done on an AMD Ryzen
9 3950x, 32GB RAM DDR4, MB X570 TUF Gaming.

I will share here, what is done, and what is not done yet. But don't forget that
this profiling is valid for my computer, but a better profiling can be found for
your computer.

Since doing this takes a lot of time, I will do it one by one.

If your use case is not done yet. There is a "default default" profile, but
clearly it is not optimized.

If you are nice, you can do a PR 🙏

# Profiling table

| Ryzen 9 3950x | voracious_sort | voracious_stable_sort | voracious_mt_sort |
|:-:|:-:|:-:|:-:|
| bool | ✓ | ✓ | ✓ |
| char | ✓ | ✓ | ✓ |
| f32 | ✓ | ✓ | ✓ |
| f64 | ✓ | ✓ | ✓ |
| u8 | ✓ | ✓ | ✓ |
| u16 | ✗ | ✗ | ✗ |
| u32 | ✓ | ✓ | ✓ |
| u64 | ✓ | ✓ | ✓ |
| u128 | ✗ | ✗ | ✗ |
| usize8 | ✓ | ✓ | ✓ |
| usize16 | ✗ | ✗ | ✗ |
| usize32 | ✓ | ✓ | ✓ |
| usize64 | ✓ | ✓ | ✓ |
| usize128 | ✗ | ✗ | ✗ |
| i8 | ✗ | ✗ | ✗ |
| i16 | ✗ | ✗ | ✗ |
| i32 | ✓ | ✓ | ✓ |
| i64 | ✓ | ✓ | ✓ |
| i128 | ✗ | ✗ | ✗ |
| isize8 | ✗ | ✗ | ✗ |
| isize16 | ✗ | ✗ | ✗ |
| isize32 | ✓ | ✓ | ✓ |
| isize64 | ✓ | ✓ | ✓ |
| isize128 | ✗ | ✗ | ✗ |
| struct (bool) | ✓ | ✓ | ✓ |
| struct (char) | ✓ | ✓ | ✓ |
| struct (f32) | ✓ | ✓ | ✓ |
| struct (f64) | ✓ | ✓ | ✓ |
| struct (u8) | ✗ | ✗ | ✗ |
| struct (u16) | ✗ | ✗ | ✗ |
| struct (u32) | ✗ | ✗ | ✗ |
| struct (u64) | ✗ | ✗ | ✗ |
| struct (u128) | ✗ | ✗ | ✗ |
| struct (usize) | ✗ | ✗ | ✗ |
| struct (i8) | ✗ | ✗ | ✗ |
| struct (i16) | ✗ | ✗ | ✗ |
| struct (i32) | ✗ | ✗ | ✗ |
| struct (i64) | ✗ | ✗ | ✗ |
| struct (i128) | ✗ | ✗ | ✗ |
| struct (isize) | ✗ | ✗ | ✗ |
