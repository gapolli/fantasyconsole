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
sudo apt install -y build-essential libsdl2-dev pkg-config
```

---

## 2. Audio Engine Framework Failures

### Issue: Lua `sfx()` scripts trigger crashes or the virtual runtime boots completely silent
**Error Symptoms:**
- `runtime error: attempt to call a nil value (global 'sfx')`

**Solution:**
Verify that your global bindings match the signatures inside `src/vm/api.rs`. Make sure the `FromLua` trait is explicitly imported and within active file scope to map primitive types:
```rust
use mlua::{Lua, Result, FromLua};
```

---

## 3. Local Sample File Path Incompatibilities

### Issue: The application loads an static screen (black or cyan) with no graphic primitive steps visible
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

## 5. Lua Runtime Type-Casting Mismatches

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
