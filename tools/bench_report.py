"""Summary of bench-cpu result files: where the reference sort (the first
column: the v2; the v1 for bool arrays) loses, by how much, and the
geometric mean and median of the ratios against each other sort.

Usage:
    python tools/bench_report.py <files or folders>... [options]

Folders are searched for bench-cpu-*.txt; the machine of a file is the
name of the first folder under the given one (e.g. ryzen/u8/...), else the
host in the file name.

Options:
    --min-loss X   a loss counts from this fraction (default 0.03: 3 %,
                   below that it is noise)
    --top N        biggest losses listed per machine (default 30)
    --ref NAME     reference column (default: the first one), e.g.
                   "v2 unstable" for the records
    --vs NAMES     only these sorts (comma-separated), e.g. std,v1
    --label TEXT   only the labels containing TEXT (e.g. "records u64")
    --sizes LIST   only these sizes (comma-separated)
    --gen TEXT     only the distributions containing TEXT
    --brief        the summary tables only (per sort, vs best, machines)
    --csv FILE     every loss written to FILE (machine, label, size,
                   distribution, sort, reference ns, sort ns, factor)

Ratios: other / reference (> 1: the reference is faster). A loss: the
other sort faster by at least --min-loss; its factor is reference / other
(1.25: the reference takes 25 % more time). "mean loss": geometric mean of
the loss factors. "vs best": per cell, the fastest of the other sorts,
leaving out the other variants of the reference (v2 unstable against the
v2, v1 stable against the v1): the v1 counts for the v2, as a regression.
"""

import argparse
import csv
import math
import os
import re
import statistics
import sys
from collections import defaultdict


def files_of(paths):
    """(machine, path) of every result file."""
    out = []
    for p in paths:
        if os.path.isfile(p):
            out.append((host_of(p), p))
            continue
        for root, _, names in os.walk(p):
            for n in sorted(names):
                if n.startswith('bench-cpu-') and n.endswith('.txt'):
                    full = os.path.join(root, n)
                    rel = os.path.relpath(full, p).split(os.sep)
                    machine = rel[0] if len(rel) > 2 else host_of(full)
                    out.append((machine, full))
    return out


def host_of(path):
    m = re.match(r'bench-cpu-(.+)-\d{4}-\d\d-\d\d', os.path.basename(path))
    return m.group(1) if m else os.path.basename(path)


def parse(path):
    """Cells: (label, size, distribution, {sort: ns}), the columns per
    label, and the count of failed checks / panics."""
    cols, cells, failures = {}, [], 0
    with open(path, encoding='utf-8', errors='replace') as f:
        for line in f:
            if 'PANICKED' in line or 'CHECK FAILED' in line:
                failures += 1
            parts = line.rstrip('\n').split(',')
            if parts[0] == 'cols':
                cols[parts[1]] = parts[4:]
            elif parts[0] == 'row' and parts[1] in cols:
                label, size, gen = parts[1], int(parts[2]), parts[3]
                values = {}
                for name, x in zip(cols[parts[1]], parts[4:]):
                    try:
                        v = float(x)
                    except ValueError:
                        continue
                    if v > 0 and not math.isnan(v):
                        values[name] = v
                cells.append((label, size, gen, values))
    return cells, cols, failures


def geo(xs):
    return math.exp(sum(math.log(x) for x in xs) / len(xs)) if xs else float('nan')


def variant_of(name, ref):
    """Another variant of the reference (same family, v1 or v2)."""
    return name.split(' ')[0] == ref.split(' ')[0]


class Stats:
    """Ratios other / reference of a group of cells, and their losses."""

    def __init__(self):
        self.ratios = []

    def add(self, r):
        self.ratios.append(r)

    def row(self, min_loss):
        rs = self.ratios
        losses = [1 / r for r in rs if 1 / r >= 1 + min_loss]
        return (f'{len(rs):>7}{geo(rs):>7.2f}{statistics.median(rs):>8.2f}'
                f'{len(losses):>7}{(geo(losses) if losses else 0):>7.2f}'
                f'{(max(losses) if losses else 0):>7.2f}')


HEAD = f'{"cells":>7}{"geo":>7}{"median":>8}{"lost":>7}{"mean":>7}{"worst":>7}'


