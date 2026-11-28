//! The benchmarks (ignored tests, run by `make bench`, `make bench-type`,
//! `make bench-records`, `make bench-big`, `make bench-big-type`).

mod bench_cpu;
mod cold;

// FRESH=1 (make bench-type and the bench-16 runs): every allocation of
// 128 KiB or more gets new pages from the system and returns them on free,
// as in an isolated sort; the system allocator would recycle a sort's
// buffer at some sizes only.
#[path = "../../../examples/common/fresh_pages.rs"] mod fresh_pages;
#[global_allocator]
static ALLOC: fresh_pages::Fresh = fresh_pages::Fresh;
