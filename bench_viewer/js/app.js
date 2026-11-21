// The bench viewer: state, filters, tabs.
'use strict';

// ---------------------------------------------------------------- format

const NF = (d) => new Intl.NumberFormat('en-US', { minimumFractionDigits: d, maximumFractionDigits: d });
const nf0 = NF(0);
const nf1 = NF(1);
const nf2 = NF(2);

function fmtRatio(r) {
  return Number.isFinite(r) ? nf2.format(r) : '—';
}

function fmtNs(v) {
  if (!Number.isFinite(v)) return '—';
  return (v < 10 ? nf2 : v < 100 ? nf1 : nf0).format(v);
}

function fmtDur(ns) {
  if (!Number.isFinite(ns)) return '—';
  const units = [
    [1e9, 's'],
    [1e6, 'ms'],
    [1e3, 'µs'],
    [1, 'ns'],
  ];
  for (const [f, u] of units) {
    if (ns >= f || f === 1) {
      const v = ns / f;
      return (v < 10 ? nf2 : v < 100 ? nf1 : nf0).format(v) + ' ' + u;
    }
  }
}

function fmtSize(n) {
  if (n >= 1e9 && n % 1e8 === 0) return nf0.format(n / 1e9).replace(/,0$/, '') + 'G';
  if (n >= 1e6) return (n % 1e6 === 0 ? nf0 : nf1).format(n / 1e6) + 'M';
  if (n >= 1e3) return (n % 1e3 === 0 ? nf0 : nf1).format(n / 1e3) + 'k';
  return String(n);
}

function fmtSizeLong(n) {
  return nf0.format(n) + ' keys';
}

function geo(xs) {
  const v = xs.filter((x) => Number.isFinite(x) && x > 0);
  if (!v.length) return NaN;
  return Math.exp(v.reduce((a, x) => a + Math.log(x), 0) / v.length);
}

// ---------------------------------------------------------------- colours of the sorts

// Each sort keeps its colour whatever is shown (colour follows the entity).
const SORT_STYLE = {
  v2: { slot: 0 },
  blazing: { slot: 1 },
  turbosort: { slot: 2 },
  glidesort: { slot: 3 },
  radsort: { slot: 4 },
  rdst: { slot: 5 },
  std: { slot: 6 },
  v1: { slot: 7 },
  'std unstable': { slot: 6, dash: '6 4' },
  'v1 stable': { slot: 7, dash: '6 4' },
  'v2 unstable': { slot: 0, dash: '6 4' },
  'v2 new': { slot: 8 },
};

function sortColor(name) {
  const st = SORT_STYLE[name];
  if (!st) return cssVar('--series-9');
  return st.slot === 0 ? cssVar('--ink') : cssVar('--series-' + st.slot);
}

function sortDash(name) {
  return (SORT_STYLE[name] || {}).dash || null;
}

// ---------------------------------------------------------------- state

const STORE_FILES = 'vbv-files';
const STORE_UI = 'vbv-ui';

const state = {
  runs: [],
  run: 0,
  label: null,
  ref: null, // name; the first column when absent from a type
  comp: 'best', // 'best' or a sort name
  variants: false, // the variants of the reference count as rivals
  hidden: new Set(),
  tab: 'overview',
  metric: 'ns',
  gen: '',
  yLog: false,
  size: null,
  lossMin: 3,
  lossAll: true,
  sortBy: { key: null, dir: 1 },
  lossSort: { key: 'factor', dir: -1 },
};

function saveUi() {
  try {
    const { runs, hidden, ...rest } = state;
    localStorage.setItem(STORE_UI, JSON.stringify({ ...rest, hidden: [...hidden] }));
  } catch (e) {}
}

function loadUi() {
  try {
    const s = JSON.parse(localStorage.getItem(STORE_UI) || 'null');
    if (!s) return;
    Object.assign(state, s, { hidden: new Set(s.hidden || []), runs: [] });
  } catch (e) {}
}

function saveFiles() {
  try {
    const files = state.runs.map((r) => ({ file: r.file, text: r.text }));
    const size = files.reduce((a, f) => a + f.text.length, 0);
    if (size > 4.5e6) {
      localStorage.removeItem(STORE_FILES);
      return;
    }
    localStorage.setItem(STORE_FILES, JSON.stringify(files));
  } catch (e) {}
}

function loadFiles() {
  try {
    const files = JSON.parse(localStorage.getItem(STORE_FILES) || '[]');
    for (const f of files) addRun(f.text, f.file, false);
  } catch (e) {}
}

function set(patch) {
  Object.assign(state, patch);
  saveUi();
  render();
}

// ---------------------------------------------------------------- model

function currentRun() {
  return state.runs[state.run] || null;
}

