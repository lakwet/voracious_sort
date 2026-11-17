"""Losses of the v2 against the v1 in a bench-cpu file: python tools/bench_v1loss.py <file>
(one short line per losing cell, then counts per type)."""
import sys, collections, math
path=sys.argv[1]
cols=None; losses=[]; per=collections.defaultdict(list)
for l in open(path,encoding='utf-8',errors='replace'):
    l=l.rstrip('\n')
    if l.startswith('cols,'):
        cols=l.split(',')[1:]
    elif l.startswith('row,') and cols:
        r=l.split(',')[1:]
        d=dict(zip(cols,r))
        if 'v1' not in d or 'v2' not in d: continue
        try: v2=float(d['v2']); v1=float(d['v1'])
        except: continue
        t=r[0]; size=int(r[1]); gen=r[2]
        per[(t,size)].append(v1/v2)
        if v1 < v2*0.95 and v2-v1 > 0.03:
            losses.append((t,size,gen,v2,v1))
print("geo v1/v2 by type and size (>1: v2 faster)")
types=collections.OrderedDict()
for (t,s),v in per.items(): types.setdefault(t,[]).append((s,math.exp(sum(map(math.log,v))/len(v)),len(v)))
for t,v in types.items(): print(f"  {t:8}", ' '.join(f"{s}:{g:.2f}" for s,g,_ in sorted(v)))
print(f"\nrows where v1 beats v2 by >5% and >0.03 ns: {len(losses)}")
by=collections.Counter((t) for t,*_ in losses)
print(' ', dict(by))
for t,s,g,v2,v1 in sorted(losses, key=lambda x: x[4]/x[3]):
    print(f"  {t:8} {s:>9} {g:16} v2 {v2:7.3f} v1 {v1:7.3f} ratio {v1/v2:.2f}")
