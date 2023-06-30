mod clock_cycles;
mod condition_codes;
mod cpu;
mod errors;

use std::{
    ffi::{c_char, CStr},
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
};

use cpu::{INTERRUPT_SENDER, PAUSE_SENDER};
pub use errors::{EmulatorErrors, InvalidFile, MemoryOutOfBounds};

pub type Result<T> = std::result::Result<T, EmulatorErrors>;

pub use cpu::Cpu8080;

pub use condition_codes::ConditionCodes;

pub use clock_cycles::cycles::CLOCK_CYCLES;

#[repr(C)]
pub struct IoCallbacks {
    /// IN port, pass port number back to app
    /// set the calculated result back to reg_a
    pub input: extern "C" fn(port: u8) -> u8,
    /// OUT port value, pass port & value back to app
    pub output: extern "C" fn(port: u8, value: u8),
}

/// # Safety
/// This function should be called with valid rom path
/// and the RAM will be allocated on the fly
#[no_mangle]
pub unsafe extern "C" fn new_cpu_instance<'a>(
    rom_path: *const c_char,
    ram_size: usize,
    callbacks: IoCallbacks,
) -> *mut Cpu8080<'a> {
    let rom_path = CStr::from_ptr(rom_path);
    let rom_path = PathBuf::from_str(rom_path.to_str().unwrap()).unwrap();
    let rom = BufReader::new(File::open(rom_path).unwrap())
        .bytes()
        .collect::<std::result::Result<Vec<u8>, std::io::Error>>()
        .unwrap();
    Box::into_raw(Box::new(Cpu8080::new(
        &*rom.leak(),
        vec![0; ram_size].leak(),
        callbacks,
    )))
}

/// # Safety
/// This function should be safe
#[no_mangle]
pub unsafe extern "C" fn run(cpu: *mut Cpu8080) {
    let cpu = &mut Box::from_raw(cpu);
    cpu.run().unwrap();
}

/// It is crucial that we don't borrow our CPU instance
/// since this function will be called from FFI thread.
/// (e.g. threads spawned by Swift language where we
/// cannot enforce any ownership mechanism)
#[no_mangle]
pub extern "C" fn send_interrupt(interrupt: u8, allow_nested_interrupt: bool) {
    INTERRUPT_SENDER
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .send((interrupt, allow_nested_interrupt))
        .unwrap()
}

#[no_mangle]
pub extern "C" fn pause_start_execution() {
    PAUSE_SENDER
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .send(())
        .unwrap()
}

/// # Safety
/// This function should be safe for accessing video ram
#[no_mangle]
pub unsafe extern "C" fn get_ram(cpu: *mut Cpu8080) -> *const u8 {
    let cpu = &*cpu;
    cpu.get_ram().as_ptr()
}
