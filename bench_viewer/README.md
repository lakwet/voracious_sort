# Bench voracious

A static page that reads the bench files of voracious_radix_sort and draws
them for humans. No server, no build, no dependency: open `index.html` in a
browser (double click), then drop one or more files on the page.

Files read: the `.csv` written by `make bench` / `make bench-type` /
`make bench-records` (in `resultsbenchs/<machine>/`), and the older
`bench-cpu-*.txt` (same raw lines). The `.txt` next to a `.csv` is the
human report and holds no raw lines.

Views:
- **Overview**: every type × size, geometric mean of the ratio
  "other sort / reference" over the distributions, and the distributions won;
- **Curves**: ns per key, duration of one sort or ratio, per size, one line
  per sort (all distributions or one), hover for the values;
- **Heatmap**: distributions × sizes for one type, the losing cells in red with
  the winning sort;
- **Detail**: one type and one size, every sort on every distribution
  (duration and ratio), sortable columns;
- **Losses**: every cell where the reference loses beyond the noise (band and floor);
- **Noise band** (every page, ± 3 % by default): a ratio within ± N % of 1 is measurement
  noise: the cell turns light grey, counts neither as won nor as lost, and the
  share of cases judged equal is shown; so is a gap under a floor in ns per
  sort (2 ns by default), whatever its ratio: 10 % of 100 ns is not 10 % of a
  second (sorted or constant arrays sort in a few ns per thousand keys);
- **Machines** (two files or more): the same type compared across machines;
- **Machine info**: CPU, caches, compiler, commit, failed checks.

The reference (v2 by default), the sort it is compared to (the best of the
others by default) and the sorts shown are chosen at the top. The loaded
files and the choices are kept in the browser (localStorage), nothing is
sent anywhere.

Files: `index.html`, `style.css`, `js/parse.js` (the file format),
`js/charts.js` (SVG charts, colour scale), `js/app.js` (views).