function typeCtx(run, label) {
  const t = run && run.labels.get(label);
  if (!t) return null;
  let refIdx = t.sorts.indexOf(state.ref);
  if (refIdx < 0) refIdx = 0;
  const refName = t.sorts[refIdx];
  const family = (n) => n.split(' ')[0];
  const others = t.sorts.map((_, i) => i).filter((i) => i !== refIdx && !state.hidden.has(t.sorts[i]));
  let rivals;
  if (state.comp === 'best') {
    rivals = others.filter((i) => state.variants || family(t.sorts[i]) !== family(refName));
  } else {
    const i = t.sorts.indexOf(state.comp);
    rivals = i >= 0 && i !== refIdx ? [i] : [];
  }
  return { t, label, refIdx, refName, others, rivals };
}

function ns(c, size, gen) {
  return c.t.cells.get(size + '|' + gen);
}

// The rival of one cell: { ratio: rival / reference, name, ns }.
function rivalOf(c, size, gen) {
  const v = ns(c, size, gen);
  if (!v) return null;
  let best = -1;
  for (const i of c.rivals) if (Number.isFinite(v[i]) && (best < 0 || v[i] < v[best])) best = i;
  if (best < 0) return null;
  return { ratio: v[best] / v[c.refIdx], name: c.t.sorts[best], ns: v[best], refNs: v[c.refIdx] };
}

// Geometric mean of the ratios rival / reference at one size, and wins.
function sizeSummary(c, size) {
  const ratios = [];
  let wins = 0;
  for (const g of c.t.gens) {
    const r = rivalOf(c, size, g);
    if (!r || !Number.isFinite(r.ratio)) continue;
    ratios.push(r.ratio);
    if (r.ratio > 1) wins++;
  }
  return { geo: geo(ratios), wins, count: ratios.length };
}

function compLabel(c) {
  if (state.comp === 'best') return 'the best other sort';
  return state.comp;
}

// ---------------------------------------------------------------- files

function addRun(text, file, persist = true) {
  const run = parseBench(text, file);
  if (!run.labels.size) {
    toast(`${file}: no measurement found. Open the bench .csv (the .txt is the human report).`);
    return false;
  }
  run.text = text;
  const same = state.runs.findIndex((r) => r.file === file);
  if (same >= 0) state.runs[same] = run;
  else state.runs.push(run);
  state.run = same >= 0 ? same : state.runs.length - 1;
  if (persist) saveFiles();
  return true;
}

async function openFiles(fileList) {
  let ok = false;
  for (const f of fileList) {
    const text = await f.text();
    ok = addRun(text, f.name) || ok;
  }
  if (ok) {
    saveUi();
    render();
  }
}

function removeRun(i) {
  state.runs.splice(i, 1);
  state.run = Math.min(state.run, state.runs.length - 1);
  saveFiles();
  set({});
}

let toastTimer;
function toast(text) {
  const el = document.getElementById('toast');
  el.textContent = text;
  el.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (el.hidden = true), 6000);
}

// ---------------------------------------------------------------- shell

const TABS = [
  ['overview', 'Overview'],
  ['curves', 'Curves'],
  ['map', 'Heatmap'],
  ['detail', 'Detail'],
  ['losses', 'Losses'],
  ['machines', 'Machines'],
  ['machine', 'Machine info'],
];

function render() {
  const run = currentRun();
  document.getElementById('runs').replaceChildren(...runChips());
  const main = document.getElementById('main');
  if (!run) {
    document.getElementById('filters').replaceChildren();
    document.getElementById('tabs').replaceChildren();
    main.replaceChildren(emptyState());
    return;
  }
  if (!run.labels.has(state.label)) state.label = run.labels.keys().next().value;
  if (state.tab === 'machines' && state.runs.length < 2) state.tab = 'overview';
  document.getElementById('tabs').replaceChildren(
    ...TABS.filter(([k]) => k !== 'machines' || state.runs.length > 1).map(([k, name]) =>
      html('button', { class: 'tab' + (state.tab === k ? ' active' : ''), role: 'tab', 'aria-selected': state.tab === k, onclick: () => set({ tab: k }), text: name }),
    ),
  );
  document.getElementById('filters').replaceChildren(filters(run));
  const view = { overview, curves, map, detail, losses, machines, machine }[state.tab] || overview;
  main.replaceChildren();
  view(main, run);
}

function runChips() {
  return state.runs.map((r, i) =>
    html('span', { class: 'run-chip' + (i === state.run ? ' active' : ''), title: r.file }, [
      html('button', { class: 'run-name', onclick: () => set({ run: i }), text: r.name }),
      html('button', { class: 'run-x', 'aria-label': 'Remove ' + r.file, onclick: () => removeRun(i), text: '×' }),
    ]),
  );
}

function emptyState() {
  return html('div', { class: 'empty' }, [
    html('div', { class: 'empty-box' }, [
      html('h2', { text: 'Drop a bench file here' }),
      html('p', {
        html: 'The <code>.csv</code> file written by <code>make bench</code> (in <code>resultsbenchs/&lt;machine&gt;/</code>), or an older <code>bench-cpu-*.txt</code>. Several files at once: one per machine, to compare them.',
      }),
      html('button', { class: 'btn primary', onclick: () => document.getElementById('file').click(), text: 'Choose files' }),
      html('p', { class: 'muted small', text: 'Everything stays in the browser: nothing is sent anywhere.' }),
    ]),
  ]);
}

