use std::io;

#[derive(Debug)]
pub struct InvalidFile;

#[derive(Debug)]
pub struct MemoryOutOfBounds;

#[derive(Debug)]
pub enum EmulatorErrors {
    Io(io::Error),
    InvalidFile(InvalidFile),
    MemoryOutOfBounds(MemoryOutOfBounds),
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
