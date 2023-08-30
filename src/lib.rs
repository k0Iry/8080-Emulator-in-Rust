mod clock_cycles;
mod condition_codes;
mod cpu;
mod errors;

#[cfg(not(feature = "cpu_diag"))]
use std::{
    ffi::{c_char, c_void, CStr},
    fs::File,
    io::{BufReader, Read},
    path::PathBuf,
    str::FromStr,
    sync::mpsc::Sender,
};

pub use errors::{EmulatorErrors, MemoryOutOfBounds};

pub type Result<T> = std::result::Result<T, EmulatorErrors>;

pub use cpu::Cpu8080;

pub use condition_codes::ConditionCodes;

pub use clock_cycles::cycles::CLOCK_CYCLES;

#[cfg(not(feature = "cpu_diag"))]
#[repr(C)]
pub struct IoCallbacks {
    /// IN port, pass port number back to app
    /// set the calculated result back to reg_a
    pub input: extern "C" fn(io_object: *const c_void, port: u8) -> u8,
    /// OUT port value, pass port & value back to app
    pub output: extern "C" fn(io_object: *const c_void, port: u8, value: u8),
}

#[cfg(not(feature = "cpu_diag"))]
#[repr(C)]
pub struct CpuSender {
    cpu: *mut Cpu8080,
    sender: *mut Sender<Message>,
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
    io_object: *const c_void,
) -> CpuSender {
    let rom_path = CStr::from_ptr(rom_path);
    let rom_path = PathBuf::from_str(rom_path.to_str().unwrap()).unwrap();
    let rom = BufReader::new(File::open(rom_path).unwrap())
        .bytes()
        .collect::<std::result::Result<Vec<u8>, std::io::Error>>()
        .unwrap();
    let (cpu, sender) = Cpu8080::new(rom, vec![0; ram_size], callbacks, io_object);
    CpuSender {
        cpu: Box::into_raw(Box::new(cpu)),
        sender: Box::into_raw(Box::new(sender)),
    }
}

/// # Safety
/// This function should be safe to start a run loop.
/// Send a `Shutdown` message can break the loop, so
/// that the CPU and the Sender will be dropped, this is
/// the only way to release the resources to the system.
#[cfg(not(feature = "cpu_diag"))]
#[no_mangle]
pub unsafe extern "C" fn run(cpu: *mut Cpu8080, sender: *mut Sender<Message>) {
    Box::from_raw(cpu).run().unwrap();
    let _ = Box::from_raw(sender);
}

/// # Safety
/// This function should be safe for accessing video ram.
#[cfg(not(feature = "cpu_diag"))]
#[no_mangle]
pub unsafe extern "C" fn get_ram(cpu: *const Cpu8080) -> *const u8 {
    (*cpu).get_ram().as_ptr()
}

/// # Safety
/// Sender needs to be present(not dropped) for
/// sending the messages to the CPU instance.
#[cfg(not(feature = "cpu_diag"))]
#[no_mangle]
pub unsafe extern "C" fn send_message(sender: *const Sender<Message>, message: Message) {
    (*sender).send(message).unwrap()
}
