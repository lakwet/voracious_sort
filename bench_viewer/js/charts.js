// Hand-written SVG charts and colour scales, no dependency.
'use strict';

const SVG_NS = 'http://www.w3.org/2000/svg';

function svg(tag, attrs, children) {
  const e = document.createElementNS(SVG_NS, tag);
  for (const k in attrs || {}) e.setAttribute(k, attrs[k]);
  for (const c of children || []) if (c) e.appendChild(c);
  return e;
}

function html(tag, attrs, children) {
  const e = document.createElement(tag);
  for (const k in attrs || {}) {
    if (k === 'text') e.textContent = attrs[k];
    else if (k === 'html') e.innerHTML = attrs[k];
    else if (k.startsWith('on')) e.addEventListener(k.slice(2), attrs[k]);
    else if (attrs[k] !== undefined && attrs[k] !== null && attrs[k] !== false) e.setAttribute(k, attrs[k]);
  }
  for (const c of children || []) if (c !== null && c !== undefined) e.append(c);
  return e;
}

function cssVar(name) {
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim();
}

// ---------------------------------------------------------------- tooltip

const Tip = {
  el: null,
  show(evt, content) {
    if (!this.el) this.el = document.getElementById('tooltip');
    this.el.replaceChildren(content);
    this.el.hidden = false;
    const pad = 14;
    const r = this.el.getBoundingClientRect();
    let x = evt.clientX + pad;
    let y = evt.clientY + pad;
    if (x + r.width > window.innerWidth - 8) x = evt.clientX - r.width - pad;
    if (y + r.height > window.innerHeight - 8) y = evt.clientY - r.height - pad;
    this.el.style.left = Math.max(8, x) + 'px';
    this.el.style.top = Math.max(8, y) + 'px';
  },
  hide() {
    if (this.el) this.el.hidden = true;
  },
};

// ---------------------------------------------------------------- colours

function hexRgb(h) {
  h = h.replace('#', '');
  return [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16));
}

function mix(a, b, t) {
  const x = hexRgb(a);
  const y = hexRgb(b);
  return 'rgb(' + x.map((v, i) => Math.round(v + (y[i] - v) * t)).join(',') + ')';
}

function ramp(stops, t) {
  // stops: hex list from t = 0 to t = 1, evenly spaced.
  const s = Math.min(Math.max(t, 0), 1) * (stops.length - 1);
  const i = Math.min(Math.floor(s), stops.length - 2);
  return mix(stops[i], stops[i + 1], s - i);
}

// Ratio other / reference: > 1 the reference is faster (blue), < 1 the
// other sort is faster (red). Log scale, saturated at 2x either way.
function ratioColor(ratio) {
  if (!Number.isFinite(ratio)) return cssVar('--cell-empty');
  const dark = document.documentElement.dataset.resolvedTheme === 'dark';
  const mid = dark ? '#383835' : '#f0efec';
  const blue = dark ? [mid, '#1c4f8f', '#2f74cc', '#6da7ec'] : [mid, '#b7d3f6', '#5598e7', '#1c5cab'];
  const red = dark ? [mid, '#7d2a2a', '#c94443', '#f08a87'] : [mid, '#f6c2b9', '#e8605f', '#b02a2c'];
  const t = Math.log2(ratio);
  if (Math.abs(t) < 0.015) return mix(mid, mid, 0);
  return t > 0 ? ramp(blue, t) : ramp(red, -t);
}

function inkOn(bg) {
  const m = bg.match(/\d+/g);
  if (!m) return 'inherit';
  const [r, g, b] = m.map(Number).map((v) => {
    v /= 255;
    return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
  });
  const L = 0.2126 * r + 0.7152 * g + 0.0722 * b;
  return L > 0.33 ? '#0b0b0b' : '#f6f5f2';
}

// ---------------------------------------------------------------- scales

function logScale(d0, d1, r0, r1) {
  const a = Math.log(d0);
  const b = Math.log(d1);
  const f = (v) => r0 + ((Math.log(v) - a) / (b - a || 1)) * (r1 - r0);
  f.invert = (p) => Math.exp(a + ((p - r0) / (r1 - r0)) * (b - a));
  return f;
}

function linScale(d0, d1, r0, r1) {
  const f = (v) => r0 + ((v - d0) / (d1 - d0 || 1)) * (r1 - r0);
  return f;
}

