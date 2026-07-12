# Troubleshooting Manual

This guide offers solutions for architectural conflicts, compiler errors, and engine edge cases.

## 1. System Dependencies (Debian / Ubuntu Linux)

### Issue: Cargo cannot link or build dependencies due to missing compiler headers
**Error Symptoms:**
- `error: linker cc not found`
- `pkg-config failure or missing sdl2 development packages`

**Solution:**
Ensure you have all necessary C development tooling and graphics packages installed via apt:
```bash
sudo apt update
sudo apt install -y build-essential libsdl2-dev pkg-config
```

---

## 2. Audio Engine Failures

### Issue: Lua `sfx()` triggers errors or app runs completely silent
**Error Symptoms:**
- `runtime error: attempt to call a nil value (global 'sfx')`

**Solution:**
Verify that your global bindings match inside `src/vm/api.rs`. Make sure `FromLua` is imported properly at the top of your API file:
```rust
use mlua::{Lua, Result, FromLua};
```

---

## 3. Window Icon Disappearances (Linux Environments)

### Issue: Window icon displays as a generic executable block under Debian 12 (Gnome / Wayland)
**Cause:**
Wayland security layers and modern X11 servers block direct frame modification inside binaries unless an explicit `.desktop` manifest matches the execution environment. 

**Workaround:**
The engine codebase maintains static pixel allocation arrays in memory to maximize support, but final production deployment requires shipping a valid system package structure matching app entry paths.

---

## 4. Lua Runtime Types Mismatches

### Issue: Rust compiler crashes on `mlua` multi-argument patterns
**Error Symptoms:**
- `the trait bound (u8, u8): FromLuaMulti is not satisfied`

**Solution:**
Do not capture tuple variables raw inside option arguments. Consume raw data pipes into sequentials explicitly via native iterators:
```rust
let mut iter = args.into_iter();
let val0 = iter.next().unwrap();
```
