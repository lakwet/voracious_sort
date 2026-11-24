"""Summarises the files of `make bench-type-placed` (one type, the same
source in several code placements): per cell (size, distribution) the range
of v2 and of each rival over the placements, and the cells won, equal or
lost against the best rival in every placement.

    python tools/bench_placements.py <csv> <csv> [<csv> ...] [--band 0.03] [--floor 2]

A cell is won when v2 beats the best rival beyond the band in every
placement, lost when it loses in every placement, equal when every
placement is within the band or under the floor (ns per sort), and
undecided otherwise: the placement alone decides it.
"""

import argparse
import csv


def load(path):
    cols, rows = None, {}
    for r in csv.reader(open(path, encoding="utf-8")):
        if r and r[0] == "cols":
            cols = r[4:]
        elif r and r[0] == "row" and cols:
            rows[(int(r[2]), r[3])] = dict(zip(cols, map(float, r[4:])))
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("files", nargs="+")
    ap.add_argument("--band", type=float, default=0.03)
    ap.add_argument("--floor", type=float, default=2.0, help="ns per sort")
    args = ap.parse_args()
    runs = [load(f) for f in args.files]
    cells = sorted(set.intersection(*(set(r) for r in runs)))
    hi, lo = 1 + args.band, 1 / (1 + args.band)
    verdicts = {"won": [], "equal": [], "lost": [], "undecided": []}
    spread = []
    for k in cells:
        size = k[0]
        marks = []
        for r in runs:
            v = r[k]
            rival = min(x for c, x in v.items() if c not in ("v2", "v1"))
            ratio = rival / max(v["v2"], 1e-9)
            if abs(rival - v["v2"]) * size < args.floor or lo <= ratio <= hi:
                marks.append("equal")
            else:
                marks.append("won" if ratio > hi else "lost")
        verdict = marks[0] if len(set(marks)) == 1 else "undecided"
        verdicts[verdict].append((k, marks))
        v2 = [r[k]["v2"] for r in runs]
        if min(v2) > 0 and max(v2) / min(v2) > hi:
            spread.append((k, min(v2), max(v2)))
    print(f"{len(runs)} placements, {len(cells)} cells")
    print("won {} / equal {} / lost {} / undecided {}".format(
        *(len(verdicts[x]) for x in ("won", "equal", "lost", "undecided"))))
    print(f"\nv2 spread beyond {args.band:.0%} over the placements: {len(spread)} cells")
    for k, a, b in sorted(spread, key=lambda m: -m[2] / m[1])[:20]:
        print(f"  {k[0]:>9} {k[1]:12} {a:.3f} .. {b:.3f} ({b / a:.2f})")
    for name in ("lost", "undecided"):
        print(f"\n{name}:")
        for k, marks in verdicts[name]:
            print(f"  {k[0]:>9} {k[1]:12} " + " ".join(m[0] for m in marks))


if __name__ == "__main__":
    main()