function select(label, value, options, onchange) {
  const s = html('select', { onchange: (e) => onchange(e.target.value) });
  for (const [v, text] of options) {
    const o = html('option', { value: v, text });
    if (String(v) === String(value)) o.selected = true;
    s.append(o);
  }
  return html('label', { class: 'field' }, [html('span', { class: 'field-label', text: label }), s]);
}

function filters(run) {
  const c = typeCtx(run, state.label);
  const t = c.t;
  const row = html('div', { class: 'filter-row' });
  const labels = [...run.labels.keys()];
  if (state.tab !== 'overview' && state.tab !== 'losses' && state.tab !== 'machine') {
    row.append(select('Type', state.label, labels.map((l) => [l, l]), (v) => set({ label: v, size: null, gen: '' })));
  }
  if (state.tab !== 'machine') {
    row.append(select('Reference', c.refName, t.sorts.map((s) => [s, s]), (v) => set({ ref: v })));
    row.append(
      select(
        'Compared with',
        state.comp,
        [['best', 'the best other sort'], ...t.sorts.filter((s) => s !== c.refName).map((s) => [s, s])],
        (v) => set({ comp: v }),
      ),
    );
    if (state.comp === 'best') {
      const cb = html('input', { type: 'checkbox', onchange: (e) => set({ variants: e.target.checked }) });
      cb.checked = state.variants;
      row.append(html('label', { class: 'check', title: 'For example “v2 unstable” when the reference is “v2”' }, [cb, html('span', { text: 'include variants of the reference' })]));
    }
  }
  const wrap = html('div', {}, [row]);
  if (state.tab !== 'machine') {
    const chips = html('div', { class: 'chips', role: 'group', 'aria-label': 'Sorts shown' });
    chips.append(html('span', { class: 'field-label', text: 'Sorts' }));
    for (const s of t.sorts) {
      const on = !state.hidden.has(s);
      const isRef = s === c.refName;
      chips.append(
        html(
          'button',
          {
            class: 'chip' + (on ? ' on' : '') + (isRef ? ' ref' : ''),
            'aria-pressed': on,
            title: isRef ? 'The reference is always shown' : on ? 'Hide ' + s : 'Show ' + s,
            onclick: () => {
              if (isRef) return;
              const h = new Set(state.hidden);
              on ? h.add(s) : h.delete(s);
              set({ hidden: h });
            },
          },
          [html('span', { class: 'swatch' + (sortDash(s) ? ' dashed' : ''), style: `--c:${sortColor(s)}` }), s + (isRef ? ' (ref.)' : '')],
        ),
      );
    }
    wrap.append(chips);
  }
  return wrap;
}

function section(title, sub, children) {
  return html('section', { class: 'card' }, [
    html('header', { class: 'card-head' }, [html('h2', { text: title }), sub ? html('p', { class: 'muted', text: sub }) : null]),
    ...children,
  ]);
}

// ---------------------------------------------------------------- overview

function overview(main, run) {
  const labels = [...run.labels.keys()];
  const sizes = [...new Set(labels.flatMap((l) => run.labels.get(l).sizes))].sort((a, b) => a - b);
  const table = html('table', { class: 'grid-table overview' });
  const head = html('tr', {}, [html('th', { text: 'Type' }), ...sizes.map((s) => html('th', { class: 'num', text: fmtSize(s) })), html('th', { class: 'num', text: 'All' })]);
  table.append(html('thead', {}, [head]));
  const body = html('tbody');
  let totalWins = 0;
  let totalCells = 0;
  for (const l of labels) {
    const c = typeCtx(run, l);
    const tr = html('tr', {}, [html('th', { class: 'row-head' }, [html('button', { class: 'link', onclick: () => set({ label: l, tab: 'curves' }), text: l })])]);
    const all = [];
    let w = 0;
    let n = 0;
    for (const s of sizes) {
      if (!c.t.sizes.includes(s)) {
        tr.append(html('td', { class: 'cell empty-cell' }));
        continue;
      }
      const sm = sizeSummary(c, s);
      for (const g of c.t.gens) {
        const r = rivalOf(c, s, g);
        if (r) all.push(r.ratio);
      }
      w += sm.wins;
      n += sm.count;
      tr.append(ratioCell(sm.geo, `${sm.wins}/${sm.count}`, () => set({ label: l, size: s, tab: 'detail' }), () => tipSummary(c, s, sm)));
    }
    totalWins += w;
    totalCells += n;
    tr.append(ratioCell(geo(all), `${w}/${n}`, () => set({ label: l, tab: 'map' }), null, 'total'));
    body.append(tr);
  }
  table.append(body);
  const pct = totalCells ? Math.round((100 * totalWins) / totalCells) : 0;
  const c0 = typeCtx(run, labels[0]);
  main.append(
    html('div', { class: 'stats' }, [
      stat(labels.length, labels.length > 1 ? 'types measured' : 'type measured'),
      stat(sizes.length, 'sizes, from ' + fmtSize(sizes[0]) + ' to ' + fmtSize(sizes[sizes.length - 1])),
      stat(pct + ' %', `of the cases won by ${c0.refName} (${nf0.format(totalWins)} of ${nf0.format(totalCells)})`),
      stat(run.failures.length, run.failures.length ? 'failures (see Machine info)' : 'failed check', run.failures.length ? 'bad' : ''),
    ]),
    section(
      `${c0.refName} against ${compLabel(c0)}`,
      'Each cell: geometric mean over the distributions of the ratio “time of the other sort / time of the reference” (> 1: the reference is faster), and the number of distributions won. Click a cell for the detail.',
      [ratioLegend(), html('div', { class: 'scroll-x' }, [table])],
    ),
  );
}

