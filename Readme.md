# netstrat
**netstrat** is a strategy backtesting and visualization tool

<img width="1728" alt="Screen Shot 2022-08-23 at 14 06 58" src="https://user-images.githubusercontent.com/32969427/186132111-9bce80c2-fe13-4208-9d02-2ede064d5ed5.png">

<img width="934" alt="Screenshot 2023-04-08 at 00 08 30" src="https://user-images.githubusercontent.com/32969427/230716449-e3346837-411f-4b3e-a7ad-a61e3b4d8fd0.png">

<img alt="Screen Shot 2022-10-21 at 13 23 18" src="https://user-images.githubusercontent.com/32969427/197162032-ba3efb31-af82-4f41-ad0b-25de5bc4834e.png">

### Status
The project is under active development. No stable version yet. First release should include:
- binance client for tick data download and visualization;
- graph based trading strategy constructor;
- backtesting tool;

Short term plan is to build and use egui based implementation for graph visualizaton and get rid of graphviz dependency

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