function niceTicks(min, max, count) {
  const span = max - min || Math.abs(max) || 1;
  const step0 = span / count;
  const mag = Math.pow(10, Math.floor(Math.log10(step0)));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * mag).find((s) => span / s <= count) || 10 * mag;
  const out = [];
  for (let v = Math.ceil(min / step) * step; v <= max + step * 1e-9; v += step) out.push(+v.toPrecision(12));
  return out;
}

function logTicks(min, max) {
  const out = [];
  const lo = Math.floor(Math.log10(min));
  const hi = Math.ceil(Math.log10(max));
  for (let e = lo; e <= hi; e++) {
    for (const m of [1, 2, 5]) {
      const v = m * Math.pow(10, e);
      if (v >= min * 0.999 && v <= max * 1.001) out.push(v);
    }
  }
  if (out.length < 3) return niceTicks(min, max, 4).filter((v) => v > 0);
  return out;
}

// ---------------------------------------------------------------- line chart

// opts: { series: [{ name, color, width, dash, points: [{ x, y }] }],
//         xs: sorted x values, xFormat, yFormat, yLog, refLine (y value),
//         height, yTitle }
function lineChart(container, opts) {
  container.replaceChildren();
  const width = Math.max(container.clientWidth, 320);
  const height = opts.height || 400;
  const m = { top: 16, right: 24, bottom: 40, left: 72 };
  const all = opts.series.flatMap((s) => s.points.map((p) => p.y)).filter((v) => Number.isFinite(v) && v > 0);
  if (!all.length) {
    container.append(html('p', { class: 'empty-note', text: 'No value to plot.' }));
    return;
  }
  let yMin = Math.min(...all);
  let yMax = Math.max(...all);
  if (opts.refLine !== undefined) {
    yMin = Math.min(yMin, opts.refLine);
    yMax = Math.max(yMax, opts.refLine);
  }
  let y;
  let yTicks;
  if (opts.yLog) {
    yMin /= 1.08;
    yMax *= 1.08;
    y = logScale(yMin, yMax, height - m.bottom, m.top);
    yTicks = logTicks(yMin, yMax);
  } else {
    yMin = opts.refLine !== undefined ? Math.min(yMin, opts.refLine) : 0;
    yMin = Math.min(0, yMin);
    yMax *= 1.06;
    y = linScale(yMin, yMax, height - m.bottom, m.top);
    yTicks = niceTicks(yMin, yMax, 6);
  }
  const xs = opts.xs;
  const x =
    xs.length > 1
      ? logScale(xs[0], xs[xs.length - 1], m.left, width - m.right)
      : () => (m.left + width - m.right) / 2;

  const root = svg('svg', { width, height, class: 'chart', role: 'img' });
  // grid + axes
  const grid = svg('g', { class: 'grid' });
  for (const t of yTicks) {
    const py = y(t);
    if (py < m.top - 1 || py > height - m.bottom + 1) continue;
    grid.append(svg('line', { x1: m.left, x2: width - m.right, y1: py, y2: py }));
    const label = svg('text', { x: m.left - 8, y: py, class: 'tick', 'text-anchor': 'end', 'dominant-baseline': 'middle' });
    label.textContent = opts.yFormat(t);
    grid.append(label);
  }
  for (const v of xs) {
    const px = x(v);
    grid.append(svg('line', { x1: px, x2: px, y1: m.top, y2: height - m.bottom, class: 'vgrid' }));
    const label = svg('text', { x: px, y: height - m.bottom + 18, class: 'tick', 'text-anchor': 'middle' });
    label.textContent = opts.xFormat(v);
    grid.append(label);
  }
  root.append(grid);
  root.append(svg('line', { x1: m.left, x2: width - m.right, y1: height - m.bottom, y2: height - m.bottom, class: 'axis' }));
  if (opts.yTitle) {
    const t = svg('text', { x: 14, y: m.top + (height - m.top - m.bottom) / 2, class: 'axis-title', transform: `rotate(-90 14 ${m.top + (height - m.top - m.bottom) / 2})`, 'text-anchor': 'middle' });
    t.textContent = opts.yTitle;
    root.append(t);
  }
  if (opts.refLine !== undefined) {
    const py = y(opts.refLine);
    root.append(svg('line', { x1: m.left, x2: width - m.right, y1: py, y2: py, class: 'ref-line' }));
  }

  // lines (reference last, on top)
  const ordered = [...opts.series].sort((a, b) => (a.top ? 1 : 0) - (b.top ? 1 : 0));
  for (const s of ordered) {
    const pts = s.points.filter((p) => Number.isFinite(p.y) && p.y > 0);
    if (!pts.length) continue;
    const d = pts.map((p, i) => (i ? 'L' : 'M') + x(p.x).toFixed(1) + ',' + y(p.y).toFixed(1)).join('');
    root.append(svg('path', { d, fill: 'none', stroke: s.color, 'stroke-width': s.width || 2, 'stroke-dasharray': s.dash || null, 'stroke-linejoin': 'round', 'stroke-linecap': 'round' }));
    for (const p of pts) {
      root.append(svg('circle', { cx: x(p.x), cy: y(p.y), r: s.top ? 3.5 : 2.5, fill: s.color, stroke: cssVar('--surface'), 'stroke-width': 1.5 }));
    }
  }

  // hover: crosshair on the nearest size
  const cross = svg('line', { class: 'crosshair', y1: m.top, y2: height - m.bottom, visibility: 'hidden' });
  const dots = svg('g', {});
  root.append(cross, dots);
  const hit = svg('rect', { x: m.left - 10, y: m.top, width: width - m.left - m.right + 20, height: height - m.top - m.bottom, fill: 'transparent' });
  root.append(hit);
  hit.addEventListener('mousemove', (evt) => {
    const box = root.getBoundingClientRect();
    const px = evt.clientX - box.left;
    let best = xs[0];
    for (const v of xs) if (Math.abs(x(v) - px) < Math.abs(x(best) - px)) best = v;
    cross.setAttribute('x1', x(best));
    cross.setAttribute('x2', x(best));
    cross.setAttribute('visibility', 'visible');
    dots.replaceChildren();
    const items = [];
    for (const s of opts.series) {
      const p = s.points.find((q) => q.x === best);
      if (!p || !Number.isFinite(p.y)) continue;
      dots.append(svg('circle', { cx: x(best), cy: y(p.y), r: 5, fill: s.color, stroke: cssVar('--surface'), 'stroke-width': 2 }));
      items.push({ s, p });
    }
    items.sort((a, b) => a.p.y - b.p.y);
    const body = html('div', {}, [html('div', { class: 'tip-title', text: opts.xTipFormat ? opts.xTipFormat(best) : opts.xFormat(best) })]);
    for (const { s, p } of items) {
      body.append(
        html('div', { class: 'tip-row' + (s.top ? ' tip-ref' : '') }, [
          html('span', { class: 'swatch', style: `background:${s.color}` }),
          html('span', { class: 'tip-name', text: s.name }),
          html('span', { class: 'tip-val', text: (opts.tipFormat || opts.yFormat)(p.y, p) }),
        ]),
      );
    }
    Tip.show(evt, body);
  });
  hit.addEventListener('mouseleave', () => {
    cross.setAttribute('visibility', 'hidden');
    dots.replaceChildren();
    Tip.hide();
  });
  if (opts.onClickX) {
    hit.style.cursor = 'pointer';
    hit.addEventListener('click', (evt) => {
      const box = root.getBoundingClientRect();
      const px = evt.clientX - box.left;
      let best = xs[0];
      for (const v of xs) if (Math.abs(x(v) - px) < Math.abs(x(best) - px)) best = v;
      opts.onClickX(best);
    });
  }
  container.append(root);
}