function stat(value, text, cls) {
  return html('div', { class: 'stat ' + (cls || '') }, [html('div', { class: 'stat-value', text: String(value) }), html('div', { class: 'stat-text', text })]);
}

function ratioCell(ratio, small, onclick, tip, cls) {
  const bg = ratioColor(ratio);
  const td = html('td', { class: 'cell clickable ' + (cls || ''), style: `background:${bg};color:${inkOn(bg)}`, onclick, tabindex: 0 }, [
    html('div', { class: 'cell-main', text: fmtRatio(ratio) }),
    small ? html('div', { class: 'cell-sub', text: small }) : null,
  ]);
  td.addEventListener('keydown', (e) => e.key === 'Enter' && onclick());
  if (tip) {
    td.addEventListener('mousemove', (e) => Tip.show(e, tip()));
    td.addEventListener('mouseleave', () => Tip.hide());
  }
  return td;
}

function tipSummary(c, size, sm) {
  const body = html('div', {}, [html('div', { class: 'tip-title', text: `${c.label} · ${fmtSizeLong(size)}` })]);
  body.append(html('div', { class: 'tip-row' }, [html('span', { class: 'tip-name', text: 'mean ratio' }), html('span', { class: 'tip-val', text: fmtRatio(sm.geo) })]));
  body.append(html('div', { class: 'tip-row' }, [html('span', { class: 'tip-name', text: 'won by ' + c.refName }), html('span', { class: 'tip-val', text: `${sm.wins} / ${sm.count}` })]));
  // Who wins the lost cells.
  const who = {};
  for (const g of c.t.gens) {
    const r = rivalOf(c, size, g);
    if (r && r.ratio < 1) who[r.name] = (who[r.name] || 0) + 1;
  }
  const lost = Object.entries(who).sort((a, b) => b[1] - a[1]);
  if (lost.length) {
    body.append(html('div', { class: 'tip-sep', text: 'lost cases, won by' }));
    for (const [n, k] of lost) {
      body.append(html('div', { class: 'tip-row' }, [html('span', { class: 'swatch', style: `background:${sortColor(n)}` }), html('span', { class: 'tip-name', text: n }), html('span', { class: 'tip-val', text: String(k) })]));
    }
  }
  return body;
}

// ---------------------------------------------------------------- curves

function curves(main, run) {
  const c = typeCtx(run, state.label);
  const t = c.t;
  if (state.gen && !t.gens.includes(state.gen)) state.gen = '';
  const gens = state.gen ? [state.gen] : t.gens;
  const shown = [c.refIdx, ...c.others];
  const series = shown.map((i) => {
    const name = t.sorts[i];
    const points = t.sizes.map((size) => {
      let y;
      if (state.metric === 'ratio') {
        y = geo(gens.map((g) => {
          const v = ns(c, size, g);
          return v ? v[i] / v[c.refIdx] : NaN;
        }));
      } else {
        y = geo(gens.map((g) => (ns(c, size, g) || [])[i]));
        if (state.metric === 'dur') y *= size;
      }
      return { x: size, y };
    });
    return { name, color: sortColor(name), dash: sortDash(name), width: i === c.refIdx ? 3 : 2, top: i === c.refIdx, points };
  });
  const controls = html('div', { class: 'filter-row tight' }, [
    select('Measure', state.metric, [['ns', 'ns per key'], ['dur', 'time of one sort'], ['ratio', 'ratio to the reference']], (v) => set({ metric: v, yLog: v === 'ratio' ? true : v === 'dur' ? true : state.yLog })),
    select('Distribution', state.gen, [['', `all (geometric mean, ${t.gens.length})`], ...t.gens.map((g) => [g, g])], (v) => set({ gen: v })),
  ]);
  const cb = html('input', { type: 'checkbox', onchange: (e) => set({ yLog: e.target.checked }) });
  cb.checked = state.yLog;
  controls.append(html('label', { class: 'check' }, [cb, html('span', { text: 'logarithmic vertical axis' })]));
  const chart = html('div', { class: 'chart-wrap' });
  const titles = { ns: 'ns per key', dur: 'time of one sort', ratio: `time / time of ${c.refName}` };
  const card = section(
    `${state.label}: ${titles[state.metric]}`,
    (state.gen ? `Distribution “${state.gen}”. ` : `Geometric mean over ${t.gens.length} distributions. `) +
      (state.metric === 'ratio' ? 'Above 1: the reference is faster. ' : 'Lower is faster. ') +
      'Hover for the values; click for the detail of that size.',
    [controls, legend(series), chart],
  );
  main.append(card);
  const yFormat = state.metric === 'ratio' ? fmtRatio : state.metric === 'dur' ? fmtDur : fmtNs;
  const draw = () =>
    lineChart(chart, {
      series,
      xs: t.sizes,
      xFormat: fmtSize,
      xTipFormat: fmtSizeLong,
      yFormat,
      tipFormat: state.metric === 'ns' ? (v) => fmtNs(v) + ' ns' : yFormat,
      yLog: state.yLog,
      refLine: state.metric === 'ratio' ? 1 : undefined,
      yTitle: titles[state.metric],
      height: 420,
      onClickX: (size) => set({ size, tab: 'detail' }),
    });
  requestAnimationFrame(draw);
  onResize(draw);
}

