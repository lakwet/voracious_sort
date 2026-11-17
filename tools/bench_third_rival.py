"""Rivals outside the two best that show a weakness of the v2: rows where
they beat v2 (unstable v2 for the records' unstable rivals is not split
here: v2 stable column) by more than 5 % while neither of the two best
does. python tools/bench_third_rival.py <file>... Prints per type the count
of such rows and a few examples."""
import sys, collections
BEST = {
    'u32': ('blazing', 'turbosort'), 'i32': ('blazing', 'turbosort'), 'f32': ('blazing', 'turbosort'),
    'u16': ('blazing', 'turbosort'), 'u8': ('blazing', 'turbosort'),
    'u64': ('blazing', 'glidesort'), 'i64': ('blazing', 'glidesort'),
    'i16': ('blazing', 'radsort'), 'i8': ('blazing', 'radsort'),
    'f64': ('turbosort', 'glidesort'), 'u128': ('radsort', 'rdst'), 'i128': ('radsort', 'rdst'),
    'char': ('radsort', 'glidesort'),
}
OWN = {'v2', 'v1', 'std', 'std unstable', 'std cached', 'v2 unstable', 'v1 stable'}
hits = collections.defaultdict(lambda: collections.defaultdict(list))
for path in sys.argv[1:]:
    cols = None
    for line in open(path, encoding='utf-8', errors='replace'):
        p = line.rstrip('\n').split(',')
        if p[0] == 'cols':
            cols = p
        elif p[0] == 'row' and cols:
            ty = p[1]
            best = BEST.get(ty) or (('radsort', 'rdst') if ty.endswith(' 8B') else ('glidesort', 'radsort'))
            vals = {}
            for c, v in zip(cols[4:], p[4:]):
                try:
                    vals[c] = float(v)
                except ValueError:
                    pass
            v2 = vals.get('v2 unstable') if ty.startswith('records') and 'v2 unstable' in vals else vals.get('v2')
            if not v2:
                continue
            if any(vals.get(b, 1e18) < 0.95 * v2 for b in best):
                continue
            for c, x in vals.items():
                if c not in OWN and c not in best and x < 0.95 * v2:
                    hits[ty][c].append((x / v2, p[2], p[3], path.split('/')[-1].split('.')[0]))
for ty, rivals in hits.items():
    for c, rows in rivals.items():
        rows.sort()
        ex = '; '.join(f'{s} {g} {m} {r:.2f}' for r, s, g, m in rows[:3])
        print(f'{ty:16} {c:10} {len(rows):3} rows  e.g. {ex}')
