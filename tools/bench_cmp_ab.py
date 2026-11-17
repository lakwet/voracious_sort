"""Before/after binaries from sentinel control logs: per cell, the median of
the rounds of each side (the 'old' column, rule off), then after/before."""
import math
import re
import statistics
import sys

prefix = sys.argv[1]  # e.g. .../surf_ or .../perf_
rounds = int(sys.argv[2]) if len(sys.argv) > 2 else 3
pat = re.compile(r'^(.*?)\s+(?:ctrl|moved) old\s+([\d.]+)')


def load(side):
    cells = {}
    for r in range(1, rounds + 1):
        for line in open(f'{prefix}{side}_{r}.log', encoding='utf-8', errors='replace'):
            m = pat.match(line)
            if m:
                cells.setdefault(m.group(1).strip(), []).append(float(m.group(2)))
    return {k: statistics.median(v) for k, v in cells.items()}


b, a = load('before'), load('after')
ratios = [(a[k] / b[k], k, b[k], a[k]) for k in b if k in a and b[k] > 0]
geo = math.exp(sum(math.log(r) for r, *_ in ratios) / len(ratios))
med = statistics.median(r for r, *_ in ratios)
print(f'{len(ratios)} cells, after/before geo {geo:.3f}, median {med:.3f}')
ratios.sort()
print('best:')
for r, k, x, y in ratios[:5]:
    print(f'  {r:.3f}  {k}  {x:.2f} -> {y:.2f}')
print('worst:')
for r, k, x, y in ratios[-8:]:
    print(f'  {r:.3f}  {k}  {x:.2f} -> {y:.2f}')