function legend(series) {
  return html(
    'div',
    { class: 'legend' },
    series.map((s) => html('span', { class: 'legend-item' + (s.top ? ' ref' : '') }, [html('span', { class: 'swatch' + (s.dash ? ' dashed' : ''), style: `--c:${s.color}` }), s.name])),
  );
}

let resizeHook = null;
function onResize(fn) {
  resizeHook = fn;
}
let resizeTimer;
window.addEventListener('resize', () => {
  clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => resizeHook && document.querySelector('.chart-wrap') && resizeHook(), 120);
});

// ---------------------------------------------------------------- map (heatmap)

function map(main, run) {
  const c = typeCtx(run, state.label);
  const t = c.t;
  const table = html('table', { class: 'grid-table heat' });
  table.append(html('thead', {}, [html('tr', {}, [html('th', { text: 'Distribution' }), ...t.sizes.map((s) => html('th', { class: 'num', text: fmtSize(s) }))])]));
  const body = html('tbody');
  for (const g of t.gens) {
    const tr = html('tr', {}, [html('th', { class: 'row-head', text: g })]);
    for (const s of t.sizes) {
      const r = rivalOf(c, s, g);
      const sub = r && state.comp === 'best' && r.ratio < 1 ? r.name : null;
      tr.append(ratioCell(r ? r.ratio : NaN, sub, () => set({ size: s, tab: 'detail' }), () => tipCell(c, s, g)));
    }
    body.append(tr);
  }
  const foot = html('tfoot');
  const geoRow = html('tr', {}, [html('th', { class: 'row-head', text: 'geometric mean' })]);
  for (const s of t.sizes) {
    const sm = sizeSummary(c, s);
    geoRow.append(ratioCell(sm.geo, `${sm.wins}/${sm.count}`, () => set({ size: s, tab: 'detail' }), () => tipSummary(c, s, sm), 'total'));
  }
  foot.append(geoRow);
  table.append(body, foot);
  main.append(
    section(
      `${state.label}: ${c.refName} against ${compLabel(c)}`,
      'One row per distribution, one column per size. Ratio “time of the other sort / time of the reference”. In red, the lost cases, with the name of the winning sort. Hover for every time; click for the detail.',
      [ratioLegend(), html('div', { class: 'scroll-x' }, [table])],
    ),
  );
}

function tipCell(c, size, gen) {
  const v = ns(c, size, gen) || [];
  const body = html('div', {}, [html('div', { class: 'tip-title', text: `${gen} · ${fmtSizeLong(size)}` })]);
  const rows = [c.refIdx, ...c.others].map((i) => ({ i, v: v[i] })).filter((x) => Number.isFinite(x.v));
  rows.sort((a, b) => a.v - b.v);
  for (const { i, v: x } of rows) {
    body.append(
      html('div', { class: 'tip-row' + (i === c.refIdx ? ' tip-ref' : '') }, [
        html('span', { class: 'swatch', style: `background:${sortColor(c.t.sorts[i])}` }),
        html('span', { class: 'tip-name', text: c.t.sorts[i] }),
        html('span', { class: 'tip-val', text: fmtDur(x * size) }),
        html('span', { class: 'tip-val muted', text: i === c.refIdx ? 'ref.' : '× ' + fmtRatio(x / v[c.refIdx]) }),
      ]),
    );
  }
  return body;
}

// ---------------------------------------------------------------- detail (one size, a table)

