"""Records like for like in a bench-cpu file: v2 (stable) against v1 stable,
v2 unstable against v1 (unstable). python tools/bench_records_like.py <file>
Prints the geometric ratios per type and size, then the cells lost by > 10 %."""
import sys, math, collections
cols = None
geo = collections.defaultdict(list)
lost = []
for line in open(sys.argv[1], encoding='utf-8', errors='replace'):
    p = line.rstrip('\n').split(',')
    if p[0] == 'cols':
        cols = {c: i for i, c in enumerate(p)}
    elif p[0] == 'row' and cols and 'v1 stable' in cols:
        ty, size, gen = p[1], int(p[2]), p[3]
        for mode, mine, ref in (('stable', 'v2', 'v1 stable'), ('unstable', 'v2 unstable', 'v1')):
            a, b = float(p[cols[mine]]), float(p[cols[ref]])
            r = b / a  # > 1: v2 faster
            geo[(ty, mode, size)].append(math.log(r))
            if r < 0.9 and a - b > 0.03:
                lost.append((r, ty, mode, size, gen, a, b))
print('geo v1/v2 (>1: v2 faster)')
keys = sorted(geo)
for ty_mode in sorted({k[:2] for k in keys}):
    cells = ' '.join(f'{k[2]}:{math.exp(sum(geo[k]) / len(geo[k])):.2f}' for k in keys if k[:2] == ty_mode)
    print(f'  {ty_mode[0]} {ty_mode[1]:8} {cells}')
print(f'cells lost by > 10 %: {len(lost)}')
for r, ty, mode, size, gen, a, b in sorted(lost)[:int(sys.argv[2]) if len(sys.argv) > 2 else 25]:
    print(f'  {ty} {mode:8} {size:>8} {gen:14} v2 {a:7.3f} v1 {b:7.3f} {r:.2f}')
