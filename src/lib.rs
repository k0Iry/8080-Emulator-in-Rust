mod clock_cycles;
mod condition_codes;
mod cpu;
mod errors;

pub use errors::{EmulatorErrors, InvalidFile, MemoryOutOfBounds};

pub type Result<T> = std::result::Result<T, EmulatorErrors>;

pub use cpu::Cpu8080;

pub use condition_codes::ConditionCodes;

pub use clock_cycles::cycles::CLOCK_CYCLES;