function detail(main, run) {
  const c = typeCtx(run, state.label);
  const t = c.t;
  if (!t.sizes.includes(state.size)) state.size = t.sizes[0];
  const size = state.size;
  const sizes = html('div', { class: 'chips sizes', role: 'group', 'aria-label': 'Size' }, [html('span', { class: 'field-label', text: 'Size' })]);
  for (const s of t.sizes) {
    sizes.append(html('button', { class: 'chip' + (s === size ? ' on' : ''), 'aria-pressed': s === size, onclick: () => set({ size: s }), text: fmtSize(s) }));
  }
  const shown = [c.refIdx, ...c.others];
  const rows = t.gens.map((g) => {
    const v = ns(c, size, g) || [];
    const r = rivalOf(c, size, g);
    return { gen: g, v, gap: r ? r.ratio : NaN, rival: r };
  });
  const sb = state.sortBy;
  if (sb.key !== null) {
    const key = sb.key;
    const val = (row) => (key === 'gen' ? row.gen : key === 'gap' ? row.gap : row.v[key] / (key === c.refIdx ? 1 : row.v[c.refIdx]));
    rows.sort((a, b) => {
      const x = val(a);
      const y = val(b);
      if (typeof x === 'string') return x.localeCompare(y) * sb.dir;
      return ((Number.isFinite(x) ? x : Infinity) - (Number.isFinite(y) ? y : Infinity)) * sb.dir;
    });
  }
  const th = (text, key, cls) => {
    const active = sb.key === key;
    return html('th', { class: (cls || '') + ' sortable' + (active ? ' sorted' : ''), 'aria-sort': active ? (sb.dir > 0 ? 'ascending' : 'descending') : 'none' }, [
      html('button', { onclick: () => set({ sortBy: { key, dir: active ? -sb.dir : key === 'gen' ? 1 : -1 } }), text: text + (active ? (sb.dir > 0 ? ' ↑' : ' ↓') : '') }),
    ]);
  };
  const table = html('table', { class: 'data-table' });
  table.append(
    html('thead', {}, [
      html('tr', {}, [
        th('Distribution', 'gen'),
        ...shown.map((i) => {
          const h = th(t.sorts[i], i, 'num');
          h.prepend(html('span', { class: 'swatch' + (sortDash(t.sorts[i]) ? ' dashed' : ''), style: `--c:${sortColor(t.sorts[i])}` }));
          return h;
        }),
        th('Gap', 'gap', 'gap-col'),
      ]),
    ]),
  );
  const body = html('tbody');
  for (const row of rows) {
    const finite = shown.map((i) => row.v[i]).filter(Number.isFinite);
    const fastest = Math.min(...finite);
    const tr = html('tr', {}, [html('th', { class: 'row-head', text: row.gen })]);
    for (const i of shown) {
      const x = row.v[i];
      const best = x === fastest;
      const ratio = x / row.v[c.refIdx];
      const td = html('td', { class: 'num' + (best ? ' best' : '') }, [
        html('div', { class: 'dur', text: fmtDur(x * size) }),
        html('div', { class: 'sub', text: i === c.refIdx ? fmtNs(x) + ' ns/key' : '× ' + fmtRatio(ratio) }),
      ]);
      if (i !== c.refIdx && Number.isFinite(ratio)) ratio < 1 && td.querySelector('.sub').classList.add('lose');
      tr.append(td);
    }
    const gapTd = html('td', { class: 'gap-col' }, [ratioBar(row.gap), html('span', { class: 'gap-text', text: fmtRatio(row.gap) + (row.rival && row.gap < 1 && state.comp === 'best' ? ' · ' + row.rival.name : '') })]);
    tr.append(gapTd);
    body.append(tr);
  }
  // Summary rows.
  const geoRow = html('tr', { class: 'sum' }, [html('th', { class: 'row-head', text: 'geometric mean' })]);
  const winRow = html('tr', { class: 'sum' }, [html('th', { class: 'row-head', text: `won by ${c.refName}` })]);
  for (const i of shown) {
    if (i === c.refIdx) {
      const g = geo(t.gens.map((gg) => (ns(c, size, gg) || [])[i]));
      geoRow.append(html('td', { class: 'num' }, [html('div', { class: 'dur', text: fmtDur(g * size) }), html('div', { class: 'sub', text: fmtNs(g) + ' ns/key' })]));
      winRow.append(html('td', { class: 'num muted', text: 'ref.' }));
      continue;
    }
    const ratios = t.gens.map((gg) => {
      const v = ns(c, size, gg) || [];
      return v[i] / v[c.refIdx];
    });
    const gm = geo(ratios);
    const valid = ratios.filter(Number.isFinite);
    geoRow.append(html('td', { class: 'num' }, [html('div', { class: 'sub' + (gm < 1 ? ' lose' : ''), text: '× ' + fmtRatio(gm) })]));
    winRow.append(html('td', { class: 'num', text: `${valid.filter((r) => r > 1).length} / ${valid.length}` }));
  }
  const sm = sizeSummary(c, size);
  geoRow.append(html('td', { class: 'gap-col' }, [ratioBar(sm.geo), html('span', { class: 'gap-text', text: fmtRatio(sm.geo) })]));
  winRow.append(html('td', { class: 'gap-col', text: `${sm.wins} / ${sm.count}` }));
  table.append(body, html('tfoot', {}, [geoRow, winRow]));
  const it = t.iters.get(size);
  const how = it ? ` Each measurement sorts ${nf0.format(it.arrays)} array${it.arrays > 1 ? 's' : ''}; ${it.measures} measurements, the median is kept.` : '';
  main.append(
    section(
      `${state.label} · ${fmtSizeLong(size)}`,
      `Time of one sort, then “× ratio”: time of this sort / time of ${c.refName} (in red: faster than the reference). In bold: the fastest of the row. Gap: the reference against ${compLabel(c)}. Click a header to sort.` + how,
      [sizes, html('div', { class: 'scroll-x' }, [table])],
    ),
  );
}

