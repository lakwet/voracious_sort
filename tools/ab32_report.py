"""Summary of an ab_u32 run (columns v2, lsd, std, blz, copy) and its trace.

Usage:
    python tools/ab32_report.py <eng-T.txt> [<trace-T.txt>] [--noise 0.03]

Per size: cells where the LSD engine alone loses to blazing (the engine's
own gap), where v2 loses to blazing, and the chain's overhead v2 / lsd on
the cells whose path ends in the LSD (the cost of the failed questions),
with the paths of the other cells.
"""
import sys
from collections import defaultdict

args = [a for a in sys.argv[1:] if not a.startswith("--")]
noise = 0.03
if "--noise" in sys.argv:
    noise = float(sys.argv[sys.argv.index("--noise") + 1])

rows = []
cols = None
for line in open(args[0], encoding="utf-8"):
    f = line.split()
    if not f:
        continue
    if f[0] == "size":
        cols = f[2:]
        if cols.count("copy") > 1:
            cols[len(cols) - 1 - cols[::-1].index("copy")] = "memcpy"
        continue
    if cols and f[0].isdigit() and len(f) == len(cols) + 2:
        rows.append((int(f[0]), f[1], dict(zip(cols, map(float, f[2:])))))

paths = {}
if len(args) > 1:
    for line in open(args[1], encoding="utf-8"):
        f = line.split()
        if len(f) >= 2 and f[0].isdigit():
            paths[(int(f[0]), f[1])] = " ".join(f[2:])

by = defaultdict(list)
for r in rows:
    by[r[0]].append(r)


def lost(a, b):
    return a > b * (1 + noise)


print(f"{'size':>8} {'cells':>5} | {'lsd<blz':>7} {'lsd>blz':>7} | {'v2>blz':>6} | "
      f"{'lsd-path':>8} {'ovh med':>7} {'ovh max':>7} | {'v2>lsd':>6}")
for n in sorted(by):
    rs = by[n]
    eng_lost = sum(lost(c["lsd"], c["blz"]) for _, _, c in rs)
    eng_won = sum(lost(c["blz"], c["lsd"]) for _, _, c in rs)
    v2_lost = sum(lost(c["v2"], c["blz"]) for _, _, c in rs)
    ovh = [c["v2"] / c["lsd"] for _, g, c in rs
           if paths.get((n, g), "").endswith("lsd") and c["lsd"] > 0]
    ovh.sort()
    med = ovh[len(ovh) // 2] if ovh else float("nan")
    mx = ovh[-1] if ovh else float("nan")
    worse = sum(lost(c["v2"], c["lsd"]) for _, _, c in rs)
    print(f"{n:>8} {len(rs):>5} | {eng_won:>7} {eng_lost:>7} | {v2_lost:>6} | "
          f"{len(ovh):>8} {med:>7.2f} {mx:>7.2f} | {worse:>6}")

engines = [e for e in ("lsd", "dlsd", "roller", "std") if cols and e in cols]
if len(engines) > 1:
    print()
    print("best engine per cell (our engines and std), and the best against blazing")
    print(f"{'size':>8} " + " ".join(f"{e:>7}" for e in engines) + f" | {'best<blz':>8} {'best>blz':>8}")
    for n in sorted(by):
        wins = defaultdict(int)
        bw = bl = 0
        for _, _, c in by[n]:
            best = min(engines, key=lambda e: c[e])
            wins[best] += 1
            bw += lost(c["blz"], c[best])
            bl += lost(c[best], c["blz"])
        print(f"{n:>8} " + " ".join(f"{wins[e]:>7}" for e in engines) + f" | {bw:>8} {bl:>8}")

if "--cells" in sys.argv:
    print()
    for n in sorted(by):
        for _, g, c in by[n]:
            if lost(c["v2"], c["blz"]):
                print(f"{n:>8} {g:14} v2 {c['v2']:6.2f} lsd {c['lsd']:6.2f} "
                      f"std {c['std']:6.2f} blz {c['blz']:6.2f}  {paths.get((n, g), '')}")
