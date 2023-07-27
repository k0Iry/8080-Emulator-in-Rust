Using Rust to implement a full Intel 8080 CPU emulator.

To verify if the emulator works well, run *cpudiag program under path diagnosis_program/*. I also modified the assembly to avoid the need of writing to ROM.

This library is intended to be reused on different platforms: macOS, iOS, Android and (if possible) Web.

The FFI design is meant to be easy to understand and use.

The most important part of this library is to give you an instance of the CPU representative, within which all the functions(CPU, RAM, IO) are provided.

If we take a look at the interfaces in *emulator.h* header file, we can see:
- `Cpu8080` opaque struct, we obtain a pointer to it on app platforms and pass it back to this lib for interpretation. e.g. see `run` method
- `IoCallbacks` struct, as the name implies, this is for IO interaction, e.g. we need to read from/write to peripheral devices, every time an IO request made from CPU, we need a way to get back to our devices.
- Interrupt, we simulate a way to receive interrupts from the outside world, the interrupts always happen asynchronously, a mpsc channel can be used for this purpose, and CPU is the receiver, the sender should be owned by the platforms.
- Pause/resume the execution, similar to handle interrupts, a mpsc should be used, but with some extra cares:
    - for receiving pause request, we check the request in a non-blocking manner (active state)
    - for receiving resume request, we check the request in a blocking manner (idle state)

## How to use
To use this library for app development, you can download the library(*libi8080emulator.a*) and header file(*emulator.h*) from the releases page and add them in your project. P.S. **Currently the releases only contain macOS(both x64 and aarch64) and iOS targets.**

If you can't find the target in the releases, you need to clone the source code and build it on your own, e.g. android (`aarch64-linux-android, arm-linux-androideabi` and etc...).

## Apps powered by this library
- [Space Invaders on macOS + iOS](https://github.com/k0Iry/SpaceInvaders)