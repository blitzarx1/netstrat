# netstrat
**netstrat** is a strategy backtesting and visualization tool using [egui](https://github.com/emilk/egui) for ui

<img width="1728" alt="Screen Shot 2022-08-23 at 14 06 58" src="https://user-images.githubusercontent.com/32969427/186132111-9bce80c2-fe13-4208-9d02-2ede064d5ed5.png">

<img alt="Screen Shot 2022-10-21 at 13 23 18" src="https://user-images.githubusercontent.com/32969427/197162032-ba3efb31-af82-4f41-ad0b-25de5bc4834e.png">

### Status
The project is under active development. No stable version yet. First release should include:

<pre>
Tool                                                          Completion
------------------------------------------------------------+------------
- binance client for tick data download and visualization   |   90%
- graph based trading strategy constructor                  |   10%
- backtesting tool                                          |    0%
- graph analysis tool to support ML based trading strategies|   40%
</pre>
Short term plan is to build and use [my own egui based implementation for graph visualizaton](https://github.com/blitzarx1/egui_graph) and get rid of graphviz dependency

<img width="801" alt="Screenshot 2023-04-08 at 14 33 14" src="https://user-images.githubusercontent.com/32969427/230716665-b86ac6c5-b94f-4354-85c8-5d55dd3d380f.png">

### Depedencies
You need to have [graphviz binary](https://graphviz.org/download/) installed

### Build
```bash
# windows 10
cargo build --target=x86_64-pc-windows-gnu --release
```
```bash
# Apple intel
cargo build --target=x86_64-apple-darwin --release
```
```bash
# Apple silicon
cargo build --target=aarch64-apple-darwin --release
```