// ---------------------------------------------------------------- losses

function losses(main, run) {
  const min = state.lossMin / 100;
  const list = [];
  const labels = state.lossAll ? [...run.labels.keys()] : [state.label];
  for (const l of labels) {
    const c = typeCtx(run, l);
    for (const s of c.t.sizes) {
      for (const g of c.t.gens) {
        const r = rivalOf(c, s, g);
        if (!r || !Number.isFinite(r.ratio)) continue;
        const factor = 1 / r.ratio;
        if (factor >= 1 + min) list.push({ label: l, size: s, gen: g, winner: r.name, refNs: r.refNs, ns: r.ns, factor, refName: c.refName });
      }
    }
  }
  const ls = state.lossSort;
  const key = ls.key;
  list.sort((a, b) => {
    const x = a[key];
    const y = b[key];
    return (typeof x === 'string' ? x.localeCompare(y) : x - y) * ls.dir;
  });
  const range = html('input', { type: 'range', min: 0, max: 50, step: 1, value: state.lossMin, oninput: (e) => (e.target.nextSibling.textContent = e.target.value + ' %'), onchange: (e) => set({ lossMin: Number(e.target.value) }) });
  const scope = html('input', { type: 'checkbox', onchange: (e) => set({ lossAll: e.target.checked }) });
  scope.checked = state.lossAll;
  const controls = html('div', { class: 'filter-row tight' }, [
    html('label', { class: 'field' }, [html('span', { class: 'field-label', text: 'Loss of at least' }), html('span', { class: 'range' }, [range, html('span', { class: 'range-val', text: state.lossMin + ' %' })])]),
    html('label', { class: 'check' }, [scope, html('span', { text: 'all types' })]),
  ]);
  if (!state.lossAll) controls.prepend(select('Type', state.label, [...run.labels.keys()].map((l) => [l, l]), (v) => set({ label: v })));
  const th = (text, k, cls) => {
    const active = ls.key === k;
    return html('th', { class: (cls || '') + ' sortable' + (active ? ' sorted' : '') }, [html('button', { onclick: () => set({ lossSort: { key: k, dir: active ? -ls.dir : typeof list[0]?.[k] === 'string' ? 1 : -1 } }), text: text + (active ? (ls.dir > 0 ? ' ↑' : ' ↓') : '') })]);
  };
  const table = html('table', { class: 'data-table' });
  table.append(html('thead', {}, [html('tr', {}, [th('Type', 'label'), th('Size', 'size', 'num'), th('Distribution', 'gen'), th('Winner', 'winner'), th('Reference', 'refNs', 'num'), th('Winner', 'ns', 'num'), th('Loss', 'factor', 'num')])]));
  const body = html('tbody');
  const MAX = 400;
  for (const x of list.slice(0, MAX)) {
    const tr = html('tr', { class: 'clickable', tabindex: 0, onclick: () => set({ label: x.label, size: x.size, tab: 'detail' }) }, [
      html('td', { text: x.label }),
      html('td', { class: 'num', text: fmtSize(x.size) }),
      html('td', { text: x.gen }),
      html('td', {}, [html('span', { class: 'swatch', style: `--c:${sortColor(x.winner)}` }), x.winner]),
      html('td', { class: 'num', text: fmtDur(x.refNs * x.size) }),
      html('td', { class: 'num', text: fmtDur(x.ns * x.size) }),
      html('td', { class: 'num lose strong', text: '× ' + fmtRatio(x.factor) }),
    ]);
    tr.addEventListener('keydown', (e) => e.key === 'Enter' && tr.click());
    body.append(tr);
  }
  table.append(body);
  // Who wins the most.
  const by = {};
  for (const x of list) by[x.winner] = (by[x.winner] || 0) + 1;
  const counts = Object.entries(by).sort((a, b) => b[1] - a[1]);
  const refName = typeCtx(run, labels[0]).refName;
  main.append(
    section(
      `Cases where ${refName} loses by at least ${state.lossMin} %`,
      `${nf0.format(list.length)} case${list.length === 1 ? '' : 's'}${list.length > MAX ? ` (the first ${MAX} shown)` : ''}. Loss = time of the reference / time of the winner. Click a row for the detail.`,
      [
        controls,
        counts.length
          ? html('div', { class: 'legend' }, counts.map(([n, k]) => html('span', { class: 'legend-item' }, [html('span', { class: 'swatch', style: `--c:${sortColor(n)}` }), `${n}: ${k}`])))
          : html('p', { class: 'empty-note', text: 'No loss at this threshold.' }),
        list.length ? html('div', { class: 'scroll-x' }, [table]) : null,
      ],
    ),
  );
}

// ---------------------------------------------------------------- machines (several files)

