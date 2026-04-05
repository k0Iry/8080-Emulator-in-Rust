#![cfg_attr(not(test), no_std)]

use core::{
    mem::{align_of, size_of},
    panic::PanicInfo,
    ptr, slice,
};

use i8080emulator::{Cpu8080, EmulatorErrors, ExecutionState, Memory, StepOutcome};

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo<'_>) -> ! {
    loop {}
}

struct ForeignMemory {
    rom_ptr: *const u8,
    rom_len: usize,
    ram_ptr: *mut u8,
    ram_len: usize,
}

impl ForeignMemory {
    fn ram_ptr(&self) -> *const u8 {
        self.ram_ptr.cast_const()
    }

    fn ram_mut_ptr(&mut self) -> *mut u8 {
        self.ram_ptr
    }

    fn ram_len(&self) -> usize {
        self.ram_len
    }
}

impl Memory for ForeignMemory {
    fn read(&self, addr: u16) -> Option<u8> {
        let addr = addr as usize;
        if addr < self.rom_len {
            // Safety: validated at initialization and never mutated afterwards.
            unsafe { slice::from_raw_parts(self.rom_ptr, self.rom_len) }
                .get(addr)
                .copied()
        } else {
            // Safety: validated at initialization and owned by the host.
            unsafe { slice::from_raw_parts(self.ram_ptr.cast_const(), self.ram_len) }
                .get(addr - self.rom_len)
                .copied()
        }
    }

    fn write(&mut self, addr: u16, value: u8) -> bool {
        let addr = addr as usize;
        if addr < self.rom_len {
            return false;
        }
        if let Some(cell) =
            // Safety: validated at initialization and owned by the host.
            unsafe { slice::from_raw_parts_mut(self.ram_ptr, self.ram_len) }
                .get_mut(addr - self.rom_len)
        {
            *cell = value;
            true
        } else {
            false
        }
    }

    fn reset_ram(&mut self) {
        // Safety: validated at initialization and owned by the host.
        unsafe { slice::from_raw_parts_mut(self.ram_ptr, self.ram_len) }.fill(0);
    }
}

pub struct I8080Cpu {
    cpu: Cpu8080<ForeignMemory>,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I8080Status {
    Ok = 0,
    NullPointer = 1,
    NoPendingInput = 2,
    MemoryOutOfBounds = 3,
    InvalidInterrupt = 4,
    InvalidArguments = 5,
    InvalidAlignment = 6,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum I8080ExecutionState {
    Continue = 0,
    Input = 1,
    Output = 2,
    Halted = 3,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct I8080StepResult {
    pub cycles: u64,
    pub state: I8080ExecutionState,
    pub port: u8,
    pub value: u8,
}

fn map_state(value: ExecutionState) -> (I8080ExecutionState, u8, u8) {
    match value {
        ExecutionState::Continue => (I8080ExecutionState::Continue, 0, 0),
        ExecutionState::Input { port } => (I8080ExecutionState::Input, port, 0),
        ExecutionState::Output { port, value } => (I8080ExecutionState::Output, port, value),
        ExecutionState::Halted => (I8080ExecutionState::Halted, 0, 0),
    }
}

impl From<StepOutcome> for I8080StepResult {
    fn from(value: StepOutcome) -> Self {
        let (state, port, io_value) = map_state(value.state);
        Self {
            cycles: value.cycles,
            state,
            port,
            value: io_value,
        }
    }
}

fn map_error(err: EmulatorErrors) -> I8080Status {
    match err {
        EmulatorErrors::NoPendingInput => I8080Status::NoPendingInput,
        EmulatorErrors::MemoryOutOfBounds(_) => I8080Status::MemoryOutOfBounds,
        EmulatorErrors::InvalidInterrupt(_) => I8080Status::InvalidInterrupt,
    }
}

unsafe fn cpu_mut<'a>(cpu: *mut I8080Cpu) -> Result<&'a mut I8080Cpu, I8080Status> {
    cpu.as_mut().ok_or(I8080Status::NullPointer)
}

unsafe fn cpu_ref<'a>(cpu: *const I8080Cpu) -> Result<&'a I8080Cpu, I8080Status> {
    cpu.as_ref().ok_or(I8080Status::NullPointer)
}

unsafe fn step_result_mut<'a>(
    result: *mut I8080StepResult,
) -> Result<&'a mut I8080StepResult, I8080Status> {
    result.as_mut().ok_or(I8080Status::NullPointer)
}

fn validate_memory_args(
    rom_ptr: *const u8,
    rom_len: usize,
    ram_ptr: *mut u8,
    ram_len: usize,
) -> I8080Status {
    if (rom_len != 0 && rom_ptr.is_null()) || (ram_len != 0 && ram_ptr.is_null()) {
        return I8080Status::InvalidArguments;
    }
    I8080Status::Ok
}

#[no_mangle]
pub extern "C" fn i8080_cpu_size() -> usize {
    size_of::<I8080Cpu>()
}

#[no_mangle]
pub extern "C" fn i8080_cpu_align() -> usize {
    align_of::<I8080Cpu>()
}

/// # Safety
/// `storage` must point to writable memory of at least `i8080_cpu_size()` bytes and aligned to `i8080_cpu_align()`.
/// `out_cpu` must be a valid writable pointer.
/// `rom_ptr`/`ram_ptr` must remain valid for the lifetime of the CPU object.
#[no_mangle]
pub unsafe extern "C" fn i8080_init_cpu(
    storage: *mut u8,
    rom_ptr: *const u8,
    rom_len: usize,
    ram_ptr: *mut u8,
    ram_len: usize,
    out_cpu: *mut *mut I8080Cpu,
) -> I8080Status {
    if storage.is_null() || out_cpu.is_null() {
        return I8080Status::NullPointer;
    }

    let status = validate_memory_args(rom_ptr, rom_len, ram_ptr, ram_len);
    if status != I8080Status::Ok {
        return status;
    }

    if !(storage as usize).is_multiple_of(align_of::<I8080Cpu>()) {
        return I8080Status::InvalidAlignment;
    }

    let memory = ForeignMemory {
        rom_ptr,
        rom_len,
        ram_ptr,
        ram_len,
    };
    ptr::write(
        storage.cast::<I8080Cpu>(),
        I8080Cpu {
            cpu: Cpu8080::with_memory(memory),
        },
    );
    ptr::write(out_cpu, storage.cast::<I8080Cpu>());
    I8080Status::Ok
}

/// # Safety
/// `cpu` must be a valid pointer previously initialized by `i8080_init_cpu`.
#[no_mangle]
pub unsafe extern "C" fn i8080_deinit_cpu(cpu: *mut I8080Cpu) -> I8080Status {
    let cpu = match cpu_mut(cpu) {
        Ok(cpu) => cpu,
        Err(status) => return status,
    };
    ptr::drop_in_place(cpu);
    I8080Status::Ok
}

/// # Safety
/// `cpu` and `result` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn i8080_run(
    cpu: *mut I8080Cpu,
    result: *mut I8080StepResult,
) -> I8080Status {
    let cpu = match cpu_mut(cpu) {
        Ok(cpu) => cpu,
        Err(status) => return status,
    };
    let result = match step_result_mut(result) {
        Ok(result) => result,
        Err(status) => return status,
    };

    match cpu.cpu.run() {
        Ok(outcome) => {
            *result = outcome.into();
            I8080Status::Ok
        }
        Err(err) => map_error(err),
    }
}

