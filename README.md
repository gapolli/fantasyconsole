# FantasyConsole

[![Crates.io](https://img.shields.io/crates/v/fantasyconsole.svg)](https://crates.io/crates/fantasyconsole)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/gapolli/fantasyconsole/actions/workflows/rust.yml/badge.svg)](https://github.com/gapolli/fantasyconsole/actions)

A modular, multi-environment fantasy console game engine and integrated development environment (IDE) written in Rust. It executes retro cartridges inspired by both **PICO-8** (`.p8`) and **TIC-80** (`.tic`) standards, powered by a high-performance CPU software rasterizer and a cross-platform SDL2 backend.

## ✨ Features

*   ⚡ **High Performance Engine:** Core runtime written entirely in pure Rust, utilizing raw pixel streaming textures to secure a locked and stable 60 FPS update rate.
*   🔀 **Polymorphic Dual-Core Architecture:** Dynamically adapts virtual hardware specs between the PICO-8 environment (128x128, 1:1 ratio) and the TIC-80 environment (240x136, 16:9 ratio, fully custom paletting).
*   🛠️ **In-Engine Development Tooling:** Booting with the `--edit` flag unlocks native pixel-art creation suites directly inside the runtime, providing inline tools for sprite composition, map painting, and tracker audio editing.
*   🌀 **Trigonometric Rotation & Compositing:** Features a native inverse-sampling software renderer capable of full 360-degree sprite rotation (`rspr`), sub-pixel scaling, lookup-table palette swapping, and multi-player axis flipping.
*   📡 **Asynchronous Netcode Skeleton:** Includes non-blocking UDP network messaging layers built for ultra-low latency inputs replication, paving the way for multi-player rollback synchronization.
*   📦 **Zero Configuration Deployment:** Standalone execution footprints utilizing statically bundled embedded SDL2 development libraries.

---

## 📦 Installation

### Prerequisites

*   Rust 1.70+ toolchain ([rustup.rs](https://rustup.rs))
*   Native C compiler toolchain (GCC, Clang, or MSVC)
*   SDL2 development libraries (optional if using automated static building configurations)

### Installation via Cargo

```bash
cargo install fantasyconsole
```

### Local Build and Manual Compilation

```bash
git clone https://github.com/gapolli/fantasyconsole.git
cd fantasyconsole
cargo build --release
./target/release/fantasyconsole <cart.p8>
```

---

## 🚀 Usage

### Command Line Interface (CLI)

```bash
# Launch a standard game cartridge file (PICO-8 or TIC-80)
fantasyconsole game.p8

# Open the Integrated Creation Suite (Sprite, Map, and Audio Editors)
fantasyconsole --edit game.p8

# Boot the runtime directly in fullscreen mode
fantasyconsole --fullscreen game.p8

# Completely disable the audio subsystem
fantasyconsole --no-audio game.p8

# Define a custom graphics window multiplier scale factor (e.g., 4x)
fantasyconsole --scale 4 game.p8

# Boot in debug mode with hot-reload listeners active
fantasyconsole --debug game.p8
```

### Default Layout and System Hotkeys

| Key Bind | Virtual Hardware Function |
| :--- | :--- |
| **Arrow Keys** | Player 1 Classic Directional D-Pad Controls |
| **WASD Keys** | Player 2 Classic Directional D-Pad Controls |
| **Z / X** | Player 1 Action Buttons (A / B) |
| **C / V** | Player 2 Action Buttons (A / B) |
| **Enter** | System Start Command |
| **Escape** | System Menu / Safe Application Shutdown |
| **F5** | Hot-Restart Cartridge Execution (*Soft Reset*) |
| **F12** | Toggle Real-Time Performance Diagnostics (*Debug Overlay*) |

---

## 📁 Project Directory Structure

```text
fantasyconsole/
├── src/
│   ├── main.rs          # CLI entry point, ASCII font blitter, and hardware loops
│   ├── lib.rs           # Core library export root engine registry
│   ├── cart/
│   │   ├── mod.rs       # Cartridge loading management and automated format detection
│   │   ├── loader.rs    # Syntactic plain-text parser engine for .p8 files
│   │   └── png_loader.rs# LZ77 steganographic decoder for .p8.png image files
│   ├── vm/
│   │   ├── mod.rs       # Controller wrapper for the embedded Lua Virtual Machine
│   │   └── api.rs       # Native environment bridges, 360 rspr mechanics, and API bindings
│   ├── renderer/
│   │   ├── mod.rs       # Graphical rendering abstract orchestration pipeline
│   │   └── editor.rs    # Integrated sprite, tilemap, and chiptune audio creation interfaces
│   ├── network/
│   │   └── mod.rs       # Non-blocking asynchronous UDP network replication subsystem
│   └── audio/
│       ├── mod.rs       # Audio mixer backend stream manager orchestrator
│       └── sfx.rs       # Multi-waveform procedural sound effects synthesizer
├── examples/
│   ├── games/           # Structured reference game script assets (.p8)
│   └── images/          # Paired high-fidelity screen capture assets (.png)
├── Cargo.toml
├── LICENSE
└─ README.md
```

---

## 🏗️ Development Tooling Commands

```bash
# Rapid compilation running in development mode
cargo build

# Highly optimized code compilation target ready for deployment (Release)
cargo build --release

# Compilation including all optional ecosystem features and flags
cargo build --release --features "audio,debug,hotreload"

# Execute the complete automated unit and integration test suites
cargo test

# Validate source code structural formatting constraints
cargo fmt --check

# Execute strict static analysis and codebase linting via Clippy
cargo clippy
```

---

## 🔌 API Implementation Roadmap

| Phase | Technological Scope | Development Status |
| :---: | :--- | :---: |
| **1** | Graphics primitives, matrix camera, clipping, and text parser | 🟢 Completed |
| **2** | Multi-channel audio mixer, procedural synthesis, and tilemaps | 🟢 Completed |
| **3** | Dynamic steganographic LZ77 decoder and pure pixel ASCII engine | 🟢 Completed |
| **4** | Polymorphic TIC-80 core integration and 360° software `rspr` | 🟢 Completed |
| **5** | In-Engine Tooling Suite (Native Sprite, Tilemap, and Sound Editors) | 🟡 In Progress |
| **6** | Rollback input netcode synchronization over UDP sockets | ⚪ Planned |

---

## 🤝 Contribution Guidelines

Contributions geared toward refining the virtual machine engine and integrated creation utilities are always welcome! Review the `CONTRIBUTING.md` manifest for explicit styling standards.

1.  Perform a manual **Fork** of this project repository.
2.  Initialize an isolated working branch: `git checkout -b feature/amazing-feature`.
3.  Validate stability via native check tools: `cargo test` and `cargo clippy`.
4.  Stage and commit your changes: `git commit -m 'Add: amazing feature'`.
5.  Push the active branch to your remote fork: `git push origin feature/amazing-feature`.
6.  Open a clear **Pull Request** overview detailing your design decisions.

---

## 📄 License

This virtual machine architecture is open-source software distributed matching compliance standards under the terms of the MIT License. Review the full `LICENSE` file text for complete parameters.

---

## 🙏 Acknowledgments and Credits

*   [Lexaloffle](https://www.lexaloffle.com/) — The innovative minds who created the original PICO-8 fantasy console.
*   [TIC-80](https://tic80.com) — For inspiring the widescreen cross-platform open-source dev standard.
*   [mlua](https://github.com/khvzak/mlua) — Exceptional, secure, and fast Lua embedding layer bindings for Rust.
*   [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) — Robust, clean, and safe idiomatic Rust abstraction over cross-platform native SDL2 hardware.

***
*Developed with ❤️ for the global preservation and thriving scene of virtual retro hardware engines.*
