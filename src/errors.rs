use std::io;

#[derive(Debug)]
pub struct InvalidFile;

#[derive(Debug)]
pub struct MemoryOutOfBounds;

#[derive(Debug)]
pub struct RomReadFailure;

#[derive(Debug)]
pub enum EmulatorErrors {
    Io(io::Error),
    InvalidFile(InvalidFile),
    MemoryOutOfBounds(MemoryOutOfBounds),
    RomReadFailure(RomReadFailure),
}

impl From<io::Error> for EmulatorErrors {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<InvalidFile> for EmulatorErrors {
    fn from(value: InvalidFile) -> Self {
        Self::InvalidFile(value)
    }
}

impl From<MemoryOutOfBounds> for EmulatorErrors {
    fn from(value: MemoryOutOfBounds) -> Self {
        Self::MemoryOutOfBounds(value)
    }
}

impl From<RomReadFailure> for EmulatorErrors {
    fn from(value: RomReadFailure) -> Self {
        Self::RomReadFailure(value)
    }
}