function machines(main) {
  const label = state.label;
  const runs = state.runs.filter((r) => r.labels.has(label));
  const slots = [1, 2, 3, 7, 4, 5, 6, 8];
  const series = runs.map((r, k) => {
    const c = typeCtx(r, label);
    return {
      name: r.name,
      color: cssVar('--series-' + slots[k % slots.length]),
      width: 2,
      points: c.t.sizes.map((s) => ({ x: s, y: sizeSummary(c, s).geo })),
    };
  });
  const xs = [...new Set(runs.flatMap((r) => r.labels.get(label).sizes))].sort((a, b) => a - b);
  const chart = html('div', { class: 'chart-wrap' });
  const c0 = typeCtx(runs[0], label);
  main.append(
    section(
      `${label}: ${c0.refName} against ${compLabel(c0)}, per machine`,
      'One line per loaded file: geometric mean over the distributions of the ratio “other / reference”. Above 1: the reference is faster on that machine.',
      [legend(series), chart],
    ),
  );
  const draw = () => lineChart(chart, { series, xs, xFormat: fmtSize, xTipFormat: fmtSizeLong, yFormat: fmtRatio, yLog: true, refLine: 1, yTitle: 'ratio other / reference', height: 380 });
  requestAnimationFrame(draw);
  onResize(draw);
}

// ---------------------------------------------------------------- machine info

const MACHINE_NAMES = {
  cpu: 'CPU',
  'cpu details': 'Details',
  power: 'Power',
  arch: 'Architecture',
  os: 'OS',
  threads: 'Threads',
  ram: 'Memory',
  l2: 'L2 used',
  caches: 'Caches',
  page: 'Page',
  'target features': 'Target features',
  rustc: 'Compiler',
  commit: 'Commit',
  settings: 'Settings',
  core: 'Core',
};

function machine(main, run) {
  const dl = html('dl', { class: 'kv' });
  for (const [k, v] of Object.entries(run.machine)) dl.append(html('dt', { text: MACHINE_NAMES[k] || k }), html('dd', { text: v }));
  dl.append(html('dt', { text: 'File' }), html('dd', { text: run.file }));
  dl.append(html('dt', { text: 'Measurements' }), html('dd', { text: `${nf0.format(run.rows)} rows, ${run.labels.size} type${run.labels.size > 1 ? 's' : ''}` + (run.done ? ` · ${run.done}` : ' · bench not finished (or stopped)') }));
  const parts = [section('Machine', null, [dl])];
  if (run.failures.length) {
    parts.push(section(`${run.failures.length} failure${run.failures.length > 1 ? 's' : ''}`, 'A sort that panicked or a result not sorted.', [html('ul', { class: 'fail-list' }, run.failures.map((f) => html('li', { text: f })))]));
  }
  if (run.skips.length) {
    parts.push(section('Skipped sizes', 'More memory than the budget (BENCH_MEM_MB).', [html('ul', { class: 'fail-list muted' }, run.skips.map((s) => html('li', { text: `${s.label} · ${fmtSize(s.size)}` })))]));
  }
  main.append(...parts);
}

// ---------------------------------------------------------------- theme

function applyTheme() {
  let pref = 'auto';
  try {
    pref = localStorage.getItem('vbv-theme') || 'auto';
  } catch (e) {}
  const root = document.documentElement;
  if (pref === 'auto') delete root.dataset.theme;
  else root.dataset.theme = pref;
  const dark = pref === 'dark' || (pref === 'auto' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  root.dataset.resolvedTheme = dark ? 'dark' : 'light';
  const btn = document.getElementById('theme');
  btn.textContent = { auto: 'Theme: auto', light: 'Theme: light', dark: 'Theme: dark' }[pref];
}

function cycleTheme() {
  let pref = 'auto';
  try {
    pref = localStorage.getItem('vbv-theme') || 'auto';
  } catch (e) {}
  const next = { auto: 'light', light: 'dark', dark: 'auto' }[pref];
  try {
    localStorage.setItem('vbv-theme', next);
  } catch (e) {}
  applyTheme();
  render();
}

// ---------------------------------------------------------------- start

function start() {
  applyTheme();
  window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', () => {
    applyTheme();
    render();
  });
  document.getElementById('theme').addEventListener('click', cycleTheme);
  const input = document.getElementById('file');
  input.addEventListener('change', () => {
    openFiles(input.files);
    input.value = '';
  });
  document.getElementById('open').addEventListener('click', () => input.click());
  let depth = 0;
  const drop = document.getElementById('drop');
  window.addEventListener('dragenter', (e) => {
    e.preventDefault();
    depth++;
    drop.hidden = false;
  });
  window.addEventListener('dragleave', () => {
    if (--depth <= 0) {
      depth = 0;
      drop.hidden = true;
    }
  });
  window.addEventListener('dragover', (e) => e.preventDefault());
  window.addEventListener('drop', (e) => {
    e.preventDefault();
    depth = 0;
    drop.hidden = true;
    if (e.dataTransfer.files.length) openFiles(e.dataTransfer.files);
  });
  loadUi();
  const keep = state.run;
  loadFiles();
  state.run = Math.min(keep, Math.max(0, state.runs.length - 1));
  render();
}

start();
