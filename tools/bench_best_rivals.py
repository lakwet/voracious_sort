"""Best rivals per type in bench-cpu files: for each type, the geometric mean
of each rival's time over all rows of all files (lower: faster), ranked.
python tools/bench_best_rivals.py <file>... (std*, v1, v2* left out)."""
import sys, math, collections
OWN = {'v2', 'v1', 'std', 'std unstable', 'std cached', 'v2 unstable', 'v1 stable'}
logs = collections.defaultdict(lambda: collections.defaultdict(list))
for path in sys.argv[1:]:
    cols = None
    for line in open(path, encoding='utf-8', errors='replace'):
        p = line.rstrip('\n').split(',')
        if p[0] == 'cols':
            cols = p
        elif p[0] == 'row' and cols:
            ty = p[1]
            for c, v in zip(cols[4:], p[4:]):
                if c in OWN:
                    continue
                try:
                    x = float(v)
                except ValueError:
                    continue
                if x > 0:
                    logs[ty][c].append(math.log(x))
for ty, rivals in logs.items():
    ranked = sorted((math.exp(sum(v) / len(v)), c, len(v)) for c, v in rivals.items())
    print(f'{ty:16} ' + '  '.join(f'{c} {g:.2f}' for g, c, n in ranked))