def main():
    ap = argparse.ArgumentParser(description=__doc__,
                                 formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('paths', nargs='+')
    ap.add_argument('--min-loss', type=float, default=0.03)
    ap.add_argument('--top', type=int, default=30)
    ap.add_argument('--ref')
    ap.add_argument('--vs')
    ap.add_argument('--label')
    ap.add_argument('--sizes')
    ap.add_argument('--gen')
    ap.add_argument('--brief', action='store_true')
    ap.add_argument('--csv')
    a = ap.parse_args()
    only_vs = set(a.vs.split(',')) if a.vs else None
    only_sizes = {int(x) for x in a.sizes.split(',')} if a.sizes else None

    machines = defaultdict(list)
    failures = defaultdict(int)
    for machine, path in files_of(a.paths):
        cells, cols, fails = parse(path)
        failures[machine] += fails
        for label, size, gen, values in cells:
            if a.label and a.label not in label:
                continue
            if only_sizes and size not in only_sizes:
                continue
            if a.gen and a.gen not in gen:
                continue
            ref = a.ref if a.ref else cols[label][0]
            if ref not in values:
                continue
            machines[machine].append((label, size, gen, ref, values))
    if not machines:
        sys.exit('no result found')

    writer = None
    if a.csv:
        fcsv = open(a.csv, 'w', newline='', encoding='utf-8')
        writer = csv.writer(fcsv)
        writer.writerow(['machine', 'label', 'size', 'distribution', 'sort', 'reference',
                         'ref_ns', 'sort_ns', 'factor'])

    overall = {}
    for machine in sorted(machines):
        cells = machines[machine]
        per_sort = defaultdict(Stats)
        best_all = Stats()
        per_label = defaultdict(Stats)
        per_size = defaultdict(Stats)
        per_gen = defaultdict(Stats)
        losses = []
        best_losses = []
        for label, size, gen, ref, values in cells:
            r_ns = values[ref]
            others = {k: v for k, v in values.items()
                      if k != ref and (not only_vs or k in only_vs)}
            for name, ns in others.items():
                per_sort[name].add(ns / r_ns)
                if r_ns / ns >= 1 + a.min_loss:
                    losses.append((r_ns / ns, label, size, gen, name, r_ns, ns, ref))
            rivals = {k: v for k, v in others.items() if not variant_of(k, ref)}
            if rivals:
                name, ns = min(rivals.items(), key=lambda kv: kv[1])
                r = ns / r_ns
                for s in (best_all, per_label[label], per_size[size], per_gen[gen]):
                    s.add(r)
                if 1 / r >= 1 + a.min_loss:
                    best_losses.append((1 / r, label, size, gen, name, r_ns, ns, ref))
        overall[machine] = best_all

        nfail = failures[machine]
        print(f'=== {machine}: {len(cells)} cells, {len({c[0] for c in cells})} labels'
              + (f', {nfail} PANICKED / CHECK FAILED lines' if nfail else ''))
        print(f'{"vs":<16}{HEAD}')
        for name in sorted(per_sort, key=lambda n: -len(per_sort[n].ratios)):
            print(f'{name:<16}{per_sort[name].row(a.min_loss)}')
        if best_all.ratios:
            print(f'{"vs best rival":<16}{best_all.row(a.min_loss)}')
        if a.brief:
            print()
            continue

        for title, groups, key in (('size', per_size, lambda k: k),
                                   ('label', per_label, lambda k: k)):
            print(f'\nvs best rival, per {title}')
            print(f'{title:<22}{HEAD}')
            for k in sorted(groups, key=key):
                print(f'{str(k):<22}{groups[k].row(a.min_loss)}')

        lost_gens = [(g, s) for g, s in per_gen.items()
                     if any(1 / r >= 1 + a.min_loss for r in s.ratios)]
        lost_gens.sort(key=lambda gs: -sum(1 / r >= 1 + a.min_loss for r in gs[1].ratios))
        print('\nvs best rival, distributions with losses (most first)')
        print(f'{"distribution":<30}{HEAD}')
        for g, s in lost_gens[:25]:
            print(f'{g[:29]:<30}{s.row(a.min_loss)}')

        best_losses.sort(reverse=True)
        print(f'\nbiggest losses against the best rival '
              f'(factor = reference / rival, from {1 + a.min_loss:.2f}):')
        for f, label, size, gen, name, rns, ons, ref in best_losses[: a.top]:
            print(f'  {f:5.2f}  {label:<20}{size:>9}  {gen[:28]:<28} '
                  f'{ref} {rns:.3f} vs {name} {ons:.3f}')
        print()
        if writer:
            for f, label, size, gen, name, rns, ons, ref in sorted(losses, reverse=True):
                writer.writerow([machine, label, size, gen, name, ref, rns, ons, round(f, 4)])

    if len(overall) > 1:
        print('=== machines, vs best rival')
        print(f'{"machine":<16}{HEAD}')
        for machine, s in sorted(overall.items()):
            print(f'{machine:<16}{s.row(a.min_loss)}')
    if writer:
        fcsv.close()


if __name__ == '__main__':
    main()
