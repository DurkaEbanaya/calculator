# mscalculator-rust

A Rust port of the Windows Calculator **Standard mode**, preserving the 2020 dark UI look and running on macOS (Intel & Apple Silicon) and Linux.

This is part of a fork of [microsoft/calculator](https://github.com/microsoft/calculator). The goal is to keep the **2020 interface** while implementing the functionality in fresh Rust code.

## Current status

- Standard mode only (scientific / programmer / converter / graphing are future work).
- Dark theme matching the Windows 10 2020 Calculator look.
- Cross-platform GUI using [egui](https://github.com/emilk/egui) + [eframe](https://github.com/emilk/egui/tree/master/crates/eframe).
- Memory functions (MC, MR, M+, M−, MS).
- macOS universal binary (`x86_64` + `arm64`).
- Linux build supported via the same codebase.

## Build

### macOS

```bash
cd rust-port
cargo build --release
```

This produces a native binary for the current architecture. To build a universal binary for both Intel and Apple Silicon:

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
lipo -create -output dist/mscalculator-rust-universal \
  target/x86_64-apple-darwin/release/mscalculator-rust \
  target/aarch64-apple-darwin/release/mscalculator-rust
```

Run it:

```bash
./dist/mscalculator-rust-universal
```

### Linux

```bash
rustup target add x86_64-unknown-linux-gnu
cd rust-port
cargo build --release --target x86_64-unknown-linux-gnu
./target/x86_64-unknown-linux-gnu/release/mscalculator-rust
```

> Linux may need X11/Wayland development libraries (`libxcb`, `libxkbcommon`, etc.) because eframe uses `winit`.

## Project structure

- `src/calculator.rs` — core calculator logic (Standard mode).
- `src/main.rs` — egui/eframe user interface and the 2020 dark theme.

## License

MIT, matching the upstream Microsoft Calculator license.
