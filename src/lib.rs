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

pub use errors::{EmulatorErrors, InvalidFile, MemoryOutOfBounds};

pub type Result<T> = std::result::Result<T, EmulatorErrors>;

pub use cpu::Cpu8080;

pub use condition_codes::ConditionCodes;

pub use clock_cycles::cycles::CLOCK_CYCLES;

#[repr(C)]
pub struct SwiftCallbacks {
    /// IN port, pass port number back to app
    /// set the calculated result back to reg_a
    pub input: extern "C" fn(port: u8) -> u8,
    /// OUT port value, pass port & value back to app
    pub output: extern "C" fn(port: u8, shift_offset: u8),
}

/// # Safety
/// This function should be called with valid rom path
/// and the RAM will be allocated on the fly
#[no_mangle]
pub unsafe extern "C" fn new_cpu_instance(
    rom_path: *const c_char,
    ram_len: usize,
    callbacks: SwiftCallbacks,
) -> *mut Cpu8080<'static> {
    let rom_path = unsafe { CStr::from_ptr(rom_path) };
    let rom_path = PathBuf::from_str(rom_path.to_str().unwrap()).unwrap();
    let bytes = BufReader::new(File::open(rom_path).unwrap())
        .bytes()
        .collect::<std::result::Result<Vec<u8>, std::io::Error>>()
        .unwrap();
    let rom = Box::leak(Box::new(bytes));
    let ram = Box::leak(Box::new(vec![0; ram_len]));
    Box::into_raw(Box::new(Cpu8080::new(rom, ram, callbacks)))
}

/// # Safety
/// This function should be safe
#[no_mangle]
pub unsafe extern "C" fn run(cpu: *mut Cpu8080) {
    let cpu = unsafe { &mut Box::from_raw(cpu) };
    cpu.run().unwrap();
}
