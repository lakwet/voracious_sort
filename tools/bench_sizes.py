"""Every sort side by side, per size: geometric mean of the ns per key over
the distributions, and the distributions where each sort is the fastest.

Usage:
    python tools/bench_sizes.py <files or folders>... [--label TEXT]

--label: only the labels containing TEXT (=TEXT: that label only).
"""

import argparse
import math
import os
import sys
from collections import defaultdict

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bench_report import files_of, parse, geo  # noqa: E402


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('paths', nargs='+')
    ap.add_argument('--label')
    a = ap.parse_args()
    for machine, path in files_of(a.paths):
        cells, cols, fails = parse(path)
        labels = defaultdict(lambda: defaultdict(list))
        for label, size, gen, values in cells:
            if a.label and (label != a.label[1:] if a.label.startswith('=')
                            else a.label not in label):
                continue
            labels[label][size].append(values)
        for label, sizes in labels.items():
            names = cols[label]
            print(f'=== {machine} {label}' + (f' ({fails} PANICKED / CHECK FAILED)' if fails else ''))
            print(f'{"size":>6}' + ''.join(f'{n:>12}' for n in names)
                  + '   fastest: ' + ' / '.join(names))
            for size in sorted(sizes):
                rows = [v for v in sizes[size] if all(n in v for n in names)]
                g = [geo([v[n] for v in rows]) for n in names]
                wins = [sum(min(v, key=v.get) == n for v in rows) for n in names]
                print(f'{size:>6}' + ''.join(f'{x:>12.3f}' for x in g)
                      + '   ' + ' / '.join(str(w) for w in wins) + f'  (of {len(rows)})')
            print()


if __name__ == '__main__':
    main()
