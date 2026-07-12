# FantasyConsole - Technical Specification

## Overview

FantasyConsole is an open-source virtual runtime machine engineered to execute PICO-8 style game cartridge configurations (plain-text `.p8` format), recreating the technical specifications of a retro fantasy system without breaking intellectual property bounds. The architecture implements a matching safe API built on top of a portable hardware abstraction layer wrapped with an SDL2 rendering backend.

**Version:** 0.1.0-dev  
**License:** MIT  
**Repository:** https://github.com/gapolli/fantasyconsole

---

## Emulated Hardware Constraints

| Parameter | Configuration Specification |
| :--- | :--- |
| **Display Resolution** | 128×128 pixels (square display) |
| **Color Space** | 16-color index-mapped lookup table (fully remappable) |
| **Hexadecimal Palettes**| #000000, #1D2B53, #7E2553, #008751, #AB5236, #5F574F, #C2C3C7, #FFF1E8, #FF004D, #FFA300, #FFEC27, #00E436, #29ADFF, #83769C, #FF77A8, #FFCCAA |
| **Graphics Asset Bounds** | 256 addressable sprite slots inside a shared sheet (8×8 cells) |
| **Tilemap Dimension Grid**| Standard 128×64 matrix block cell registry array |
| **System Update Pace** | Fixed 60 frames per second regulated via monolithic frame loops |
| **Virtual Addressable RAM**| 32 KB structural memory mapping block configuration |

---

## Runtime API Surface (PICO-8 Core Compatible)

### Graphics and Rasterization Subsystem

| Function Signature | Technical Description |
| :--- | :--- |
| `cls(color)` | Clears either the active display space or active clipping window using a specific color index. |
| `pset(x, y, color)` | Modifies a target screen coordinate applying active camera vectors and clipping boundaries. |
| `line(x0, y0, x1, y1, color)` | Draws a single primitive line segment leveraging an unrolled CPU implementation of Bresenham's Algorithm. |
| `circ(x, y, r, color)` | Draws a hollow circular perimeter primitive leveraging the classic Midpoint Circle Algorithm. |
| `circfill(x, y, r, color)` | Draws a filled circle shape executing balanced parallel horizontal row rasterization routines. |
| `spr(n, x, y, flip_x, flip_y)` | Renders an 8×8 pixel bitmap block with optional runtime hardware axis reflection parameters. |
| `sspr(sx, sy, sw, sh, dx, dy, dw, dh, fx, fy)` | Samples and projects variable texture blocks executing native *Nearest-Neighbor* layout interpolation. |
| `map(mx, my, sx, sy, w, h)` | Iterates and renders rows of structural cell assets from the active memory tilemap array down onto display space. |
| `mget(x, y)` | Reads and returns the explicit structural tile ID index saved at the target cell row location on the map. |
| `mset(x, y, v)` | Modifies the target tile index mapping slot inside the active structural map grid array block. |
| `pal(c0, c1)` | Updates a dynamic lookup map entry, re-routing pixel index pipelines downstream during hardware composition. |
| `pal()` | Reinitializes the dynamic color routing index mapping array back to native hardware standards. |
| `clip(x, y, w, h)` | Establishes a hardware scissor bounding rectangle mask protecting pixel buffer write segments. |
| `camera(x, y)` | Injects an axis shift displacement vector altering world-space render calculations for subsequent calls. |
| `print(str, x, y, color)` | Blits text elements down into pixel buffers utilizing a pure 3x5 matrix bitmask binary lookup font engine. |

### Peripherals and Input Subsystem

| Function Signature | Technical Description |
| :--- | :--- |
| `btn(button_index)` | Polls and returns the active physical state boolean flag of a virtual controller index registry. |

### Audio and Synthesis Subsystem (Phase 2)

| Function Signature | Technical Description |
| :--- | :--- |
| `sfx(n, channel)` | Fires a real-time procedural oscillator wave (Sine, Square, Triangle, or Sawtooth) down an active mixer pipeline. |
| `music(track)` | Coordinates background loop audio pattern generation streams over the virtual tracker channels (*In active development*). |

### Engine Persistence and State Systems

| Function Signature | Technical Description |
| :--- | :--- |
| `save_game(slot)` | Dumps a comprehensive binary copy of memory matrices and state flags out into structured static JSON storage files. |
| `load_game(slot)` | Parses existing persistence JSON save states, restoring system positions, buffers, and flags immediately. |

---

## Supported Storage File Formats

### 1. Primary Text Layout Configuration: `.p8` Files
Cartridges are managed via plain-text files using explicit header brackets to section off virtual hardware fields:
*   `__lua__`: Houses the complete raw game program source logic.
*   `__gfx__`: Holds hexadecimal row character strings (0-f) that decode down into sprite color matrices.
*   `__map__`: Layout data arrays mapping matrix cell tile configurations for world scenarios.

### 2. Compressed Binary Steganographic Format: `.p8.png` Files
A retro data storage mechanism where game scripts and asset streams are distributed hidden invisibly within the two least significant bits (LSB) of color channels inside a regular 160×205 pixel PNG image shell.

---

## Architectural Engine Topology

```text
┌────────────────────────────────────────────────────────┐
│                   CLI Main Entry Point                 │
│                      (src/main.rs)                     │
└──────────────────────────┬─────────────────────────────┘
                           │
┌──────────────────────────▼─────────────────────────────┐
│                    FantasyRuntime                      │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                  CartLoader                      │  │
│  │  • Main syntax parser engine for the .p8 format  │  │
│  │  • Section extractor loops (__lua__, __gfx__)    │  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                    LuaVM                         │  │
│  │  • Embedded Lua 5.4 platform instance via `mlua`│  │
│  │  • Hardware bridge bindings injector middleware  │  │
│  │  • Game lifestyle callback caller (_update,_draw)│  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │               GraphicsRenderer                   │  │
│  │  • SDL2 window engine scaled matrix canvas layer │  │
│  │  • Stream converter maps Indexed Buffer -> RGB24 │  │
│  │  • Core diagnostic overlay monitoring tools (F12)│  │
│  └──────────────────────────────────────────────────┘  │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │                 AudioEngine                      │  │
│  │  • Asynchronous thread safe oscillators (44.1kHz)│  │
│  │  • Non-blocking crossbeam channel command piping │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
```

---

## Application Runtime Lifecycle

Game software scripts must register (or selectively choose to implement) these core environment hooks to drive the runtime:

*   **`_init()`**: Fired once on cold bootup to set up variable baselines, states, arrays, and map modifications.
*   **`_update()`**: Driven strictly 60 times a second to clear inputs, recalculate vectors, logic, and physics states.
*   **`_draw()`**: Synchronized directly with native monitor refresh loops to clear layouts and commit graphics out to display space.

---

## Exception Strategies and Memory Management

*   **Syntax Compilation Flaws:** Safely intercept and log compilation or interpreter breakdowns cleanly via terminal crash output traps without locking physical operating system drivers.
*   **Buffer Bounds Protection:** Out-of-bounds pixel positions or clipping parameter limits drop writes automatically without altering adjacent hardware indices, protecting native CPU stack footprints against indexing corruption.
