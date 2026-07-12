# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0-dev] - 2026-07-12

### Added
- **Core Engine:** Built integrated Game Loop running strictly at 60 FPS using standard library timers.
- **Pure Software Rasterization:** Implemented Bresenham's line algorithm and Midpoint circle algorithm drawing directly to CPU index buffers.
- **Procedural Sound Synth:** Integrated an asynchronous multithreaded audio mixer with support for Sine, Square, Triangle, and Sawtooth oscillators.
- **Dynamic Palette Swapping:** Added remappable look-up tables via `pal()` and `palette()` bindings.
- **Matrix Scaling & Flipping:** Implemented full custom nearest-neighbor sampling for `spr()` and `sspr()` routines with Axis flipping flags.
- **Trigonometric Rotation:** Introduced `rspr()` function executing full 360-degree software transformations via inverse sampling.
- **Tilemaps System:** Implemented structural cell grade reading and rendering routines (`map`, `mget`, `mset`).
- **Save States Architecture:** Integrated local storage checkpoints utilizing data serialization via `serde_json`.
- **Pure Pixel ASCII Engine:** Mapped the full printable ASCII character block from 32 to 126 in binary 3x5 matrix glyphs for `print()` operations.
- **Diagnostics Panel:** Added a performance overlay toggle (`F12`) displaying dynamic rendering bar metrics and exact numerical FPS tracking.

### Fixed
- Fixed memory lifespan drops causing app icons to fail under specific Wayland desktop environment setups on Debian 12.
- Cleaned and removed redundant compiler warnings and unsat trait imports across `src/main.rs` and `src/audio/mod.rs`.
