mod condition_codes;
mod cpu;
mod errors;

pub use errors::{EmulatorErrors, InvalidFile, MemoryOutOfBounds, RomReadFailure};

pub type Result<T> = std::result::Result<T, EmulatorErrors>;

pub use cpu::Cpu8080;

pub use condition_codes::ConditionCodes;
