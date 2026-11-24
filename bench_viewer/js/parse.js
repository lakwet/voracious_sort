// Reads a bench file written by src/tests/bench/bench_cpu.rs (the .csv, or an
// older bench-cpu-*.txt that holds the same raw lines).
//
// Lines used:
//   == machine            then key,value lines (cpu, arch, os, caches...)
//   cols,<label>,size,gen,<sort>,<sort>...
//   row,<label>,<size>,<distribution>,<ns per key>,...   (one per sort)
//   iter,<label>,<size>,<arrays per measurement>,<measurements>
//   skip,<label>,<size>,<reason>
//   CHECK FAILED,... / SORT PANICKED,... / PANIC,...
//   == done in N min
'use strict';

const MACHINE_KEYS = new Set([
  'cpu', 'cpu details', 'power', 'arch', 'os', 'threads', 'ram', 'l2', 'caches', 'page',
  'target features', 'rustc', 'commit', 'settings', 'core',
]);

function parseBench(text, fileName) {
  const run = {
    file: fileName,
    machine: {},
    labels: new Map(), // label -> { sorts, sizes, gens, cells: Map("size|gen" -> ns[]), iters }
    failures: [],
    skips: [],
    done: null,
    rows: 0,
  };
  const lines = text.split(/\r?\n/);
  for (const line of lines) {
    if (!line) continue;
    if (line.startsWith('== done in')) {
      run.done = line.slice(3);
      continue;
    }
    if (line.startsWith('CHECK FAILED') || line.startsWith('SORT PANICKED') || line.startsWith('PANIC')) {
      run.failures.push(line.replace(/,/g, ' · '));
      continue;
    }
    const comma = line.indexOf(',');
    if (comma < 0) continue;
    const kind = line.slice(0, comma);
    if (MACHINE_KEYS.has(kind)) {
      run.machine[kind] = line.slice(comma + 1);
      continue;
    }
    const p = line.split(',');
    if (kind === 'cols') {
      const label = p[1];
      if (!run.labels.has(label)) {
        run.labels.set(label, { sorts: p.slice(4), sizes: [], gens: [], cells: new Map(), iters: new Map() });
      }
    } else if (kind === 'row') {
      const t = run.labels.get(p[1]);
      if (!t) continue;
      const size = Number(p[2]);
      const gen = p[3];
      const ns = p.slice(4).map((x) => {
        const v = Number(x);
        return Number.isFinite(v) && v > 0 ? v : NaN;
      });
      if (!t.sizes.includes(size)) t.sizes.push(size);
      if (!t.gens.includes(gen)) t.gens.push(gen);
      t.cells.set(size + '|' + gen, ns);
      run.rows++;
    } else if (kind === 'iter') {
      const t = run.labels.get(p[1]);
      if (t) t.iters.set(Number(p[2]), { arrays: Number(p[3]), measures: Number(p[4]) });
    } else if (kind === 'skip') {
      run.skips.push({ label: p[1], size: Number(p[2]), reason: p.slice(3).join(',') });
    }
  }
  for (const t of run.labels.values()) t.sizes.sort((a, b) => a - b);
  // Drop labels without any row (a part announced but not measured).
  for (const [label, t] of run.labels) if (t.cells.size === 0) run.labels.delete(label);
  run.name = runName(run);
  return run;
}

// "Ryzen 9 3950X · 2026-11-18 11:06" from the cpu line and the file name.
function runName(run) {
  const cpu = (run.machine.cpu || '')
    .replace(/\(R\)|\(TM\)|CPU|Processor|\d+-Core/gi, '')
    .replace(/^(AMD|Intel|Qualcomm)\s+/i, '')
    .replace(/\s+@.*$/, '')
    .replace(/\s+/g, ' ')
    .trim();
  const m = run.file.match(/(\d{4})-(\d{2})-(\d{2})-(\d{2})(\d{2})/);
  const date = m ? `${m[1]}-${m[2]}-${m[3]} ${m[4]}:${m[5]}` : '';
  const base = run.file.replace(/\.(csv|txt)$/, '');
  return [cpu || base, date].filter(Boolean).join(' · ');
}
