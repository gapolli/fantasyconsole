# Troubleshooting Manual

This guide offers solutions for architectural conflicts, compile-time errors, multi-environment issues, and runtime edge cases inside the `fantasyconsole` ecosystem.

## 1. System Dependencies & Compilation (Debian 12 / Ubuntu Linux)

### Issue: Cargo cannot link or compile dependencies due to missing hardware headers
**Error Symptoms:**
- `error: linker cc not found`
- `pkg-config failure or missing sdl2 development packages`

**Solution:**
Ensure you have all necessary C development tooling, compilation packages, and native system graphics libraries installed via apt before attempting to compile the runtime:
```bash
sudo apt update
sudo apt install -y build-essential libsdl2-dev libsdl2-ttf-dev pkg-config cmake
```

### Issue: `sdl2-sys` build script failure caused by missing `cmake` tool
**Error Symptoms:**
- `failed to execute command: No such file or directory (os error 2)`
- `is cmake not installed?`
- `process didn't exit successfully ... sdl2-sys/build-script-build`

**Solution:**
By default, the `sdl2` crate tries to compile its own vendor version using CMake. If you are running on a clean Linux environment like Debian 12, you can either install CMake or force Cargo to link directly against the system-installed SDL2 libraries using `pkg-config`:

*   **Option A (Recommended - Fast Link):** Prepend the native configuration path variable before building:
    ```bash
    SDL_CONFIG=/usr/bin/sdl2-config cargo run -- game.p8
    ```
*   **Option B (System Install):** Give the compiler the build utility it requested:
    ```bash
    sudo apt install -y cmake
    ```

---

## 2. Audio Engine & Arpeggiator Framework Failures

### Issue: Lua `sfx()` scripts trigger crashes or the virtual runtime boots completely silent
**Error Symptoms:**
- `runtime error: attempt to call a nil value (global 'sfx')`

**Solution:**
Verify that your global bindings match the signatures inside `src/vm/api.rs`. Make sure the `FromLua` trait is explicitly imported and within active file scope to map primitive types:
```rust
use mlua::{Lua, Result, FromLua};
```

### Issue: Variant `AudioCommand::PlaySfx` compile error after arpeggiator refactoring
**Error Symptoms:**
- `error[E0559]: variant AudioCommand::PlaySfx has no field named note`
- `error[E0026]: variant PlaySfx does not have a field named note`

**Cause:**
This happens when you upgrade the audio subsystem to handle structural multi-frequency vector tables (`notes`) for chords, but old legacy single-note emitters (`sfx`) or event unboxers inside `main.rs` still attempt to map a singular `note: f32` parameter.

**Solution:**
*   **In `src/vm/api.rs` (SFX Binder):** Wrap the singular frequency variable into a native vector layout allocator:
    ```rust
    notes: vec![base_frequency]
    ```
*   **In `src/main.rs` (Event Loop Receiver):** Update the pattern matching structure to desubstitute `note` with `notes`, capture the lock safely, and stream the complete buffer to the sound channel array:
    ```rust
    AudioCommand::PlaySfx { channel, waveform, notes, duration_ms } => {
        let mut lock = audio_device.lock();
        lock.channels[channel].arpeggio_notes = notes;
    }
    ```

---

## 3. Local Sample File Path Incompatibilities

### Issue: The application loads a static screen (black or cyan) with no graphic primitive steps visible
**Cause:**
This is commonly caused by path naming mismatches between the execution terminal shell argument and the actual localized workspace directories (such as feeding `exemplos/` instead of the configured english directory standard `examples/`), which prevents the `CartLoader` module from feeding the Lua VM environment.

**Solution:**
Always double-check that your target asset scripts match the localized directory array exactly when firing the binary execution command:
```bash
cargo run -- examples/05_multiplayer_arena.p8
```

---

## 4. Asynchronous Network & Game Loop Locks

### Issue: The graphics engine freezes or stalls immediately on boot during multiplayer tests
**Cause:**
If the UDP network socket abstraction layer (`src/network/mod.rs`) is called synchronously without the proper non-blocking flag, the Linux kernel (Debian 12) will pause the main execution frame thread while waiting for remote incoming packets, completely stalling the SDL2 render cycles.

**Solution:**
Verify that the `set_nonblocking(true)` method is bound directly during the socket initialization routine to enforce smooth, non-blocking frame update steps:
```rust
let socket = UdpSocket::bind(bind_addr)?;
socket.set_nonblocking(true)?;
```

---

## 5. Lua Runtime Type-Casting & Memory Mismatches

### Issue: Rust compiler crashes on `mlua::create_function` multi-argument patterns
**Error Symptoms:**
- `the trait bound (u8, u8): FromLuaMulti is not satisfied`

**Solution:**
Do not capture Rust tuples raw directly inside function option wrappers. Intercept native arguments as an un-typed array sequence (`mlua::MultiValue`), and then extract their elements sequentially using native iterator consumption:
```rust
let mut iter = args.into_iter();
let val0 = iter.next().unwrap();
let val1 = iter.next().unwrap();
```

### Issue: Vector rendering primitives crashing with array out-of-bounds index panics
**Cause:**
When writing fast CPU loops or injecting untrusted pixel colors from Lua scripts into primitive shaders like `rectfill` or `polyfill`, providing an arbitrary index beyond `15` will step outside the standard `P8_RGB` array stack limits.

**Solution:**
Enforce strict hardware-level bitmask wrapping constraints on the color value before querying index references inside your software blitter loop:
```rust
let mapped_color = self.palette_map[(color & 0x0F) as usize] & 0x0F;
```