// ---------------------------------------------------------------- diverging bar (inline)

// A small bar centred on ratio 1, log scale saturated at 4x.
function ratioBar(ratio) {
  const wrap = html('div', { class: 'rbar' });
  if (!Number.isFinite(ratio)) return wrap;
  const t = Math.max(-1, Math.min(1, Math.log2(ratio) / 2));
  const bar = html('div', { class: 'rbar-fill' });
  bar.style.background = ratioColor(ratio > 1 ? Math.max(ratio, 1.6) : Math.min(ratio, 0.62));
  if (t >= 0) {
    bar.style.left = '50%';
    bar.style.width = t * 50 + '%';
  } else {
    bar.style.left = 50 + t * 50 + '%';
    bar.style.width = -t * 50 + '%';
  }
  wrap.append(html('div', { class: 'rbar-mid' }), bar);
  return wrap;
}

function ratioLegend() {
  const stops = [0.5, 0.62, 0.77, 0.9, 1, 1.11, 1.3, 1.6, 2];
  const bar = html('div', { class: 'legend-ramp' });
  for (const r of stops) bar.append(html('span', { style: `background:${ratioColor(r)}` }));
  return html('div', { class: 'legend-div' }, [
    html('span', { class: 'legend-end', text: 'other sort faster (≤ 0.5)' }),
    bar,
    html('span', { class: 'legend-end', text: 'reference faster (≥ 2)' }),
  ]);
}
