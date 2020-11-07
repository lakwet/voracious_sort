# Peeka sort vs Regions sort

Computer: Ryzen 9 3950x (16 physical cores, 32 threads), 32GB RAM DDR4, MB X570 TUF Gaming

Peeka sort:
- Language: Rust
- Voracious radix sort v1.0.0
- threads: 16
```
Please note that with more than 16 threads given to Rayon threadpool,
performance decreases.
```
```
Peeka sort can sort all types supported by the crate. A dedicated implementation
for unsigned integer should be faster.
```

Regions sort: Clone from the [repository](https://github.com/omarobeya/parallel-inplace-radixsort).
- Language: C++
- threads: 32

## Results
<table>
  <thead>
    <tr><td colspan = 2></td><td colspan = 2>u32</td><td colspan = 2>u64</td></tr>
    <tr>
      <td>Array size</td><td>Distribution</td>
      <td>Peeka sort (5 runs)</td><td>Regions sort (1 run)</td>
      <td>Peeka sort (5 runs)</td><td>Regions sort (1 run)</td>
    </tr>
  </thead>
  <tbody>
    <tr><td rowspan = 2>1_000_000</td>
      <td>Unif</td><td>3582us</td><td>15ms</td><td>4538us</td><td>15ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>3781us</td><td>4ms</td><td>4445us</td><td>4ms</td></tr>
    <tr><td rowspan = 2>5_000_000</td>
      <td>Unif</td><td>10677us</td><td>17ms</td><td>13904us</td><td>21ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>9472us</td><td>8ms</td><td>15403us</td><td>9ms</td></tr>
    <tr><td rowspan = 2>10_000_000</td>
      <td>Unif</td><td>15561us</td><td>20ms</td><td>24992us</td><td>30ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>16618us</td><td>15ms</td><td>25392us</td><td>24ms</td></tr>
    <tr><td rowspan = 2>20_000_000</td>
      <td>Unif</td><td>27316us</td><td>30ms</td><td>46706us</td><td>45ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>26586us</td><td>31ms</td><td>45298us</td><td>48ms</td></tr>
    <tr><td rowspan = 2>50_000_000</td>
      <td>Unif</td><td>69710us</td><td>63ms</td><td>111295us</td><td>133ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>62739us</td><td>66ms</td><td>106242us</td><td>123ms</td></tr>
    <tr><td rowspan = 2>100_000_000</td>
      <td>Unif</td><td>117199us</td><td>143ms</td><td>194142us</td><td>254ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>113328us</td><td>135ms</td><td>193344us</td><td>246ms</td></tr>
    <tr><td rowspan = 2>200_000_000</td>
      <td>Unif</td><td>264979us</td><td>266ms</td><td>465165us</td><td>514ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>249163us</td><td>275ms</td><td>442913us</td><td>498ms</td></tr>
    <tr><td rowspan = 2>300_000_000</td>
      <td>Unif</td><td>411247us</td><td>402ms</td><td>706385us</td><td>769ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>420591us</td><td>393ms</td><td>691201us</td><td>749ms</td></tr>
    <tr><td rowspan = 2>400_000_000</td>
      <td>Unif</td><td>489960us</td><td>556ms</td><td>980324us</td><td>1013ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>485013us</td><td>514ms</td><td>969742us</td><td>996ms</td></tr>
    <tr><td rowspan = 2>500_000_000</td>
      <td>Unif</td><td>632402us</td><td>711ms</td><td>1310269us</td><td>1278ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>627480us</td><td>641ms</td><td>1244246us</td><td>1238ms</td></tr>
    <tr><td rowspan = 2>600_000_000</td>
      <td>Unif</td><td>771349us</td><td>870ms</td><td>1551487us</td><td>1536ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>730904us</td><td>766ms</td><td>1522778us</td><td>1479ms</td></tr>
    <tr><td rowspan = 2>700_000_000</td>
      <td>Unif</td><td>925669us</td><td>1013ms</td><td>1864540us</td><td>1796ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>890647us</td><td>893ms</td><td>1840039us</td><td>1722ms</td></tr>
    <tr><td rowspan = 2>800_000_000</td>
      <td>Unif</td><td>1075767us</td><td>1124ms</td><td>2153855us</td><td>2063ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>1012213us</td><td>1018ms</td><td>2121924us</td><td>1960ms</td></tr>
    <tr><td rowspan = 2>900_000_000</td>
      <td>Unif</td><td>1168676us</td><td>1290ms</td><td>2465361us</td><td>2325ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>1172123us</td><td>1141ms</td><td>2427304us</td><td>2192ms</td></tr>
    <tr><td rowspan = 2>1_000_000_000</td>
      <td>Unif</td><td>1328195us</td><td>1369ms</td><td>2798789us</td><td>2585ms</td></tr>
    <tr>
      <td>Unif 10^9</td><td>1316730us</td><td>1281ms</td><td>2763619us</td><td>2422ms</td></tr>
  </tbody>
</table>
