# FantasyConsole

[![Crates.io](https://img.shields.io/crates/v/fantasyconsole.svg)](https://crates.io/crates/fantasyconsole)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://github.com/gapolli/fantasyconsole/actions/workflows/rust.yml/badge.svg)](https://github.com/gapolli/fantasyconsole/actions)

A fantasy console virtual machine written in Rust that executes games inspired by PICO-8 (using the `.p8` format) powered by a high-performance SDL2 backend.

## ✨ Features

*   ⚡ **High Performance:** Core engine written entirely in pure Rust, ensuring a locked and stable 60 FPS update rate.
*   🎮 **PICO-8 Compatibility:** Progressive API implementation built for stable execution of existing community cartridges.
*   🖥️ **Cross-Platform:** Native support out-of-the-box for Windows, macOS, and Linux (Debian/Ubuntu).
*   📦 **Zero Dependencies:** Simplified standalone deployment using statically bundled embedded SDL2 development libraries.
*   🔧 **Extensible:** Fully decoupled modular internal architecture, making it seamless to introduce custom emulated hardware systems.

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
# Run a standard game cartridge file
fantasyconsole cart.p8

# Boot the runtime directly in fullscreen mode
fantasyconsole --fullscreen cart.p8

# Completely disable the audio subsystem
fantasyconsole --no-audio cart.p8

# Define a custom graphics window multiplier scale factor (e.g., 4x)
fantasyconsole --scale 4 cart.p8

# Boot in debug mode with hot-reload listeners active
fantasyconsole --debug cart.p8
```

### Default Layout and System Hotkeys

| Key Bind | Virtual Hardware Function |
| :--- | :--- |
| **Arrow Keys / WASD** | Classic Directional D-Pad Game Controls |
| **Z / X / Spacebar** | Primary Hardware Action Buttons (A / B) |
| **Enter** | System Start Command |
| **Escape** | System Menu / Safe Application Shutdown |
| **F5** | Hot-Restart Cartridge Execution (*Soft Reset*) |
| **F12** | Toggle Real-Time Performance Diagnostics (*Debug Overlay*) |

---

## 🛠️ Minimal Cartridge Example

Below is the standard structural layout of a functional `.p8` game file used to evaluate primitive rendering layers:

```lua
-- hello.p8
function _init()
  print("Hello, FantasyConsole!")
end

function _update()
  if btn(4) then -- Evaluates if virtual hardware button Z is held down
    cls(0)
    circfill(64, 64, 20, 7)
  end
end

function _draw()
  cls(1)
  print("Hold Z to draw!", 10, 50, 7)
end
```

To boot this explicit sample file locally, pass it as a terminal argument:
```bash
fantasyconsole hello.p8
```

---

## 📁 Project Directory Structure

```text
fantasyconsole/
├── src/
│   ├── main.rs          # CLI application entry point wrapper
│   ├── lib.rs           # Core library export root engine registry
│   ├── cart/
│   │   ├── mod.rs       # Cartridge abstraction loading module manager
│   │   ├── loader.rs    # Syntactic plain-text parser engine for .p8 files
│   │   └── data.rs      # Internal structured game data definitions
│   ├── vm/
│   │   ├── mod.rs       # Controller wrapper for the embedded Lua Virtual Machine
│   │   ├── api.rs       # Native environment bridges and virtual hardware bindings
│   │   └── callbacks.rs # Primary game loop execution pipeline hooks (_init, _update, _draw)
│   ├── renderer/
│   │   ├── mod.rs       # Graphical rendering abstract orchestration pipeline
│   │   ├── sdl2_impl.rs # Concrete backend engine drawing wrapper for SDL2
│   │   └── palette.rs   # Management arrays for dynamic indexed look-up color tables
│   ├── input/
│   │   ├── mod.rs       # Processing handlers for peripheral input events
│   │   └── mapper.rs    # Logical keystroke mapping translators: Keyboard -> Virtual Buttons
│   └── audio/
│       ├── mod.rs       # Audio mixer backend stream manager orchestrator
│       └── sfx.rs       # Multi-waveform procedural sound effects synthesizer
├── tests/
│   ├── carts/           # Collection of test cartridge assets for internal evaluation
│   └── integration.rs   # Suite for end-to-end integration and platform testing
├── examples/
│   └── minimal.p8       # Minimal reference game cartridge asset
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
| **1** | Graphics primitives, user inputs, matrix camera, clipping, and text parser | 🟢 Completed |
| **2** | Multi-channel audio mixer and procedural sound synthesis oscillators | 🟢 Completed |
| **3** | Dynamic steganographic `.p8.png` decoder and inline software debugger | 🟡 In Progress |
| **4** | Multi-slot memory state persistence (*save states*) and network mechanics | 🟢 Completed |

---

## 🤝 Contribution Guidelines

Contributions geared toward refining the virtual machine engine are always welcome! Review the `CONTRIBUTING.md` manifest for explicit styling standards.

### Recommended Pull Request Workflow

1. Perform a manual **Fork** of this project repository.
2. Initialize an isolated working branch: `git checkout -b feature/amazing-feature`.
3. Validate stability via native check tools: `cargo test` and `cargo clippy`.
4. Stage and commit your changes: `git commit -m 'Add: amazing feature'`.
5. Push the active branch branch to your remote fork: `git push origin feature/amazing-feature`.
6. Open a clear **Pull Request** overview detailing your design decisions.

---

## 📄 License

This virtual machine architecture is open-source software distributed matching compliance standards under the terms of the MIT License. Review the full [LICENSE](LICENSE) file text for complete parameters.

---

## 🙏 Acknowledgments and Credits

*   [Lexaloffle](https://www.lexaloffle.com/) — The innovative minds who created the original PICO-8 fantasy console.
*   [mlua](https://github.com/khvzak/mlua) — Exceptional, secure, and fast Lua embedding layer bindings for Rust.
*   [rust-sdl2](https://github.com/Rust-SDL2/rust-sdl2) — Robust, clean, and safe idiomatic Rust abstraction over cross-platform native SDL2 hardware.
*   The PICO-8 Community — For providing years of retro game design assets, testing tools, and continuous inspiration.

***
*Developed with ❤️ for the global preservation and thriving scene of virtual retro hardware engines.*
