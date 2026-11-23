"""Compares the builds of `make abench-align`: the same source compiled in
several code placements (loops aligned on 16, 32 or 64 bytes, 1 or 16
codegen units). The placement of the code alone can move a cell by 5-35 %
(Haswell's uop cache, Zen 2), so a variant counts only where it wins in
every placement (Mytkowicz et al., ASPLOS 2009; STABILIZER, Curtsinger and
Berger, ASPLOS 2013: compare over randomized layouts).

    python tools/ab_align.py <log> <log> [<log> ...] [--band 0.03]

Each log is the table printed by examples/ab_u16.rs or ab_i16.rs (size, gen,
one column per variant, blazing, copy). The first column is the reference.
Prints the spread of the reference and of blazing over the placements, then
per variant the cells won or lost in every placement, and the cells where
the placements disagree.
"""

import argparse


def load(path):
    head, rows = None, {}
    for line in open(path, encoding="utf-8", errors="replace"):
        cols = line.split()
        if len(cols) >= 4 and cols[0] == "size" and cols[1] == "gen":
            head = cols[2:]
        elif head and len(cols) == len(head) + 2 and cols[0].isdigit():
            rows[(int(cols[0]), cols[1])] = dict(zip(head, map(float, cols[2:])))
    return head, rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("logs", nargs="+")
    ap.add_argument("--band", type=float, default=0.03)
    args = ap.parse_args()
    loaded = [load(p) for p in args.logs]
    head = loaded[0][0]
    runs = [r for _, r in loaded]
    ref = head[0]
    variants = [h for h in head[1:] if h != "copy"]
    cells = sorted(set.intersection(*(set(r) for r in runs)))
    hi, lo = 1 + args.band, 1 / (1 + args.band)
    print(f"{len(runs)} placements, {len(cells)} cells")

    for col in (ref, "blazing"):
        if col not in head:
            continue
        moved = []
        for k in cells:
            vals = [r[k][col] for r in runs]
            if min(vals) > 0 and max(vals) / min(vals) > hi:
                moved.append((k, min(vals), max(vals)))
        print(f"\n{col}: {len(moved)} cells spread beyond {args.band:.0%} over the placements")
        for k, a, b in sorted(moved, key=lambda m: -m[2] / m[1])[:15]:
            print(f"  {k[0]:>9} {k[1]:12} {a:.3f} .. {b:.3f} ({b / a:.2f})")

    for v in variants:
        if v == ref:
            continue
        won, lost, split = [], [], []
        for k in cells:
            rs = [r[k][v] / r[k][ref] for r in runs if r[k][ref] > 0]
            if not rs:
                continue
            if max(rs) < lo:
                won.append((k, rs))
            elif min(rs) > hi:
                lost.append((k, rs))
            elif min(rs) < lo or max(rs) > hi:
                split.append((k, rs))
        print(f"\n{v} / {ref}: won {len(won)}, lost {len(lost)}, placements disagree {len(split)}")
        for name, group in (("won", won), ("lost", lost), ("disagree", split)):
            for k, rs in group:
                print(f"  {name:8} {k[0]:>9} {k[1]:12} " + " ".join(f"{x:.2f}" for x in rs))


if __name__ == "__main__":
    main()
