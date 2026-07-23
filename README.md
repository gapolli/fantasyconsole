# FantasyConsole

[![Crates.io](https://img.shields.io/crates/v/fantasyconsole.svg)](https://crates.io/crates/fantasyconsole)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/gapolli/fantasyconsole/actions/workflows/rust.yml/badge.svg)](https://github.com/gapolli/fantasyconsole/actions)

A modular, multi-environment fantasy console game engine and integrated development environment (IDE) written in Rust. It executes retro cartridges inspired by **PICO-8** (`.p8`), **TIC-80** (`.tic`) standards, and its own high-performance chunk-based native binary format (`.fc`), powered by a high-performance CPU software rasterizer and a cross-platform SDL2 backend.

## ✨ Features

*   ⚡ **High Performance Engine:** Core runtime written entirely in pure Rust, utilizing raw pixel streaming textures to secure a locked and stable 60 FPS update rate.
*   🔀 **Polymorphic Dual-Core Architecture:** Dynamically adapts virtual hardware specs between the PICO-8 environment (128x128, 1:1 ratio) and the TIC-80 environment (240x136, 16:9 ratio, fully custom paletting).
*   📦 **Native Binary Format (`.fc`):** Features a proprietary chunk-based serialization format (`FCST` specification) that bundles metadata, target console mode headers, code payloads, and custom assets directly into unified, compiled distributions.
*   📐 **Vector & Software Rasterization:** Features an optimized scanline-filling renderer capable of drawing flat rectangles (`rect`/`rectfill`), full 360-degree sprite rotation (`rspr`), and n-sided regular polygons (`polygon`/`polyfill`) natively on the CPU.
*   🎵 **Chiptune Synthesis & Arpeggiator:** Multi-waveform asynchronous procedural audio mixer featuring a real-time, sample-accurate circular arpeggiator engine for advanced retro melody sequencing.
*   🛠️ **In-Engine Development Tooling:** Booting with the `--edit` flag unlocks native asset creation suites directly inside the runtime, providing inline tools for sprite composition (with queue-based Flood Fill) and standalone tilemap level design.
*   📡 **Asynchronous Netcode Skeleton:** Includes non-blocking UDP network messaging layers built for ultra-low latency inputs replication, paving the way for multi-player rollback synchronization.

---

## 📦 Installation

### Prerequisites

*   Rust 1.70+ toolchain ([rustup.rs](https://rustup.rs))
*   Native C compiler toolchain (GCC, Clang, or MSVC)
*   `pkg-config` utility and `libsdl2-dev` libraries (for native system compilation bindings, especially on Linux distros like Debian 12)

### Linux (Debian/Ubuntu) Setup

```bash
sudo apt update
sudo apt install -y build-essential libsdl2-dev libsdl2-ttf-dev pkg-config cmake
```

### Installation via Cargo

```bash
cargo install fantasyconsole
```

### Local Build and Manual Compilation

```bash
git clone https://github.com/gapolli/fantasyconsole.git
cd fantasyconsole
SDL_CONFIG=/usr/bin/sdl2-config cargo build --release
./target/release/fantasyconsole <cart.p8/.fc>
```

---

## 🚀 Usage

### Command Line Interface (CLI)

```bash
# Launch a standard plain-text cartridge file (PICO-8 or TIC-80)
fantasyconsole game.p8

# Launch a native, compiled chunk-based binary cartridge
fantasyconsole game.fc

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
| **Arrow Keys** | Player 1 Classic Directional D-Pad Controls / Map Editor Focus Navigation |
| **WASD Keys** | Player 2 Classic Directional D-Pad Controls |
| **Z / X** | Player 1 Action Buttons (A / B) / Map Editor Stamp Placement Command |
| **C / V** | Player 2 Action Buttons (A / B) |
| **Space** | Secondary Native Action Command / Alternative Tilemap Carimber |
| **`[` / `]`** | Decrement / Increment currently active Sprite ID asset within the IDE Workspace |
| **Enter** | System Start Command |
| **Escape** | System Menu / Safe Application Shutdown |
| **F1** | Toggle In-Engine **Sprite Editor** Workspace Canvas (Mouse & Palette Driven) |
| **F2** | Toggle In-Engine **Map Editor** Workspace Canvas (**Work In Progress / Under Construction**) |
| **F5** | Hot-Restart Cartridge Execution (*Soft Reset*) |
| **F6** | Restore State Dump Layout (*Savestate Load*) |
| **F7** | Compile current live execution environment into a native binary cartridge (`.fc`) |
| **F12** | Toggle Real-Time Performance Diagnostics (*Debug Overlay*) |

---

## 📁 Project Directory Structure

```text
fantasyconsole/
├── src/
│   ├── main.rs          # CLI entry point, IDE routing layers, hardware loops, and audio command unboxing
│   ├── lib.rs           # Core library export root engine registry
│   ├── cart/
│   │   ├── mod.rs       # Cartridge loading management and automated format detection router
│   │   ├── loader.rs    # Plain-text .p8 parser and proprietary .fc chunk-based serialization engine
│   │   └── png_loader.rs# LZ77 steganographic decoder for .p8.png image files
│   ├── vm/
│   │   ├── mod.rs       # Controller wrapper for the embedded Lua Virtual Machine
│   │   └── api.rs       # Native environment bridges, mouse input registers, and core API Lua bindings
│   ├── renderer/
│   │   ├── mod.rs       # Graphical rendering abstract orchestration pipeline and IDE public registry
│   │   └── editor.rs    # Integrated sprite (with Flood Fill) and tilemap creation interface structures
│   ├── network/
│   │   └── mod.rs       # Non-blocking asynchronous UDP network replication subsystem
│   └── audio/
│       ├── mod.rs       # Audio mixer callback manager and sample-accurate arpeggiator sequencer
│       └── sfx.rs       # Multi-waveform procedural sound effects phase-driven synthesizer
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
SDL_CONFIG=/usr/bin/sdl2-config cargo run -- game.p8

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
| **4.5**| Native `.fc` chunk format, multi-note arpeggiator and vector APIs | 🟢 Completed |
| **5** | In-Engine Tooling Suite (Native Sprite Editor) | 🟢 Completed |
| **5.1**| In-Engine Tooling Suite (Tilemap Editor) | 🟡 Under Construction |
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
