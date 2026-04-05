#[derive(Debug)]
pub struct MemoryOutOfBounds;

#[derive(Debug)]
pub enum EmulatorErrors {
    MemoryOutOfBounds(MemoryOutOfBounds),
    NoPendingInput,
    InvalidInterrupt(u8),
}

impl From<MemoryOutOfBounds> for EmulatorErrors {
    fn from(value: MemoryOutOfBounds) -> Self {
        Self::MemoryOutOfBounds(value)
    }
}
