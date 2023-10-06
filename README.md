A full Intel 8080 CPU emulator in Rust.

To verify if the emulator works well, run *cpudiag program under diagnosis_program/*. The program was modified to avoid the need of writing to ROM.

This library is intended to be reusable on different platforms: macOS, iOS, Android and (if possible) Web.

The FFI design is meant to be easy to understand and use.

The most important part of this library is to give you a representative of CPU, within which all the functions(CPU, RAM, IO) are provided. Apart from this, a channel is created for the communications between CPU and outside world, CPU owns a message receiver, and the corresponding message sender is exposed to the outside world for use.

If we take a look at *emulator.h* header file, we can see:
- `Cpu8080` opaque struct, we obtain a reference of this and pass it back for interpretation. e.g. see `run` method
- `IoCallbacks`, this is for IO interaction, e.g. we need to read from/write to peripheral devices, we use this way to get back to our devices. IO interfaces normally depend on the actual hardware, so you can pass an object (e.g. an opaque pointer `const void *io_object`) representing your specific IO models. This can be helpful if you want to run multiple games with different hardware specifications under same app process.
- A message sender for deliverying messages pre-defined:
    - Interrupt, we simulate a way to receive interrupts from the outside world, the interrupts always happen asynchronously, a mpsc channel can be used for this purpose, and CPU is the receiver, the sender should be owned by the platforms.
    - Pause/resume control signal, similar to handle interrupts, but with extra cares:
        - we check the pausing signal in a non-blocking manner (active state)
        - we check the resuming signal in a blocking manner (idle state)
    - Restarting from scratch, by clearing the RAM and resetting the PC and other general registers.
    - Shutdown the game machine, you can send a `Shutdown` message to the CPU, the CPU instance and the message sender will **both** be dropped, subsequent message deliveries and RAM access **will not be valid**, and doing so will cause undefined behavior! Make sure to shutdown only after you stop sending any messages and accessing the RAM. This can be helpful if you want to load a new game ROM file, so that you need to call `new_cpu_instance` to create a new CPU instance with new memory size and new IO callbacks.

## How to use
To use this library for app development, you can download the library(*libi8080emulator.a*) and header file(*emulator.h*) from the releases page and add them in your project. P.S. **Currently the releases only contain macOS(both x64 and aarch64) and iOS targets.**

If you can't find the target in the releases, you need to clone the source code and build it on your own, e.g. android (`aarch64-linux-android, arm-linux-androideabi` and etc...).
### Usage
- Load the ROM, allocate the runtime memory and provide IO callback functions by calling `new_cpu_instance`.
- Start the emulation by calling `run`, this function will not return unless:
    - You send a `Shutdown` message, in this case all resources will be freed, e.g. runtime memory, ROM memory and the message sender
    - OR an exception happens, in this case, please open an issue.
- `get_ram` will allow you to have the access to read the runtime memory, within which you can access video RAM.
- Call `send_message` to send messages including interrupts, control messages like: pause, resume, shutdown and reload.

## Apps powered by this library
- [Space Invaders on macOS + iOS](https://github.com/k0Iry/SpaceInvaders)
