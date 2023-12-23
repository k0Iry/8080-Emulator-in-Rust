A full Intel 8080 CPU emulator in Rust.

To verify the emulation, run below:

`cargo run --features="cpu_diag"`, making sure *CPU IS OPERATIONAL* gets popped up.

This library is intended to be portable on different platforms: macOS, iOS, Android and (if possible) Web.

The FFI design is meant to be easy to understand and use.

This library gives all the functions: CPU, RAM & IO. Apart from these, a channel is created for communicating between CPU and the outside world, CPU is the events receiver, and the corresponding message sender is exposed to/owned by the outside world.

If we take a look at *emulator.h* header file, we can see:
- `Cpu8080`, we obtain a reference of this object and then pass back for interpretation. e.g. see `run` method
- `IoCallbacks`, for IO interfaces. IO interfaces normally depend on the actual hardware spec, similar to `Cpu8080` you can pass an object (e.g. an opaque pointer `const void *io_object`) representing specific IO models. This can be helpful if you want to run multiple games with different hardware specifications under same context.
- A message sender for deliverying messages pre-defined:
    - Interrupt, simulating a way to receive async interrupts from the outside world, a mpsc channel can be used for this purpose
    - Pause/resume control signal, similar to handle interrupts, but with extra cares:
        - check the pausing signal in a non-blocking manner (active state)
        - check the resuming signal in a blocking manner (idle state)
    - Restart from scratch, by clearing the RAM and resetting the PC and other general registers.
    - Shutdown, you can send a `Shutdown` message to the CPU, the CPU instance and the message sender will **both** be dropped, subsequent message deliveries and RAM access **will not be valid**, and doing so will cause undefined behavior! Make sure to shutdown only after you stop sending any messages and accessing the RAM. This can be helpful if you want to load a new game ROM file, but you need to call `new_cpu_instance` again to create a new CPU instance with new rom, new memory size & new IO callbacks.

## How to use
To use this library for app development, you can download the library(*libi8080emulator.a*) and header(*emulator.h*) from the releases page and add them in your project. Please be noted that **Currently releases only contain macOS(both x64 and aarch64) and iOS targets.**

If you can't find the target in the releases, you need to clone the source code and build it on your own, e.g. android (`aarch64-linux-android, arm-linux-androideabi` and etc...).
### Usage
1. Load the ROM, allocate the runtime memory and provide IO callback functions by calling `new_cpu_instance`.
2.  Start the emulation by calling `run`, this function will not return unless:

    1. You send a `Shutdown` message, in this case all resources will be freed, e.g. runtime memory, ROM memory and the message sende
    2. OR an exception happens, in this case, please open an issue.
4. `get_ram` allows you to have the access to read the runtime memory, within which you can access video RAM.
5. Call `send_message` to send messages including interrupts, control messages like: pause, resume, shutdown and reload.

## Apps powered by this library
- [Space Invaders on macOS + iOS](https://github.com/k0Iry/SpaceInvaders)
