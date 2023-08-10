mod clock_cycles;
mod condition_codes;
mod cpu;
mod errors;

#[cfg(not(feature = "cpu_diag"))]
use std::{
    ffi::{c_char, CStr},
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
};

#[cfg(not(feature = "cpu_diag"))]
use cpu::MESSAGE_SENDER;

pub use errors::{EmulatorErrors, MemoryOutOfBounds};

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

#[cfg(not(feature = "cpu_diag"))]
#[repr(C)]
pub enum Message {
    Interrupt {
        irq_no: u8,
        allow_nested_interrupt: bool,
    },
    Suspend,
    Restart,
    Shutdown,
}

/// # Safety
/// This function should be called with valid rom path
/// and the RAM will be allocated on the fly
#[cfg(not(feature = "cpu_diag"))]
#[no_mangle]
pub unsafe extern "C" fn new_cpu_instance(
    rom_path: *const c_char,
    ram_size: usize,
    callbacks: IoCallbacks,
) -> *mut Cpu8080 {
    let rom_path = CStr::from_ptr(rom_path);
    let rom_path = PathBuf::from_str(rom_path.to_str().unwrap()).unwrap();
    let rom = BufReader::new(File::open(rom_path).unwrap())
        .bytes()
        .collect::<std::result::Result<Vec<u8>, std::io::Error>>()
        .unwrap();
    Box::into_raw(Box::new(Cpu8080::new(rom, vec![0; ram_size], callbacks)))
}

/// # Safety
/// This function should be safe
#[cfg(not(feature = "cpu_diag"))]
#[no_mangle]
pub unsafe extern "C" fn run(cpu: *mut Cpu8080) {
    let cpu = &mut Box::from_raw(cpu);
    cpu.run().unwrap();
}

/// # Safety
/// This function should be safe for accessing video ram
#[cfg(not(feature = "cpu_diag"))]
#[no_mangle]
pub unsafe extern "C" fn get_ram(cpu: *mut Cpu8080) -> *const u8 {
    let cpu = &*cpu;
    cpu.get_ram().as_ptr()
}

/// Always called from a separated thread!
/// It is crucial that we don't borrow our CPU instance
/// since this function will be called from FFI thread.
/// (e.g. threads spawned by Swift language where we
/// cannot enforce any ownership mechanism)
#[cfg(not(feature = "cpu_diag"))]
#[no_mangle]
pub extern "C" fn send_message(message: Message) {
    MESSAGE_SENDER
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .send(message)
        .unwrap()
}
