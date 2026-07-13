# FantasyConsole - Technical Specification

## Overview

FantasyConsole is a polymorphic, open-source fantasy virtual machine and integrated development environment (IDE) built in Rust. It executes codebases matching both **PICO-8** (`.p8`) and **TIC-80** (`.tic`) specifications without infringing on intellectual property boundaries. The engine implements a decoupled software-rendered graphics rasterizer mapped onto a hardware abstraction layer via an SDL2 backend.

**Version:** 0.1.0-dev  
**License:** MIT  
**Repository:** https://github.com/gapolli/fantasyconsole

---

## Hardware Architecture & Constraints

The engine dynamically reshapes its core structural allocations at boot time depending on the loaded cartridge metadata criteria:

| Parameter | PICO-8 Core Mode Specification | TIC-80 Core Mode Specification |
| :--- | :--- | :--- |
| **Display Resolution** | 128×128 pixels (1:1 Ratio) | **240×136 pixels** (16:9 Widescreen Ratio) |
| **Color Lookup (LUT)** | 16 fixed indexed slots | **16 fully custom dynamic RAM registers** |
| **Addressable VRAM** | 8 KB screen buffer array | 16.32 KB screen buffer array |
| **Graphics Asset Sheet**| 128×128 pixel sprite matrix | **Dual-bank 128×256 pixel asset matrix** |
| **Tilemap Array Bounds**| 128×64 matrix block cells | **240×136 matrix block cells** |
| **Input Hardware Caps** | 4 local/remote players × 6 buttons | 4 local/remote players × 6 buttons |
| **Virtual Address Space**| 32 KB structural layout mapping | 96 KB linear memory mapping layout |
| **Target Refresh Rate** | Stable 60 Frames Per Second (FPS) | Stable 60 Frames Per Second (FPS) |

### Native Hardware Colors Index Mapping
```text
[0] #000000  [1] #1D2B53  [2] #7E2553  [3] #008751
[4] #AB5236  [5] #5F574F  [6] #C2C3C7  [7] #FFF1E8
[8] #FF004D  [9] #FFA300  [A] #FFEC27  [B] #00E436
[C] #29ADFF  [D] #83769C  [E] #FF77A8  [F] #FFCCAA
```

---

## Runtime API Surface (Dual-Core Matrix)

### Graphical Rasterization Core

| Function Signature | Subsystem Behaviour & Calculations |
| :--- | :--- |
| `cls(color)` | Flushes target coordinate layers with an implicit color index respecting scissors. |
| `pset(x, y, color)` | Injects a color index downstream after evaluating world-space offsets and active clipping boundaries. |
| `line(x0, y0, x1, y1, color)` | Evaluates and plots linear pixel steps executing an unrolled CPU Bresenham's Line Algorithm. |
| `circ(x, y, r, color)` | Renders a hollow circular perimeter utilizing Midpoint Integer Circle calculus. |
| `circfill(x, y, r, color)` | Rasterizes a filled circle shape executing parallel symmetrical horizontal sweep lines. |
| `spr(n, x, y, fx, fy)` | Blits a standard 8×8 graphic block applying safe index bounds-checking and coordinate reflection flags. |
| `sspr(sx, sy, sw, sh, dx, dy, dw, dh, fx, fy)` | Scales variable asset sections down into the drawing buffer via pure *Nearest-Neighbor* layout interpolation. |
| `rspr(n, dx, dy, angle, sx, sy)` | **Rotated Sprite:** Executes 360° trigonometric transformations using inverse coordinate sampling to eliminate gaps. |
| `map(mx, my, sx, sy, w, h)` | Traverses structural memory cells, laying tile blocks onto the pixel matrix. |
| `mget(x, y)` | Returns the unique tile ID index registered inside a specific map cell coordinate. |
| `mset(x, y, v)` | Modifies the current tile ID index mapping entry located inside the target map grid slot. |
| `pal(c0, c1)` | Updates the lookup table data stream, re-routing color indices on-the-fly during screen composition. |
| `pal()` | Clears the look-up table re-routing array back to native hardware palette constants. |
| `clip(x, y, w, h)` | Constructs an explicit hardware coordinate scissoring bounding-box protecting adjacent VRAM segments. |
| `camera(x, y)` | Offsets the absolute drawing coordinates array calculation layer by injecting camera transformation vectors. |
| `print(str, x, y, color)` | Decodes text strings into direct matrix graphics using a custom built-in 3x5 bitmask binary font engine. |

