"""Per size, won / lost of column A against column B (and against blazing)
in ab_u32 outputs. Usage: python tools/ab32_cmp.py FILE A B [--noise 0.03]"""
import sys
from collections import defaultdict
f, a, b = sys.argv[1:4]
noise = float(sys.argv[sys.argv.index("--noise") + 1]) if "--noise" in sys.argv else 0.03
cols = None
by = defaultdict(list)
for line in open(f, encoding="utf-8"):
    x = line.split()
    if x and x[0] == "size":
        cols = x[2:]
        if cols.count("copy") > 1:
            cols[len(cols) - 1 - cols[::-1].index("copy")] = "memcpy"
        continue
    if cols and x and x[0].isdigit() and len(x) == len(cols) + 2:
        by[int(x[0])].append((x[1], dict(zip(cols, map(float, x[2:])))))
print(f"{'size':>8} | {a}<{b} {a}>{b} med | {a}>blz {b}>blz | Unif {a} {b} blz | lost by {a} vs {b}")
for n in sorted(by):
    r = by[n]
    w = [g for g, c in r if c[b] > c[a] * (1 + noise)]
    l = [g for g, c in r if c[a] > c[b] * (1 + noise)]
    q = sorted(c[a] / c[b] for g, c in r if c[b] > 0)
    la = sum(c[a] > c["blz"] * (1 + noise) for g, c in r)
    lb = sum(c[b] > c["blz"] * (1 + noise) for g, c in r)
    u = [c for g, c in r if g == "Unif"][0]
    print(f"{n:>8} | {len(w):>5} {len(l):>5} {q[len(q)//2]:.2f} | {la:>6} {lb:>6} | "
          f"{u[a]:.2f} {u[b]:.2f} {u['blz']:.2f} | {' '.join(l[:8])}")
