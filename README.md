A full Intel 8080 CPU emulator in Rust.

This crate is now a `no_std` CPU core. It focuses on Intel 8080 execution only; file loading, threading, timers, host messaging and platform-specific I/O are expected to live outside the crate.

To verify the emulation, run below:

`cargo run --features="cpu_diag"`, making sure *CPU IS OPERATIONAL* gets popped up.

This library is intended to be portable on different platforms: macOS, iOS, Android and (if possible) Web.

Feature flags:
- `cpu_diag`: enables the bundled diagnosis flow and its debug output.

## How to use
Create a CPU with ROM bytes and RAM, then drive it from your host loop.

### Usage
1. Construct `Cpu8080` with `Cpu8080::new(rom, ram)`, where `rom: &[u8]` and `ram: &mut [u8]` are owned by the caller.
2. If you need a different address-space layout, implement the `Memory` trait and build the CPU with `Cpu8080::with_memory(...)`.
3. Call `step()` to execute one instruction, or `run()` to keep executing until the CPU halts or needs host-visible I/O.
4. When you receive `ExecutionState::Input { port }`, fetch the value from your platform and pass it back with `provide_input(value)`.
5. When you receive `ExecutionState::Output { port, value }`, forward it to your platform-specific device model.
6. Use `interrupt()` and `restart()` from your host when the platform needs to inject interrupts or reset the CPU.
7. Use `get_ram()` with the default `SplitMemory` layout, or `memory()` / `memory_mut()` if you are using a custom memory implementation.

### Example
```rust
use i8080emulator::{Cpu8080, ExecutionState};

let mut ram = [0; 0x4000];
let mut cpu = Cpu8080::new(rom, &mut ram);

loop {
    match cpu.run()?.state {
        ExecutionState::Continue => {}
        ExecutionState::Input { port } => {
            let value = platform_read(port);
            cpu.provide_input(value)?;
        }
        ExecutionState::Output { port, value } => {
            platform_write(port, value);
        }
        ExecutionState::Halted => break,
    }
}
```

## Apps powered by this library
- [Space Invaders on macOS + iOS](https://github.com/k0Iry/SpaceInvaders)

## Swift / C FFI
The repo now also contains a thin FFI crate for Swift and other C-compatible hosts.

- Build it with `cargo build -p i8080emulator-ffi --release`
- The exported header is at `ffi/include/i8080emulator_ffi.h`
- The static library will be emitted under `target/release/`

The FFI layer is now also `no_std`/no-allocation. The host owns CPU storage, ROM, and RAM:
- `i8080_cpu_size` / `i8080_cpu_align`
- `i8080_init_cpu` / `i8080_deinit_cpu`
- `i8080_run` / `i8080_step`
- `i8080_provide_input`
- `i8080_interrupt`
- `i8080_restart`
- `i8080_ram_ptr` / `i8080_ram_len`