### Peripherals & Input Management

| Function Signature | Subsystem Behaviour & Calculations |
| :--- | :--- |
| `btn(button, [player])` | Polls the real-time boolean flag of a virtual controller mapping. Supports 4 players (`0` to `3`). |

### Asynchronous Audio Mixer

| Function Signature | Subsystem Behaviour & Calculations |
| :--- | :--- |
| `sfx(n, channel)` | Pipes a real-time procedural waveform command (Sine, Square, Triangle, Sawtooth) down to a separate audio thread mixer. |
| `music(track)` | Synchronizes structural audio loop tracker patterns over available virtual audio streams. |

### Structural Memory Operations

| Function Signature | Subsystem Behaviour & Calculations |
| :--- | :--- |
| `save_game(slot)` | Serializes the state of virtual arrays, state flags, and memories directly into compressed local JSON assets. |
| `load_game(slot)` | Parses existing persistence JSON save checkpoints, immediately restoring the virtual state registers. |

---

## File Layout Frameworks

### 1. Plain-Text Layout Input: `.p8` Format
Operates via explicit syntax bracket demarcations to separate virtual hardware modules:
*   `__lua__`: Houses core software script game program state logic.
*   `__gfx__`: Holds inline hexadecimal character data stream mappings (0-f) that decode down into sprite matrices.
*   `__map__`: Layout tables housing grid index cell designations for constructing environment scenes.

### 2. Compressed Steganographic Framework: `.p8.png` Format
A compact distribution layer where game scripts and layout streams are embedded directly within the two least significant bits (LSB) of color channels inside a regular 160×205 PNG pixel layout canvas shell.

---

## Application Runtime Lifecycle Flow

```text
       Cold Boot Execution
                │
     ┌──────────▼──────────┐
     │  _init() Lifecycle  │ ◄─── Once per environment initialization
     └──────────┬──────────┘
                │
  ┌─────────────►─────────────┐
  │                           │
┌─┴───────────────────────────┴─┐
│     _update() Loop (60Hz)     │ ◄─── Resolves network replication, button maps, logic
└─┬───────────────────────────┬─┘
  │                           │
┌─▼───────────────────────────▼─┐
│      _draw() Loop (60Hz)      │ ◄─── Compiles primitive geometry and outputs frame
└─┬───────────────────────────┬─┘
  │                           │
  └─────────────◄─────────────┘
```

---

## Architectural Topology Diagram

```text
┌────────────────────────────────────────────────────────┐
│               Command Line Interface / CLI             │
│                    (src/main.rs)                       │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│                    FantasyRuntime                      │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                  CartLoader                      │  │
│  │  • Automatic polymorphism: PICO-8 / TIC-80 checks│  │
│  │  • LZ77 text desegmentation code restoration loop│  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                    LuaVM                         │  │
│  │  • Embedded Lua 5.4 context platform standard    │  │
│  │  • Mutex shared multi-player binding middleware  │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │               GraphicsRenderer                   │  │
│  │  • Dynamic widescreen viewport scaler (SDL2)     │  │
│  │  • Pure CPU Software Rasterizer & Text blitter   │  │
│  │  • Integrated In-Engine Tooling Suite (--edit)   │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │               NetworkSubsystem                   │  │
│  │  • Non-blocking asynchronous UDP network sockets │  │
│  │  • Bitmask compression input replication engine  │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

---

## Robustness & Security Constrains

*   **Scissoring Integrity:** Virtual coordinate parameters injected outside bounds or current clipping rects are discarded safely on the CPU layer, completely locking out the possibility of heap memory overflow or stack memory corruption vulnerabilities during rendering.
*   **Virtual VM Traps:** Execution loops broken by incorrect script syntax are halted cleanly by the Rust runtime environment, dumping structured crash logs back into the command line shell without blocking native desktop window environments.