/// # Safety
/// `cpu` and `result` must be valid non-null pointers.
#[no_mangle]
pub unsafe extern "C" fn i8080_step(
    cpu: *mut I8080Cpu,
    result: *mut I8080StepResult,
) -> I8080Status {
    let cpu = match cpu_mut(cpu) {
        Ok(cpu) => cpu,
        Err(status) => return status,
    };
    let result = match step_result_mut(result) {
        Ok(result) => result,
        Err(status) => return status,
    };

    match cpu.cpu.step() {
        Ok(outcome) => {
            *result = outcome.into();
            I8080Status::Ok
        }
        Err(err) => map_error(err),
    }
}

/// # Safety
/// `cpu` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn i8080_provide_input(cpu: *mut I8080Cpu, value: u8) -> I8080Status {
    let cpu = match cpu_mut(cpu) {
        Ok(cpu) => cpu,
        Err(status) => return status,
    };

    match cpu.cpu.provide_input(value) {
        Ok(()) => I8080Status::Ok,
        Err(err) => map_error(err),
    }
}

/// # Safety
/// `cpu` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn i8080_interrupt(cpu: *mut I8080Cpu, irq_no: u8) -> I8080Status {
    let cpu = match cpu_mut(cpu) {
        Ok(cpu) => cpu,
        Err(status) => return status,
    };

    match cpu.cpu.interrupt(irq_no) {
        Ok(()) => I8080Status::Ok,
        Err(err) => map_error(err),
    }
}

/// # Safety
/// `cpu` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn i8080_restart(cpu: *mut I8080Cpu) -> I8080Status {
    let cpu = match cpu_mut(cpu) {
        Ok(cpu) => cpu,
        Err(status) => return status,
    };

    cpu.cpu.restart();
    I8080Status::Ok
}

/// # Safety
/// `cpu` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn i8080_is_halted(cpu: *const I8080Cpu) -> bool {
    cpu_ref(cpu).map(|cpu| cpu.cpu.is_halted()).unwrap_or(true)
}

/// # Safety
/// `cpu` must be a valid non-null pointer. The returned pointer is borrowed from the host-provided RAM.
#[no_mangle]
pub unsafe extern "C" fn i8080_ram_ptr(cpu: *const I8080Cpu) -> *const u8 {
    cpu_ref(cpu)
        .map(|cpu| cpu.cpu.memory().ram_ptr())
        .unwrap_or(ptr::null())
}

/// # Safety
/// `cpu` must be a valid non-null pointer. The returned pointer is borrowed from the host-provided RAM.
#[no_mangle]
pub unsafe extern "C" fn i8080_ram_mut_ptr(cpu: *mut I8080Cpu) -> *mut u8 {
    cpu_mut(cpu)
        .map(|cpu| cpu.cpu.memory_mut().ram_mut_ptr())
        .unwrap_or(ptr::null_mut())
}

/// # Safety
/// `cpu` must be a valid non-null pointer.
#[no_mangle]
pub unsafe extern "C" fn i8080_ram_len(cpu: *const I8080Cpu) -> usize {
    cpu_ref(cpu)
        .map(|cpu| cpu.cpu.memory().ram_len())
        .unwrap_or(0)
}
