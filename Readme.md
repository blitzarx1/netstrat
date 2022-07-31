# netstrat
**netstrat** is a strategy backtesting and visualization tool

![Screen Shot 2022-07-31 at 14 40 17](https://user-images.githubusercontent.com/32969427/182022345-0fd69e39-8f62-4aa0-a1cf-511cc2c36c21.png)

## executables
There are prebuilded versions for windows and mac supplied with every [release](https://github.com/qzarx1/netstrat/releases).

### build
```bash
# windows 10
cargo build --target=x86_64-pc-windows-gnu --release
```
```bash
# apple intel
cargo build --target aarch64-apple-darwin --release
```
```bash
# apple silicon
cargo build --release --target x86_64-apple-darwin --release
```
