# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com),
and this project adheres to Semantic Versioning.

## [0.1.0-dev] - 2026-07-22

### Added
- **Native Binary Container Specification (`.fc`):** Engineered a high-performance proprietary chunk-based serialization format utilizing the `FCST` structural layout protocol, enabling monolithic distribution of compiled game logic and asset blocks.
- **Scanline-Optimized Vector Primitives:** Developed a software rasterizer for filled rectangles (`rectfill`) and regular convex n-sided polygons (`polyfill`) implementing sorted horizontal edge intersection points on the CPU.
- **Hollow Geometric Boundaries APIs:** Added boundary-tracing APIs for hollow rectangles (`rect`) and rotational regular polygons (`polygon`) using an unrolled implementation over the core Bresenham line algorithm.
- **Chiptune Arpeggiator Sequencer:** Built a sample-accurate, real-time circular chord sequencer inside the `AudioCallback` thread loop, capable of parsing dynamic multi-frequency tables from Lua at 50ms ticks.
- **Asynchronous Audio Multi-frequency Payload:** Extended the `AudioCommand::PlaySfx` crossbeam-channel communication layer to transmit vector-based note packages safely between thread boundaries.
- **Live Cartridge Packer Utility:** Injected a runtime physical compiler shortcut mapping on the `F7` hotkey, enabling immediate extraction and binary serialization of active execution data structures directly to disc.
- **Linux Environment Portability:** Added formal support and initialization parameters to bypass CMake compilation overhead under Debian 12 by utilizing native system `pkg-config` hooks for SDL2 development binaries.

### Fixed
- Fixed a multi-threading data race condition by consolidating audio device mutex trancation (`lock()`) routines into a single atomic sequence per processed instruction.
- Fixed VRAM array out-of-bounds corruption risks by implementing rigid bit-mask clamping (`color_idx & 0x0F`) on pixel index retrieval loops.
- Fixed coordinate matrix calculation mismatches between the debug frame rate performance overlay and dynamic core viewport limits.
- Silenced compiler warning logs regarding untracked virtual arguments inside plain-text syntax loops.

## [0.1.0-dev] - 2026-07-12

### Added
- **Polymorphic Dual-Core System:** Re-engineered the foundational `BackendState` block to dynamically adapt memory limits and viewport canvas scaling criteria between **PICO-8** standard specs (128x128, 1:1 ratio) and **TIC-80** standard widescreen specs (240x136, 16:9 ratio).
- **Trigonometric Rotation Subsystem:** Introduced the raw CPU software `rspr()` rendering api executing full 360-degree matrix transformations via inverse pixel sampling to eliminate rasterization holes.
- **Pure Pixel ASCII Engine:** Expanded the binary lookup font engine to map the complete printable ASCII character block (from index 32 to 126) in custom high-readability 3x5 matrix glyphs for native multi-color `print()` routines.
- **Integrated Tooling Interface Structure:** Laid the software foundation for the upcoming native In-Engine creation suite (Sprite, Map, and Sound tracker editors) triggered via the new standalone execution CLI parameter `--edit`.
- **Enhanced Local Multi-player Matrix:** Expanded the input system arrays into full multidimensional maps, providing distinct physical controller mapping bindings (Player 0 on Arrow Keys, Player 1 on classic `WASD` keys) supporting up to 4 concurrent players.
- **Asynchronous UDP Network Subsystem:** Implemented a non-blocking network core configuration (`src/network/mod.rs`) capable of compressing structural virtual button frames into lightweight bitmask packet payloads over UDP.
- **Advanced Steganographic LZ77 Decoder:** Built a precise text segment parsing routine into `png_loader.rs` to intercept hidden payload buffers and reverse custom compression headers.
- **Look-Up Table Palette Swapping:** Implemented dynamic index color routing arrays allowing games to execute complex lookup switches on-the-fly (`pal` / `palette`).
- **Diagnostics Dashboard Overhaul:** Refined the `F12` overlay rendering layers to place exact numerical frame rate readouts (`xxFPS`) flush right with a surgical 2-pixel margin constraint, while stretching the performance health bar across all available leftover visor space.

### Fixed
- Fixed visual blending artifacts on modern desktop environments by adjusting the bitmask layout structure of the 3x5 font glyph array (such as sharpening the top stem profile separating 'F' from 'P').
- Fixed Wayland and modern X11 architecture display device context drops under Debian 12 environments by pinning the application surface reference layers statically into stack routines during hardware cycles.
- Fixed a boundary evaluation conflict inside the plain-text parser file loop (`src/cart/loader.rs`) allowing the engine to successfully ingest uneven or partial graphics hexadecimal blocks.
- Cleared and stripped redundant compiler check indicators, unsat trait dependencies, and dangling imports across `src/main.rs` and `src/audio/mod.rs`.
