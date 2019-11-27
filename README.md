# Voracious sort

Welcome on our GitHub dear visitor.

## Introduction

This project's purpose is to have a Rust implementation of a **state
of the art radix sort**.

We started with a single thread radix sort, but the goal is also to have
a multithread radix sort.

This crate should be easy to use and the sort should be able to sort almost
"everything". Radix sort is criticized because people think it can only sort
unsigned integers. This project proves this wrong, **Voracious sort can sort all
Rust primitive types** (except Array for now) **and custom struct**. **It is way
faster than Rust standard sort and Rust unstable sort** on most of the types and
data distribution.

You will find here:
- Version
- License
- A word about the sort’s name
- The **documentation** on how to use this sort,
- Radix sort: basic notions,
- **Performance** analysis,
- For developers and researchers
- **References** we used for this project,
- Futur work,

## Version

Last version tested/used:
- Rustc: 1.38.0 stable
- Rustfmt: 1.4.4 stable
- Cargo: 1.38.0 stable
- Clippy: 0.0.212

## License

This project is under the **MIT** license.

See the license file.

## A word about the sort's name

When I asked what name to give to this sort, a colleague of mine proposed
Voracious sort (_Variation Of Radixsort Axelle Cleverly Imagined Over Uncountable
Sessions_). The name was fun so I took it. It is true that I spent uncountable
sessions to code this project.

You can also think about it like someone very voracious of radish...

## Documentation: How to use it ?

Since it is alreay explained in the crate documentation, we just provide the link:
- [Voracious sort Rust Doc](./blob/master/src/lib.rs) <!-- TODO: replace this link by the real link -->

And we assume that you know how to code in Rust...

Other implementation examples:
- [tuple](./blob/master/src/types/tuple.rs) <!-- TODO: replace this link by the real link -->
- [struct](./blob/master/src/types/custom.rs) <!-- TODO: replace this link by the real link -->

Don't forget to visit other `types` to see how the trait is implemented for each
of them.

## Radix sort: basic notions

Radix sort is a **non-comparative** sort. It requires a key and **uses the binary
representation** to sort the input instead of a comparative function. Amongst
radix sorts there are two sub groups:

- LSD: **L**east **S**ignificant **D**igit; (or LSB: **L**east **S**ignificant **B**it)
- MSD: **M**ost **S**ignificant **D**igit; (or MSB: **M**ost **S**ignificant **B**it)

Their differences are based on how the algorithm iterates through the binary
representation.

Sorts can be **stable** or **unstable**; in general, LSD are stable whereas MSD
tend to be unstable. Please note that the term stable can be ambiguous in the
computer science field, we are refering to this one [wikipedia page on sorting algorithm
stability](https://en.wikipedia.org/wiki/Sorting_algorithm#Stability).

Sorts can be **in place** or **out of place**. A sort is said to be **in place**
if it does not require more memory than the initial memory used by the input. A
sort is said **out of place** if it does require more memory (usually, in order
of `O(n)`). LSD sorts are out of place, and MSD sorts can be either one.

**Diversion** is something we do because radix sorts perform badly on small
inputs. In order to avoid this loss of performance, a radix sort fallbacks on
another sort when the input becomes too small. Insertion sort is very good for
input's length less or equal to 32 elements. Diversion can occur at the
beginning of the algorithm, we check the input's length and we choose the sort
accordingly, or we trigger it at the end of recursive calls when the remaining
input is small enough.

The **radix** itself, is the number of bits the sorting algorithm take into
account per pass. The radix of choice is often 8, however, it may vary depending
on your use case.

In a radix sort, you use an **histogram** which has a size of `2.pow(radix)`.
Given a **level**<sup>[1]</sup> and a radix, we can compute the histogram. This
histogram give all the bucket sizes and thus, the algorithm know where to move
each elements. For a detail example, I let the reader read the Wikipedia page on
[radix sort](https://en.wikipedia.org/wiki/Radix_sort).

<sup>[1]</sup>: If a radix of 8 is choosen, level 1 is the first 8 bits, level
2 is the next 8 following bits, and so on until there is no more bit to handle.

## Performances analysis

Coming soon !

## For developers and researchers

- Voracious sort is a MSD radix sort. It is an improvement of the
[Ska sort](https://probablydance.com/2016/12/27/i-wrote-a-faster-sorting-algorithm/)
and it uses the [Verge sort pre-processing heuristic](https://github.com/Morwenn/vergesort). Depending on the type and the input size, another sort might be choosen (LSD sort, Counting sort, etc...).
The purpose is to implement a multithread radix sort (see
[Regions sort](https://github.com/omarobeya/parallel-inplace-radixsort) and
the [article](https://people.csail.mit.edu/jshun/RegionsSort.pdf)).

- DLSD (Diverting LSD radix sort) is a simpler version of the
[DFR sort](https://github.com/ramou/dfr) with a different diversion and
a variable radix (see [article](https://users.encs.concordia.ca/~sthiel/DS/SEA2015_FastRadix.pdf)).

- All sorts fallback on the
[PDQ sort](https://github.com/stjepang/pdqsort)
(Rust Unstable sort) for very small inputs.

- For now, both Voracious sort and DLSD sort have generic code. But I notice
that dedicated implementation per type is faster (~10-15%). For maintainability
reason, this Crate has an as generic code as possible. For research article,
dedicated implementation per type is be used.

## References

- [Ska sort](https://probablydance.com/2016/12/27/i-wrote-a-faster-sorting-algorithm/)
- [Verge sort pre-processing heuristic](https://github.com/Morwenn/vergesort)
- [Regions sort](https://github.com/omarobeya/parallel-inplace-radixsort): [Theoretically-Efficient and Practical Parallel In-Place Radix Sorting](https://people.csail.mit.edu/jshun/RegionsSort.pdf)
- [DFR sort](https://github.com/ramou/dfr)
- [Fast radix sort](https://github.com/AwardOfSky/Fast-Radix-Sort): [Relaxing the Counting Requirement for Least Significant Digit Radix Sorts](https://users.encs.concordia.ca/~sthiel/DS/SEA2015_FastRadix.pdf)
- [PDQ sort](https://github.com/stjepang/pdqsort)


## Futur work

- Add multithread sort.
- Improve k-way-merge algorithm.
- Use more statistical hypothesis.
- Finish 2-arity tuple implementation.
- Add more generators (for tests).
- Replace the MSD sort for string by a Burstsort or Spreadsort implementation
or something else.
