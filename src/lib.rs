#![no_std]

#[cfg(any(test, feature = "cpu_diag"))]
extern crate std;

mod clock_cycles;
mod condition_codes;
mod cpu;
mod errors;
mod memory;

pub use errors::{EmulatorErrors, MemoryOutOfBounds};

pub type Result<T> = core::result::Result<T, EmulatorErrors>;

pub use cpu::{Cpu8080, ExecutionState, StepOutcome};

pub use condition_codes::ConditionCodes;

pub use clock_cycles::cycles::CLOCK_CYCLES;
pub use memory::{Memory, SplitMemory};
